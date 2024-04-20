use std::collections::HashMap;
use std::fmt::Display;
use std::io;

use super::{Token, tcp_stream};

use bevy::log;

pub struct BroadcastError {
    token: Token,
    error: io::Error,
}

impl Display for BroadcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "broadcast failure on client ({}): {}",
            self.token.0, self.error
        )
    }
}
#[derive(Default)]
pub struct ConnectedClients {
    map: HashMap<Token, tcp_stream::ClientStream>,
}

impl ConnectedClients {
    pub fn add(&mut self, token: Token, stream: tcp_stream::ClientStream) {
        self.map.insert(token, stream);
    }

    pub fn remove(&mut self, token: Token) -> Option<tcp_stream::ClientStream> {
        self.map.remove(&token)
    }

    pub fn broadcast(&mut self, data: &[u8]) -> Vec<BroadcastError> {
        self.map
            .iter_mut()
            .map(|(token, client)| {
                client.write_prefixed(data).map_err(|error| BroadcastError {
                    error,
                    token: *token,
                })
            })
            .filter_map(|v| v.is_err().then(|| v.unwrap_err()))
            .collect()
    }

    pub fn try_receive(&mut self, token: Token, buf: &mut [u8]) -> io::Result<usize> {
        let client = match self.map.get_mut(&token) {
            Some(client) => client,
            None => {
                return Ok(0);
            }
        };
        let read = match client.read_fill(buf) {
            Ok(read) => read,
            Err(err) => {
                log::warn!("client read failure: {}", err);
                self.map.remove(&token);
                return Err(err); 
            }
        };
        log::debug!("read {} bytes from {:?}", read, token);
        Ok(read)
    }
}
