use std::collections::HashMap;

use bevy::{ecs::system::SystemId, log, prelude::*};

use crate::{world_connection, GameState, GameStates};

/// Maps server entities to our entities
#[derive(Resource, Debug, Default)]
struct ServerClientMap(HashMap<Entity, Entity>);

/// Marker
#[derive(Component, Debug, Default)]
struct GameObject;

#[derive(Bundle, Default)]
struct GameCameraBundle {
    go: GameObject,
    bundle: Camera3dBundle,
}

fn sys_handle_world_messages(
    mut game_state: ResMut<GameState>,
    world_conn: Res<world_connection::WorldConnection>,
) {
    // todo: conn and message might need to be different resources
    if world_conn.is_changed() {
        if let Some(world_connection::WorldConnectionMessage::Error(err)) = &world_conn.message {
            log::info!("kicking back to menu: {}", err);
            game_state.0 = GameStates::Menu;
        }
    }
}

fn sys_handle_world_state(
    mut commands: Commands,
    mut server_client_map: ResMut<ServerClientMap>,
    world_state: Res<world_connection::WorldConnection>,
) {
}

fn sys_spawn_game_world(mut commands: Commands, game_state: Res<GameState>) {
    if !(game_state.is_changed() && game_state.0 == GameStates::Game) {
        return;
    }
    log::info!("spawning game world");
    commands.spawn(GameCameraBundle::default());
}

fn sys_cleanup_game_world(
    mut commands: Commands,
    game_state: Res<GameState>,
    go_query: Query<Entity, With<GameObject>>,
) {
    if !(game_state.is_changed() && game_state.0 != GameStates::Game) {
        return;
    }
    log::info!("cleaning up game world");
    for entity in go_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerClientMap::default());
        app.add_systems(
            Update,
            (
                sys_spawn_game_world,
                sys_handle_world_messages,
                sys_handle_world_state,
                sys_cleanup_game_world,
            )
                .in_set(GameStates::Game),
        );
    }
}
