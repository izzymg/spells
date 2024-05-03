use crate::terrain;
use bevy::log;
use bevy::prelude::*;

pub fn sys_create_map(mut terrain_event_send: EventWriter<terrain::GenerateTerrainEvent>) {
    log::info!("creating map");

    let mut terrain = terrain::VoxelTerrain::default();
    for x in 0..50 {
        for y in 0..25 {
            terrain.0.push(terrain::Voxel(x, 0, y));
        }
    }
    terrain_event_send.send(terrain::GenerateTerrainEvent { terrain });
}

