use core::fmt;
use std::{
    error::Error,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

use bevy::{
    app::Update, ecs::system::NonSend, DefaultPlugins
};

const SERVER_ADDR: &str = "127.0.0.1:7776";
const EXPECT_SERVER_HEADER: &str = "SPELLSERVER 0.1\n";
const CLIENT_RESPONSE: &str = "SPELLCLIENT OK 0.1\n";

pub struct ConnectionHandler {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl ConnectionHandler {
    pub fn handle(connection: Connection) -> io::Result<ConnectionHandler> {
        let writer = io::BufWriter::new(connection.stream.try_clone()?);
        let reader = io::BufReader::new(connection.stream);
        Ok(Self { reader, writer })
    }

    /// Block until data received, and return if the data matches the given.
    fn expect(&mut self, data: &str) -> io::Result<bool> {
        let mut buf = data.to_string();
        buf.clear();
        self.reader.read_line(&mut buf)?;
        Ok(buf == data)
    }

    /// Block until we receive the expected server header response from Spells Server.
    fn expect_header(&mut self) -> io::Result<bool> {
        self.expect(EXPECT_SERVER_HEADER)
    }

    fn write_client_ok(&mut self) -> io::Result<()> {
        self.writer.write_all(CLIENT_RESPONSE.as_bytes())?;
        self.writer.flush()
    }

    /// block until we get more state
    pub fn listen(&mut self, tx: Sender<io::Result<String>>) {
        let mut buf = String::new();
        loop {
            buf.clear();
            match self.reader.read_line(&mut buf) {
                Ok(read) => {
                    println!("read! {}", read);
                    if read > 0 {
                        tx.send(Ok(buf.clone())).unwrap();
                    }
                }
                Err(err) => {
                    println!("err! {}", err);
                    tx.send(Err(err)).unwrap();
                }
            }
        }
    }
}

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn connect_retry(addr: &str) -> io::Result<Connection> {
        let stream = loop {
            match TcpStream::connect(addr) {
                Ok(s) => {
                    println!("connected to {}", addr);
                    break s;
                }
                Err(err) => {
                    println!("failed to connect to {}, retrying ({})", SERVER_ADDR, err)
                }
            }
        };
        Ok(Connection { stream })
    }
}

#[derive(Debug, Clone)]
pub struct ServerFetchError(String);
impl fmt::Display for ServerFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to fetch world state, closing. reason: {}",
            self.0
        )
    }
}

pub struct ServerStateReceiver(Receiver<io::Result<String>>);

fn check_world_server_data_system(fetch: NonSend<ServerStateReceiver>) {
    match fetch.0.try_recv() {
        Ok(msg) => match msg {
            Ok(world_state) => {
                println!("NEW WORLD STATE {}", world_state)
            }
            Err(err) => {
                panic!("{}", err);
            }
        },
        Err(err) => match err {
            mpsc::TryRecvError::Disconnected => {
                panic!("CHANNEL DC");
            }
            _ => {}
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::connect_retry(SERVER_ADDR)?;
    {
        let mut connection_handler = ConnectionHandler::handle(connection)?;

        if !connection_handler.expect_header()? {
            return Err("invalid response from server (wrong version?)".into());
        }
        println!("OK header from server");
        connection_handler.write_client_ok()?;
        println!("sent OK");

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            connection_handler.listen(tx);
        });

        bevy::app::App::new()
            .add_plugins(DefaultPlugins)
            .insert_non_send_resource(ServerStateReceiver(rx))
            .add_systems(Update, check_world_server_data_system)
            .run();
        Ok(())
    }
}
