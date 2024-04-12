use bevy::prelude::*;

fn sys_create_camera_light() {
    let camera_and_light_transform =
        Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y);

    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    commands.spawn(PointLightBundle {
        transform: camera_and_light_transform,
        ..default()
    });
}

fn sys_create_mesh() -> Mesh {
    Mesh::new(PrimitiveTopo)
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {}
}
