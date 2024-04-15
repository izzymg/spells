use crate::{GameState, GameStates};
use bevy::{
    ecs::system::SystemParam,
    log,
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

const VOXEL_SIZE: i32 = 1;

pub struct Voxel(pub i32, pub i32, pub i32);

#[derive(Resource)]
pub struct VoxelTerrain(pub Vec<Voxel>);

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

#[derive(Resource, Default)]
struct TerrainAssets {
    quad_mesh: Handle<Mesh>,
    default_mat: Handle<StandardMaterial>,
    blue_mat: Handle<StandardMaterial>,
}

/// Load & create the terrain asset data
fn sys_populate_assets_ev(
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut terrain_assets: ResMut<TerrainAssets>,
) {
    log::info!("creating assets");
    terrain_assets.quad_mesh = mesh_assets.add(create_quad());
    terrain_assets.default_mat = material_assets.add(StandardMaterial {
        base_color: Color::RED,
        ..default()
    });
    terrain_assets.blue_mat = material_assets.add(StandardMaterial {
        base_color: Color::BLUE,
        ..default()
    });
}

/// Instruct a generation of the given terrain data
#[derive(Event)]
pub struct GenerateTerrainEvent {
    pub terrain: VoxelTerrain,
}

#[derive(SystemParam)]
struct TerrainGenerationSysParams<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, TerrainAssets>,
}

impl<'w, 's> TerrainGenerationSysParams<'w, 's> {
    fn spawn_quad(&mut self, transform: Transform, blue: bool) -> Entity {
        let mat = if blue {
            self.assets.blue_mat.clone()
        } else {
            self.assets.default_mat.clone()
        };
        self.commands
            .spawn((
                PbrBundle {
                    mesh: self.assets.quad_mesh.clone(),
                    material: mat,
                    transform,
                    ..default()
                },
                CustomUV,
            ))
            .id()
    }
}

fn sys_generate_terrain(
    mut sys_params: TerrainGenerationSysParams,
    mut voxel_terrain_ev: EventReader<GenerateTerrainEvent>,
) {
    for ev in voxel_terrain_ev.read() {
        log::info!("regenerating terrain");
        // spawn quads at each voxel position
        for i in ev.terrain.0.iter() {
            // facing -z
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_y((180.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing +z
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_y((0.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing -y
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_x((90.0_f32).to_radians());
                sys_params.spawn_quad(tr, true);
            }
            // facing +y
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_x((270.0_f32).to_radians());
                sys_params.spawn_quad(tr, true);
            }
            // facing +x
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_y((90.0_f32).to_radians());
                sys_params.spawn_quad(tr, true);
            }
            // facing -x
            {
                let mut tr = Transform::from_xyz((i.0 * VOXEL_SIZE) as f32, 0.0, 0.0);
                tr.rotate_y((270.0_f32).to_radians());
                sys_params.spawn_quad(tr, true);
            }
        }
    }
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TerrainAssets::default());
        app.add_event::<GenerateTerrainEvent>();
        app.add_systems(Startup, sys_populate_assets_ev);
        app.add_systems(Update, sys_generate_terrain);
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
