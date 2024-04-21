/*! movement of network objects */
use crate::game::net::packet;
use bevy::prelude::*;
use std::time::{Duration, Instant};
use strum_macros::FromRepr;

/// Calculate a new position based on a velocity that was `time_passed` ago
fn find_position(position: Vec3, velocity: Vec3, time_passed: Duration) -> Vec3 {
    position + (velocity * time_passed.as_secs_f32())
}

/// Describes a last known velocity. Only used for network tracking of movement.
#[derive(Component, Debug, Copy, Clone)]
pub struct VelocityInstant {
    timestamp: Instant,
    velocity: Vec3,
}

impl VelocityInstant {
    fn new(timestamp: Instant, velocity: Vec3) -> Self {
        Self {
            timestamp,
            velocity,
        }
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
    fn to_3d(&self) -> Vec3 {
        match &self {
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
pub struct MovementPacket {
    timestamp: Instant,
    direction: MovementDirection,
}

impl TryFrom<&packet::IncomingPacket> for MovementPacket {
    type Error = &'static str;

    fn try_from(value: &packet::IncomingPacket) -> Result<Self, Self::Error> {
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

/// Given packets should belong to a single client, in order of their timestamp
/// todo: copying these shouldn't be expensive, but check
pub fn integrate_movement_packets(
    position: Vec3,
    velocity_instant: VelocityInstant,
    packets: impl Iterator<Item = MovementPacket>,
) -> (Vec3, VelocityInstant) {
    packets.fold((position, velocity_instant), |(pos, vel), packet| {
        let passed = packet.timestamp.saturating_duration_since(vel.timestamp);
        let dir = packet.direction.to_3d();
        let n_pos = find_position(pos, dir, passed);
        let n_vel = VelocityInstant::new(packet.timestamp, dir);
        (n_pos, n_vel)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_position() {
        let position = Vec3::new(3.0, 4.0, 0.0);
        let velocity = Vec3::Y * 2.0;

        let time_passed = Duration::from_secs_f32(2.5);

        assert_eq!(
            find_position(position, velocity, time_passed),
            position + (velocity * 2.5)
        );
    }

    use std::thread;
    use std::time::Duration;
    #[test]
    fn test_to_movement_packet() {
        let cases = vec![0_u8, 1, 2, 3, 4];
        let fail = 5_u8;

        for case in cases {
            let packet = packet::IncomingPacket {
                timestamp: Instant::now(),
                command: packet::PacketCommand::Move,
                stamp: 0,
                payload: case.to_le_bytes().to_vec(),
            };
            let movement_packet =
                MovementPacket::try_from(&packet).expect("should convert correctly");
            assert_eq!(
                movement_packet.direction,
                MovementDirection::try_from(case).unwrap()
            );
        }
        assert!(MovementPacket::try_from(&packet::IncomingPacket {
            timestamp: Instant::now(),
            command: packet::PacketCommand::Move,
            stamp: 0,
            payload: fail.to_le_bytes().to_vec(),
        })
        .is_err());
    }

    #[test]
    #[ignore] // `Instant` is opaque, thread sleeping test
    fn test_integrate_velocity() {
        let accept_margin = 0.001;
        let starting_velocity_inst = VelocityInstant::new(Instant::now(), Vec3::ZERO);
        let dir = MovementDirection::Forward;
        let starting_pos = Vec3::new(1.0, 5.0, 2.0);
        let duration = Duration::from_millis(3500);

        // order important here
        thread::sleep(duration);
        let packets = [
            MovementPacket {
                timestamp: Instant::now(),
                direction: dir,
            },
            MovementPacket {
                timestamp: Instant::now(),
                direction: MovementDirection::Still,
            },
        ];

        let expected_pos = starting_pos + (dir.to_3d() * duration.as_secs_f32());
        let (pos, _vel) = integrate_movement_packets(
            starting_pos,
            starting_velocity_inst,
            packets.iter().copied(),
        );
        assert!(expected_pos.abs_diff_eq(pos, accept_margin));
    }
}
