use super::{tcp_stream, Token};
use crate::game::net::packet;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;

pub struct BroadcastError {
    pub token: Token,
    pub error: io::Error,
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
    stamps: HashMap<Token, u8>,
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

    /// returns `None` if `token` isn't a connected client
    pub fn try_receive(
        &mut self,
        token: Token,
    ) -> Result<Option<packet::Packet>, packet::InvalidPacketError> {
        let client = match self.map.get_mut(&token) {
            Some(client) => client,
            None => {
                return Ok(None);
            }
        };

        let size = packet::read_packet_header(client)?;
        let mut buf = vec![0_u8; size];
        let contents = packet::read_packet_contents(client, &mut buf)?;
        let inc_packet: packet::IncomingPacket = contents.try_into()?;
        // store the most recently read packet stamp so we can re-transmit to the client later
        self.stamps.insert(token, inc_packet.stamp);
        let packet = packet::Packet::from_incoming(token, inc_packet)?;
        Ok(Some(packet))
    }
}
