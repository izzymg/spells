mod movement;
mod packet;
mod server;
use crate::game;
use bevy::{app, log, prelude::*, tasks::IoTaskPool, utils::dbg};
use lib_spells::{net, shared};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

type ClientID = u32;

#[derive(Resource, Debug, Clone)]
struct ClientEntityMap(HashMap<ClientID, Entity>);

// assumes all packets belong to the same client
fn sys_parse_client_packets(
    In((client_id, packets)): In<(ClientID, &[packet::IncomingPacket])>,
    client_entity_map: Res<ClientEntityMap>,
    server: NonSend<ServerComms>,
) {
    let client_entity = *client_entity_map
        .0
        .get(&client_id)
        .expect("clients passed must have a mapped entity");

    let movement_packets: Result<Vec<movement::MovementPacket>, &'static str> = packets
        .iter()
        .filter(|p| p.command == packet::PacketCommand::Velocity)
        .map(movement::MovementPacket::try_from)
        .collect();

    match movement_packets {
        Ok(packets) => {}
        Err(err) => {}
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

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let (broadcast_tx, broadcast_rx) = mpsc::channel();
        let (incoming_tx, incoming_rx) = mpsc::channel();
        let mut server = server::Server::create(incoming_tx, broadcast_rx).unwrap();

        IoTaskPool::get()
            .spawn(async move {
                log::debug!("client event loop task spawned");
                server.event_loop();
            })
            .detach();

        app.insert_non_send_resource(ServerComms::new(incoming_rx, broadcast_tx));

        app.add_systems(
            FixedLast,
            (sys_create_state
                .pipe(sys_update_component_world_state::<shared::Health>)
                .pipe(sys_update_component_world_state::<shared::Aura>)
                .pipe(sys_update_component_world_state::<shared::SpellCaster>)
                .pipe(sys_update_component_world_state::<shared::CastingSpell>)
                .pipe(sys_update_component_world_state::<shared::Position>)
                .pipe(sys_broadcast_state)
                .map(dbg))
            .in_set(game::ServerSets::NetworkSend),
        );
    }
}
