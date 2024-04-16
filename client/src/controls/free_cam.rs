use bevy::prelude::*;

use crate::input;

pub struct FreeCameraPlugin;
impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sys_speed_camera, sys_free_camera_look, sys_free_camera_move)
                .after(input::InputSystemSet),
        );
    }
}

/// Tags a camera as capable of free movement.
#[derive(Component)]
pub struct FreeCamera {
    pub look_sensitivity: f32,
    pub move_speed: f32,
    pub invert_pitch: bool,
    pub invert_yaw: bool,
    yaw: f32,
    pitch: f32,
    speed: f32,
}

impl Default for FreeCamera {
    fn default() -> Self {
        Self {
            move_speed: 9.0,
            look_sensitivity: 0.5,
            invert_pitch: false,
            invert_yaw: false,
            yaw: 0.0,
            pitch: 0.0,
            speed: 1.0,
        }
    }
}

impl FreeCamera {
    /// Create a new camera with the applied rotations `yaw` and `pitch` in radians.
    pub fn new_with_angle(pitch: f32, yaw: f32) -> Self {
        Self {
            yaw,
            pitch,
            ..default()
        }
    }
}

fn sys_free_camera_look(
    input_axes: Res<input::InputAxes>,
    mut query: Query<(&mut Transform, &mut FreeCamera)>,
) {
    let (mut cam_trans, mut cam) = query.single_mut();
    let mut delta_y = input_axes.look.y;
    if cam.invert_pitch {
        delta_y *= -1.0;
    }
    let mut delta_x = input_axes.look.x;
    if cam.invert_yaw {
        delta_x *= -1.0;
    }
    cam.pitch -= (cam.look_sensitivity * delta_y)
        .clamp(-90.0, 90.0)
        .to_radians();
    cam.yaw -= (cam.look_sensitivity * delta_x).to_radians();

    cam_trans.rotation =
        Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);
}

fn sys_free_camera_move(
    input_axes: Res<input::InputAxes>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FreeCamera)>,
    mut gizmos: Gizmos,
) {
    if input_axes.movement.length() <= 0.0 {
        return;
    }
    let (mut cam_trans, cam) = query.single_mut();
    let tr = cam_trans.rotation
        * input_axes.get_movement_3d().normalize()
        * cam.speed
        * time.delta_seconds();
    cam_trans.translation += tr;
    gizmos.ray(cam_trans.translation, tr, Color::BLUE);
}

fn sys_speed_camera(
    mut buttons_evr: EventReader<input::ActionEvent<input::ButtonAction>>,
    mut query: Query<&mut FreeCamera>,
) {
    let mut cam = query.single_mut();
    cam.speed = cam.move_speed;
    for ev in buttons_evr.read() {
        if ev.action == input::Action::Secondary {
            cam.speed = cam.move_speed * 3.0;
        }
    }
}
