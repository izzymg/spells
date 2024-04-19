/*! movement of network objects */
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Calculate a new position based on a velocity that was `time_passed` ago
pub(super) fn find_position(position: Vec3, velocity: Vec3, time_passed: Duration) -> Vec3 {
    position + (velocity * time_passed.as_secs_f32())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_position() {
        let position = Vec3::new(3.0, 4.0, 0.0);
        let velocity = Vec3::Y * 2.0;

        let time_passed = Duration::from_secs_f32(2.5);

        assert_eq!(find_position(position, velocity, time_passed), position + (velocity * 2.5));

    }
}
