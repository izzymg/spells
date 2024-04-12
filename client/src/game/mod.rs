use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, log, prelude::*, reflect::Map};
use lib_spells::net;

use crate::{world_connection, GameState, GameStates};

//// 00000000000000000000000000000000000000000
//// 00000000000000000000000000000000000000000
//// 0000 WE LOVE CASTING SPELLS AND SHIT 0000
//// 00000000000000000000000000000000000000000
//// 00000000000000000000000000000000000000000

/// Maps server entities to our entities
#[derive(Resource, Debug, Default)]
struct ServerClientMap(HashMap<u32, Entity>);

/// Marker
#[derive(Component, Debug, Default)]
struct GameObject;

#[derive(Bundle, Default)]
struct GameCameraBundle {
    go: GameObject,
    cam: Camera3dBundle,
}
impl GameCameraBundle {
    fn new() -> Self {
        Self {
            go: GameObject,
            cam: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 12.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
        }
    }
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

#[derive(SystemParam)]
struct WorldStateSysParam<'w, 's> {
    commands: Commands<'w, 's>,
}

impl<'w, 's> WorldStateSysParam<'w, 's> {
    fn push_state(&mut self, entity: Entity, state: &Option<impl Component + Clone>) {
        if let Some(s) = state {
            self.commands.entity(entity).insert(s.clone());
        }
    }

    // todo: probably just re-export EntityState from world_conn tbh
    /// Replicate some `serialization::EntityState` onto the given `Entity`.
    fn push_entity_state(&mut self, entity: Entity, state: &net::EntityState) {
        self.push_state(entity, &state.aura);
        self.push_state(entity, &state.health);
        self.push_state(entity, &state.spellcaster);
        self.push_state(entity, &state.casting_spell);
    }

    fn spawn_gameobject(&mut self) -> Entity {
        self.commands.spawn(GameObject).id()
    }

    fn despawn(&mut self, entity: Entity) {
        self.commands.entity(entity).despawn_recursive();
    }
}

/// Handle replication & syncing of world state to this game's world.
fn sys_handle_world_state(
    mut sys_params: WorldStateSysParam,
    mut server_client_map: ResMut<ServerClientMap>,
    world_conn: Res<world_connection::WorldConnection>,
) {
    if !world_conn.is_changed() {
        return;
    }
    if let Some(world_state) = &world_conn.cached_state {
        log::debug!("game processing world state");

        if let Some(world_change) = &world_conn.state_change {
            for (server_entity, is_new_entity) in world_change.new_server_keys.iter().copied() {
                let client_entity = if is_new_entity {
                    sys_params.spawn_gameobject()
                } else {
                    *server_client_map
                        .0
                        .get(&server_entity)
                        .expect("client server map should hold existing server entities")
                };
                sys_params.push_entity_state(
                    client_entity,
                    world_state
                        .entity_state_map
                        .get(&server_entity)
                        .expect("any key in `new server keys` should be in `cached state`"),
                );
                log::debug!("updated entity {:?} with new state", client_entity);
            }

            for dead_entity in world_change.lost_server_keys.iter() {
                let client_entity = server_client_map
                    .0
                    .remove(dead_entity)
                    .expect("expected lost server key in client server map");
                log::debug!("lost {:?}, despawned", client_entity);
                sys_params.despawn(client_entity);
            }
        } else {
            // we just want to spawn everything in here
            log::debug!("this is a first time world gen");
            for (server_entity, state) in world_state.entity_state_map.iter() {
                let new_entity = sys_params.spawn_gameobject();
                server_client_map.0.insert(*server_entity, new_entity);
                sys_params.push_entity_state(new_entity, state);
                log::debug!("spawned {:?} with state: {:?}", new_entity, state);
            }
        }
    }
}

/// Spawn in the basic game world objects when the game state changes.
fn sys_spawn_game_world(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut server_client_map: ResMut<ServerClientMap>,
) {
    if !(game_state.is_changed() && game_state.0 == GameStates::Game) {
        return;
    }
    log::info!("spawning game world");
    commands.spawn(GameCameraBundle::new());
    server_client_map.0.clear();
}

fn sys_cleanup_game_world(
    mut commands: Commands,
    game_state: Res<GameState>,
    go_query: Query<Entity, With<GameObject>>,
    mut server_client_map: ResMut<ServerClientMap>,
) {
    if !(game_state.is_changed() && game_state.0 != GameStates::Game) {
        return;
    }
    log::info!("cleaning up game world");
    for entity in go_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    server_client_map.0.clear();
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerClientMap::default());
        app.add_systems(
            Update,
            (
                sys_spawn_game_world,
                (
                    sys_handle_world_messages,
                    sys_handle_world_state,
                    sys_cleanup_game_world,
                )
                    .chain(),
            )
                .in_set(GameStates::Game),
        );
    }
}
