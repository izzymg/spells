use crate::input;
use bevy::{log, prelude::*};

pub struct FollowCameraPlugin;
impl Plugin for FollowCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sys_follow_camera_target,
                sys_follow_camera_look.after(input::InputSystemSet),
            ),
        );
    }
}

/// Marks this entity as being followed. Should have a transform.
#[derive(Component)]
pub struct FollowCameraTarget;

/// Tags a camera as capable of follow movement.
#[derive(Component, Debug)]
pub struct FollowCamera {
    pub look_sensitivity: f32,
    pub invert_pitch: bool,
    pub invert_yaw: bool,
    pub z_offset: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.5,
            invert_pitch: true,
            invert_yaw: false,
            yaw: 0.0,
            pitch: 0.0,
            z_offset: 5.0,
        }
    }
}

impl FollowCamera {
    /// Create a new camera with the applied rotations `yaw` and `pitch` in radians.
    pub fn new_with_angle(pitch: f32, yaw: f32) -> Self {
        Self {
            yaw,
            pitch,
            ..default()
        }
    }
}

fn sys_follow_camera_target(
    mut camera_query: Query<(&FollowCamera, &mut Transform), Without<FollowCameraTarget>>,
    target_query: Query<&Transform, With<FollowCameraTarget>>,
) {
    let (cam, mut cam_trans) = match camera_query.get_single_mut() {
        Ok((c, t)) => (c, t),
        _ => return,
    };
    let follow_trans = match target_query.get_single() {
        Ok(t) => t,
        _ => return,
    };

    cam_trans.translation = follow_trans.translation + (Vec3::Z * cam.z_offset);
}

fn sys_follow_camera_look(
    input_axes: Res<input::ActionAxes>,
    mut query: Query<(&mut Transform, &mut FollowCamera), Without<FollowCameraTarget>>,
    target_query: Query<&Transform, With<FollowCameraTarget>>
) {
    let (mut cam_trans, mut cam) = match query.get_single_mut() {
        Ok((ct, c)) => (ct, c),
        _ => return,
    };
    let follow_trans = match target_query.get_single() {
        Ok(t) => t,
        _ => return,
    };
    let mut delta_y = input_axes.look.y;
    if cam.invert_pitch {
        delta_y *= -1.0;
    }
    let mut delta_x = input_axes.look.x;
    if cam.invert_yaw {
        delta_x *= -1.0;
    }
    cam.pitch = (cam.pitch - (cam.look_sensitivity * delta_y)).clamp(-70.0, 70.0);
    cam.yaw -= cam.look_sensitivity * delta_x;
    cam_trans.translation = *Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, cam.yaw.to_radians()) * Quat::from_axis_angle(Vec3::X, cam.pitch.to_radians())).forward() * cam.z_offset;
    cam_trans.look_at(follow_trans.translation, Vec3::Y);
}
