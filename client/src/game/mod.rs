use std::{collections::HashMap, time::Duration};

use bevy::{log, prelude::*};

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

fn sys_handle_world_state(
    mut commands: Commands,
    mut server_client_map: ResMut<ServerClientMap>,
    world_conn: Res<world_connection::WorldConnection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !world_conn.is_changed() {
        return;
    }
    if let Some(state) = &world_conn.world_state {
        log::debug!("world state process");

        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::hsl(0.0, 0.0, 0.1));
        for caster in state.casting_spell.iter() {
            // MAKE GENERIC OR SYSTEM PARAM

            // get or spawn caster
            let client_entity = match server_client_map.0.get(&caster.entity) {
                Some(&client_entity) => client_entity,
                None => commands
                    .spawn(MeshMatGameObjectBundle::new(mesh.clone(), material.clone()))
                    .id(),
            };

            server_client_map.0.insert(caster.entity, client_entity);

            // update caster
            let casting = commands
                .entity(client_entity)
                .insert(CastingSpell::new(
                    caster.spell_id,
                    caster.timer,
                    caster.max_timer,
                ))
                .id();
            log::debug!("casting {:?}", casting);
        }

        for caster in state.spell_casters.iter() {
            // get or spawn caster
            let client_entity = match server_client_map.0.get(&caster.0) {
                Some(&client_entity) => client_entity,
                None => commands.spawn(GameObject::default()).id(),
            };

            server_client_map.0.insert(caster.0, client_entity);

            let caster = commands.entity(client_entity).insert(SpellCaster).id();
            log::debug!("caster {:?}", caster);
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
