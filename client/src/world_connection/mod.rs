mod stream;
use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use std::sync::mpsc;

#[derive(Debug, Event)]
pub struct ClientInfoEvent(pub lib_spells::net::ClientInfo);

#[derive(Debug, Event)]
pub struct WorldStateEvent(pub lib_spells::net::WorldState);

/// This should be a `NonSend` resource
#[derive(Default)]
struct ConnectorComms {
    rx: Option<mpsc::Receiver<stream::Incoming>>,
    tx: Option<mpsc::Sender<stream::Outgoing>>,
}

/// These should be `Send`
#[derive(Debug, Resource)]
pub struct WorldConnection {
    pub connect_system: SystemId<(String, Option<String>)>,
    listen_handle: Option<tasks::Task<Result<(), stream::ConnectionError>>>,
}

impl WorldConnection {
    fn new(connect_system: SystemId<(String, Option<String>)>) -> Self {
        Self {
            connect_system,
            listen_handle: None,
        }
    }
}

// Check for incoming messages
fn sys_check_incoming(recv: NonSend<ConnectorComms>) {
    if let Some(recv) = &recv.rx {
        if let Ok(msg) = recv.try_recv() {
            log::debug!("received server message");
        }
    }
}

/// Check for world disconnections and handle appropriately
fn sys_check_disconnect(mut conn: ResMut<WorldConnection>) {
    if let Some(res) = tasks::block_on(tasks::poll_once(conn.listen_handle.as_mut().unwrap())) {
        log::info!("disconnected from world {:?}", res);
        conn.listen_handle = None;
        if let Err(err) = res {}
    }
}

/// Handle requests to connect to a world.
fn sys_connect(
    In((addr, password)): In<(String, Option<String>)>,
    mut comms: NonSendMut<ConnectorComms>,
    mut conn: ResMut<WorldConnection>,
) {
    let (out_tx, out_rx) = mpsc::channel();
    let (inc_tx, inc_rx) = mpsc::channel();
    let handle = tasks::IoTaskPool::get()
        .spawn(async move { stream::get_connection(&addr.clone(), password.as_deref()) });

    comms.tx = Some(out_tx);
    comms.rx = Some(inc_rx);
}

pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let connect_system = app.world.register_system(sys_connect);
        app.insert_resource(WorldConnection::new(connect_system));
        app.insert_non_send_resource(ConnectorComms::default());
        app.add_systems(
            Update,
            (sys_check_disconnect, sys_check_incoming)
                .run_if(|con: Res<WorldConnection>| con.listen_handle.is_some()),
        );
    }
}
