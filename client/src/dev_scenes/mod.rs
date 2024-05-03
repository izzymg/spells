use crate::{controls::follow_cam, render};
use bevy::prelude::*;
const TERRAIN_SIZE: i32 = 30;

pub enum Scene {
    FollowCamera,
}

pub struct DevScenesPlugin {
    pub scene: Scene,
}

fn sys_dev_follow_cam(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_ev_w: EventWriter<render::GenerateTerrainEvent>,
) {
    commands.spawn((
        follow_cam::FollowCamera::default(),
        Camera3dBundle::default(),
    ));

    let target_mesh = meshes.add(Capsule3d::new(0.5, 1.75));
    let target_mat = mats.add(Color::WHITE);

    commands.spawn((
        PbrBundle {
            mesh: target_mesh,
            material: target_mat,
            ..default()
        },
        follow_cam::FollowCameraTarget,
    ));

    let mut terrain = render::VoxelTerrain::default();
    for x in 0..TERRAIN_SIZE {
        for y in 0..TERRAIN_SIZE {
            terrain.add(render::Voxel(x, 0, y));
        }
    }

    terrain_ev_w.send(render::GenerateTerrainEvent { terrain });
}

fn sys_move_dev_followed(
    time: Res<Time>,
    mut followed: Query<&mut Transform, With<follow_cam::FollowCameraTarget>>,
) {
    let mut trans = followed.single_mut();
    trans.translation += Vec3::Z * time.delta_seconds();
}

impl Plugin for DevScenesPlugin {
    fn build(&self, app: &mut App) {
        match self.scene {
            Scene::FollowCamera => {
                app.insert_state(crate::GameStates::Game);
                app.add_plugins(follow_cam::FollowCameraPlugin);
                app.add_systems(Startup, sys_dev_follow_cam);
                app.add_systems(Update, sys_move_dev_followed);
            }
        }
    }
}
