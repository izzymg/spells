use bevy::{
    input::{
        mouse::{MouseButton, MouseMotion, MouseWheel},
        ButtonInput,
    },
    log,
    prelude::*,
};

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub(super) struct FreeCamera {
    pub speed: f32,
    pub invert_pitch: bool,
    pub invert_yaw: bool,
    yaw: f32,
    pitch: f32,
}

impl Default for FreeCamera {
    fn default() -> Self {
        Self {
            speed: 1.0,
            invert_pitch: true,
            invert_yaw: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl FreeCamera {
    pub fn new_with_angle(pitch: f32, yaw: f32) -> Self {
        Self {
            yaw,
            pitch,
            ..default()
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub(super) fn sys_free_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FreeCamera)>,
) {
    for ev in ev_motion.read() {
        let (mut cam_trans, mut cam) = query.single_mut();

        cam.pitch -= (cam.speed * ev.delta.y).clamp(-90.0, 90.0).to_radians();
        cam.yaw -= (cam.speed * ev.delta.x).to_radians();

        log::info!("{}, {}", cam.pitch.to_degrees(), cam.yaw.to_degrees());
        cam_trans.rotation =
            Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);
    }
}
