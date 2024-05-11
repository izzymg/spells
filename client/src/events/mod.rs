use bevy::prelude::*;
use lib_spells::net;
use crate::render::terrain;

/// World connected
#[derive(Debug, Event)]
pub struct ConnectedEvent;

/// New world state is available
#[derive(Debug, Event)]
pub struct WorldStateEvent {
    pub stamp: Option<u8>,
    pub state: net::WorldState,
    pub client_info: net::ClientInfo,
}

/// World disconnected
#[derive(Debug, Event)]
pub struct DisconnectedEvent(pub Option<String>);

/// Instruct a generation of the given terrain data
#[derive(Debug, Event)]
pub struct GenerateTerrainEvent {
    pub terrain: terrain::VoxelTerrain,
}

#[derive(Debug, Event)]
pub struct ReplicationCompleted;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<WorldStateEvent>>();
        app.add_event::<ConnectedEvent>();
        app.add_event::<DisconnectedEvent>();
        app.add_event::<GenerateTerrainEvent>();
        app.add_event::<ReplicationCompleted>();
    }
}
