/*! Buffered, message parsing mio TCP stream wrapper */
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::net;

pub const HEADER_BYTES: usize = 2;
pub const MAX_MESSAGE_BYTES: usize = u16::MAX as usize;

#[derive(Debug)]
pub enum MessageStreamError {
    InvalidHeaderSize,
    WriteMessageErr,
    IO(io::Error),
}

pub type Result<T> = std::result::Result<T, MessageStreamError>;

impl Display for MessageStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeaderSize => {
                write!(f, "invalid header size")
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

fn create_header(data: &[u8]) -> [u8;2] {
    (data.len() as u16).to_le_bytes()
}

/// Attempt to parse 2 bytes into a message length
fn parse_message_length(buf: &[u8;2]) -> io::Result<usize> {
    let to_read = u16::from_le_bytes(*buf) as usize;
    if to_read < 1 || to_read > MAX_MESSAGE_BYTES {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(to_read)
}

fn parse_messages(buf: &[u8], start: usize, len: usize, mut messages: Vec<Vec<u8>>) -> Result<(usize, usize, Vec<Vec<u8>>)> {
    // assume we start from the position of the header
    let our_bit = &buf[start..len];
    if our_bit.len() < HEADER_BYTES {
        return Ok((start, len, messages));
    }
    let message_len = parse_message_length(our_bit[..HEADER_BYTES].try_into().unwrap())?;
    let total_read_size = HEADER_BYTES + message_len;
    if our_bit.len() < total_read_size {
        // we didn't have enough data for the complete message
        return Ok((start, len, messages));
    }
    // add a full message
    messages.push(our_bit[HEADER_BYTES..HEADER_BYTES+message_len].to_vec());
    let more = len - (total_read_size);
    if more > 0 {
        parse_messages(buf, start + total_read_size, len, messages)
    } else {
        Ok((message_len, len, messages))
    }
}

/// Provides for buffered message read & writes to a `mio` TCP stream.
/// Methods should not `WouldBlock` but drain, buffer & parse header-prefixed data.
#[derive(Debug)]
pub struct MessageStream {
    stream: net::TcpStream,
    addr: String,

    read_buffer: Vec<u8>,
    read_start: usize,
    read_end: usize,
}

impl MessageStream {
    /// Consume & configure a stream. Can fail.
    pub fn create(stream: net::TcpStream) -> io::Result<Self> {
        let addr = stream.peer_addr()?.to_string();

        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;

        Ok(Self {
            stream,
            read_buffer: vec![0; HEADER_BYTES + MAX_MESSAGE_BYTES],
            read_start: 0,
            read_end: 0,
            addr,
        })
    }

    pub fn into_inner(self) -> net::TcpStream {
        self.stream
    }

    pub fn inner(&mut self) -> &mut net::TcpStream {
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
        match self.stream.read(&mut self.read_buffer) {
            Ok(n) if n < 1 => {
                Err(io::ErrorKind::UnexpectedEof.into())
            }
            Ok(n) => {
               let (start, end, messages) = parse_messages(&self.read_buffer, self.read_start, self.read_end + n, messages)?;
               self.read_start = start;
               self.read_end = end;
               Ok(messages)
            }
            Err(ref io_err) if is_would_block(io_err) => {
                Ok(messages)
            }
            Err(ref io_err) if is_interrupted(io_err) => {
                self.try_read_messages()
            }
            Err(io_err) => Err(io_err.into())
        }
    }
}

impl Display for MessageStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_message_length() {
        const SIZE: usize = MAX_MESSAGE_BYTES - 10;
        let header = create_header(&[0; SIZE]);
        let res = parse_message_length(&header);
        assert!(res.is_ok());
        assert!(res.unwrap() == SIZE);
    }

    #[test]
    fn test_read_complete_messages() {
        let messages = [
            "123".as_bytes(),
            "abc".as_bytes(),
            "zxcb".as_bytes(),
        ];

        let buf = messages.iter().flat_map(|msg| {
            let header = create_header(msg); 
            [&header[..], msg].concat()
        }).collect::<Vec<u8>>();
    
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

    #[test]
    fn test_try_write_prefixed() {
        let message = "bonguscan".as_bytes();
        let server = net::TcpListener::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            let stream = loop {
                match server.accept() {
                    Ok((stream, _)) => break stream,
                    Err(_err) => continue,
                }
            };
            MessageStream::create(stream).unwrap()
                .try_write_prefixed(message)
                .unwrap();
        });

        let mut client = std::net::TcpStream::connect(server_addr).unwrap();
        handle.join().unwrap();
        let mut buf = vec![0; message.len()];
        assert_eq!(
            message.len() + HEADER_BYTES,
            client.read_to_end(&mut buf).unwrap()
        );
    }
}
