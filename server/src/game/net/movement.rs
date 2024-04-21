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

/// Given packets should belong to a single client, in order of their timestamp
/// todo: copying these shouldn't be expensive, but check
pub fn integrate_movement_packets(
    position: Vec3,
    velocity_instant: VelocityInstant,
    packets: impl Iterator<Item = (Instant, packet::MovementDirection)>,
) -> (Vec3, VelocityInstant) {
    packets.fold(
        (position, velocity_instant),
        |(pos, vel), (timestamp, dir)| {
            let passed = timestamp.saturating_duration_since(vel.timestamp);
            let dir = dir.to_3d();
            let n_pos = find_position(pos, dir, passed);
            let n_vel = VelocityInstant::new(timestamp, dir);
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
        let packets = [
            (Instant::now(), dir),
            (Instant::now(), packet::MovementDirection::Still),
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
