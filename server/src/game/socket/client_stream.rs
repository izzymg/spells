use std::io::{self, Read, Write};

use mio::{Interest, Token};

use super::SERVER_HEADER;

#[derive(Debug)]
pub struct ClientStream {
    stream: mio::net::TcpStream,
}

impl ClientStream {
    pub fn new(stream: mio::net::TcpStream) -> io::Result<Self> {
        Ok(Self { stream })
    }

    fn ip_or_unknown(&self) -> String {
        if let Ok(addr) = self.stream.peer_addr() {
            addr.to_string()
        } else {
            "unknown".into()
        }
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        println!(
            "client stream write {}: {} bytes",
            self.ip_or_unknown(),
            data.len(),
        );
        self.stream.write_all(data)
    }

    pub fn write_header(&mut self) -> io::Result<()> {
        self.write(SERVER_HEADER.as_bytes())
    }

    pub fn read_fill(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    pub fn register_to_poll(&mut self, token: Token, poll: &mut mio::Poll) -> io::Result<()> {
        poll.registry().register(
            &mut self.stream,
            token,
            Interest::READABLE | Interest::WRITABLE,
        )
    }

    pub fn deregister_from_poll(&mut self, poll: &mut mio::Poll) -> io::Result<()> {
        poll.registry().deregister(&mut self.stream)
    }
}

impl Drop for ClientStream {
    fn drop(&mut self) {
        println!("dropped client: {}", self.ip_or_unknown());
    }
}
