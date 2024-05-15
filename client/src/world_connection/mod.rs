mod stream;
use crate::{events, SystemSets};
use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use lib_spells::net::{self, packet};
use std::time::Duration;

const PING_FREQ: Duration = Duration::from_secs(4);

#[derive(Resource, Debug)]
pub struct Connection {
    connection: stream::Connection,
    ping_timer: Timer,
    client_info: net::ClientInfo,
    movement_inputs: Vec<(Duration, u8, Vec3)>,
}

impl Connection {
    /// Returns last-recorded round trip latency
    pub fn latency(&self) -> Option<Duration> {
        self.connection.last_ping_rtt
    }

    /// Returns client info for this connection
    pub fn client_info(&self) -> net::ClientInfo {
        self.client_info
    }

    /// Queue a movement input to be sent out
    pub fn enqueue_input(&mut self, timestamp: Duration, seq: u8, input: Vec3) {
        self.movement_inputs.push((timestamp, seq, input));
    }

    fn new(conn: stream::Connection, client_info: net::ClientInfo) -> Self {
        Self {
            connection: conn,
            client_info,
            ping_timer: Timer::new(PING_FREQ, TimerMode::Repeating),
            movement_inputs: Vec::new(),
        }
    }
}

/// Ping the server on a timer
fn sys_net_send_ping(time: Res<Time>, mut conn: ResMut<Connection>) -> stream::Result<()> {
    conn.ping_timer.tick(time.delta());
    if conn.ping_timer.just_finished() {
        conn.connection.ping()?;
    }
    Ok(())
}

/// Write all buffered movement inputs
/// TODO: batching
fn sys_net_send_movement(mut conn: ResMut<Connection>) -> stream::Result<()> {
    conn.movement_inputs
        .drain(..)
        .collect::<Vec<(Duration, u8, Vec3)>>()
        .into_iter()
        .try_for_each(|(timestamp, seq, dir)| {
            conn.connection
                .send_packet(lib_spells::net::packet::Packet {
                    timestamp,
                    seq,
                    command_type: packet::PacketType::Move,
                    command_data: packet::PacketData::Movement(dir.into())
                })?;
            Ok(())
        })
}

fn sys_net_handle_error(
    In(err): In<stream::Result<()>>,
    mut dc_ev_w: EventWriter<events::DisconnectedEvent>,
) {
    if let Err(err) = err {
        log::warn!("caught network send error: {}", err);
        dc_ev_w.send(events::DisconnectedEvent(Some(err.to_string())));
    }
}

// Currently connecting
#[derive(Resource, Debug)]
struct Connecting {
    handle: tasks::Task<stream::Result<(stream::Connection, net::ClientInfo)>>,
}

/// Stores one shot connect system
#[derive(Debug, Resource)]
pub struct WorldConnectSys {
    pub connect_system: SystemId<(String, Option<String>)>,
}

impl WorldConnectSys {
    fn new(connect_system: SystemId<(String, Option<String>)>) -> Self {
        Self { connect_system }
    }
}

/// Check for disconnection and dispatch incoming data.
fn sys_check_connection(world: &mut World) {
    world.resource_scope(|world, mut connection: Mut<Connection>| {
        match connection.connection.read() {
            Ok(reads) => {
                for (seq, state) in reads {
                    world
                        .get_resource_mut::<Events<events::WorldStateEvent>>()
                        .unwrap()
                        .send(events::WorldStateEvent {
                            seq,
                            client_info: connection.client_info,
                            state,
                        });
                }
            }
            Err(err) => {
                world
                    .get_resource_mut::<Events<events::DisconnectedEvent>>()
                    .unwrap()
                    .send(events::DisconnectedEvent(Some(err.to_string())));
                world.remove_resource::<Connection>();
                log::debug!("removed connection: {:?}", err);
            }
        }
    });
}

fn sys_check_connecting(world: &mut World) {
    let mut connecting = match world.get_resource_mut::<Connecting>() {
        Some(connecting) => connecting,
        None => return,
    };
    let res = match tasks::block_on(tasks::poll_once(&mut connecting.handle)) {
        Some(res) => res,
        None => return,
    };

    world.remove_resource::<Connecting>().unwrap();

    match res {
        Ok((conn, client_info)) => {
            log::debug!("inserted connection resource");
            world
                .get_resource_mut::<Events<events::ConnectedEvent>>()
                .unwrap()
                .send(events::ConnectedEvent);
            world.insert_resource(Connection::new(conn, client_info));
        }
        Err(err) => {
            log::info!("connection failure: {}", err);
            world
                .get_resource_mut::<Events<events::DisconnectedEvent>>()
                .unwrap()
                .send(events::DisconnectedEvent(Some(err.to_string())));
        }
    }
}

/// Handle requests to connect to a world.
fn sys_connect(In((addr, password)): In<(String, Option<String>)>, world: &mut World) {
    let handle = tasks::IoTaskPool::get()
        .spawn(async move { stream::get_connection(&addr.clone(), password.as_deref()) });

    world.insert_resource(Connecting { handle });
}
pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let connect_system = app.world.register_system(sys_connect);
        app.insert_resource(WorldConnectSys::new(connect_system));
        app.add_systems(
            Update,
            (
                sys_check_connecting.run_if(resource_exists::<Connecting>),
                (
                    sys_check_connection.in_set(SystemSets::NetFetch),
                    (
                        sys_net_send_ping.pipe(sys_net_handle_error),
                        sys_net_send_movement.pipe(sys_net_handle_error),
                    )
                        .in_set(SystemSets::NetSend),
                )
                    .run_if(resource_exists::<Connection>),
            ),
        );
    }
}
