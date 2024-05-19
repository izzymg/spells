mod server;

use crate::game;
use bevy::{log, prelude::*, tasks::IoTaskPool};
use lib_spells::{
    net::{self, packet},
    shared,
};
use std::{collections::HashMap, sync::mpsc, time::Duration};

#[derive(Component, Debug, Default)]
struct LastPacketTime(Option<Duration>);

#[derive(Component, Debug, Default)]
struct LastPacketSequence(u8);

#[derive(Component, Debug)]
struct ServerPlayer(server::Token);

#[derive(Bundle, Debug)]
struct ServerPlayerBundle {
    sp: ServerPlayer,
    lps: LastPacketSequence,
    lpt: LastPacketTime,
    name: shared::Name,
    pos: shared::Position,
    player: shared::Player,
    vel: shared::Velocity,
}

impl ServerPlayerBundle {
    fn new(token: server::Token) -> Self {
        Self {
            sp: ServerPlayer(token),
            lps: Default::default(),
            lpt: Default::default(),
            pos: Default::default(),
            vel: Default::default(),
            player: Default::default(),
            name: shared::Name(format!("Player {}", token)),
        }
    }
}

fn sys_process_incoming(
    mut commands: Commands,
    server: NonSend<ServerComms>,
    server_player_query: Query<(Entity, &ServerPlayer)>,
) -> HashMap<Entity, Vec<packet::Packet>> {
    let mut client_packets: HashMap<Entity, Vec<packet::Packet>> = HashMap::default();

    for inc in server.incoming.try_iter() {
        match inc {
            server::Incoming::Joined(token) => {
                commands.spawn(ServerPlayerBundle::new(token));
            }
            server::Incoming::Left(token) => {
                if let Some((entity, _)) = server_player_query.iter().find(|(_, p)| p.0 == token) {
                    commands.entity(entity).despawn_recursive();
                }
            }
            server::Incoming::Data(token, packet) => {
                if let Some((entity, _)) = server_player_query.iter().find(|(_, p)| p.0 == token) {
                    if let Some(packets) = client_packets.get_mut(&entity) {
                        packets.push(packet);
                    } else {
                        client_packets.insert(entity, vec![packet]);
                    }
                }
            }
        }
    }

    // pass an updated copy of our client info
    client_packets
}

fn sys_process_client_packets(
    In(packets): In<HashMap<Entity, Vec<packet::Packet>>>,
    mut q_velocity_pos: Query<(
        Entity,
        &mut shared::Position,
        &mut shared::Velocity,
        &mut LastPacketTime,
        &mut LastPacketSequence,
    )>,
) {
    for (entity, mut pos, mut vel, mut last_t, mut last_sequence) in q_velocity_pos.iter_mut() {
        let entity_packets = packets.get(&entity);
        let movement_packets = entity_packets.iter().flat_map(|p| {
            p.iter().filter_map(|p| match p.command_data {
                packet::PacketData::Movement(dir) => Some((p.timestamp, p.seq, dir)),
                _ => None,
            })
        });

        for (time, seq, dir) in movement_packets {
            if let Some(lts) = last_t.0 {
                let t = (time - lts).as_secs_f32();
                pos.0 += vel.0 * t;
            }
            vel.0 = Vec3::from(dir);
            last_t.0 = Some(time);
            last_sequence.0 = seq;
            log::debug!("velocity: {}, pos: {}", vel.0, pos.0);
        }
    }
}

fn sys_broadcast_state(
    In(world_state): In<net::WorldState>,
    server: NonSend<ServerComms>,
    players_query: Query<(&ServerPlayer, &LastPacketSequence)>,
) {
    for (player, sequence) in players_query.iter() {
        server
            .outgoing
            .send(server::Outgoing::ClientState(
                player.0,
                server::ClientStateUpdate {
                    seq: sequence.0,
                    world_state: world_state.clone(),
                },
            ))
            .unwrap();
    }
}

fn sys_on_player_spawned(
    server: NonSend<ServerComms>,
    query: Query<(Entity, &ServerPlayer), Added<ServerPlayer>>,
) {
    for (entity, player) in query.iter() {
        server
            .outgoing
            .send(server::Outgoing::ClientInfo(
                player.0,
                net::ClientInfo { you: entity },
            ))
            .unwrap();
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
        app.add_systems(
            FixedUpdate,
            sys_on_player_spawned.after(sys_process_incoming),
        );
    }
}
