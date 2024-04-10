use std::{collections::HashMap, time::Duration};

use bevy::{ecs::system::SystemParam, log, prelude::*};
use lib_spells::serialization;

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

#[derive(Bundle, Default)]
struct MeshMatGameObjectBundle {
    go: GameObject,
    cube: PbrBundle,
}

impl MeshMatGameObjectBundle {
    fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
        Self {
            go: GameObject,
            cube: PbrBundle {
                mesh,
                material,
                transform: Transform::from_xyz(0.0, 0.5, 1.0),
                ..Default::default()
            },
        }
    }
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

/// Cast timer for a spell
#[derive(Component)]
pub struct CastingSpell {
    pub spell_id: usize,
    pub timer: Timer,
}

/// This entity is ALIVE
#[derive(Component, Debug)]
pub struct Health(pub i64);

/// List of aura IDs
#[derive(Component, Debug)]
pub struct Auras(pub Vec<usize>);

/// Marker
#[derive(Component)]
pub struct SpellCaster;

impl CastingSpell {
    fn new(spell_id: usize, current_ms: u64, max_ms: u64) -> Self {
        let mut timer = Timer::new(Duration::from_millis(max_ms), TimerMode::Once);
        timer.set_elapsed(Duration::from_millis(current_ms));
        Self { spell_id, timer }
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
    query_auras: Query<'w, 's, &'static mut Auras>,
}

impl<'w, 's> WorldStateSysParam<'w, 's> {
    // todo: probably just re-export EntityState from world_conn tbh
    fn push_entity_state(&mut self, entity: Entity, state: &serialization::EntityState) {
        if state.spell_caster.is_some() {
            self.commands.entity(entity).insert(SpellCaster);
        }
        if let Some(casting) = state.casting_spell {
            self.commands.entity(entity).insert(CastingSpell::new(
                casting.spell_id,
                casting.timer,
                casting.max_timer,
            ));
        }

        if let Some(health) = state.health {
            self.commands.entity(entity).insert(Health(health.health));
        }

        for aura in state.auras.iter() {
            if let Ok(mut entity_auras) = self.query_auras.get_mut(entity) {
                // already have some to add to
                entity_auras.0.push(aura.aura_id);
            }
        }
    }

    fn spawn_gameobject(&mut self) -> Entity {
        self.commands.spawn(GameObject).id()
    }
}
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
            // we need to process change
        } else {
            // we just want to spawn everything in here
            log::debug!("this is a first time world gen");
            for (entity, state) in world_state.entity_state_map.iter() {
                let new_entity = sys_params.spawn_gameobject();
                server_client_map.0.insert(*entity, new_entity);
                sys_params.push_entity_state(new_entity, state);
                log::debug!("spawned {:?} with state: {:?}", new_entity, state);
            }
        }
    }
}

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
