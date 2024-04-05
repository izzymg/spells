use std::io::{self, Read, Write};

use mio::{Interest, Token};

use super::SERVER_HEADER;

#[derive(Debug)]
pub struct ClientStream {
    stream: mio::net::TcpStream,
}

impl ClientStream {

    pub fn new(stream: mio::net::TcpStream) -> io::Result<Self> {
        Ok(Self {
            stream,
        })
    }

    pub fn write(&mut self, data: &str) -> io::Result<()> {
        println!("stream write: {}", data);
        self.stream.write_all(data.as_bytes())?;
        self.stream.flush()
    }

    pub fn write_header(&mut self) -> io::Result<()> {
        self.write(SERVER_HEADER.into())
    }

    pub fn read_fill(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    pub fn register_to_poll(&mut self, token: Token, poll: &mut mio::Poll) -> io::Result<()> {
        poll.registry().register(&mut self.stream, token, Interest::READABLE | Interest::WRITABLE)
    }

    pub fn deregister_from_poll(&mut self, poll: &mut mio::Poll) -> io::Result<()> {
        poll.registry().deregister(&mut self.stream)
    }

}