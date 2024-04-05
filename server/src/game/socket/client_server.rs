use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use std::{io, time};

use super::client_stream::ClientStream;
use super::CLIENT_EXPECT;

const SERVER_ADDR: &str = "0.0.0.0:7776";
const SERVER_TOKEN: Token = Token(0); // uniquely identify TCP listener events
const EVENT_BUFFER_SIZE: usize = 1028;
const PENDING_TIMEOUT: Duration = Duration::from_millis(50); // how long to wait on a client to verify before we kick them
const MIN_TICK: Duration = Duration::from_millis(2000); // how often at minimum we should check for pending clients, clean up dead connectios

#[derive(Debug)]
pub struct PendingClient {
    created_at: time::Instant,
    client: ClientStream,
}

pub struct ClientValidationError {
    pub error: String,
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
    pub fn got_valid_response(&mut self) -> Result<bool, ClientValidationError> {
        let mut buf: [u8; CLIENT_EXPECT.as_bytes().len()] = [0; CLIENT_EXPECT.as_bytes().len()];
        match self.client.read_fill(&mut buf) {
            Ok(_) => {
                println!("{:?} | {:?} | {}", buf, CLIENT_EXPECT.as_bytes(), buf == CLIENT_EXPECT.as_bytes());
                return Ok(CLIENT_EXPECT.as_bytes() == buf)
            }
            Err(err) => return Err(ClientValidationError{ error: err.to_string() })
        }
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

pub struct ClientServer {
    listener: TcpListener,
    events: Events,
    poll: Poll,

    connected_clients: HashMap<Token, ClientStream>,
    pending_clients: HashMap<Token, PendingClient>,
    next_socket: usize,
}

impl ClientServer {
    pub fn create() -> io::Result<ClientServer> {
        println!("binding server to {SERVER_ADDR}");
        let mut listener = TcpListener::bind(SERVER_ADDR.parse().unwrap())?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;
        let events = Events::with_capacity(EVENT_BUFFER_SIZE);
        Ok(ClientServer {
            listener,
            poll,
            events,
            connected_clients: HashMap::new(),
            pending_clients: HashMap::new(),
            next_socket: 1,
        })
    }

    /// try to accept a client on the listener
    pub fn try_accept(&self) -> Option<(ClientStream, SocketAddr)> {
        if let Ok((stream, addr)) = self.listener.accept() {
            return match ClientStream::new(stream) {
                Ok(client) => Some((client, addr)),
                Err(err) => {
                    println!("failed to create client stream {}", err);
                    None
                }
            };
        }
        None
    }

    /// allocate an ID for the client and assign them to pending clients
    pub fn pend_new_client(&mut self, mut client: ClientStream) -> io::Result<()> {
        let token = Token(self.next_socket);
        client.register_to_poll(token, &mut self.poll)?;
        self.next_socket += 1; // todo: decouple from token/next socket so we don't push this up for every req?
        self.pending_clients
            .insert(token, PendingClient::new(client));
        Ok(())
    }

    /// block on event look waiting for new clients, adding them by their token to a map of active cleint
    pub fn block_get_client(&mut self, broadcast: Receiver<String>) {
        loop {
            println!("polling");
            if let Err(poll_err) = self.poll.poll(&mut self.events, Some(MIN_TICK)) {
                println!("poll error: {}", poll_err);
            }
            // drop all expired clients
            self.pending_clients
                .iter()
                .filter_map(|(k, v)| v.is_expired().then_some(k))
                .copied()
                .collect::<Vec<Token>>()
                .iter()
                .for_each(|k| {
                    self.drop_pending_client(k);
                });

            // find new clients
            let mut new_client_requests = vec![];
            
            if let Some((client, addr)) = self.try_accept() {
                new_client_requests.push((client, addr));
            }

            for ev in self.events.iter() {
                println!("{:?}", ev);
                match ev.token() {
                    SERVER_TOKEN => (),
                    client_token => {
                        // try to pull pending client out
                        if let Some(mut pending_client) = self.pending_clients.remove(&client_token)
                        {
                            if let Ok(valid) = pending_client.got_valid_response() {
                                if valid {
                                    // good data, insert into actual clients
                                    self.connected_clients
                                        .insert(client_token, pending_client.client);
                                    println!("connected valid client: {:?}", client_token);
                                } else {
                                    println!("re-inserting invalid client: {:?}", client_token);
                                    self.pending_clients.insert(client_token, pending_client);
                                }
                            } else {
                                println!("re-inserting invalid client: {:?}", client_token);
                                self.pending_clients.insert(client_token, pending_client);
                            }
                        }
                    }
                }
            }

            // start pending new clients
            for (mut client, addr) in new_client_requests {
                if let Err(err) = client.write_header() {
                    println!("failed to send server header: {}", err)
                    // client is dropped here
                } else {
                    match self.pend_new_client(client) {
                        Ok(()) => {
                            println!("pending client: {}", addr);
                        }
                        Err(err) => {
                            println!("failed to pend client: {}", err);
                        }
                    }
                }
            }

            if let Ok(data) = broadcast.try_recv() {
                self.broadcast(data.as_str());
            }
        }
    }

    pub fn drop_pending_client(&mut self, client: &Token) -> Option<PendingClient> {
        println!("dropping pending client: ({:?})", client);
        if let Some(mut pending) = self.pending_clients.remove(client) {
            pending.client.deregister_from_poll(&mut self.poll).unwrap();
            return Some(pending)
        }
        None
    }

    pub fn drop_connected_client(&mut self, client: &Token) -> Option<ClientStream> {
        println!("dropping connected client: ({:?})", client);
        self.connected_clients.remove(client)
    }

    // broadcast on all clients, drop dead ones
    pub fn broadcast(&mut self, data: &str) {
        let failures: Vec<BroadcastError> = self
            .connected_clients
            .iter_mut()
            .map(|(token, client)| {
                client.write(data).map_err(|error| BroadcastError {
                    error,
                    token: *token,
                })
            })
            .filter_map(|v| v.is_err().then(|| v.unwrap_err()))
            .collect();

        for failure in failures {
            println!("broadcast failure {}", failure);
            self.drop_connected_client(&failure.token);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        thread, time::Duration,
    };

    use crate::game::socket::{CLIENT_EXPECT, SERVER_HEADER};

    #[test]
    fn test_getter() {
        // let mut client_getter = ClientGetter::create().unwrap();
        // // create a client stream
        // // create a thread that blocks & fetches our clients
        // // assert we grab the server header correctly
        // // panic the thread if it doesn't process the client

        // let handle = thread::spawn(move || {
        //     client_getter.block_get_client();
        // });

        {
            let mut stream = TcpStream::connect("127.0.0.1:7776").unwrap();
            println!("stream connected");
            let mut first_response: [u8; SERVER_HEADER.len()] = [0; SERVER_HEADER.len()];
            stream.read_exact(&mut first_response).unwrap();
            assert_eq!(SERVER_HEADER.as_bytes(), first_response);
            stream.write(CLIENT_EXPECT.as_bytes()).unwrap();
        }
        thread::sleep(Duration::from_secs(4));
        {
            let mut stream = TcpStream::connect("127.0.0.1:7776").unwrap();
            println!("stream connected {}", stream.local_addr().unwrap());
            let mut first_response: [u8; SERVER_HEADER.len()] = [0; SERVER_HEADER.len()];
            stream.read_exact(&mut first_response).unwrap();
            assert_eq!(SERVER_HEADER.as_bytes(), first_response);
            stream.write(CLIENT_EXPECT.as_bytes()).unwrap();
        }
    }
}
