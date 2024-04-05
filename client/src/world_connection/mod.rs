use std::{
    fmt::Display, io::{self, BufRead, BufReader, BufWriter, Write}, net::{self, TcpStream}, sync::mpsc::Sender, thread, time::Duration
};

const EXPECT_SERVER_HEADER: &str = "SPELLSERVER 0.1\n";
const CLIENT_RESPONSE: &str = "SPELLCLIENT OK 0.1\n";

#[derive(Debug)]
pub enum WorldConnectionError {
    InvalidServer(),
    IO(io::Error),
}

impl std::error::Error for WorldConnectionError {}

impl Display for WorldConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldConnectionError::IO(io_err) => {
                write!(f, "IO error: {}", io_err)
            },
            WorldConnectionError::InvalidServer() => {
                write!(f, "invalid server response")
            }
        }
    }
}

impl From<io::Error> for WorldConnectionError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

type Result<T> = std::result::Result<T, WorldConnectionError>;

/// Try to create a new world connection from the given address
pub fn connect_retry(addr: &str, delay: Duration) -> Result<WorldConnection> {
    let stream = loop {
        match net::TcpStream::connect(addr) {
            Ok(s) => {
                println!("connected to {}", addr);
                break s;
            }
            Err(err) => {
                println!("failed to connect to {}, retrying ({}) in {}s", addr, err, delay.as_secs());
                thread::sleep(delay);
            }
        }
    };

    WorldConnection::handle(stream)
}

pub struct WorldConnection {
    reader: BufReader<net::TcpStream>,
    writer: BufWriter<net::TcpStream>,
}

impl WorldConnection {
    /// Consume a TCP stream as world connection
    pub fn handle(stream: TcpStream) -> Result<WorldConnection> {
        let writer = io::BufWriter::new(stream.try_clone()?);
        let reader = io::BufReader::new(stream);
        Ok(Self { reader, writer })
    }

    /// Block until data received, and return if the data matches the given.
    fn expect(&mut self, data: &str) -> Result<bool> {
        let mut buf = data.to_string();
        buf.clear();
        self.reader.read_line(&mut buf)?;
        Ok(buf == data)
    }

    /// Block until we receive the expected server header response from Spells Server.
    fn expect_header(&mut self) -> Result<bool> {
        self.expect(EXPECT_SERVER_HEADER)
    }

    fn write_client_ok(&mut self) -> Result<()> {
        self.writer.write_all(CLIENT_RESPONSE.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn handshake(&mut self) -> Result<()> {
        if !self.expect_header()? {
            return Err(WorldConnectionError::InvalidServer())
        }
        println!("OK header from server");
        self.write_client_ok()?;
        println!("sent OK");
        Ok(())
    }

    /// block until we get more state
    pub fn listen(&mut self, tx: Sender<io::Result<String>>) -> io::Result<()> {
        let mut buf = String::new();
        loop {
            buf.clear();
            match self.reader.read_line(&mut buf) {
                Ok(read) => {
                    if read > 0 {
                        println!("server listen: read {} bytes", read);
                        tx.send(Ok(buf.clone())).expect("server listen: No state receiver");
                    } else {
                        tx.send(Err(io::ErrorKind::UnexpectedEof.into())).expect("server listen: No state receiver");
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }
}
