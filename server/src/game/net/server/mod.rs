/*! TCP server implementation for managing connected game clients */

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
const PENDING_TIMEOUT: Duration = Duration::from_millis(1000);
const MAX_INCOMING_BYTES: usize = 6;
// ^

#[derive(Debug)]
pub enum ClientValidationError {
    IO(io::Error),
    ErrInvalidHeader,
}

impl Display for ClientValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientValidationError::IO(io_err) => {
                write!(f, "IO error: {}", io_err)
            }
            ClientValidationError::ErrInvalidHeader => {
                write!(f, "invalid header response from client")
            }
        }
    }
}

impl From<io::Error> for ClientValidationError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

#[derive(Debug)]
struct PendingClient {
    created_at: time::Instant,
    client: ClientStream,
}

impl PendingClient {
    pub fn new(client: ClientStream) -> Self {
        Self {
            client,
            created_at: Instant::now(),
        }
    }

    /// has this client been pending for longer than our timeout
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > PENDING_TIMEOUT
    }

    /// read the client stream and return if the response
    pub fn validate(&mut self) -> Result<(), ClientValidationError> {
        let mut buf = [0_u8; lib_spells::CLIENT_EXPECT.as_bytes().len()];
        self.client.read_fill(&mut buf)?;
        if lib_spells::CLIENT_EXPECT.as_bytes() != buf {
            return Err(ClientValidationError::ErrInvalidHeader);
        }
        Ok(())
    }
}

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
        let mut pending_clients: HashMap<Token, PendingClient> = HashMap::default();

        loop {
            self.check_accept(&mut pending_clients);
            self.check_outgoing(&mut connected_clients);
            self.drop_expired(&mut pending_clients);

            if let Err(poll_err) = self.poll.poll(&mut self.events, Some(MIN_TICK)) {
                log::warn!("poll error: {}", poll_err);
            }

            let client_events = self.events.iter().filter(|ev| ev.token() != SERVER_TOKEN);
            for ev in client_events {
                let client_token = ev.token();

                if let Some(mut pending_client) = pending_clients.remove(&client_token) {
                    if self.receive_pending_data(client_token, &mut pending_client) {
                        pending_clients.insert(client_token, pending_client);
                    }
                } else if let Some(mut connected_client) = connected_clients.remove(&client_token) {
                    if self.receive_connected_data(client_token, &mut connected_client) {
                        connected_clients.insert(client_token, connected_client);
                    }
                } else {
                    unreachable!("event from untracked client");
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

    // returns true if the client validated correctly
    fn receive_pending_data(
        &self,
        client_token: Token,
        pending_client: &mut PendingClient,
    ) -> bool {
        if let Ok(()) = pending_client.validate() {
            log::info!("valid client: {:?}", client_token);
            self.inc_tx.send(Incoming::Joined(client_token)).unwrap();
            return true;
        }
        false
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
            },
            _ => { }, // empty, we don't care
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
                client
                    .write_prefixed(data)
                    .map_err(|error| BroadcastError {
                        error,
                        token: *token,
                    })
            })
            .filter_map(|v| v.is_err().then(|| v.unwrap_err()))
            .collect()
    }

    /// remove pending clients that haven't validated within the timeframe
    fn drop_expired(&mut self, clients: &mut HashMap<Token, PendingClient>) {
        clients
            .iter()
            .filter_map(|(k, v)| v.is_expired().then_some(k))
            .copied()
            .collect::<Vec<Token>>()
            .iter()
            .for_each(|k| {
                clients
                    .remove(k)
                    .unwrap()
                    .client
                    .deregister_from_poll(&mut self.poll)
                    .unwrap();
            });
    }

    /// check for new clients and accept them into pending
    /// client value gets dropped if we don't successfully pend them
    fn check_accept(&mut self, pending_clients: &mut HashMap<Token, PendingClient>) {
        let (stream, addr) = match self.listener.accept() {
            Ok(s) => s,
            Err(_) => return, // would block, just exit
        };
        let mut client = match ClientStream::new(stream) {
            Ok(client) => client,
            Err(err) => {
                log::info!("failed to create client stream: {}", err);
                return;
            }
        };
        if let Err(err) = client.write_header() {
            log::info!("failed to send server header: {}", err);
            return;
        }
        match self.pend_new_client(client, pending_clients) {
            Ok(()) => {
                log::info!("pending client: {}", addr);
            }
            Err(err) => {
                log::warn!("failed to pend client: {}", err);
            }
        }
    }

    /// allocate an ID for the client and assign them to pending clients
    fn pend_new_client(
        &mut self,
        mut client: ClientStream,
        pending: &mut HashMap<Token, PendingClient>,
    ) -> io::Result<()> {
        let token = Token(self.next_socket);
        client.register_to_poll(token, &mut self.poll)?;
        self.next_socket += 1;
        pending.insert(token, PendingClient::new(client));
        Ok(())
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
