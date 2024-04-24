/*! Manages a set of `tcp_stream::ClientStream` connections, providing event handling, kick,
broadcast, etc */
use crate::game::net::server;
use bevy::log;
use mio::event::Event;
use std::sync::mpsc;

mod connected_clients;
mod pending_clients;
pub mod tcp_stream;

pub struct ConnectionManager {
    inc_tx: mpsc::Sender<server::Incoming>,
    out_rx: mpsc::Receiver<server::Outgoing>,
    connected: connected_clients::ConnectedClients,
    pending: pending_clients::PendingClients,
    dead: Vec<tcp_stream::ClientStream>,
}

impl ConnectionManager {
    pub fn new(
        inc_tx: mpsc::Sender<server::Incoming>,
        out_rx: mpsc::Receiver<server::Outgoing>,
        password: Option<String>,
    ) -> Self {
        Self {
            inc_tx,
            out_rx,
            connected: connected_clients::ConnectedClients::default(),
            pending: pending_clients::PendingClients::new(password),
            dead: vec![],
        }
    }

    /// Check channels & internals, clean up dead stuff
    pub fn tick(&mut self) {
        self.check_outgoing();
        self.mark_expired_as_dead();
        self.pending.try_writes();
        self.connected.try_write_client_info();
    }

    /// Take ownership of a stream to be managed. Once it's kicked, it'll be available in
    /// `collect_dead`.
    pub fn manage_stream(&mut self, token: server::Token, stream: tcp_stream::ClientStream) {
        self.pending.add_client(token, stream);
    }

    /// Figure out who an event is for and handle appropriately. Panics if an event was passed for
    /// a client we don't have.
    pub fn handle_event(&mut self, event: &Event) {
        let token = event.token();

        if self.pending.has_client(token) {
            if event.is_readable() {
                self.read_pending_validation(token);
            }
            if event.is_writable() {}
        } else if self.connected.has_client(token) {
            if event.is_readable() {
                self.read_client_packets(token);
            }
        } else {
            panic!("mismanaged client");
        }
    }

    pub fn collect_dead<F>(&mut self, f: F)
    where
        F: FnOnce(tcp_stream::ClientStream) + Copy,
    {
        for stream in self.dead.drain(..) {
            f(stream);
        }
    }

    fn check_outgoing(&mut self) {
        match self.out_rx.try_recv() {
            Ok(server::Outgoing::Broadcast(data)) => {
                self.connected.broadcast(&self.connected.get_all(), &data);
            }
            Ok(server::Outgoing::Kick(token)) => {
                self.kick_client(token);
            }
            Ok(server::Outgoing::ClientInfo(info)) => {
                self.connected.set_current_client_info(info);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("receiver disconnected");
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }

    fn read_pending_validation(&mut self, token: server::Token) {
        match self.pending.try_validate(token) {
            Ok(did_validate) => {
                if did_validate {
                    self.pending_to_connected(token);
                }
            }
            Err(err) => {
                log::warn!("validation error: {}", err);
                self.kick_client(token);
            }
        }
    }

    fn read_client_packets(&mut self, token: server::Token) {
        match self.connected.try_receive(token) {
            Ok(packets) => {
                for packet in packets {
                    self.inc_tx
                        .send(server::Incoming::Data(token, packet))
                        .expect("receiver died");
                }
            }
            Err(err) => {
                log::warn!("read error: {}", err);
                self.kick_client(token);
            }
        }
    }

    /// Moves a client from pending status to connected status
    fn pending_to_connected(&mut self, token: server::Token) {
        if let Some(client) = self.pending.remove_client(token) {
            self.connected.add_client(token, client);
            self.inc_tx
                .send(server::Incoming::Joined(token))
                .expect("receiver dead");
        }
    }

    // we don't need to notify above us about non-connected clients
    // they only care about verified people
    fn kick_client(&mut self, token: server::Token) {
        if let Some(client) = self.connected.remove_client(token) {
            self.inc_tx
                .send(server::Incoming::Left(token))
                .expect("dead receiver");
            self.dead.push(client);
        }

        if let Some(client) = self.pending.remove_client(token) {
            self.dead.push(client);
        }
    }

    fn mark_expired_as_dead(&mut self) {
        for dead in self.pending.remove_expired() {
            self.dead.push(dead);
        }
    }
}
