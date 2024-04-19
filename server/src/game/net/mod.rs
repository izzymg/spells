mod movement;
mod packet;
mod socket;
use crate::game;
use bevy::{app, log, prelude::*, tasks::IoTaskPool, utils::dbg};
use lib_spells::{net, shared};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::Instant;
use strum_macros::FromRepr;

type ClientID = u32;

#[derive(Resource, Debug, Clone)]
struct ClientEntityMap(HashMap<ClientID, Entity>);

/// Describes a last known velocity. Only used for network tracking of movement.
#[derive(Component, Debug, Copy, Clone)]
pub struct VelocityInstant {
    timestamp: Instant,
    velocity: Vec3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct VelocityError;

impl Display for VelocityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "velocity error")
    }
}

/// Movement states including no movement, going clockwise from forward.
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr, Hash)]
#[repr(u8)]
enum MovementDirection {
    Still = 0,
    Forward,
    Right,
    Backward,
    Left,
}

impl TryFrom<u8> for MovementDirection {
    type Error = &'static str;

    /// Produce a movement direction from a single le byte.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if let Some(dir) = MovementDirection::from_repr(u8::from_le_bytes([value])) {
            Ok(dir)
        } else {
            Err("invalid movement direction")
        }
    }
}

impl MovementDirection {
    /// Convert a movement direction to a direction in -z forward y up 3D space.
    fn to_3d(dir: MovementDirection) -> Vec3 {
        match dir {
            MovementDirection::Still => Vec3::ZERO,
            MovementDirection::Forward => Vec3::NEG_Z,
            MovementDirection::Right => Vec3::X,
            MovementDirection::Backward => Vec3::Z,
            MovementDirection::Left => Vec3::NEG_X,
        }
    }
}

/// Request to move in a direction
#[derive(Debug, Copy, Clone)]
struct MovementPacket {
    timestamp: Instant,
    direction: MovementDirection,
}

impl TryFrom<packet::IncomingPacket> for MovementPacket {
    type Error = &'static str;

    fn try_from(value: packet::IncomingPacket) -> Result<Self, Self::Error> {
        if let Some(byte) = value.payload.first() {
            match MovementDirection::try_from(*byte) {
                Ok(direction) => Ok(MovementPacket {
                    direction,
                    timestamp: value.timestamp,
                }),
                Err(err) => Err(err),
            }
        } else {
            Err("packet too small")
        }
    }
}

// assumes all packets belong to the same client
fn sys_parse_client_packets(
    In((client_id, packets)): In<(ClientID, &[packet::IncomingPacket])>,
    client_entity_map: Res<ClientEntityMap>,
) -> Vec<ClientID> {
    let client_entity = *client_entity_map
        .0
        .get(&client_id)
        .expect("clients passed must have a mapped entity");
    let mut dead_clients = vec![];

    for packet in packets.iter() {
        if packet::PacketCommand::Velocity == packet.command {}
    }

    dead_clients
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
    mut sender: ResMut<ClientStreamSender>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) -> net::WorldState {
    if !sender.send_data(
        world_state
            .serialize()
            .expect("world serialization failure"),
    ) {
        log::info!("client sender died, exiting");
        exit_events.send(app::AppExit);
    }
    world_state
}

// wraps a send channel
#[derive(Resource)]
struct ClientStreamSender(mpsc::Sender<Vec<u8>>);

impl ClientStreamSender {
    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self(tx)
    }

    // Returns false if sending is now impossible (very bad)
    pub fn send_data(&mut self, data: Vec<u8>) -> bool {
        self.0.send(data).is_ok()
    }
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client_getter = socket::client_server::ClientServer::create().unwrap();
        let (tx, rx) = mpsc::channel();

        IoTaskPool::get()
            .spawn(async move {
                log::debug!("client event loop task spawned");
                client_getter.event_loop(rx);
            })
            .detach();

        app.insert_resource(ClientStreamSender::new(tx));

        app.add_systems(
            FixedLast,
            (sys_create_state
                .pipe(sys_update_component_world_state::<shared::Health>)
                .pipe(sys_update_component_world_state::<shared::Aura>)
                .pipe(sys_update_component_world_state::<shared::SpellCaster>)
                .pipe(sys_update_component_world_state::<shared::CastingSpell>)
                .pipe(sys_broadcast_state)
                .map(dbg))
            .in_set(game::ServerSets::NetworkSend),
        );
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_to_movement_packet() {
        let cases = vec![0_u8, 1, 2, 3, 4];
        let fail = 5_u8;

        for case in cases {
            let packet = packet::IncomingPacket {
                timestamp: Instant::now(),
                command: packet::PacketCommand::Velocity,
                stamp: 0,
                payload: case.to_le_bytes().to_vec(),
            };
            let movement_packet =
                MovementPacket::try_from(packet).expect("should convert correctly");
            assert_eq!(
                movement_packet.direction,
                MovementDirection::try_from(case).unwrap()
            );
        }
        assert!(MovementPacket::try_from(packet::IncomingPacket {
            timestamp: Instant::now(),
            command: packet::PacketCommand::Velocity,
            stamp: 0,
            payload: fail.to_le_bytes().to_vec(),
        })
        .is_err());
    }
}
