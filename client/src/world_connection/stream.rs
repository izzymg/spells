use lib_spells::tcp_stream;
use std::{
    fmt::Display,
    io::{self, Read, Write},
    sync::mpsc,
    time::Duration,
};

const PREFIX_BYTES: usize = 4;
const MAX_MESSAGE_SIZE: u32 = 10 * 1000;
const SERVER_READ_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));
const SERVER_WRITE_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));

pub type Result<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    InvalidServer,
    ConnectionEnded,
    BigMessage(u32),
    IO(io::ErrorKind),
    BadAddress(std::net::AddrParseError),
    BadData,
}

impl std::error::Error for ConnectionError {}

impl Display for ConnectionError {
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
            Self::BadAddress(addr_err) => {
                write!(f, "bad address: {}", addr_err)
            }
            Self::BadData => {
                write!(f, "bad data")
            }
        }
    }
}

impl From<std::net::AddrParseError> for ConnectionError {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::BadAddress(value)
    }
}

impl From<io::Error> for ConnectionError {
    fn from(value: io::Error) -> Self {
        if value.kind() == io::ErrorKind::UnexpectedEof {
            Self::ConnectionEnded
        } else {
            Self::IO(value.kind())
        }
    }
}

pub enum Incoming {
    WorldState(lib_spells::net::WorldState),
}

pub enum Outgoing {
    Disconnect,
    Movement(bevy::math::Vec3),
}

#[derive(Debug)]
pub struct Connection {
    stream: tcp_stream::ClientStream,
}

impl Connection {
    pub fn listen_outgoing(out_rx: mpsc::Receiver<Outgoing>) -> Result<()> {
        Ok(())
    }

    pub fn listen_incoming(inc_tx: mpsc::Sender<Incoming>) -> Result<()> {
        Ok(())
    }
}

pub fn get_connection(addr: &str, password: Option<&str>) -> Result<(Connection, lib_spells::net::ClientInfo)> {
    let mut raw_stream = mio::net::TcpStream::connect(addr.parse()?)?;
    let mut poll = mio::Poll::new()?;
    let mut events = mio::Events::with_capacity(128);
    poll.registry().register(
        &mut raw_stream,
        mio::Token(0),
        mio::Interest::READABLE | mio::Interest::WRITABLE,
    )?;

    let mut client_stream = tcp_stream::ClientStream::new(raw_stream);
    poll.poll(&mut events, None)?;

    let mut messages = vec![];
    let mut wrote_pass = false;

    loop {
        for ev in events.iter() {
            if ev.is_readable() {
                read_messages(&mut client_stream, &mut messages)?;
                println!("read messages");
            }
            if !wrote_pass && ev.is_writable() {
                if let Some(password) = password {
                    wrote_pass = write_data(&mut client_stream, password.as_bytes())?;
                    println!("wrote password: {}", wrote_pass);
                }
            }
            if let Some(client_info) = validate_server_messages(&messages)? {
                println!("verified server");
                return Ok((Connection { stream: client_stream }, client_info));
            }
        }
    }
}

fn validate_server_messages(messages: &[Vec<u8>]) -> Result<Option<lib_spells::net::ClientInfo>> {
    if let Some(msg) = messages.first() {
        if msg != lib_spells::SERVER_HEADER {
            return Err(ConnectionError::InvalidServer);
        }
    } else {
        return Ok(None);
    }

    let client_info_raw = if let Some(msg) = messages.get(1) {
        msg
    } else {
        return Ok(None);
    };

    let client_info = match lib_spells::net::ClientInfo::deserialize(client_info_raw) {
        Ok(ci) => ci,
        Err(_) => return Err(ConnectionError::BadData),
    };

    Ok(Some(client_info))
}

fn read_messages(stream: &mut lib_spells::tcp_stream::ClientStream, messages: &mut Vec<Vec<u8>>) -> Result<()> {
    match stream.try_read_messages() {
        Ok(mut received) => {
            messages.append(&mut received);
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn write_data(stream: &mut lib_spells::tcp_stream::ClientStream, data: &[u8]) -> Result<bool> {
    Ok(stream.try_write_prefixed(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_client_stream() {
        get_connection("0.0.0.0:7776", Some("cat")).unwrap();
        println!("connected");
    }
}
