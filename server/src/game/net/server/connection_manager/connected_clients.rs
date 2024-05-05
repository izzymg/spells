use crate::game::net::{packet, server};
use lib_spells::message_stream;
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
    stream: message_stream::MessageStream<T>,
    stamp: Option<u8>,
}

pub struct ConnectedClients<T: std::io::Read + std::io::Write> {
    map: HashMap<server::Token, ConnectedClient<T>>,
    // clients that need to be sent `ClientInfo`
    needs_info: HashSet<server::Token>,
    // clients that are OK to send broadcast data to
    send_targets: HashSet<server::Token>,
    current_client_info: server::ActiveClientInfo,
}

impl<T: std::io::Read + std::io::Write> ConnectedClients<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            needs_info: HashSet::default(),
            send_targets: HashSet::default(),
            current_client_info: server::ActiveClientInfo::default(),
        }
    }

    pub fn set_current_client_info(&mut self, info: server::ActiveClientInfo) {
        self.current_client_info = info;
    }

    /// Tries to write info to any clients that need info and we have updated info state for
    pub fn try_write_client_info(
        &mut self,
    ) -> Vec<(server::Token, message_stream::MessageStreamError)> {
        let mut errors = vec![];
        self.needs_info.retain(|token| {
            if let Some(info) = self.current_client_info.0.get(token) {
                let conn_client = self.map.get_mut(token).unwrap();
                match conn_client
                    .stream
                    .try_write_prefixed(&info.serialize().unwrap())
                {
                    Ok(is_done) => {
                        if is_done {
                            self.send_targets.insert(*token);
                            return false;
                        }
                        true
                    }
                    Err(err) => {
                        errors.push((*token, err));
                        false
                    }
                }
            } else {
                true
            }
        });
        errors
    }

    pub fn add_client(&mut self, token: server::Token, stream: message_stream::MessageStream<T>) {
        self.map.insert(
            token,
            ConnectedClient {
                stream,
                stamp: None,
            },
        );
        self.needs_info.insert(token);
    }

    pub fn remove_client(
        &mut self,
        token: server::Token,
    ) -> Option<message_stream::MessageStream<T>> {
        if let Some(stream) = self.map.remove(&token) {
            self.needs_info.remove(&token);
            self.send_targets.remove(&token);
            Some(stream.stream)
        } else {
            None
        }
    }

    pub fn get_send_targets(&self) -> Vec<server::Token> {
        self.map.keys().copied().collect()
    }

    pub fn has_client(&self, token: server::Token) -> bool {
        self.map.contains_key(&token)
    }

    /// Returns a list of failed writes
    pub fn send_to(
        &mut self,
        _clients: &[server::Token],
        data: &[u8],
    ) -> Vec<(server::Token, message_stream::MessageStreamError)> {
        self.map
            .iter_mut()
            .filter_map(|(token, client)| {
                let res = client.stream.try_write_prefixed(data);
                res.is_err().then(|| (*token, res.unwrap_err()))
            })
            .collect()
    }

    pub fn try_receive(&mut self, token: server::Token) -> Result<Vec<packet::Packet>> {
        let mut packets = vec![];
        let client = self.map.get_mut(&token).unwrap();
        for message in client.stream.try_read_messages()? {
            if message_is_ping(&message) {
                // pong
                let _ = client.stream.try_write_prefixed(&[0]);
                continue;
            }
            let inc_packet: packet::IncomingPacket = (&message[..]).try_into()?;
            client.stamp = Some(inc_packet.stamp);
            packets.push(packet::Packet::from_incoming(token, inc_packet)?);
        }

        Ok(packets)
    }
}
fn message_is_ping(message: &[u8]) -> bool {
    message.len() == 1 && message[0] == 0
}
