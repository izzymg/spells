use lib_spells::message_stream;
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

const MAX_MESSAGE_SIZE: u16 = u16::MAX;

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
    fn from(_value: lib_spells::net::SerializationError) -> Self {
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
pub struct Connection {
    stream: message_stream::MessageStream<std::net::TcpStream>,
    stamp: u8,
    last_ping: Option<Instant>,
    pub last_ping_rtt: Option<Duration>,
}

impl Connection {
    pub fn new(stream: message_stream::MessageStream<std::net::TcpStream>) -> Self {
        Self {
            stream,
            stamp: 0,
            last_ping: None,
            last_ping_rtt: None,
        }
    }

    /// Handle incoming messages from the world
    pub fn read(&mut self) -> Result<Vec<lib_spells::net::WorldState>> {
        let messages = self.stream.try_read_messages()?;

        messages
            .iter()
            .filter(|m| message_is_ping(m))
            .for_each(|_| {
                if let Some(last_ping) = self.last_ping {
                    self.last_ping_rtt = Some(Instant::now().duration_since(last_ping));
                    self.last_ping = None;
                }
            });

        Ok(messages
            .iter()
            .filter(|m| !message_is_ping(m))
            .map(|m| lib_spells::net::WorldState::deserialize(m))
            .collect::<std::result::Result<
                Vec<lib_spells::net::WorldState>,
                lib_spells::net::SerializationError,
            >>()?)
    }

    pub fn ping(&mut self) -> Result<bool> {
        if self.stream.try_write_prefixed(&[0])? {
            self.last_ping = Some(Instant::now());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if the input was actually sent
    pub fn send_command(&mut self, command: u8, data: u8) -> Result<bool> {
        let sent = self
            .stream
            .try_write_prefixed(&[command, self.stamp, data])?;
        if sent {
            self.stamp = self.stamp.checked_add(1).unwrap_or(0);
        }
        println!("sent command: {}, {}, {}", sent, command, data);
        Ok(sent)
    }
}

pub fn get_connection(
    addr: &str,
    password: Option<&str>,
) -> Result<(Connection, lib_spells::net::ClientInfo)> {
    let raw_stream = std::net::TcpStream::connect(addr)?;
    raw_stream.set_nonblocking(true)?;
    raw_stream.set_nodelay(true)?;

    let mut message_stream =
        message_stream::MessageStream::create(raw_stream, MAX_MESSAGE_SIZE.into())?;
    let mut messages = vec![];
    let mut wrote_pass = false;

    loop {
        if !wrote_pass {
            if let Some(password) = password {
                wrote_pass = write_data(&mut message_stream, password.as_bytes())?;
            }
        }
        read_messages(&mut message_stream, &mut messages)?;
        if let Some(client_info) = validate_server_messages(&messages)? {
            return Ok((Connection::new(message_stream), client_info));
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
        Err(_) => {
            dbg!(messages);
            return Err(ConnectionError::BadData);
        }
    };

    Ok(Some(client_info))
}

fn read_messages(
    stream: &mut message_stream::MessageStream<std::net::TcpStream>,
    messages: &mut Vec<Vec<u8>>,
) -> Result<()> {
    let mut received = stream.try_read_messages()?;
    messages.append(&mut received);
    Ok(())
}

fn write_data(
    stream: &mut message_stream::MessageStream<std::net::TcpStream>,
    data: &[u8],
) -> Result<bool> {
    Ok(stream.try_write_prefixed(data)?)
}

fn message_is_ping(message: &[u8]) -> bool {
    message.len() == 1 && message[0] == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_client_stream() {
        let init_time = Instant::now();
        let (mut conn, client_info) = get_connection("0.0.0.0:7776", Some("cat")).unwrap();
        dbg!(
            client_info,
            Instant::now().duration_since(init_time).as_millis()
        );
        conn.send_command(0, 1).unwrap();
        println!("!!!!!");
        let sent_first_cmd_time = Instant::now();
        loop {
            let state = conn.read().unwrap();
            if state.is_empty() {
                continue;
            }
            dbg!(state);
            println!(
                "elapsed since command: {}ms",
                Instant::now()
                    .duration_since(sent_first_cmd_time)
                    .as_millis()
            );
            if let Some(latency) = conn.last_ping_rtt {
                println!("{}ms", latency.as_millis());
            }
        }
    }
}
