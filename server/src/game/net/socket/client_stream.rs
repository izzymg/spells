use std::io::{self, Read, Write};

use mio::{Interest, Token};

use super::SERVER_HEADER;

#[derive(Debug)]
pub(super) struct ClientStream {
    stream: mio::net::TcpStream,
}

impl ClientStream {
    pub(super) fn new(stream: mio::net::TcpStream) -> io::Result<Self> {
        Ok(Self { stream })
    }

    fn ip_or_unknown(&self) -> String {
        if let Ok(addr) = self.stream.peer_addr() {
            addr.to_string()
        } else {
            "unknown".into()
        }
    }

    pub(super) fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        println!(
            "client stream write {}: {} bytes",
            self.ip_or_unknown(),
            data.len(),
        );
        self.stream.write_all(data)
    }

    /// send a 4 byte LE prefix length header then write the data
    pub(super) fn write_prefixed(&mut self, data: &[u8]) -> io::Result<usize> {
        let size_prefix: u32 = data.len() as u32;
        let header_written = self.stream.write(&size_prefix.to_le_bytes())?;
        let data_written = self.stream.write(&data)?;
        Ok(header_written + data_written)
    }

    pub(super) fn write_header(&mut self) -> io::Result<()> {
        self.write_all(SERVER_HEADER.as_bytes())
    }

    pub(super) fn read_fill(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    pub(super) fn register_to_poll(
        &mut self,
        token: Token,
        poll: &mut mio::Poll,
    ) -> io::Result<()> {
        poll.registry().register(
            &mut self.stream,
            token,
            Interest::READABLE | Interest::WRITABLE,
        )
    }

    pub(super) fn deregister_from_poll(&mut self, poll: &mut mio::Poll) -> io::Result<()> {
        poll.registry().deregister(&mut self.stream)
    }
}

impl Drop for ClientStream {
    fn drop(&mut self) {
        println!("dropped client: {}", self.ip_or_unknown());
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
        let mut client_stream = ClientStream::new(client_stream.0).unwrap();

        let data = b"hello!";
        let written = client_stream.write_prefixed(data).unwrap();
        // prefixed should be len + the length of the u32 size header we prefix first
        assert_eq!(written, data.len() + size_of::<u32>());
    }
}
