use crate::game::net::server;
use std::collections::{HashMap, HashSet};
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
    sent_header: bool,
}

impl TimedClient {
    pub fn new(client: tcp_stream::ClientStream) -> Self {
        Self {
            stream: client,
            created_at: Instant::now(),
            sent_header: false,
        }
    }

    /// has this client been pending for longer than our timeout
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > PENDING_TIMEOUT
    }

    pub fn try_send_header(&mut self) -> io::Result<()> {
        if self.sent_header {
            return Ok(());
        }
        if self.stream.try_write(lib_spells::SERVER_HEADER)? {
            self.sent_header = true;
        }
        Ok(())
    }
}

pub struct PendingClients {
    pending: HashMap<server::Token, TimedClient>,
    password: Option<String>,
}

impl PendingClients {
    pub fn new(password: Option<String>) -> Self {
        Self {
            password,
            pending: HashMap::default(),
        }
    }

    pub fn add_client(&mut self, token: server::Token, client: tcp_stream::ClientStream) {
        let pending = TimedClient::new(client);
        self.pending.insert(token, pending);
    }

    pub fn remove_client(&mut self, token: server::Token) -> Option<tcp_stream::ClientStream> {
        Some(self.pending.remove(&token)?.stream)
    }

    /// Moves all the expired streams out to the caller.
    pub fn remove_expired(&mut self) -> Vec<tcp_stream::ClientStream> {
        self.pending
            .iter()
            .filter_map(|(t, s)| s.is_expired().then_some(t))
            .copied()
            .collect::<Vec<server::Token>>()
            .iter()
            .map(|t| self.pending.remove(t).unwrap().stream)
            .collect()
    }

    /// Returns a list of failed writes.
    pub fn try_writes(&mut self) -> Vec<(server::Token, std::io::Error)> {
        self.pending
            .iter_mut()
            .filter_map(|(token, client)| {
                let res = client.try_send_header();
                res.is_err().then(|| (*token, res.unwrap_err()))
            })
            .collect()
    }

    /// Returns `Ok(true)` if a correct password was given. Returns `Ok(false)` if nothing was
    /// provided and the caller should wait. Bad passwords are errors. Returns `Ok(true)`
    /// immediately with no read if no password is configured.
    pub fn try_validate(
        &mut self,
        token: server::Token,
    ) -> Result<bool, ClientValidationError> {
        let client = self.pending.get_mut(&token).unwrap();
        if let Some(password) = &self.password {
            match client.stream.try_read_messages() {
                Ok(messages) if messages.is_empty() => Ok(false),
                Ok(messages) => {
                    if password.as_bytes() == messages[0] {
                        Ok(true)
                    } else {
                        Err(ClientValidationError::BadPassword)
                    }
                }
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(true)
        }
    }

    pub fn has_client(&self, token: server::Token) -> bool {
        self.pending.contains_key(&token)
    }
}
