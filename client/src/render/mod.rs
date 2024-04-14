use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
mod orbit_cam;

const VOXEL_SIZE: i32 = 1;

struct Voxel(i32, i32, i32);

#[derive(Resource)]
struct VoxelTerrain(Vec<Voxel>);

impl VoxelTerrain {
    fn neighbor_right(&self, index: usize) -> bool {
        self.neighbor_x(index, 1)
    }
    fn neighbor_left(&self, index: usize) -> bool {
        self.neighbor_x(index, -1)
    }
    fn neighbor_x(&self, index: usize, dir: i32) -> bool {
        let uv = self.0.get(index).unwrap();
        self.0
            .iter()
            .filter(|v| if dir < 0 { v.0 < uv.0 } else { v.0 > uv.0 })
            .any(|v| v.0 - uv.0 == dir)
    }
}

#[derive(Component)]
struct CustomUV;

fn create_quad() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![0, 3, 1, 1, 3, 2]))
}

fn sys_create_camera_light(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_terrain: Res<VoxelTerrain>,
) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, -10.0),
        ..default()
    }, orbit_cam::FreeCamera::new_with_angle(-53.0_f32.to_radians(), 180.0_f32.to_radians())));

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 1.0, -1.0).looking_at(Vec3::Z, Vec3::Y),
        ..default()
    });

    let mesh_handle: Handle<Mesh> = meshes.add(create_quad());

    // spawn quads at each voxel position
    for i in voxel_terrain.0.iter() {
        // facing -z
        {
            let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
            tr.rotate_y((180.0_f32).to_radians());
            commands.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::RED,
                        ..default()
                    }),
                    transform: tr,
                    ..default()
                },
                CustomUV,
            ));
        }// facing y
        {
            let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
            tr.rotate_y((180.0_f32).to_radians());
            tr.rotate_x((90.0_f32).to_radians());
            commands.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::BLUE,
                        ..default()
                    }),
                    transform: tr,
                    ..default()
                },
                CustomUV,
            ));
        }
    }
}
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, orbit_cam::sys_free_camera);
        app.add_systems(Startup, sys_create_camera_light);
        app.insert_resource(VoxelTerrain(vec![
            Voxel(-4, 0, 0),
            Voxel(-3, 0, 0),
            Voxel(-2, 0, 0),
            Voxel(-1, 0, 0),
            Voxel(0, 0, 0),
            Voxel(1, 0, 0),
            Voxel(2, 0, 0),
            Voxel(3, 0, 0),
            Voxel(4, 0, 0),
        ]));
    }
}

#[cfg(test)]
mod tests {

    use super::{Voxel, VoxelTerrain};
    #[test]
    fn test_neighbors() {
        let voxel_terrain = VoxelTerrain(vec![
            Voxel(-1, 0, 0),
            Voxel(0, 0, 0),
            Voxel(1, 0, 0),
            Voxel(2, 0, 0),
            Voxel(3, 0, 0),
            Voxel(5, 0, 0),
        ]);

        assert!(voxel_terrain.neighbor_right(0));
        assert!(!voxel_terrain.neighbor_left(0));
        assert!(voxel_terrain.neighbor_right(0) && !voxel_terrain.neighbor_left(0));
        assert!(voxel_terrain.neighbor_right(1) && voxel_terrain.neighbor_left(1));
        assert!(voxel_terrain.neighbor_right(2) && voxel_terrain.neighbor_left(2));
        assert!(voxel_terrain.neighbor_right(3) && voxel_terrain.neighbor_left(3));
        assert!(!voxel_terrain.neighbor_right(4) && voxel_terrain.neighbor_left(4));
        assert!(!voxel_terrain.neighbor_right(5) && !voxel_terrain.neighbor_left(5));
    }
}
