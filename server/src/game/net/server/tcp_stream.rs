use bevy::log;
use mio::{Interest, Token};
use std::fmt::Display;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub(super) struct ClientStream {
    stream: mio::net::TcpStream,
}

impl ClientStream {
    pub fn new(stream: mio::net::TcpStream) -> Self {
        Self { stream }
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
        self.write_all(lib_spells::SERVER_HEADER.as_bytes())
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

impl Drop for ClientStream {
    fn drop(&mut self) {
        log::info!("dropped client: {}", self.ip_or_unknown());
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
