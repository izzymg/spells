use crate::{SystemSets, input};
use bevy::prelude::*;

pub struct FollowCameraPlugin;
impl Plugin for FollowCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sys_follow_camera_look, sys_camera_input)
                .chain()
                .in_set(SystemSets::Controls),
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
            invert_pitch: false,
            invert_yaw: false,
            yaw: 0.0,
            pitch: 0.0,
            z_offset: 15.0,
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

fn sys_camera_input(
    input_axes: Res<input::ActionAxes>,
    mut query: Query<&mut FollowCamera, Without<FollowCameraTarget>>,
) {
    let mut cam = match query.get_single_mut() {
        Ok(ct) => ct,
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
}

fn sys_follow_camera_look(
    mut query: Query<(&mut Transform, &mut FollowCamera), Without<FollowCameraTarget>>,
    target_query: Query<&Transform, With<FollowCameraTarget>>,
) {
    let (mut cam_trans, cam) = match query.get_single_mut() {
        Ok((ct, c)) => (ct, c),
        _ => return,
    };
    let follow_trans = match target_query.get_single() {
        Ok(t) => t,
        _ => return,
    };

    let rot = Transform::from_rotation(
        Quat::from_axis_angle(Vec3::Y, cam.yaw.to_radians())
            * Quat::from_axis_angle(Vec3::X, cam.pitch.to_radians()),
    );

    cam_trans.translation = follow_trans.translation + (rot.back() * cam.z_offset);
    cam_trans.look_at(follow_trans.translation, Vec3::Y);
}
