mod server;

use crate::game;
use bevy::{app, log, prelude::*, tasks::IoTaskPool};
use lib_spells::{
    net::{self, packet},
    shared,
};
use std::{collections::HashMap, sync::mpsc, time::Duration};

#[derive(Component, Debug)]
struct LastMovementPacket(Option<Duration>);

#[derive(Resource, Debug, Default)]
struct ActiveClientInfo(server::ActiveClientInfo);

fn spawn_client(commands: &mut Commands, id: &str) -> Entity {
    log::debug!("spawning player client entity");
    commands
        .spawn((
            shared::Player,
            shared::Name(format!("Player {}", id)),
            shared::Position(Vec3::ZERO),
            shared::Velocity(Vec3::ZERO),
            LastMovementPacket(None),
        ))
        .id()
}

fn sys_process_incoming(
    mut commands: Commands,
    active_client_info: ResMut<ActiveClientInfo>,
    server: NonSend<ServerComms>,
) -> HashMap<Entity, Vec<packet::Packet>> {
    let active_client_info = &mut active_client_info.into_inner().0;
    let mut client_packets: HashMap<Entity, Vec<packet::Packet>> = HashMap::default();

    for inc in server.incoming.try_iter() {
        match inc {
            server::Incoming::Joined(token) => {
                active_client_info.0.insert(
                    token,
                    net::ClientInfo {
                        you: spawn_client(&mut commands, &token.to_string()),
                    },
                );
                server
                    .outgoing
                    .send(server::Outgoing::ClientInfo(active_client_info.clone()))
                    .unwrap();
            }
            server::Incoming::Left(token) => {
                commands
                    .entity(active_client_info.0.get(&token).unwrap().you)
                    .despawn_recursive();
                active_client_info.0.remove(&token);
                server
                    .outgoing
                    .send(server::Outgoing::ClientInfo(active_client_info.clone()))
                    .unwrap();
            }
            server::Incoming::Data(token, packet) => {
                let entity = active_client_info.0.get(&token).unwrap().you;
                if let Some(packets) = client_packets.get_mut(&entity) {
                    packets.push(packet);
                } else {
                    client_packets.insert(entity, vec![packet]);
                }
            }
        }
    }

    // pass an updated copy of our client info
    client_packets
}

fn sys_process_client_packets(
    In(packets): In<HashMap<Entity, Vec<packet::Packet>>>,
    mut q_velocity_pos: Query<(Entity, &mut shared::Position, &mut shared::Velocity, &mut LastMovementPacket)>,
) {
    for (entity, mut pos, mut vel, mut last_packet) in q_velocity_pos.iter_mut() {
        let entity_packets = packets.get(&entity);
        let movement_packets = entity_packets.iter().flat_map(|p| {
            p.iter().filter_map(|p| match p.command_data {
                packet::PacketData::Movement(dir) => Some((p.timestamp, dir)),
                _ => None,
            })
        });

        for (time, dir) in movement_packets {
            if let Some(lts) = last_packet.0 {
                let t = (time - lts).as_secs_f32();
                pos.0 += vel.0 * t;
                vel.0 = Vec3::from(dir);
            }
            last_packet.0 = Some(time);
        }
    }
}

fn sys_broadcast_state(
    In(world_state): In<net::WorldState>,
    server: NonSend<ServerComms>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) {
    if server
        .outgoing
        .send(server::Outgoing::Broadcast(world_state))
        .is_err()
    {
        log::info!("client sender died, exiting");
        exit_events.send(app::AppExit);
    }
}

struct ServerComms {
    outgoing: mpsc::Sender<server::Outgoing>,
    incoming: mpsc::Receiver<server::Incoming>,
}

impl ServerComms {
    pub fn new(
        incoming: mpsc::Receiver<server::Incoming>,
        outgoing: mpsc::Sender<server::Outgoing>,
    ) -> Self {
        Self { outgoing, incoming }
    }
}

pub struct NetPlugin {
    pub server_password: Option<String>,
}

impl Plugin for NetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let (broadcast_tx, broadcast_rx) = mpsc::channel();
        let (incoming_tx, incoming_rx) = mpsc::channel();
        let mut server = server::Server::create().unwrap();

        let password = self.server_password.clone();
        IoTaskPool::get()
            .spawn(async move {
                log::debug!("client event loop task spawned");
                if let Err(err) = server.event_loop(incoming_tx, broadcast_rx, password) {
                    log::error!("client event loop exited: {}", err);
                }
            })
            .detach();

        app.insert_non_send_resource(ServerComms::new(incoming_rx, broadcast_tx));
        app.insert_resource(ActiveClientInfo::default());
        app.add_systems(
            FixedUpdate,
            ((net::query_world_state.pipe(sys_broadcast_state).map(drop)),)
                .in_set(game::ServerSets::NetworkSend),
        );
        app.add_systems(
            FixedUpdate,
            (sys_process_incoming.pipe(sys_process_client_packets))
                .in_set(game::ServerSets::NetworkFetch),
        );
    }
}
