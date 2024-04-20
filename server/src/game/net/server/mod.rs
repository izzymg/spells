/*! TCP server implementation for managing connected game clients */

mod pending_clients;
mod tcp_stream;

use mio::net::TcpListener;
pub use mio::Token;
use mio::{Events, Interest, Poll};

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, time};

use bevy::log;

use tcp_stream::ClientStream;

const SERVER_TOKEN: Token = Token(0);
const EVENT_BUFFER_SIZE: usize = 1028;
const MIN_TICK: Duration = Duration::from_millis(250);

// these should be in a passed in config
const SERVER_ADDR: &str = "0.0.0.0:7776";
const MAX_INCOMING_BYTES: usize = 6;
// ^

struct BroadcastError {
    token: Token,
    error: io::Error,
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

pub enum Incoming {
    Joined(Token),
    Left(Token),
    Data(Token, Vec<u8>),
}

pub enum Outgoing {
    Drop(Token),
    Broadcast(Vec<u8>),
}

pub struct Server {
    listener: TcpListener,
    events: Events,
    poll: Poll,

    next_socket: usize,

    inc_tx: mpsc::Sender<Incoming>,
    out_rx: mpsc::Receiver<Outgoing>,
}

impl Server {
    pub fn create(
        inc_tx: mpsc::Sender<Incoming>,
        out_rx: mpsc::Receiver<Outgoing>,
    ) -> io::Result<Server> {
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

            inc_tx,
            out_rx,
        })
    }

    /// block on event look waiting for new clients, adding them by their token to a map of active cleint
    pub fn event_loop(&mut self) {
        let mut connected_clients: HashMap<Token, ClientStream> = HashMap::default();
        let mut pending_clients: pending_clients::PendingClients =
            pending_clients::PendingClients::default();

        loop {
            self.check_outgoing(&mut connected_clients);
            if let Some(mut new_client) = self.check_accept() {
                let token = Token(self.next_socket);
                self.next_socket += 1;
                new_client.register_to_poll(token, &mut self.poll).unwrap();
                pending_clients.add_stream(token, new_client);
            }
            for mut dead_client in pending_clients.kill_expired() {
                dead_client.deregister_from_poll(&mut self.poll).unwrap();
            }

            if let Err(poll_err) = self.poll.poll(&mut self.events, Some(MIN_TICK)) {
                log::warn!("poll error: {}", poll_err);
            }

            let client_events = self.events.iter().filter(|ev| ev.token() != SERVER_TOKEN);
            for ev in client_events {
                let client_token = ev.token();

                if let Some(new_client) = pending_clients.try_validate(client_token) {
                    connected_clients.insert(client_token, new_client);
                    self.inc_tx.send(Incoming::Joined(client_token)).unwrap();
                }
                if let Some(mut connected_client) = connected_clients.remove(&client_token) {
                    if self.receive_connected_data(client_token, &mut connected_client) {
                        connected_clients.insert(client_token, connected_client);
                    }
                }
            }
        }
    }

    // returns true if data was received correctly
    fn receive_connected_data(&self, token: Token, client: &mut ClientStream) -> bool {
        let mut buf = [0_u8; MAX_INCOMING_BYTES];
        let read = match client.read_fill(&mut buf) {
            Ok(read) => read,
            Err(err) => {
                log::warn!("client read failure: {}", err);
                return false; // TODO: do we ever get events here we can't read?
            }
        };
        log::debug!("read {} bytes from {:?}", read, token);
        self.inc_tx
            .send(Incoming::Data(token, buf.to_vec()))
            .unwrap();
        true
    }

    /// receive data from the outside world to interact with clients (non-blocking)
    fn check_outgoing(&mut self, connected: &mut HashMap<Token, ClientStream>) {
        match self.out_rx.try_recv() {
            Ok(outgoing) => match outgoing {
                Outgoing::Broadcast(data) => {
                    self.broadcast(connected, &data);
                }
                Outgoing::Drop(token) => {
                    self.drop_connected_client(token, connected);
                }
            },
            Err(err) if err == mpsc::TryRecvError::Disconnected => {
                panic!("receiver died: {}", err)
            }
            _ => {} // empty, we don't care
        }
    }

    // broadcast on all clients, drop dead ones
    fn broadcast(
        &mut self,
        clients: &mut HashMap<Token, ClientStream>,
        data: &[u8],
    ) -> Vec<BroadcastError> {
        clients
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

    /// check for new clients and accept them into pending
    /// client value gets dropped if we don't successfully pend them
    fn check_accept(&mut self) -> Option<tcp_stream::ClientStream> {
        let (stream, _) = match self.listener.accept() {
            Ok(s) => s,
            Err(_) => return None, // would block, just exit
        };
        let mut client = match ClientStream::new(stream) {
            Ok(client) => client,
            Err(err) => {
                log::info!("failed to create client stream: {}", err);
                return None;
            }
        };
        if let Err(err) = client.write_header() {
            log::info!("failed to send server header: {}", err);
            return None;
        }
        Some(client)
    }

    /// connected removals should go through here to notify send channel properly
    fn drop_connected_client(
        &mut self,
        client: Token,
        connected_clients: &mut HashMap<Token, ClientStream>,
    ) -> Option<ClientStream> {
        self.inc_tx.send(Incoming::Left(client)).unwrap();
        connected_clients.remove(&client)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::mpsc,
        thread,
    };

    use super::Server;

    #[test]
    fn test_incoming_client_recv() {
        let (_keep, rx) = mpsc::channel();
        let (tx, _keep) = mpsc::channel();
        let mut server = Server::create(tx, rx).unwrap();
        // create a client stream
        // create a thread that blocks & fetches our clients
        // assert we grab the server header correctly
        // panic the thread if it doesn't process the client

        let server = thread::spawn(move || {
            server.event_loop();
        });

        let connect = || {
            let mut stream = TcpStream::connect("127.0.0.1:7776").unwrap();
            let mut first_response = [0; lib_spells::SERVER_HEADER.len()];
            stream.read_exact(&mut first_response).unwrap();
            assert_eq!(lib_spells::SERVER_HEADER.as_bytes(), first_response);
            stream
                .write_all(lib_spells::SERVER_HEADER.as_bytes())
                .unwrap();
        };

        connect();
        connect();
        dbg!("exiting");
    }
}
