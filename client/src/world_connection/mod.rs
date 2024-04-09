mod stream;

use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use lib_spells::serialization;

use std::{fmt::Display, sync::mpsc};

use self::stream::ServerStreamMessage;

#[derive(Debug)]
pub enum WorldConnectionMessage {
    Error(stream::ServerStreamError),
    Message(String),
}

impl From<stream::ServerStreamError> for WorldConnectionMessage {
    fn from(value: stream::ServerStreamError) -> Self {
        Self::Error(value)
    }
}

impl Display for WorldConnectionMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(err) => write!(f, "world connection... {}", err),
            Self::Message(msg) => write!(f, "world connection... {msg}"),
        }
    }
}

/// Non-send receiver of new World State data.
// An "active connection" for our purposes just means we have Some(handle) to the thread that runs it.
// If that dies we catch it (somewhere, somehow) and set everything back to None
#[derive(Default)]
struct ThreadReceiver {
    rx: Option<mpsc::Receiver<stream::ServerStreamMessage>>,
}

// for some fucking reason this has to be in a different resource
// because if we put it in our non send and then try to access anything to do with it
// that counts as "accessing a non send resource from a different thread"
// probably because blocking on a thread counts as integrating it xdddd
#[derive(Resource, Default)]
struct ThreadHandle {
    handle: Option<tasks::Task<Result<(), stream::ServerStreamError>>>,
}

#[derive(Debug, Resource)]
pub struct WorldConnection {
    pub connect_system: SystemId<String>,
    pub world_state: Option<serialization::WorldState>,
    pub message: Option<WorldConnectionMessage>,
}

impl WorldConnection {
    fn new(connect_system: SystemId<String>) -> Self {
        Self {
            connect_system,
            message: None,
            world_state: None,
        }
    }
}

/// Run only if there's some connection
fn run_if_conn(thread_handle: Res<ThreadHandle>) -> bool {
    thread_handle.handle.is_some()
}

fn sys_check_receiver(recv: NonSend<ThreadReceiver>, mut conn: ResMut<WorldConnection>) {
    if let Some(recv) = &recv.rx {
        if let Ok(msg) = recv.try_recv() {
            match msg {
                ServerStreamMessage::Info(msg) => {
                    conn.message = Some(WorldConnectionMessage::Message(msg))
                }
                _ => (),
            }
        }
    }
}

/// Check for world disconnections and handle appropriately. ONLY DO THIS WHEN THERE'S A CONNECTION.
fn sys_check_disconnect(
    mut thread_handle: ResMut<ThreadHandle>,
    mut conn: ResMut<WorldConnection>,
) {
    if let Some(res) = tasks::block_on(tasks::poll_once(thread_handle.handle.as_mut().unwrap())) {
        log::info!("got disconnection {:?}", res);
        thread_handle.handle = None;
        if let Err(err) = res {
            conn.message = Some(WorldConnectionMessage::Error(err.into()));
        } else {
            unreachable!("the server shouldn't quietly fail");
        }
    }
}

/// Handle requests to connect to a world.
fn sys_connect(
    In(addr): In<String>,
    mut connection_res: NonSendMut<ThreadReceiver>,
    mut thread_handle: ResMut<ThreadHandle>,
    mut conn: ResMut<WorldConnection>,
) {
    conn.message = Some(WorldConnectionMessage::Message("connecting...".into()));
    let (tx, rx) = mpsc::channel();
    let handle = tasks::IoTaskPool::get().spawn(async move {
        let mut connection = stream::connect(&addr.clone())?;
        connection.listen_handshake(tx)
    });

    thread_handle.handle = Some(handle);
    connection_res.rx = Some(rx);
}

pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let connect_system = app.world.register_system(sys_connect);
        app.insert_resource(WorldConnection::new(connect_system));
        app.insert_resource(ThreadHandle::default());
        app.insert_non_send_resource(ThreadReceiver::default());
        app.add_systems(
            Update,
            (sys_check_disconnect, sys_check_receiver).run_if(run_if_conn),
        );
    }
}
