/*! TCP server implementation for managing connected game clients */

mod connected_clients;
pub mod packet;
mod pending_clients;
mod tcp_stream;

use mio::net::TcpListener;
pub use mio::Token;
use mio::{net::TcpStream, Events, Interest, Poll, Registry};

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
}

impl ConnectionManager {
    pub fn new(inc_tx: mpsc::Sender<Incoming>, out_rx: mpsc::Receiver<Outgoing>) -> Self {
        Self {
            inc_tx,
            out_rx,
            connected: connected_clients::ConnectedClients::default(),
            pending: pending_clients::PendingClients::default(),
        }
    }

    fn broadcast(&mut self, clients: &[Token], data: &[u8], registry: &Registry) {
        let errors = self.connected.broadcast(&self.connected.get_all(), &data);
        let interrupted: Vec<Token> = errors
            .iter()
            .filter(|(_, err)| err.kind() == io::ErrorKind::Interrupted)
            .map(|(t, _)| t)
            .copied().collect();
        self.broadcast(&interrupted, data, registry);
        for (token, err) in errors.iter().filter(|(_, err)| (err.kind() != io::ErrorKind::Interrupted) && (err.kind() != io::ErrorKind::WouldBlock)) {
            log::info!("broadcast failure to client {}: {}", token.0, err);
            self.kick_client(registry, *token);
        }
    }

    fn check_outgoing(&mut self, registry: &Registry) {
        match self.out_rx.try_recv() {
            Ok(Outgoing::Broadcast(data)) => {
                self.broadcast(&self.connected.get_all(), &data, registry);
            }
            Ok(Outgoing::Kick(token)) => {
                self.kick_client(registry, token);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("receiver disconnected");
            }
            _ => {}
        }
    }

    /// Returns true if the operation is finished, false if it should be retried
    fn read_pending_validation(&mut self, registry: &Registry, token: Token) -> bool {
        match self.pending.try_validate(token) {
            Ok(None) => true,
            Ok(Some(mut client)) => {
                self.connected.add(token, client);
                self.inc_tx.send(Incoming::Joined(token)).unwrap();
                true
            }

            Err(pending_clients::ClientValidationError::ErrInvalidHeader) => {
                log::info!("bad header from client {}", token.0);
                if let Some(mut stream) = self.pending.remove_client(token) {
                    stream.deregister_from_poll(registry);
                }
                true
            }

            Err(pending_clients::ClientValidationError::IO(ref io_err))
                if io_err.kind() == io::ErrorKind::WouldBlock =>
            {
                true
            }

            Err(pending_clients::ClientValidationError::IO(ref io_err))
                if io_err.kind() == io::ErrorKind::Interrupted =>
            {
                false
            }

            Err(pending_clients::ClientValidationError::IO(io_err)) => {
                log::warn!("pending client io error: {}", io_err);
                if let Some(mut stream) = self.pending.remove_client(token) {
                    stream.deregister_from_poll(registry);
                }
                true
            }
        }
    }

    /// Returns true if the operation is finished, false if it should be retried
    fn read_client_packets(&mut self, registry: &Registry, token: Token) -> bool {
        match self.connected.try_receive(token) {
            Ok(Some(packet)) => {
                self.inc_tx
                    .send(Incoming::Data(token, packet))
                    .expect("receiver died");
                true
            }
            Ok(None) => true,
            Err(packet::InvalidPacketError::IoError(ref io_err))
                if io_err.kind() == io::ErrorKind::WouldBlock =>
            {
                true
            }
            Err(packet::InvalidPacketError::IoError(ref io_err))
                if io_err.kind() == io::ErrorKind::Interrupted =>
            {
                false
            }
            Err(packet::InvalidPacketError::IoError(io_err)) => {
                log::warn!("client io error: {}", io_err);
                self.kick_client(registry, token);
                true
            }
            Err(err) => {
                log::warn!("invalid packet from client {}: {}", token.0, err);
                self.kick_client(registry, token);
                true
            }
        }
    }

    fn add_client_to_connected(&mut self, token: Token, client: tcp_stream::ClientStream) {
        self.connected.add(token, client);
        self.inc_tx.send(Incoming::Joined(token));
    }

    fn kick_client(&mut self, registry: &Registry, token: Token) {
        if let Some(mut client) = self.connected.remove(token) {
            client.deregister_from_poll(registry).unwrap();
            self.inc_tx
                .send(Incoming::Left(token))
                .expect("dead receiver");
        }
    }

    fn begin_pending(
        &mut self,
        registry: &Registry,
        token: Token,
        stream: TcpStream,
    ) -> io::Result<()> {
        let mut client = tcp_stream::ClientStream::new(stream);
        client.register_to_poll(token, registry)?;
        self.pending.add_client(token, client);
        Ok(())
    }

    fn clean_expired_pending(&mut self, registry: &Registry) -> io::Result<()> {
        for mut expired_pending_client in self.pending.remove_expired() {
            expired_pending_client.deregister_from_poll(registry)?;
        }
        Ok(())
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

    #[test]
    #[ignore]
    fn test_tcp_things() {
        let (inc_tx, inc_rx) = mpsc::channel();
        let (_out_tx, out_rx) = mpsc::channel();
        let mut server = Server::create().unwrap();
        let handle = std::thread::spawn(move || {
            server.event_loop(inc_tx, out_rx);
        });

        let mut stream = TcpStream::connect(SERVER_ADDR.parse().unwrap()).unwrap();
        stream.set_nodelay(true).unwrap();
        stream
            .write_all(lib_spells::CLIENT_EXPECT.as_bytes())
            .unwrap();

        std::thread::spawn(move || loop {
            dbg!(inc_rx.recv().unwrap());
        });

        let sleep = Duration::from_millis(1000);
        println!("waiting {}s...", sleep.as_secs_f32());
        thread::sleep(sleep);
        stream.write_all("abcdefg".as_bytes()).unwrap();
        println!("write");
        stream.write_all("h".as_bytes()).unwrap();
        println!("write");
        handle.join().unwrap();
    }
}
