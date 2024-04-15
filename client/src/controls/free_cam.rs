use bevy::{
    input::{mouse::MouseMotion, ButtonInput},
    log,
    prelude::*,
};

pub struct FreeCameraPlugin;
impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sys_free_camera_look, sys_free_camera_move));
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
}

impl Default for FreeCamera {
    fn default() -> Self {
        Self {
            move_speed: 9.0,
            look_sensitivity: 0.2,
            invert_pitch: false,
            invert_yaw: false,
            yaw: 0.0,
            pitch: 0.0,
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

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn sys_free_camera_look(
    mut ev_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FreeCamera)>,
) {
    for ev in ev_motion.read() {
        let (mut cam_trans, mut cam) = query.single_mut();
        let mut delta_y = ev.delta.y;
        if cam.invert_pitch {
            delta_y *= -1.0;
        }
        let mut delta_x = ev.delta.x;
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
}

fn sys_free_camera_move(
    buttons: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FreeCamera)>,
    mut gizmos: Gizmos,
) {
    let mut wish_dir = Vec3::ZERO;
    if buttons.pressed(KeyCode::KeyW) {
        wish_dir.z -= 1.0;
    }

    if buttons.pressed(KeyCode::KeyS) {
        wish_dir.z += 1.0;
    }

    if buttons.pressed(KeyCode::KeyD) {
        wish_dir.x += 1.0;
    }

    if buttons.pressed(KeyCode::KeyA) {
        wish_dir.x -= 1.0;
    }

    if wish_dir.length() <= 0.0 {
        return;
    }

    let (mut cam_trans, cam) = query.single_mut();
    let speed = if mouse_buttons.pressed(MouseButton::Right) {
        cam.move_speed * 3.0
    } else {
        cam.move_speed
    };

    let tr = cam_trans.rotation * (wish_dir).normalize() * speed * time.delta_seconds();
    cam_trans.translation += tr;
    gizmos.ray(cam_trans.translation, tr, Color::BLUE);
}
