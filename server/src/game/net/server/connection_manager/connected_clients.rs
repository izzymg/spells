use crate::game::net::server;
use lib_spells::{message_stream, net::packet};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

#[derive(Debug)]
pub enum ClientError {
    StreamError(message_stream::MessageStreamError),
    PacketError(packet::InvalidPacketError),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreamError(err) => {
                write!(f, "stream error: {}", err)
            }
            Self::PacketError(err) => {
                write!(f, "packet error: {}", err)
            }
        }
    }
}

impl From<message_stream::MessageStreamError> for ClientError {
    fn from(value: message_stream::MessageStreamError) -> Self {
        Self::StreamError(value)
    }
}

impl From<packet::InvalidPacketError> for ClientError {
    fn from(value: packet::InvalidPacketError) -> Self {
        Self::PacketError(value)
    }
}

pub type Result<T> = std::result::Result<T, ClientError>;

struct ConnectedClient<T: std::io::Read + std::io::Write> {
    info_sent: bool,
    stream: message_stream::MessageStream<T>,
    client_info: Option<lib_spells::net::ClientInfo>,
}

pub struct ConnectedClients<T: std::io::Read + std::io::Write> {
    map: HashMap<server::Token, ConnectedClient<T>>,
    // clients that are OK to send broadcast data to
    send_targets: HashSet<server::Token>,
}

impl<T: std::io::Read + std::io::Write> ConnectedClients<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            send_targets: HashSet::default(),
        }
    }

    /// Set the `ClientInfo` for the given client. Noop if the token isn't a connected client.
    pub fn set_client_info(
        &mut self,
        token: server::Token,
        client_info: lib_spells::net::ClientInfo,
    ) {
        if let Some(client) = self.map.get_mut(&token) {
            client.client_info = Some(client_info);
        }
    }

    /// Tries to write info to any clients that need info and we have updated info state for
    pub fn try_write_client_info(
        &mut self,
    ) -> Vec<(server::Token, message_stream::MessageStreamError)> {
        let mut errors = vec![];
        for (token, client) in self.map.iter_mut() {
            if self.send_targets.contains(token) {
                continue;
            }

            let info = match client.client_info {
                Some(ci) => ci,
                None => continue,
            };

            let serialized_client_info = lib_spells::net::serialize(&info).unwrap();
            match client.stream.try_write_prefixed(&serialized_client_info) {
                Ok(did_send) if did_send => {
                    self.send_targets.insert(*token);
                }
                Err(err) => errors.push((*token, err)),
                _ => {}
            };
        }
        errors
    }

    pub fn add_client(&mut self, token: server::Token, stream: message_stream::MessageStream<T>) {
        self.map.insert(
            token,
            ConnectedClient {
                stream,
                client_info: None,
                info_sent: false,
            },
        );
    }

    pub fn remove_client(
        &mut self,
        token: server::Token,
    ) -> Option<message_stream::MessageStream<T>> {
        if let Some(stream) = self.map.remove(&token) {
            self.send_targets.remove(&token);
            Some(stream.stream)
        } else {
            None
        }
    }

    pub fn has_client(&self, token: server::Token) -> bool {
        self.map.contains_key(&token)
    }

    pub fn send_state(
        &mut self,
        token: server::Token,
        seq: u8,
        state: lib_spells::net::WorldState,
    ) -> Result<()> {
        if !self.send_targets.contains(&token) {
            return Ok(());
        }

        let target = self.map.get_mut(&token).unwrap();
        let serialized_state = lib_spells::net::serialize(&state).unwrap();
        let data = [&[seq], &serialized_state[..]].concat();
        target.stream.try_write_prefixed(&data)?;
        Ok(())
    }

    pub fn try_receive(&mut self, token: server::Token) -> Result<Vec<packet::Packet>> {
        let mut packets = vec![];
        let client = self.map.get_mut(&token).unwrap();
        for message in client.stream.try_read_messages()? {
            // ping -> pong
            if message_is_ping(&message) {
                let _ = client.stream.try_write_prefixed(&[0])?;
                continue;
            }
            let packet = packet::Packet::deserialize(&message)?;
            packets.push(packet);
        }

        Ok(packets)
    }
}

fn message_is_ping(message: &[u8]) -> bool {
    message.len() == 1 && message[0] == 0
}
