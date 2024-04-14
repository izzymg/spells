/*!
    WE LOVE CASTING SPELLS
*/

use bevy::{
    ecs::{
        entity::{EntityHashMap, MapEntities},
        system::SystemParam,
    },
    log,
    prelude::*,
};
use lib_spells::net;

use crate::{world_connection, GameState, GameStates};
/// Maps server entities to our entities
#[derive(Resource, Debug, Default)]
struct WorldLocalEntityMap(EntityHashMap<Entity>);

impl EntityMapper for WorldLocalEntityMap {
    fn map_entity(&mut self, entity: Entity) -> Entity {
        // todo: this could crash
        self.0.get(&entity).copied().unwrap()
    }
}

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
                transform: Transform::from_xyz(0.0, 12.0, 0.0),
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
    mapper: ResMut<'w, WorldLocalEntityMap>,
}

impl<'w, 's> WorldStateSysParam<'w, 's> {
    fn push_state(
        &mut self,
        entity: Entity,
        state: &Option<impl Component + Clone + core::fmt::Debug>,
    ) {
        if let Some(s) = state {
            self.commands.entity(entity).insert(s.clone());
            log::debug!("added {:?} to {:?}", s, entity);
        }
    }

    /// Replicate some `serialization::EntityState` onto the given `Entity`.
    fn push_entity_state(&mut self, world_entity: Entity, state: &net::EntityState) {
        log::debug!("pushing state to server entity {:?}", world_entity);
        let entity = *self
            .mapper
            .0
            .get(&world_entity)
            .expect("client entity should be mapped");
        if let Some(aura) = &state.aura {
            let mut aura = aura.clone();
            aura.map_entities::<WorldLocalEntityMap>(&mut self.mapper);
            log::debug!("added {:?} to {:?}", aura, entity);
            self.commands.entity(entity).insert(aura.clone());
        }

        self.push_state(entity, &state.health);
        self.push_state(entity, &state.spellcaster);
        self.push_state(entity, &state.casting_spell);
    }

    fn spawn_gameobject(&mut self, server_entity: Entity) -> Entity {
        let entity = self.commands.spawn(GameObject).id();
        self.mapper.0.insert(server_entity, entity);
        log::debug!(
            "replicated server entity {:?} -> {:?}",
            server_entity,
            entity
        );
        entity
    }

    fn despawn(&mut self, server_entity: Entity) {
        let entity = self
            .mapper
            .0
            .remove(&server_entity)
            .expect("client entity should be mapped");
        self.commands.entity(entity).despawn_recursive();
        log::debug!("despawned server entity {:?}", entity);
    }
}

/// Handle replication & syncing of world state to this game's world.
fn sys_handle_world_state(
    mut sys_params: WorldStateSysParam,
    world_conn: Res<world_connection::WorldConnection>,
) {
    if !world_conn.is_changed() {
        return;
    }
    if let Some(world_state) = &world_conn.cached_state {
        log::debug!("game processing world state");

        if let Some(world_change) = &world_conn.state_change {
            for (server_entity, is_new_entity) in world_change.new_server_keys.iter().copied() {
                if is_new_entity {
                    sys_params.spawn_gameobject(server_entity);
                }
                sys_params.push_entity_state(
                    server_entity,
                    world_state
                        .entity_state_map
                        .get(&server_entity)
                        .expect("any key in `new server keys` should be in `cached state`"),
                );
            }

            for dead_entity in world_change.lost_server_keys.iter().copied() {
                sys_params.despawn(dead_entity);
            }
        } else {
            // we just want to spawn everything in here
            log::debug!("this is a first time world gen");
            for (server_entity, state) in world_state.entity_state_map.iter() {
                sys_params.spawn_gameobject(*server_entity);
                sys_params.push_entity_state(*server_entity, state);
            }
        }
    }
}

/// Spawn in the basic game world objects when the game state changes.
fn sys_spawn_game_world(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut server_client_map: ResMut<WorldLocalEntityMap>,
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
    mut server_client_map: ResMut<WorldLocalEntityMap>,
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
        app.insert_resource(WorldLocalEntityMap::default());
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
