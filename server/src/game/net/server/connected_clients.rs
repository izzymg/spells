use super::{tcp_stream, Token};
use crate::game::net::packet;
use std::collections::HashMap;

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

    pub fn get_all(&self) -> Vec<Token> {
        self.map.keys().copied().collect()
    }

    /// returns all errors for associated client tokens
    pub fn broadcast(&mut self, clients: &[Token], data: &[u8]) -> Vec<(Token, std::io::Error)> {
        self.map
            .iter_mut()
            .filter_map(|(token, client)| {
                let res = client.write_prefixed(data);
                res.is_err().then(|| (*token, res.unwrap_err()))
            })
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
