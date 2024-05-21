use crate::{controls::{wish_dir, cameras::follow_cam}, input, replication, world_connection};
use bevy::prelude::*;

const CAPSULE_HEIGHT: f32 = 1.65;

fn sys_build_replication_dev_scenes(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // camera
    commands.spawn((
        follow_cam::FollowCamera::default(),
        Camera3dBundle::default(),
    ));

    // replication spawns a predicted player while in LoadGame state after it receives the first world state.
    // we'll spawn one ourselves as we skip straight into Game
    let target_mesh = meshes.add(Capsule3d::new(0.5, CAPSULE_HEIGHT));
    let target_mat = mats.add(Color::WHITE);
    commands.spawn((
        PbrBundle {
            mesh: target_mesh,
            material: target_mat,
            transform: Transform::from_translation(Vec3::new(0., CAPSULE_HEIGHT, 0.)),
            ..default()
        },
        follow_cam::FollowCameraTarget,
        replication::PredictedPlayer,
    ));

    // origin cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::default().mesh()),
        material: mats.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
        material: mats.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 850.,
    });
}

pub struct ReplicationDevScenePlugin;

impl Plugin for ReplicationDevScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            world_connection::WorldConnectionPlugin,
            replication::ReplicationPlugin,
            follow_cam::FollowCameraPlugin,
            wish_dir::WishDirPlugin,
        ));
        app.add_systems(Startup, sys_build_replication_dev_scenes);
    }
}
