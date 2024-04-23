/*! Buffered, message parsing mio TCP stream wrapper */
use std::fmt::Display;
use std::io::{self, Read, Write};

const MAX_MESSAGE_BYTES: u8 = 50;

// Try to read a single-byte message length header into the first byte of `buf`
fn try_read_message_length(
    max_bytes: u8,
    buf: &mut [u8],
    stream: &mut impl io::Read,
) -> io::Result<()> {
    let read = stream.read(&mut buf[0..1])?;
    if read == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    let to_read = u8::from_le_bytes(buf[0..1].try_into().unwrap());
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

/// Provides for buffered message read & writes to a `mio` TCP stream.
/// Methods should not `WouldBlock` but drain, buffer & parse header-prefixed data.
#[derive(Debug)]
pub struct ClientStream {
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

    /// Returns all readable messages on the stream.
    pub fn try_read_messages(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut messages = vec![];
        if self.read_bytes == 0 {
            match try_read_message_length(
                self.max_message_bytes(),
                &mut self.read_buffer,
                &mut self.stream,
            ) {
                Ok(()) => {
                    self.read_bytes = 1;
                }
                Err(ref io_err) if is_interrupted(io_err) => {
                    return self.try_read_messages();
                }
                Err(ref io_err) if is_would_block(io_err) => {
                    return Ok(messages);
                }
                Err(io_err) => return Err(io_err),
            }
        }

        loop {
            match self.stream.read(&mut self.read_buffer[self.read_bytes..]) {
                Ok(n) if n < 1 => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => {
                    let message_len = self.read_buffer[0] as usize;
                    let total_read = self.read_bytes + n;
                    let to_read = message_len + 1;
                    if total_read >= to_read {
                        messages.push(self.read_buffer[1..=message_len].to_vec());
                        let next_message_bytes = total_read - to_read;
                        self.read_bytes = next_message_bytes;
                        if next_message_bytes > 0 {
                            let next_message =
                                &self.read_buffer[to_read..to_read + next_message_bytes].to_vec();
                            self.read_buffer[0..next_message_bytes].copy_from_slice(next_message);
                        }
                    } else {
                        self.read_bytes += n;
                    }
                }
                Err(ref io_err) if is_would_block(io_err) => {
                    return Ok(messages);
                }
                Err(ref io_err) if is_interrupted(io_err) => {
                    continue;
                }
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
        let header_written = self.stream.write(&size_prefix.to_le_bytes())?;
        let data_written = self.stream.write(data)?;
        Ok(header_written + data_written)
    }
    /// write the server header
    pub fn write_header(&mut self) -> io::Result<()> {
        //self.write(lib_spells::SERVER_HEADER.as_bytes())
        Ok(())
    }
}

impl Display for ClientStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.ip_or_unknown())
    }
}

#[cfg(test)]
mod tests {
    use mio;
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_write_prefixed() {
        let listener = mio::net::TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let _unused = mio::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let client_stream = listener.accept().unwrap();
        let mut client_stream = ClientStream::new(client_stream.0);

        let data = b"hello!";
        let written = client_stream.write_prefixed(data).unwrap();
        // prefixed should be len + the length of the u32 size header we prefix first
        assert_eq!(written, data.len() + size_of::<u32>());
    }

    struct FakeReader {
        length: usize,
    }

    impl std::io::Read for FakeReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            buf[0] = self.length.to_le_bytes()[0];
            Ok(1)
        }
    }

    #[test]
    fn test_read_message_length() {
        let mut buf = vec![0; 1];
        let res = try_read_message_length(100, &mut buf, &mut FakeReader { length: 101 });
        assert!(res.is_err());
        let res = try_read_message_length(100, &mut buf, &mut FakeReader { length: 100 });
        assert!(res.is_ok());
        assert!(buf[0] == 100);
    }

    #[test]
    fn test_read_messaging() {
        let messages = vec![
            "abcde".as_bytes(),
            "fghij".as_bytes(),
            "bingusss".as_bytes(),
            "fongusbeep".as_bytes(),
        ];
        let n_messages = messages.len();
        let server = mio::net::TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let server_addr = server.local_addr().unwrap();

        let (tx, rx) = std::sync::mpsc::channel();

        let handle = std::thread::spawn(move || {
            let stream = loop {
                tx.send(true).unwrap();
                match server.accept() {
                    Ok((stream, _)) => break stream,
                    Err(_err) => continue,
                }
            };
            let mut client_stream = ClientStream::new(stream);
            let mut received = vec![];
            loop {
                let mut recv_messages = client_stream.try_read_messages().unwrap();
                if !recv_messages.is_empty() {
                    received.append(&mut recv_messages);
                }
                if received.len() == n_messages {
                    return received;
                }
            }
        });

        let mut client = std::net::TcpStream::connect(server_addr).unwrap();
        rx.recv().unwrap();
        for message in messages.iter() {
            let len = &[message.len() as u8];
            // prefix header
            client.write_all(len).unwrap();
            for byte in message.iter() {
                client.write_all(&[*byte]).unwrap();
            }
        }

        let res = handle.join().unwrap();
        assert_eq!(res, messages);
        let strs = res
            .iter()
            .map(|b| String::from_utf8(b.to_vec()).unwrap())
            .collect::<Vec<String>>();
        dbg!(strs);
    }
}
