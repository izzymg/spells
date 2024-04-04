use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, time::Duration};

const CLIENT_TIMEOUT: Duration = Duration::from_millis(3500);
const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub struct ClientGetter {
    listener: TcpListener,
}

impl ClientGetter {
    pub fn create() -> io::Result<ClientGetter> {

        let listener = TcpListener::bind("127.0.0.1:7776")?;

        Ok(ClientGetter {
            listener,
        })
    }

    pub fn block_get_client(&self) -> io::Result<ClientStream> {
        println!("waiting for client...");
        match self.listener.accept() {
            Ok((stream, addr)) => Ok({
                println!("client acccepted {}", addr);
                ClientStream::new(stream)?
            }),
            Err(err) => Err(err)
        }
    }
}

pub struct ClientStream {
    stream: TcpStream,
}

impl ClientStream {

    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
        Ok(Self {
            stream,
        })
    }

    pub fn shutdown(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }

    pub fn write(&mut self, data: String) -> io::Result<()> {
        println!("stream write: {}", data);
        self.stream.write_all(data.as_bytes())?;
        self.stream.flush()
    }

    pub fn write_header(&mut self) -> io::Result<()> {
        self.write(SERVER_HEADER.into())
    }

    pub fn expect_client_response(&mut self, timeout: Option<Duration>) -> io::Result<bool> {
        self.stream.set_read_timeout(timeout)?;
        let mut buf = [0; CLIENT_EXPECT.len()];
        self.stream.read_exact(&mut buf)?;
        self.stream.set_read_timeout(None)?;
        return Ok(buf == *CLIENT_EXPECT.as_bytes())
    }

}