/*! TCP server implementation for managing connected game clients */

mod connected_clients;
pub mod packet;
mod pending_clients;
mod tcp_stream;

use mio::net::TcpListener;
pub use mio::Token;
use mio::{event::Event, net::TcpStream, Events, Interest, Poll, Registry};

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use bevy::log;

const SERVER_TOKEN: Token = Token(0);
const EVENT_BUFFER_SIZE: usize = 1028;
const MIN_TICK: Duration = Duration::from_millis(100);

// these should be in a passed in config
const SERVER_ADDR: &str = "0.0.0.0:7776";
// ^

struct ConnectionManager {
    inc_tx: mpsc::Sender<Incoming>,
    out_rx: mpsc::Receiver<Outgoing>,
    connected: connected_clients::ConnectedClients,
    pending: pending_clients::PendingClients,
    dead: Vec<tcp_stream::ClientStream>,
}

impl ConnectionManager {
    pub fn new(inc_tx: mpsc::Sender<Incoming>, out_rx: mpsc::Receiver<Outgoing>) -> Self {
        Self {
            inc_tx,
            out_rx,
            connected: connected_clients::ConnectedClients::default(),
            pending: pending_clients::PendingClients::default(),
            dead: vec![],
        }
    }

    /// Take ownership of a stream to be managed. Once it's kicked, it'll be available in
    /// `collect_dead`.
    pub fn manage_stream(&mut self, token: Token, stream: tcp_stream::ClientStream) -> io::Result<()> {
        self.pending.add_client(token, stream);
        Ok(())
    }

    /// Figure out who an event is for and handle appropriately. Panics if an event was passed for
    /// a client we don't have.
    pub fn handle_event(&mut self, event: Event) {
        let token = event.token();
        if self.pending.has_client(token) {
        } else if self.connected.has_client(token) {
            // connected
        }
    }

    pub fn collect_dead<F>(&mut self, f: F) where F: FnOnce(tcp_stream::ClientStream) + Copy {
        for stream in self.dead.drain(..) {
            f(stream);
        }
    }

    fn broadcast(&mut self, clients: &[Token], data: &[u8]) {
    }

    fn check_outgoing(&mut self) {
        match self.out_rx.try_recv() {
            Ok(Outgoing::Broadcast(data)) => {
                self.broadcast(&self.connected.get_all(), &data);
            }
            Ok(Outgoing::Kick(token)) => {
                self.kick_client(token);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("receiver disconnected");
            }
            _ => {}
        }
    }

    fn read_pending_validation(&mut self, token: Token) {}

    fn read_client_packets(&mut self, token: Token) {
        match self.connected.try_receive(token) {
            Ok(packets) => {
                for packet in packets {
                    self.inc_tx
                        .send(Incoming::Data(token, packet))
                        .expect("receiver died");
                }
            }
            Err(err) => {
                log::warn!("client io error: {}", err);
                self.kick_client(token);
            }
        }
    }

    /// Moves a client from pending status to connected status
    fn pending_to_connected(&mut self, token: Token) {
        if let Some(client) = self.pending.remove_client(token) {
            self.connected.add_client(token, client);
            self.inc_tx.send(Incoming::Joined(token));
        }
    }

    // we don't need to notify above us about non-connected clients
    // they only care about verified people
    fn kick_client(&mut self, token: Token) {
        if let Some(client) = self.connected.remove_client(token) {
            self.inc_tx
                .send(Incoming::Left(token))
                .expect("dead receiver");
            self.dead.push(client);
        }

        if let Some(client) = self.pending.remove_client(token) {
            self.dead.push(client);
        }
    }

    fn clean_expired_pending(&mut self, registry: &Registry) -> io::Result<()> {
        for mut expired_pending_client in self.pending.remove_expired() {
            expired_pending_client.deregister_from_poll(registry)?;
        }
        Ok(())
    }

    fn try_say_hello(&mut self, token: Token, registry: &Registry) {
        match self.pending.write_header(token) {
            Err(ref io_err) if io_err.kind() == io::ErrorKind::Interrupted => {
                self.try_say_hello(token, registry);
            }
            Err(ref io_err) => {
                log::info!("failed to write header to {}: {}", token.0, io_err);
                if let Some(mut client) = self.pending.remove_client(token) {
                    client.deregister_from_poll(registry).unwrap();
                }
            }
            Ok(_) => {
                log::debug!("wrote client header to {}", token.0);
            }
        }
    }
}

#[derive(Debug)]
pub enum Incoming {
    Joined(Token),
    Left(Token),
    Data(Token, packet::Packet),
}

#[derive(Debug)]
pub enum Outgoing {
    Kick(Token),
    Broadcast(Vec<u8>),
}

pub struct Server {
    listener: TcpListener,
    events: Events,
    poll: Poll,

    next_socket: usize,
}

impl Server {
    pub fn create() -> io::Result<Server> {
        log::info!("binding server to {SERVER_ADDR}");
        let mut listener = TcpListener::bind(SERVER_ADDR.parse().unwrap())?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;
        let events = Events::with_capacity(EVENT_BUFFER_SIZE);
        Ok(Server {
            listener,
            poll,
            events,
            next_socket: 1,
        })
    }

    /// block on event look waiting for new clients, adding them by their token to a map of active cleint
    pub fn event_loop(&mut self, inc_tx: mpsc::Sender<Incoming>, out_rx: mpsc::Receiver<Outgoing>) {
        let manager = ConnectionManager::new(inc_tx, out_rx);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        sync::mpsc,
        thread,
    };

    use super::*;

    #[test]
    fn test_incoming_client_recv() {
        let (_keep, rx) = mpsc::channel();
        let (tx, _keep) = mpsc::channel();
        let mut server = Server::create().unwrap();
        // create a client stream
        // create a thread that blocks & fetches our clients
        // assert we grab the server header correctly
        // panic the thread if it doesn't process the client

        thread::spawn(move || {
            server.event_loop(tx, rx);
        });

        let connect = || {
            // use a std stream so it blocks
            let mut stream = std::net::TcpStream::connect(SERVER_ADDR).unwrap();
            let mut first_response = [0; lib_spells::SERVER_HEADER.len()];
            stream.read_exact(&mut first_response).unwrap();
            assert_eq!(lib_spells::SERVER_HEADER.as_bytes(), first_response);
        };

        connect();
        connect();
    }

}
