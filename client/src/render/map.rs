use crate::{events, render::terrain};
use bevy::prelude::*;

pub fn sys_create_map(mut terrain_event_send: EventWriter<events::GenerateTerrainEvent>) {
    let mut terrain = terrain::VoxelTerrain::default();
    for x in 0..50 {
        for y in 0..25 {
            terrain.0.push(terrain::Voxel(x, 0, y));
        }
    }
    terrain_event_send.send(events::GenerateTerrainEvent { terrain });
}

