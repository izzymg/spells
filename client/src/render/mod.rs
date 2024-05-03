use bevy::{
    ecs::system::SystemParam,
    log,
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use serde::{Deserialize, Serialize};
use std::fmt;

// Size in world space of voxels
pub const VOXEL_SIZE: i32 = 1;

#[derive(Serialize, Deserialize, Default, Copy, Clone, PartialEq, Eq)]
pub struct Voxel(pub i32, pub i32, pub i32);

impl From<(i32, i32, i32)> for Voxel {
    fn from(v: (i32, i32, i32)) -> Self {
        Self(v.0, v.1, v.2)
    }
}

impl From<Vec3> for Voxel {
    fn from(value: Vec3) -> Self {
        Self(value.x as i32, value.y as i32, value.z as i32)
    }
}

impl From<Voxel> for Vec3 {
    fn from(value: Voxel) -> Self {
        Self::new(value.0 as f32, value.1 as f32, value.2 as f32)
    }
}

impl fmt::Display for Voxel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "voxel: {}, {}, {}", self.0, self.1, self.2)
    }
}

#[derive(Resource, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct VoxelTerrain(pub Vec<Voxel>);

pub enum Direction {
    Up,
    Left,
    Down,
    Right,
    Forward,
    Backward,
}

impl VoxelTerrain {
    pub fn add(&mut self, v: Voxel) {
        self.0.push(v);
    }

    pub fn remove(&mut self, index: usize) -> Voxel {
        self.0.remove(index)
    }

    pub fn find(&self, pos: Voxel) -> Option<usize> {
        for (i, v) in self.0.iter().enumerate() {
            if *v == pos {
                return Some(i);
            }
        }
        None
    }

    pub fn has_neighbor(&self, index: usize, dir: Direction) -> bool {
        match dir {
            Direction::Up => self.neighbor_y(index, 1),
            Direction::Down => self.neighbor_y(index, -1),
            Direction::Left => self.neighbor_x(index, -1),
            Direction::Right => self.neighbor_x(index, 1),
            Direction::Forward => self.neighbor_z(index, 1),
            Direction::Backward => self.neighbor_z(index, -1),
        }
    }

    fn neighbor_x(&self, index: usize, dir: i32) -> bool {
        let uv = self.0.get(index).unwrap();
        self.0
            .iter()
            .filter(|v| {
                let in_range = if dir < 0 { v.0 < uv.0 } else { v.0 > uv.0 };
                in_range && (v.1 == uv.1 && v.2 == uv.2)
            })
            .any(|v| v.0 - uv.0 == dir)
    }
    fn neighbor_y(&self, index: usize, dir: i32) -> bool {
        let uv = self.0.get(index).unwrap();
        self.0
            .iter()
            .filter(|v| {
                let in_range = if dir < 0 { v.1 < uv.1 } else { v.1 > uv.1 };
                in_range && (v.0 == uv.0 && v.2 == uv.2)
            })
            .any(|v| v.1 - uv.1 == dir)
    }
    fn neighbor_z(&self, index: usize, dir: i32) -> bool {
        let uv = self.0.get(index).unwrap();
        self.0
            .iter()
            .filter(|v| {
                let in_range = if dir < 0 { v.2 < uv.2 } else { v.2 > uv.2 };
                in_range && (v.0 == uv.0 && v.1 == uv.1)
            })
            .any(|v| v.2 - uv.2 == dir)
    }
}

#[derive(Component)]
struct VoxelEntity;

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
    query_voxels: Query<'w, 's, Entity, With<VoxelEntity>>,
}

impl<'w, 's> TerrainGenerationSysParams<'w, 's> {
    fn despawn_all_voxels(&mut self) {
        for entity in &self.query_voxels {
            self.commands.entity(entity).despawn_recursive();
        }
    }
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
                VoxelEntity,
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

        sys_params.despawn_all_voxels();

        // spawn quads at each voxel position
        for (index, i) in ev.terrain.0.iter().enumerate() {
            let (x, y, z): (f32, f32, f32) = (
                (i.0 * VOXEL_SIZE) as f32,
                (i.1 * VOXEL_SIZE) as f32,
                (i.2 * VOXEL_SIZE) as f32,
            );
            let tr = Transform::from_xyz(x, y, z);
            // facing -z
            if !ev.terrain.has_neighbor(index, Direction::Backward) {
                let mut tr = tr;
                tr.rotate_y((180.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing +z
            if !ev.terrain.has_neighbor(index, Direction::Forward) {
                let mut tr = tr;
                tr.rotate_y((0.0_f32).to_radians());
                sys_params.spawn_quad(tr, true);
            }
            // facing -y
            if !ev.terrain.has_neighbor(index, Direction::Down) {
                let mut tr = tr;
                tr.rotate_x((90.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing +y
            if !ev.terrain.has_neighbor(index, Direction::Up) {
                let mut tr = tr;
                tr.rotate_x((270.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing +x
            if !ev.terrain.has_neighbor(index, Direction::Right) {
                let mut tr = tr;
                tr.rotate_y((90.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
            }
            // facing -x
            if !ev.terrain.has_neighbor(index, Direction::Left) {
                let mut tr = tr;
                tr.rotate_y((270.0_f32).to_radians());
                sys_params.spawn_quad(tr, false);
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

    use super::{Direction, Voxel, VoxelTerrain};
    #[test]
    fn test_x_neighbors() {
        let voxel_terrain = VoxelTerrain(vec![
            Voxel(-1, 0, 0), // 0
            Voxel(0, 0, 0),  // 1
            Voxel(1, 0, 0),  // 2
            Voxel(5, 0, 0),  // 3
        ]);

        assert!(
            voxel_terrain.has_neighbor(0, Direction::Right)
                && !voxel_terrain.has_neighbor(0, Direction::Left)
        );
        assert!(
            voxel_terrain.has_neighbor(1, Direction::Right)
                && voxel_terrain.has_neighbor(1, Direction::Left)
        );
        assert!(
            !voxel_terrain.has_neighbor(2, Direction::Right)
                && voxel_terrain.has_neighbor(2, Direction::Left)
        );
        assert!(
            !voxel_terrain.has_neighbor(3, Direction::Right)
                && !voxel_terrain.has_neighbor(3, Direction::Left)
        );
    }

    #[test]
    fn test_y_neighbors() {
        let voxel_terrain = VoxelTerrain(vec![
            Voxel(0, -1, 0), // 0
            Voxel(0, 0, 0),  // 1
            Voxel(0, 1, 0),  // 2
            Voxel(0, 5, 0),  // 3
        ]);

        assert!(
            voxel_terrain.has_neighbor(0, Direction::Up)
                && !voxel_terrain.has_neighbor(0, Direction::Down)
        );
        assert!(
            voxel_terrain.has_neighbor(1, Direction::Up)
                && voxel_terrain.has_neighbor(1, Direction::Down)
        );
        assert!(
            !voxel_terrain.has_neighbor(2, Direction::Up)
                && voxel_terrain.has_neighbor(2, Direction::Down)
        );
        assert!(
            !voxel_terrain.has_neighbor(3, Direction::Down)
                && !voxel_terrain.has_neighbor(3, Direction::Up)
        );
    }

    #[test]
    fn test_z_neighbors() {
        let voxel_terrain = VoxelTerrain(vec![
            Voxel(0, 0, -1), // 0
            Voxel(0, 0, 0),  // 1
            Voxel(0, 0, 1),  // 2
            Voxel(0, 0, 5),  // 3
        ]);

        assert!(
            voxel_terrain.has_neighbor(0, Direction::Forward)
                && !voxel_terrain.has_neighbor(0, Direction::Backward)
        );
        assert!(
            voxel_terrain.has_neighbor(1, Direction::Forward)
                && voxel_terrain.has_neighbor(1, Direction::Backward)
        );
        assert!(
            !voxel_terrain.has_neighbor(2, Direction::Forward)
                && voxel_terrain.has_neighbor(2, Direction::Backward)
        );
        assert!(
            !voxel_terrain.has_neighbor(3, Direction::Forward)
                && !voxel_terrain.has_neighbor(3, Direction::Backward)
        );
    }

    // diagonals should not be returned as neighbors
    #[test]
    fn test_diagonal_neighbors() {
        {
            let voxel_terrain = VoxelTerrain(vec![
                Voxel(0, 0, 0),
                Voxel(-1, 0, 1),  // left back
                Voxel(-1, 0, -1), // left front
                Voxel(1, 0, 1),   // right front
                Voxel(1, 0, -1),  // right back
                Voxel(1, 1, 0),   // left top
                Voxel(-1, 1, 0),  // right top
                Voxel(0, 1, 1),   // forward top
                Voxel(0, 1, -1),  // backward top
                Voxel(-1, -1, 0), // left bottom
                Voxel(1, -1, 0),  // right bottom
                Voxel(0, -1, 1),  // forward bottom
                Voxel(0, -1, -1), // backward bottom
            ]);

            // no dang neighbors, all alone
            assert!(!voxel_terrain.has_neighbor(0, Direction::Forward));
            assert!(!voxel_terrain.has_neighbor(0, Direction::Backward));
            assert!(!voxel_terrain.has_neighbor(0, Direction::Up));
            assert!(!voxel_terrain.has_neighbor(0, Direction::Down));
            assert!(!voxel_terrain.has_neighbor(0, Direction::Left));
            assert!(!voxel_terrain.has_neighbor(0, Direction::Right));
            // she's just like me fr
        }
    }
}
