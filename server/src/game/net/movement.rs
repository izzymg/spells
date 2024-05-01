/*! movement of network objects */
use crate::game::net::packet;
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Calculate a new position based on a velocity that was `time_passed` ago
fn find_position(position: Vec3, velocity: Vec3, time_passed: Duration) -> Vec3 {
    position + (velocity * time_passed.as_secs_f32())
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
    init_vel: Vec3,
    init_t: Instant,
    packets: impl Iterator<Item = MovementPacket>,
) -> (Vec3, Vec3, Instant) {
    packets
        .fold(
        (init_pos, init_vel, init_t),
        |(pos, vel, t), packet| {
            let passed = packet.timestamp.saturating_duration_since(t);
            let n_pos = find_position(pos, vel, passed);
            (n_pos, packet.direction.to_3d(), packet.timestamp)
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
        let dir = packet::MovementDirection::Forward;
        let start_time = Instant::now();
        let starting_pos = Vec3::new(1.0, 5.0, 2.0);
        let duration = Duration::from_millis(3500);

        // order important here
        thread::sleep(duration);
        let now = Instant::now();
        let packets = [
            MovementPacket {
                timestamp: start_time, 
                direction: dir,
            },
            MovementPacket {
                timestamp: now,
                direction: packet::MovementDirection::Still,
            }
        ];

        let expected_pos = starting_pos + (dir.to_3d() * duration.as_secs_f32());
        let (pos, _vel, _inst) = integrate_movement_packets(
            starting_pos,
            Vec3::ZERO,
            start_time,
            packets.iter().copied(),
        );
        assert!(expected_pos.abs_diff_eq(pos, accept_margin));
        dbg!(pos, expected_pos);
    }
}
