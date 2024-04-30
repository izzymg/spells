use lib_spells::message_stream;
use std::{
    fmt::Display,
    io::{self, Read, Write},
    ops::Deref,
    sync::mpsc,
    time::Duration,
};

const PREFIX_BYTES: usize = 4;
const MAX_MESSAGE_SIZE: u32 = 10 * 1000;
const SERVER_READ_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));
const SERVER_WRITE_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));

pub type Result<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug)]
pub enum ConnectionError {
    IOError(std::io::Error),
    StreamError(message_stream::MessageStreamError),
    InvalidServer,
    ConnectionEnded,
    BigMessage(u32),
    BadAddress(std::net::AddrParseError),
    BadData,
}

impl std::error::Error for ConnectionError {}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(err) => {
                write!(f, "io error: {}", err)
            }
            Self::StreamError(err) => {
                write!(f, "stream error: {}", err)
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

impl From<lib_spells::net::SerializationError> for ConnectionError {
    fn from(value: lib_spells::net::SerializationError) -> Self {
        Self::BadData
    }
}

impl From<std::net::AddrParseError> for ConnectionError {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::BadAddress(value)
    }
}

impl From<message_stream::MessageStreamError> for ConnectionError {
    fn from(value: message_stream::MessageStreamError) -> Self {
        Self::StreamError(value)
    }
}

impl From<std::io::Error> for ConnectionError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

#[derive(Debug)]
pub enum Incoming {
    WorldState(lib_spells::net::WorldState),
}

#[derive(Debug)]
pub struct Connection {
    stream: message_stream::MessageStream,
    stamp: u8,
}

impl Connection {
    
    /// Fetch all world state we can read
    pub fn read_get_world_state(&mut self) -> Result<Vec<lib_spells::net::WorldState>> {
        let messages = self.stream.try_read_messages()?;
        Ok(messages
            .iter()
            .map(|m| lib_spells::net::WorldState::deserialize(m))
            .collect::<std::result::Result<
                Vec<lib_spells::net::WorldState>,
                lib_spells::net::SerializationError,
            >>()?)
    }

    /// Returns true if the input was actually sent
    pub fn send_input(&mut self, command: u8, data: u8) -> Result<bool> {
        let sent = self
            .stream
            .try_write_prefixed(&[command, self.stamp, data])?;
        if sent {
            self.stamp += 1;
        }
        Ok(sent)
    }
}

pub fn get_connection(
    addr: &str,
    password: Option<&str>,
) -> Result<(Connection, lib_spells::net::ClientInfo)> {
    let mut raw_stream = std::net::TcpStream::connect(addr)?;
    raw_stream.set_nonblocking(true)?;
    raw_stream.set_nodelay(true)?;

    let mut message_stream = message_stream::MessageStream::create(raw_stream)?;
    let mut messages = vec![];
    let mut wrote_pass = false;

    loop {
        println!("read messages");
        if !wrote_pass {
            if let Some(password) = password {
                wrote_pass = write_data(&mut message_stream, password.as_bytes())?;
                println!("wrote password: {}", wrote_pass);
            }
        }
        read_messages(&mut message_stream, &mut messages)?;
        dbg!(&messages.len());
        if let Some(client_info) = validate_server_messages(&messages)? {
            println!("verified server");
            return Ok((
                Connection {
                    stream: message_stream,
                    stamp: 0,
                },
                client_info,
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
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

fn read_messages(
    stream: &mut message_stream::MessageStream,
    messages: &mut Vec<Vec<u8>>,
) -> Result<()> {
    let mut received = stream.try_read_messages()?;
    messages.append(&mut received);
    Ok(())
}

fn write_data(stream: &mut message_stream::MessageStream, data: &[u8]) -> Result<bool> {
    Ok(stream.try_write_prefixed(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_client_stream() {
        let (mut conn, client_info) = get_connection("0.0.0.0:7776", Some("cat")).unwrap();
        dbg!(client_info);
        loop {
            let sent = conn.send_input(0, 1).unwrap();
            println!("sent data? {}", sent);
            let states = conn.read_get_world_state().unwrap();
            dbg!(states);
        }
    }
}
