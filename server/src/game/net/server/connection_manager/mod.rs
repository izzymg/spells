/*! Manages a set of `tcp_stream::ClientStream` connections, providing event handling, kick,
broadcast, etc */
use crate::game::net::server;
use bevy::log;
use std::sync::mpsc;
use lib_spells::tcp_stream;

mod connected_clients;
mod pending_clients;

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
        self.pending.try_send_headers();
        self.connected.try_write_client_info();
        self.connect_validated_pending();
        self.kick_expired();
        self.check_outgoing();
    }

    /// Take ownership of a stream to be managed. Once it's kicked, it'll be available in
    /// `collect_dead`.
    pub fn manage_stream(
        &mut self,
        token: server::Token,
        stream: tcp_stream::ClientStream,
        is_readable: bool,
    ) {
        log::info!("pending: {}", token);
        self.pending.add_client(token, stream);
        self.pending.try_send_headers();
        if is_readable {
            self.read_pending_validation(token)
        }
    }

    /// Try to read from the client at the given token
    pub fn try_read(&mut self, token: server::Token) {
        if self.pending.has_client(token) {
            self.read_pending_validation(token);
        }
        if self.connected.has_client(token) {
            self.read_client_packets(token);
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
                self.connected.send_to(&self.connected.get_send_targets(), &data);
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
        if let Err(err) = self.pending.try_read_password(token) {
            log::info!("validation error {}: {}", token, err);
            self.kick_client(token);
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
                log::info!("read error {}: {}", token, err);
                self.kick_client(token);
            }
        }
    }

    /// Take all validated pending clients and move them to `connected`
    fn connect_validated_pending(&mut self) {
        for (token, client) in self.pending.remove_validated() {
            log::info!("client validated & connected: {} {}", token, client);
            self.connected.add_client(token, client);
            self.inc_tx
                .send(server::Incoming::Joined(token))
                .expect("receiver dead");
        }
    }

    /// Tries to pull `token` out of either `connected` or `pending` and move it into `dead`.
    fn kick_client(&mut self, token: server::Token) {
        log::info!("kick: {}", token);
        if let Some(client) = self.connected.remove_client(token) {
            // make sure we notify about connected clients leaving
            self.inc_tx
                .send(server::Incoming::Left(token))
                .expect("dead receiver");
            self.dead.push(client);
        }

        if let Some(client) = self.pending.remove_client(token) {
            // we don't need to notify above us about non-connected clients
            // they only care about verified people
            self.dead.push(client);
        }
    }

    fn kick_expired(&mut self) {
        for dead in self.pending.get_expired() {
            self.kick_client(dead);
        }
    }
}
