use crate::game::net::server;
use std::collections::HashMap;
use std::fmt::Display;
use std::time::{Duration, Instant};
use std::{io, time};

use super::tcp_stream;

const PENDING_TIMEOUT: Duration = Duration::from_millis(1000);

pub enum ClientValidationError {
    IO(io::Error),
    BadPassword,
}

impl Display for ClientValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientValidationError::IO(io_err) => {
                write!(f, "IO error: {}", io_err)
            }
            ClientValidationError::BadPassword => {
                write!(f, "wrong password")
            }
        }
    }
}

impl From<io::Error> for ClientValidationError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

#[derive(Debug)]
struct TimedClient {
    created_at: time::Instant,
    stream: tcp_stream::ClientStream,
}

impl TimedClient {
    pub fn new(client: tcp_stream::ClientStream) -> Self {
        Self {
            stream: client,
            created_at: Instant::now(),
        }
    }

    /// has this client been pending for longer than our timeout
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > PENDING_TIMEOUT
    }
}

#[derive(Default)]
pub struct PendingClients {
    map: HashMap<server::Token, TimedClient>,
    password: String,
}

impl PendingClients {
    pub fn add_client(&mut self, token: server::Token, client: tcp_stream::ClientStream) {
        let pending = TimedClient::new(client);
        self.map.insert(token, pending);
    }

    pub fn remove_client(&mut self, token: server::Token) -> Option<tcp_stream::ClientStream> {
        Some(self.map.remove(&token)?.stream)
    }

    /// Moves all the expired streams out to the caller.
    pub fn remove_expired(&mut self) -> Vec<tcp_stream::ClientStream> {
        self.map
            .iter()
            .filter_map(|(t, s)| s.is_expired().then_some(t))
            .copied()
            .collect::<Vec<server::Token>>()
            .iter()
            .map(|t| self.map.remove(t).unwrap().stream)
            .collect()
    }
   
    /// Returns `Ok(true)` if a correct password was given. Returns Ok(`false`) if nothing was
    /// provided and the caller should wait. Bad passwords are errors.
    pub fn try_read_password(&mut self, token: server::Token) -> Result<bool, ClientValidationError> {
        let client = self.map.get_mut(&token).unwrap();
        match client.stream.try_read_messages() {
            Ok(messages) if messages.is_empty() => {
                Ok(false) 
            },
            Ok(messages) => {
                if self.password.as_bytes() == messages[0] {
                    Ok(true)
                } else {
                    Err(ClientValidationError::BadPassword)
                }
            },
            Err(err) => Err(err.into())
        }
    }

    pub fn has_client(&self, token: server::Token) -> bool {
        self.map.contains_key(&token)
    }

    pub fn write_header(&mut self, token: server::Token) -> io::Result<()> {
        self.map.get_mut(&token).unwrap().stream.write_header()
    }
}

