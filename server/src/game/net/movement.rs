/*! movement of network objects */
use crate::game::net::packet;
use bevy::prelude::*;
use std::time::{Duration, Instant};

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

#[derive(Debug, Copy, Clone)]
pub struct MovementPacket {
    pub timestamp: Instant,
    pub direction: packet::MovementDirection,
}

impl MovementPacket {
    pub fn from_packet(packet: packet::Packet) -> Option<Self> {
        if let packet::PacketData::Movement(dir) = packet.data {
            Some(Self {
                timestamp: packet.timestamp,
                direction: dir,
            })
        } else {
            None
        }
    }
}

pub fn integrate_movement_packets(
    init_pos: Vec3,
    init_vel_i: VelocityInstant,
    packets: impl Iterator<Item = MovementPacket>,
) -> (Vec3, VelocityInstant) {
    packets
        .fold(
        (init_pos, init_vel_i),
        |(pos, vel_i), packet| {
            let passed = packet.timestamp.saturating_duration_since(vel_i.timestamp);
            let dir = packet.direction.to_3d();
            let n_pos = find_position(pos, dir, passed);
            let n_vel = VelocityInstant::new(packet.timestamp, dir);
            (n_pos, n_vel)
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;

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

    #[test]
    #[ignore] // `Instant` is opaque, thread sleeping test
    fn test_integrate_velocity() {
        let accept_margin = 0.001;
        let starting_velocity_inst = VelocityInstant::new(Instant::now(), Vec3::ZERO);
        let dir = packet::MovementDirection::Forward;
        let starting_pos = Vec3::new(1.0, 5.0, 2.0);
        let duration = Duration::from_millis(3500);

        // order important here
        thread::sleep(duration);
        let now = Instant::now();
        let packets = [
            MovementPacket {
                timestamp: now, 
                direction: dir,
            },
            MovementPacket {
                timestamp: now,
                direction: packet::MovementDirection::Still,
            }
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
