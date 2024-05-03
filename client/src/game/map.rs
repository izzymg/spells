use crate::{controls, game::GameObject, render, world_connection};
use bevy::log;
use bevy::prelude::*;

pub fn sys_create_map(mut terrain_event_send: EventWriter<render::GenerateTerrainEvent>) {
    log::info!("creating map");

    let mut terrain = render::VoxelTerrain::default();
    for x in 0..50 {
        for y in 0..25 {
            terrain.0.push(render::Voxel(x, 0, y));
        }
    }
    terrain_event_send.send(render::GenerateTerrainEvent { terrain });
}

