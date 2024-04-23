/*! Buffered, message parsing mio TCP stream wrapper */
use bevy::log;
use mio::{Interest, Token};
use std::fmt::Display;
use std::io::{self, Read, Write};

const MAX_MESSAGE_BYTES: u8 = 50;

// Try to read a single-byte message length header into the first byte of `buf`
fn try_read_message_length(max_bytes: u8, buf: &mut [u8], stream: &mut impl io::Read) -> io::Result<()> {
    stream.read(&mut buf[0..1])?;
    let to_read = u8::from_le_bytes(buf.try_into().unwrap());
    if to_read < 1 || to_read > max_bytes {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(())
}

fn is_interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

fn is_would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

#[derive(Debug)]
pub(super) struct ClientStream {
    stream: mio::net::TcpStream,
    read_buffer: Vec<u8>,
    read_bytes: usize,
}

impl ClientStream {
    fn max_message_bytes(&self) -> u8 {
        // exclude the byte for the header
        (self.read_buffer.len() - 1).try_into().unwrap()
    }

    pub fn new(stream: mio::net::TcpStream) -> Self {
        Self {
            stream,
            // extra byte is so we can store the header here + content len
            read_buffer: vec![0; (MAX_MESSAGE_BYTES + 1).into()],
            read_bytes: 0,
        }
    }

    pub fn into_inner(self) -> mio::net::TcpStream {
        self.stream
    }

    pub fn try_read_messages(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut messages = vec![];
        if self.read_bytes == 0 {
            match try_read_message_length(self.max_message_bytes(), &mut self.read_buffer, &mut self.stream) {
                Ok(()) => {
                    self.read_bytes = 1;
                },
                Err(ref io_err) if is_interrupted(io_err) => {
                    return self.try_read_messages(); 
                },
                Err(ref io_err) if is_would_block(io_err) => {
                    return Ok(messages);
                },
                Err(io_err) => return Err(io_err),
            }
        }

        loop {
            let message_len = self.read_buffer[0] as usize;
            match self.stream.read(&mut self.read_buffer[self.read_bytes..]) {
                 Ok(n) if n < 1 => {
                     return Err(io::ErrorKind::UnexpectedEof.into())
                 },
                 Ok(n) => {
                     // we've read at least one whole message
                     let total_read = self.read_bytes + n;
                     if total_read >= message_len {
                        messages.push(self.read_buffer[1..=message_len].to_vec());
                        // start again reading messages from here
                        let next_message_bytes = total_read - message_len;
                        self.read_bytes = next_message_bytes;
                        if next_message_bytes > 0 {
                            // we picked up some of the next message too
                            let next_message_start = message_len + 1;
                            let next_message_end = next_message_start + next_message_bytes;
                            let next_message = &self.read_buffer[next_message_start..next_message_end].to_vec();
                            self.read_buffer[0..next_message_bytes].copy_from_slice(next_message);
                        }
                     } else {
                         self.read_bytes += n;
                     }
                 },
                 Err(ref io_err) if is_would_block(io_err) => {
                     return Ok(messages);
                 },
                 Err(ref io_err) if is_interrupted(io_err) => {
                    continue;
                 },
                 Err(io_err) => return Err(io_err),
            }
        }
    }

    pub fn ip_or_unknown(&self) -> String {
        if let Ok(addr) = self.stream.peer_addr() {
            addr.to_string()
        } else {
            "Unknown Address".into()
        }
    }

    /// prefix a 4 byte LE length header then write the data
    pub fn write_prefixed(&mut self, data: &[u8]) -> io::Result<usize> {
        let size_prefix: u32 = data.len() as u32;
        let header_written = self.write(&size_prefix.to_le_bytes())?;
        let data_written = self.write(data)?;
        Ok(header_written + data_written)
    }

    /// write the server header
    pub fn write_header(&mut self) -> io::Result<()> {
        //self.write(lib_spells::SERVER_HEADER.as_bytes())
        Ok(())
    }

    pub fn register_to_poll(&mut self, token: Token, registry: &mio::Registry) -> io::Result<()> {
        registry.register(
            &mut self.stream,
            token,
            Interest::READABLE.add(Interest::WRITABLE),
        )
    }

    pub fn deregister_from_poll(&mut self, registry: &mio::Registry) -> io::Result<()> {
        registry.deregister(&mut self.stream)
    }
}

impl Read for ClientStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for ClientStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl Display for ClientStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.ip_or_unknown())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use mio::net::TcpListener;
    use mio::net::TcpStream;

    use super::ClientStream;

    #[test]
    fn test_write_prefixed() {
        let listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let _unused = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let client_stream = listener.accept().unwrap();
        let mut client_stream = ClientStream::new(client_stream.0);

        let data = b"hello!";
        let written = client_stream.write_prefixed(data).unwrap();
        // prefixed should be len + the length of the u32 size header we prefix first
        assert_eq!(written, data.len() + size_of::<u32>());
    }
}
