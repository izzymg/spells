use std::{
    fmt::Display,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{self, TcpStream},
    sync::mpsc::Sender,
};

use bevy::log;

const PREFIX_BYTES: usize = 4;
const MAX_MESSAGE_SIZE: u32 = 300;

#[derive(Debug, PartialEq)]
pub enum ServerStreamError {
    InvalidServer,
    ConnectionEnded,
    BigMessage(u32),
    IO(io::ErrorKind),
}

impl std::error::Error for ServerStreamError {}

impl Display for ServerStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(io_err) => {
                write!(f, "IO error: {}", io_err)
            }
            Self::InvalidServer => {
                write!(f, "invalid server response")
            }
            Self::ConnectionEnded => {
                write!(f, "server connection ended")
            }
            Self::BigMessage(size) => {
                write!(f, "message too big: {} bytes", size)
            }
        }
    }
}

impl From<io::Error> for ServerStreamError {
    fn from(value: io::Error) -> Self {
        if value.kind() == io::ErrorKind::UnexpectedEof {
            Self::ConnectionEnded
        } else {
            Self::IO(value.kind())
        }
    }
}

pub type ServerStreamResult<T> = std::result::Result<T, ServerStreamError>;
type Result<T> = ServerStreamResult<T>;

#[derive(Debug, PartialEq)]
pub enum ServerStreamMessage {
    Info(String),
    Data(Vec<u8>),
}

/// Try to create a new world connection from the given address
pub fn connect(addr: &str) -> Result<ServerStream> {
    let stream = net::TcpStream::connect(addr)?;
    ServerStream::handle(stream)
}

pub struct ServerStream {
    reader: BufReader<net::TcpStream>,
    writer: BufWriter<net::TcpStream>,
}

impl ServerStream {
    /// Consume a TCP stream as world connection
    pub fn handle(stream: TcpStream) -> Result<ServerStream> {
        let writer = io::BufWriter::new(stream.try_clone()?);
        let reader = io::BufReader::new(stream);
        Ok(Self { reader, writer })
    }

    /// Block until data received, and return if the data matches the given.
    fn expect_line(&mut self, data: &str) -> Result<bool> {
        let mut buf = data.to_string();
        buf.clear();
        self.reader.read_line(&mut buf)?;
        Ok(buf == data)
    }

    /// Block until we receive the expected server header response from Spells Server.
    fn expect_header(&mut self) -> Result<bool> {
        self.expect_line(lib_spells::SERVER_HEADER)
    }

    fn write_client_ok(&mut self) -> Result<()> {
        self.writer
            .write_all(lib_spells::CLIENT_EXPECT.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn handshake(&mut self) -> Result<()> {
        if !self.expect_header()? {
            return Err(ServerStreamError::InvalidServer);
        }
        log::info!("OK header from server");
        self.write_client_ok()?;
        log::info!("sent OK");
        Ok(())
    }

    /// Block and listen to the world stream, sending new informaton to tx. Does handshake first.
    pub fn listen_handshake(&mut self, tx: Sender<ServerStreamMessage>) -> Result<()> {
        tx.send(ServerStreamMessage::Info("handshaking".into()))
            .expect("server listen: no receiver");
        self.handshake()?;
        tx.send(ServerStreamMessage::Info("connected".into()))
            .expect("server listen: no receiver");
        self.listen(tx)
    }

    /// Block and listen to the world stream, sending new informaton to tx.
    pub fn listen(&mut self, tx: Sender<ServerStreamMessage>) -> Result<()> {
        // wait for length header
        let mut header_buffer = [0_u8; PREFIX_BYTES];

        loop {
            // read header
            self.reader.read_exact(&mut header_buffer)?;
            let message_size: u32 = u32::from_le_bytes(header_buffer);

            // read message
            if message_size > MAX_MESSAGE_SIZE {
                log::info!("big message {}", message_size);
                return Err(ServerStreamError::BigMessage(message_size));
            }

            let mut message = vec![0; message_size as usize];
            self.reader.read_exact(&mut message)?;
            tx.send(ServerStreamMessage::Data(message))
                .expect("server listen: no state receiver");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    use crate::world_connection::stream::ServerStreamMessage;

    use super::{ServerStream, ServerStreamError};

    struct ListenTest {
        data: Vec<u8>,
    }

    #[test]
    fn test_listen_loop() {
        let tests = vec![
            ListenTest {
                data: "bingus".as_bytes().to_vec(),
            },
            ListenTest {
                data: "b".as_bytes().to_vec(),
            },
            ListenTest {
                data: "0".as_bytes().to_vec(),
            },
            ListenTest { data: vec![5] },
            ListenTest {
                data: vec![5, 50, 30],
            },
        ];

        for test in tests {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
            let mut client_to_world_conn = ServerStream::handle(stream).unwrap();
            let (world_to_client_conn, _) = listener.accept().unwrap();

            let message = test.data;
            let (tx, rx) = mpsc::channel();

            // write header bytes
            (&world_to_client_conn)
                .write_all(&(message.len() as u32).to_le_bytes())
                .unwrap();
            // write actual bytes
            (&world_to_client_conn).write_all(&message).unwrap();
            let handle = thread::spawn(move || client_to_world_conn.listen(tx));
            let val = rx.recv().unwrap();
            assert_eq!(val, ServerStreamMessage::Data(message));
            world_to_client_conn
                .shutdown(std::net::Shutdown::Both)
                .unwrap();

            let val = handle.join().unwrap();
            assert_eq!(val.unwrap_err(), ServerStreamError::ConnectionEnded);
        }
    }
}
