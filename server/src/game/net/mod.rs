mod movement;
mod server;
use crate::game;
use bevy::{app, log, prelude::*, tasks::IoTaskPool};
use lib_spells::{net, shared};
use server::packet;
use std::sync::mpsc;

#[derive(Resource, Debug, Default)]
struct ActiveClientInfo(server::ActiveClientInfo);

fn sys_incoming_server(
    mut commands: Commands,
    mut client_entity_map: ResMut<ActiveClientInfo>,
    server: NonSend<ServerComms>,
) {
    match server.incoming.try_recv() {
        Ok(incoming) => match incoming {
            server::Incoming::Joined(token) => {
                let entity = commands
                    .spawn((
                        shared::Player,
                        shared::Name(format!("Player {}", token)),
                        shared::Position(Vec3::ZERO),
                    ))
                    .id();
                client_entity_map
                    .0
                     .0
                    .insert(token, net::ClientInfo { you: entity });
                server
                    .outgoing
                    .send(server::Outgoing::ClientInfo(client_entity_map.0.clone()))
                    .unwrap();
            }
            server::Incoming::Left(token) => {
                commands
                    .entity(client_entity_map.0 .0.get(&token).unwrap().you)
                    .despawn_recursive();
                client_entity_map.0 .0.remove(&token);
                server
                    .outgoing
                    .send(server::Outgoing::ClientInfo(client_entity_map.0.clone()))
                    .unwrap();
            }
            server::Incoming::Data(token, packet) => {}
        },
        Err(mpsc::TryRecvError::Disconnected) => {
            panic!("server thread died");
        }
        Err(mpsc::TryRecvError::Empty) => {}
    }
}

// assumes all packets belong to the same client
fn sys_parse_client_packets(
    In((client_id, _packets)): In<(server::Token, &[packet::Packet])>,
    client_entity_map: Res<ActiveClientInfo>,
) {
    let _client_entity = client_entity_map
        .0
         .0
        .get(&client_id)
        .expect("clients passed must have a mapped entity");
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
            FixedLast,
            (
                sys_incoming_server,
                (sys_create_state
                    .pipe(sys_update_component_world_state::<shared::Health>)
                    .pipe(sys_update_component_world_state::<shared::Aura>)
                    .pipe(sys_update_component_world_state::<shared::SpellCaster>)
                    .pipe(sys_update_component_world_state::<shared::CastingSpell>)
                    .pipe(sys_update_component_world_state::<shared::Position>)
                    .pipe(sys_update_component_world_state::<shared::Player>)
                    .pipe(sys_update_component_world_state::<shared::Name>)
                    .pipe(sys_broadcast_state)
                    .map(drop)),
            )
                .in_set(game::ServerSets::NetworkSend),
        );
    }
}
