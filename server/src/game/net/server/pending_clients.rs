use super::Token;

use std::collections::HashMap;
use std::fmt::Display;
use std::time::{Duration, Instant};
use std::{io::{self, Read}, time};

use super::tcp_stream;

const PENDING_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct PendingClient {
    created_at: time::Instant,
    client: tcp_stream::ClientStream,
}

impl PendingClient {
    pub fn new(client: tcp_stream::ClientStream) -> Self {
        Self {
            client,
            created_at: Instant::now(),
        }
    }

    /// has this client been pending for longer than our timeout
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > PENDING_TIMEOUT
    }

    /// read the client stream and return if the response
    pub fn validate(&mut self) -> Result<(), ClientValidationError> {
        let mut buf = [0_u8; lib_spells::CLIENT_EXPECT.as_bytes().len()];
        self.client.read_exact(&mut buf)?;
        if lib_spells::CLIENT_EXPECT.as_bytes() != buf {
            dbg!(String::from_utf8(buf.to_vec()).unwrap(), String::from_utf8(lib_spells::CLIENT_EXPECT.as_bytes().to_vec()).unwrap());
            return Err(ClientValidationError::ErrInvalidHeader);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct PendingClients {
    map: HashMap<Token, PendingClient>,
}

impl PendingClients {
    pub fn add_stream(&mut self, token: Token, stream: tcp_stream::ClientStream) {
        let pending = PendingClient::new(stream);
        self.map.insert(token, pending);
    }

    /// Moves all the expired streams out to the caller.
    pub fn remove_expired(&mut self) -> Vec<tcp_stream::ClientStream> {
        self.map
            .iter()
            .filter_map(|(t, s)| s.is_expired().then_some(t))
            .copied()
            .collect::<Vec<Token>>()
            .iter()
            .map(|t| self.map.remove(t).unwrap().client)
            .collect()
    }
    pub fn try_validate(&mut self, token: Token) -> Option<tcp_stream::ClientStream> {
        let mut stream = self.map.remove(&token)?;
        if stream.validate().is_ok() {
            Some(stream.client)
        } else {
            self.map.insert(token, stream);
            None
        }
    }
}

pub enum ClientValidationError {
    IO(io::Error),
    ErrInvalidHeader,
}

impl Display for ClientValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientValidationError::IO(io_err) => {
                write!(f, "IO error: {}", io_err)
            }
            ClientValidationError::ErrInvalidHeader => {
                write!(f, "invalid header response from client")
            }
        }
    }
}

impl From<io::Error> for ClientValidationError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}
