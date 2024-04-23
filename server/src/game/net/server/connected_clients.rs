use super::{tcp_stream, Token};
use crate::game::net::packet;
use std::collections::HashMap;

#[derive(Default)]
pub struct ConnectedClients {
    map: HashMap<Token, tcp_stream::ClientStream>,
    stamps: HashMap<Token, u8>,
}

impl ConnectedClients {
    pub fn add_client(&mut self, token: Token, stream: tcp_stream::ClientStream) {
        self.map.insert(token, stream);
    }

    pub fn remove_client(&mut self, token: Token) -> Option<tcp_stream::ClientStream> {
        self.map.remove(&token)
    }

    pub fn get_all(&self) -> Vec<Token> {
        self.map.keys().copied().collect()
    }

    pub fn has_client(&self, token: Token) -> bool {
        self.map.contains_key(&token)
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

    pub fn try_receive(
        &mut self,
        token: Token,
    ) -> Result<Vec<packet::Packet>, packet::InvalidPacketError> {
        let mut packets = vec![];
        let client = self.map.get_mut(&token).unwrap();
        for message in client.try_read_messages()? {
            let inc_packet: packet::IncomingPacket = (&message[..]).try_into()?;
            self.stamps.insert(token, inc_packet.stamp);
            packets.push(packet::Packet::from_incoming(token, inc_packet)?);
        }

        Ok(packets)
    }
}
