/*! Buffered, message parsing mio TCP stream wrapper */
use std::fmt::Display;
use std::io;

pub const HEADER_BYTES: usize = 2;

#[derive(Debug)]
pub enum MessageStreamError {
    InvalidHeaderSize(usize),
    WriteMessageErr,
    IO(io::Error),
}

pub type Result<T> = std::result::Result<T, MessageStreamError>;

impl Display for MessageStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeaderSize(size) => {
                write!(f, "invalid header size {}", size)
            }
            Self::WriteMessageErr => {
                write!(f, "failed to write full message")
            }
            Self::IO(err) => {
                write!(f, "io error: {}", err)
            }
        }
    }
}

impl From<io::Error> for MessageStreamError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<io::ErrorKind> for MessageStreamError {
    fn from(value: io::ErrorKind) -> Self {
        Self::IO(value.into())
    }
}

fn is_interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

fn is_would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn create_header(data: &[u8]) -> [u8; 2] {
    (data.len() as u16).to_le_bytes()
}

/// Attempt to parse 2 bytes into a message length
fn parse_message_length(buf: &[u8; 2], max: usize) -> Result<usize> {
    let to_read = u16::from_le_bytes(*buf) as usize;
    if to_read < 1 || to_read > max {
        return Err(MessageStreamError::InvalidHeaderSize(to_read));
    }
    Ok(to_read)
}

/// Recursively parse header-messages from `buf`. Rejects headers for messages that are larger than `buf` + a header.
fn parse_messages(
    buf: &[u8],
    start: usize,
    len: usize,
    mut messages: Vec<Vec<u8>>,
) -> Result<(usize, usize, Vec<Vec<u8>>)> {
    // assume we start from the position of the header
    let our_bit = &buf[start..len];
    if our_bit.len() < HEADER_BYTES {
        return Ok((start, len, messages));
    }
    let message_len = parse_message_length(
        our_bit[..HEADER_BYTES].try_into().unwrap(),
        buf.len() - HEADER_BYTES,
    )?;
    let total_read_size = HEADER_BYTES + message_len;
    if our_bit.len() < total_read_size {
        // we didn't have enough data for the complete message
        return Ok((start, len, messages));
    }
    // add a full message
    messages.push(our_bit[HEADER_BYTES..HEADER_BYTES + message_len].to_vec());
    let more = len - (total_read_size);
    if more > 0 {
        parse_messages(buf, start + total_read_size, len, messages)
    } else {
        Ok((start + total_read_size, len, messages))
    }
}

#[derive(Debug)]
pub struct MessageStream<T: io::Read + io::Write> {
    stream: T,

    read_buffer: Vec<u8>,
    last_read: usize,
    msg_start: usize,
    msg_end: usize,
}

impl<T: io::Read + io::Write> MessageStream<T> {
    /// Consume a stream as a message stream. You should set options like no_delay, non_blocking
    /// etc before passing or after via `inner()`. Can fail.
    pub fn create(stream: T, max_message_bytes: usize) -> Result<Self> {
        Ok(Self {
            stream,
            read_buffer: vec![0; HEADER_BYTES + max_message_bytes],
            last_read: 0,
            msg_start: 0,
            msg_end: 0,
        })
    }

    pub fn into_inner(self) -> T {
        self.stream
    }

    pub fn inner(&mut self) -> &mut T {
        &mut self.stream
    }

    /// Try to write all of what's buffered with a length prefix. Returns true if all of the buffer
    /// was written, false if nothing was written. Errors on partial writes.
    pub fn try_write_prefixed(&mut self, buffer: &[u8]) -> Result<bool> {
        // messages headers are hard set at 2 bytes (i.e. u16)
        let header_bytes = create_header(buffer);
        match self.stream.write_all(&[&header_bytes, buffer].concat()) {
            Ok(_) => Ok(true),
            Err(ref err) if is_would_block(err) => Ok(false),
            Err(ref err) if is_interrupted(err) => self.try_write_prefixed(buffer),
            Err(err) => Err(err.into()),
        }
    }

    /// Returns all readable messages on the stream.
    pub fn try_read_messages(&mut self) -> Result<Vec<Vec<u8>>> {
        let messages = Vec::with_capacity(1);
        match self.stream.read(&mut self.read_buffer[self.last_read..]) {
            Ok(n) if n < 1 => Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => {
                self.last_read += n;
                let (start, end, messages) = parse_messages(
                    &self.read_buffer,
                    self.msg_start,
                    self.msg_end + n,
                    messages,
                )?;

                // buffer full
                if self.last_read >= self.read_buffer.len() {
                    self.msg_start = 0;
                    self.msg_end = end - start;
                    self.last_read = self.msg_end;
                    let (l, r) = self.read_buffer.split_at_mut(start);
                    l[self.msg_start..self.msg_end].copy_from_slice(r);
                } else {
                    self.msg_start = start;
                    self.msg_end = end;
                }
                Ok(messages)
            }
            Err(ref io_err) if is_would_block(io_err) => Ok(messages),
            Err(ref io_err) if is_interrupted(io_err) => self.try_read_messages(),
            Err(io_err) => Err(io_err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct FakeReader {
        data: Vec<u8>,
        pos: usize,
    }
    impl std::io::Read for FakeReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut read = 0;
            for i in buf.iter_mut() {
                if self.pos >= self.data.len() {
                    return Ok(read);
                }
                *i = self.data[self.pos];
                self.pos += 1;
                read += 1;
            }
            Ok(read)
        }
    }

    impl std::io::Write for FakeReader {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!();
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!()
        }
    }

    #[test]
    fn test_buffer_shift() {
        let data = vec![3_u8, 0, 1, 2, 3, 2_u8, 0, 9, 8, 3_u8, 0, 7, 6];
        let reader = FakeReader {
            data: data.clone(),
            pos: 0,
        };

        let mut message_stream = MessageStream::create(reader, 3).unwrap();
        let messages = message_stream.try_read_messages().unwrap();
        assert_eq!(messages[0], data[2..5]);
        let messages = message_stream.try_read_messages().unwrap();
        assert_eq!(messages[0], data[7..9]);
        message_stream.try_read_messages().unwrap();
        // buffer should have wrapped our data to the front
        assert_eq!(message_stream.read_buffer[0..3], data[9..12]);
        assert_eq!(message_stream.last_read, 4);
    }

    #[test]
    fn test_get_message_length() {
        {
            const SIZE: usize = 10;
            let header = create_header(&[0; SIZE]);
            let res = parse_message_length(&header, SIZE);
            assert!(res.is_ok());
            assert!(res.unwrap() == SIZE);
        }
        {
            const SIZE: usize = 15;
            let header = create_header(&[0; SIZE + 1]);
            let res = parse_message_length(&header, SIZE);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_read_complete_messages() {
        let messages = ["123".as_bytes(), "abc".as_bytes(), "zxcb".as_bytes()];

        let buf = messages
            .iter()
            .flat_map(|msg| {
                let header = create_header(msg);
                [&header[..], msg].concat()
            })
            .collect::<Vec<u8>>();

        let (start, end, received) = parse_messages(&buf, 0, buf.len(), vec![]).unwrap();
        assert!(start == buf.len() && end == buf.len());
        for (i, recv) in received.iter().enumerate() {
            assert_eq!(messages[i], recv);
        }
    }

    #[test]
    fn test_read_incomplete_messages() {
        let buf = [2_u8, 0, 1, 2, 3_u8, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0]; // missing one byte at the end
        let expect_start = 4; // we should get back where the next partial message begins
        let actual_bytes = 7; // simulate padded buffer

        let (start, end, received) = parse_messages(&buf, 0, actual_bytes, vec![]).unwrap();
        assert_eq!(start, expect_start);
        assert_eq!(end, actual_bytes);
        assert!(received.len() == 1);
        assert!(received[0] == [1, 2]);
    }
}
