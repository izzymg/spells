mod movement;
mod server;
use crate::game;
use bevy::{app, log, prelude::*, tasks::IoTaskPool};
use lib_spells::{net, shared};
use server::packet;
use std::{
    collections::HashMap,
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Component, Debug)]
struct LastPacketRead(pub Instant);

#[derive(Resource, Debug, Default)]
struct ActiveClientInfo(server::ActiveClientInfo);

fn spawn_client(commands: &mut Commands, id: &str) -> Entity {
    commands
        .spawn((
            shared::Player,
            shared::Name(format!("Player {}", id)),
            shared::Position(Vec3::ZERO),
            shared::Velocity(Vec3::ZERO),
            LastPacketRead(Instant::now()),
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
            }
            server::Incoming::Left(token) => {
                commands
                    .entity(active_client_info.0.get(&token).unwrap().you)
                    .despawn_recursive();
                active_client_info.0.remove(&token);
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
    server
        .outgoing
        .send(server::Outgoing::ClientInfo(active_client_info.clone()))
        .unwrap();
    client_packets
}

fn sys_process_client_packets(
    In(packets): In<HashMap<Entity, Vec<packet::Packet>>>,
    mut commands: Commands,
    q_velocity_pos: Query<(&shared::Position, &shared::Velocity, &LastPacketRead)>,
) {
    for (entity, packets) in packets.iter() {
        let (pos, vel, t) = match q_velocity_pos.get(*entity) {
            Ok((pos, vel, t)) => (pos, vel, t),
            Err(_) => {
                log::warn!("skipping packet for entity {:?}", *entity);
                continue;
            }
        };

        let (new_pos, new_vel, new_t) = movement::integrate_movement_packets(
            pos.0,
            vel.0,
            t.0,
            packets
                .iter()
                .filter_map(|p| movement::MovementPacket::from_packet(*p)),
        );

        commands.entity(*entity).try_insert((
            shared::Position(new_pos),
            shared::Velocity(new_vel),
            LastPacketRead(new_t),
        ));
    }
}

fn sys_create_state() -> net::WorldState {
    net::WorldState::default()
}

fn sys_update_component_world_state<T: Component + Into<net::EntityState> + Clone>(
    In(mut world_state): In<net::WorldState>,
    query: Query<(Entity, &T)>,
) -> net::WorldState {
    query.iter().for_each(|(entity, comp)| {
        // clone is here so components can have uncopyable types like "timer"
        // however we should check performance of this and consider custom serialization of timer values if performance is bad
        world_state.update(entity, comp.clone().into());
    });

    world_state
}

fn sys_broadcast_state(
    In(world_state): In<net::WorldState>,
    server: NonSend<ServerComms>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) -> net::WorldState {
    if server
        .outgoing
        .send(server::Outgoing::Broadcast(
            world_state
                .serialize()
                .expect("world serialization failure"),
        ))
        .is_err()
    {
        log::info!("client sender died, exiting");
        exit_events.send(app::AppExit);
    }
    world_state
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
            ((sys_create_state
                .pipe(sys_update_component_world_state::<shared::Health>)
                .pipe(sys_update_component_world_state::<shared::Aura>)
                .pipe(sys_update_component_world_state::<shared::SpellCaster>)
                .pipe(sys_update_component_world_state::<shared::CastingSpell>)
                .pipe(sys_update_component_world_state::<shared::Position>)
                .pipe(sys_update_component_world_state::<shared::Player>)
                .pipe(sys_update_component_world_state::<shared::Name>)
                .pipe(sys_broadcast_state)
                .map(drop)),)
                .in_set(game::ServerSets::NetworkSend),
        );
        app.add_systems(
            FixedUpdate,
            (sys_process_incoming.pipe(sys_process_client_packets))
                .in_set(game::ServerSets::NetworkFetch),
        );
    }
}
