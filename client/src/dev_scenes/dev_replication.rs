use crate::{controls::cameras::follow_cam, replication, world_connection};
use bevy::{input::ButtonInput, prelude::*};

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
        replication::WishDir::default(),
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

/// Map keys to wish dir on predicted player
fn sys_set_wish_dir(
    mut wish_dir_q: Query<&mut replication::WishDir>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut dir = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        dir.z = -1.;
    }
    if input.pressed(KeyCode::KeyS) {
        dir.z = 1.;
    }
    if input.pressed(KeyCode::KeyA) {
        dir.x = -1.;
    }
    if input.pressed(KeyCode::KeyD) {
        dir.x = 1.;
    }
    wish_dir_q.single_mut().set_if_neq(replication::WishDir(dir));
}

pub struct ReplicationDevScenePlugin;

impl Plugin for ReplicationDevScenePlugin {
    fn build(&self, app: &mut App) {
        // bypass main menu
        app.add_plugins((
            world_connection::WorldConnectionPlugin,
            replication::ReplicationPlugin,
            follow_cam::FollowCameraPlugin,
        ));
        app.add_systems(Startup, sys_build_replication_dev_scenes);
        app.add_systems(Update, sys_set_wish_dir);
    }
}
