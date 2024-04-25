use super::tcp_stream;
use crate::game::net::{packet, server};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

struct ConnectedClient {
    stream: tcp_stream::ClientStream,
    stamp: Option<u8>,
}

#[derive(Default)]
pub struct ConnectedClients {
    map: HashMap<server::Token, ConnectedClient>,
    needs_info: HashSet<server::Token>,
    current_client_info: server::ActiveClientInfo,
}

impl ConnectedClients {
    pub fn set_current_client_info(&mut self, info: server::ActiveClientInfo) {
        self.current_client_info = info;
    }

    /// Tries to write info to any clients that need info and we have updated info state for
    pub fn try_write_client_info(&mut self) -> Vec<(server::Token, std::io::Error)> {
        let mut errors = vec![];
        self.needs_info.retain(|token| {
            if let Some(info) = self.current_client_info.0.get(token) {
                let conn_client = self.map.get_mut(token).unwrap();
                match conn_client
                    .stream
                    .try_write_prefixed(&info.serialize().unwrap())
                {
                    Ok(is_done) => !is_done, // retain the client if we're not done writing
                    Err(err) => {
                        errors.push((*token, err));
                        false
                    }
                }
            } else {
                // keep them to try again
                true
            }
        });
        errors
    }

    pub fn add_client(&mut self, token: server::Token, stream: tcp_stream::ClientStream) {
        self.map.insert(
            token,
            ConnectedClient {
                stream,
                stamp: None,
            },
        );
        self.needs_info.insert(token);
    }

    pub fn remove_client(&mut self, token: server::Token) -> Option<tcp_stream::ClientStream> {
        if let Some(stream) = self.map.remove(&token) {
            self.needs_info.remove(&token);
            Some(stream.stream)
        } else {
            None
        }
    }

    pub fn get_all(&self) -> Vec<server::Token> {
        self.map.keys().copied().collect()
    }

    pub fn has_client(&self, token: server::Token) -> bool {
        self.map.contains_key(&token)
    }

    /// Returns a list of failed writes
    pub fn broadcast(
        &mut self,
        _clients: &[server::Token],
        data: &[u8],
    ) -> Vec<(server::Token, std::io::Error)> {
        self.map
            .iter_mut()
            .filter_map(|(token, client)| {
                let res = client.stream.try_write_prefixed(data);
                res.is_err().then(|| (*token, res.unwrap_err()))
            })
            .collect()
    }

    pub fn try_receive(
        &mut self,
        token: server::Token,
    ) -> Result<Vec<packet::Packet>, packet::InvalidPacketError> {
        let mut packets = vec![];
        let client = self.map.get_mut(&token).unwrap();
        for message in client.stream.try_read_messages()? {
            let inc_packet: packet::IncomingPacket = (&message[..]).try_into()?;
            client.stamp = Some(inc_packet.stamp);
            packets.push(packet::Packet::from_incoming(token, inc_packet)?);
        }

        Ok(packets)
    }
}
