/*! Dev scene: testing "follow camera" */
use crate::{cameras::follow_cam, GameStates};
use bevy::prelude::*;
const CAPSULE_HEIGHT: f32 = 1.75;
const MOVE_SPEED: f32 = 3.;

fn sys_create_scene(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        follow_cam::FollowCamera::default(),
        Camera3dBundle::default(),
    ));

    // target
    let target_mesh = meshes.add(Capsule3d::new(0.5, CAPSULE_HEIGHT));
    let target_mat = mats.add(Color::WHITE);
    commands.spawn((
        PbrBundle {
            mesh: target_mesh,
            material: target_mat,
            transform: Transform::from_translation(Vec3::new(0., CAPSULE_HEIGHT / 2., 0.)),
            ..default()
        },
        follow_cam::FollowCameraTarget,
    ));

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 100.0)),
        material: mats.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 150.,
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(1.0, 2.0, 0.0),
        point_light: PointLight {
            intensity: 100_000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

fn sys_move_followed(
    time: Res<Time>,
    mut followed: Query<&mut Transform, With<follow_cam::FollowCameraTarget>>,
) {
    let mut trans = followed.single_mut();
    trans.translation += Vec3::Z * MOVE_SPEED * time.delta_seconds();
}

pub struct FollowCamDevScenePlugin;

impl Plugin for FollowCamDevScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameStates::Game);
        app.add_plugins(follow_cam::FollowCameraPlugin);
        app.add_systems(Startup, sys_create_scene);
        app.add_systems(Update, sys_move_followed);
    }
}
