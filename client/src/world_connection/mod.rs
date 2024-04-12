mod stream;

pub use stream::ServerStreamStatus;

use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use lib_spells::shared;

use std::{fmt::Display, sync::mpsc};

use self::stream::ServerStreamMessage;

#[derive(Debug)]
pub enum WorldConnectionMessage {
    Error(stream::ServerStreamError),
    Status(stream::ServerStreamStatus),
}

impl From<stream::ServerStreamError> for WorldConnectionMessage {
    fn from(value: stream::ServerStreamError) -> Self {
        Self::Error(value)
    }
}

impl Display for WorldConnectionMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(err) => write!(f, "world connection error... {}", err),
            Self::Status(msg) => write!(f, "world connection status... {}", msg),
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

// Describes a change made to a stored WorldState
#[derive(Debug, Resource, Default)]
pub struct WorldStateChange {
    /// If true, this server entity is new.
    pub new_server_keys: Vec<(u32, bool)>,
    pub lost_server_keys: Vec<u32>,
}

#[derive(Debug, Resource)]
pub struct WorldConnection {
    pub connect_system: SystemId<String>,
    pub cached_state: Option<shared::WorldState>,
    pub state_change: Option<WorldStateChange>,
    pub message: Option<WorldConnectionMessage>,
}

impl WorldConnection {
    fn new(connect_system: SystemId<String>) -> Self {
        Self {
            connect_system,
            message: None,
            cached_state: None,
            state_change: None,
        }
    }
}

/// Run only if there's some connection
fn run_if_conn(thread_handle: Res<ThreadHandle>) -> bool {
    thread_handle.handle.is_some()
}

/// Check the receiver for the connection thread and handle new messages.
fn sys_check_receiver(recv: NonSend<ThreadReceiver>, mut conn: ResMut<WorldConnection>) {
    if let Some(recv) = &recv.rx {
        if let Ok(msg) = recv.try_recv() {
            log::debug!("received server message");
            match msg {
                ServerStreamMessage::Status(msg) => {
                    conn.message = Some(WorldConnectionMessage::Status(msg));
                }
                ServerStreamMessage::Data(data) => {
                    let new_state = shared::WorldState::deserialize(&data)
                        .expect("deserialization shouldn't fail");
                    // if we never had state set new state
                    if conn.cached_state.is_none() {
                        conn.cached_state = Some(new_state);
                        return;
                    }
                    let cached_state = conn.cached_state.as_ref().unwrap();
                    // every entity that's in new, that wasn't in the cache
                    let state_change = WorldStateChange {
                        new_server_keys: new_state
                            .entity_state_map
                            .keys()
                            .copied()
                            .map(|k| (k, !cached_state.entity_state_map.contains_key(&k)))
                            .collect(),

                        // every entity that was in our cache that isn't in new
                        lost_server_keys: cached_state
                            .entity_state_map
                            .keys()
                            .copied()
                            .filter(|k| !new_state.entity_state_map.contains_key(k))
                            .collect(),
                    };

                    conn.cached_state = Some(new_state);
                    conn.state_change = Some(state_change);
                }
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
            conn.message = Some(WorldConnectionMessage::Error(err));
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
    conn.message = None;
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
