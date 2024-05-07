use crate::{world_connection, GameStates, SystemSets};
use bevy::{
    ecs::{
        entity::{EntityHashMap, MapEntities},
        system::SystemParam,
    },
    log,
    prelude::*,
};
use std::collections::VecDeque;

const MAX_INPUTS_CACHED: usize = 200;

/// Marks the player that is being controlled by this client
#[derive(Component, Debug, Default)]
pub struct PredictedPlayer;

/// Which direction we would like our `PredictedPlayer` to go.
#[derive(Debug, Component, PartialEq, Default)]
pub struct WishDir(pub Vec3);

/// Maps World entities to Game entities
#[derive(Resource, Debug, Default)]
struct WorldGameEntityMap(EntityHashMap<Entity>);

impl EntityMapper for WorldGameEntityMap {
    fn map_entity(&mut self, entity: Entity) -> Entity {
        // todo: this could crash
        self.0.get(&entity).copied().unwrap()
    }
}

#[derive(SystemParam)]
struct ReplicationSys<'w, 's> {
    commands: Commands<'w, 's>,
    world_to_game: ResMut<'w, WorldGameEntityMap>,
}

impl<'w, 's> ReplicationSys<'w, 's> {
    fn update_world_entity(
        &mut self,
        world_entity: Entity,
        mut state: lib_spells::net::EntityState,
    ) {
        let game_entity = *self
            .world_to_game
            .0
            .get(&world_entity)
            .expect("should be mapped");

        state.map_entities(self.world_to_game.as_mut());
        self.commands.add(lib_spells::net::AddEntityStateCommand {
            entity: game_entity,
            entity_state: state,
        });
    }

    fn spawn_world_entity(&mut self, world_entity: Entity) {
        let game_entity = self.commands.spawn(super::GameObject).id();
        self.world_to_game.0.insert(world_entity, game_entity);
        log::debug!(
            "spawned world entity {:?} -> {:?}",
            world_entity,
            game_entity
        );
    }

    fn despawn_world_entity(&mut self, world_entity: Entity) {
        let game_entity = self.world_to_game.0.remove(&world_entity).unwrap();
        self.commands.entity(game_entity).despawn_recursive();
        log::debug!("despawned world entity {:?}", world_entity);
    }

    fn has_world_entity(&self, world_entity: Entity) -> bool {
        self.world_to_game.0.contains_key(&world_entity)
    }

    fn integrate(&mut self, mut state: lib_spells::net::WorldState) {
        // find entities we're tracking that don't exist in this state, and kill them
        let lost = self
            .world_to_game
            .0
            .iter()
            .filter_map(|(world_entity, _)| {
                state
                    .entity_state_map
                    .get(world_entity)
                    .is_none()
                    .then_some(world_entity)
            })
            .copied()
            .collect::<Vec<Entity>>();
        for entity in lost {
            self.despawn_world_entity(entity);
        }

        for (world_entity, state) in state.entity_state_map.drain() {
            if !self.has_world_entity(world_entity) {
                self.spawn_world_entity(world_entity);
            }
            self.update_world_entity(world_entity, state);
        }
        log::debug!("world state integration done");
    }

    /// Marks the given world entity as being controlled by this client.
    fn mark_controlled_player(&mut self, world_entity: Entity) {
        let game_entity = self.world_to_game.0.get(&world_entity).unwrap();
        self.commands
            .entity(*game_entity)
            .insert((WishDir::default(), PredictedPlayer::default()));
        log::debug!("controlled player: {:?} -> {:?}", world_entity, game_entity);
    }
}

fn sys_sync_positions(
    mut commands: Commands,
    pos_query: Query<
        (Entity, &lib_spells::shared::Position, &Transform),
        Changed<lib_spells::shared::Position>,
    >,
) {
    for (entity, world_pos, actual_pos) in pos_query.iter() {
        let error_amt = (world_pos.0.length() - actual_pos.translation.length()).abs();
        log::debug!("transform sync pass: {:?} error: {}", entity, error_amt);
        commands
            .entity(entity)
            .insert(Transform::from_translation(world_pos.0));
    }
}

/// Build the world and swap the game state.
fn sys_on_first_world_state(
    mut state_events: ResMut<Events<world_connection::WorldStateEvent>>,
    world_conn: Res<world_connection::Connection>,
    mut replication: ReplicationSys,
    mut next_game_state: ResMut<NextState<GameStates>>,
) {
    if let Some(state_ev) = state_events.drain().next() {
        log::info!("got initial world state");
        replication.integrate(state_ev.state);
        replication.mark_controlled_player(world_conn.client_info().you);
        next_game_state.set(GameStates::Game);
    }
}

/// Received new world state. Need to generate comparison against current state if it exists.
fn sys_on_world_state(
    mut state_events: ResMut<Events<world_connection::WorldStateEvent>>,
    mut replication: ReplicationSys,
    cached: ResMut<InputCache>,
) {
    for state_ev in state_events.drain() {
        log::debug!(
            "state - last read input: {}, current input: {}",
            state_ev.stamp,
            cached.0.len()
        );
        replication.integrate(state_ev.state);
    }
}

fn sys_destroy_gos(mut commands: Commands, go_query: Query<Entity, With<super::GameObject>>) {
    log::info!("cleaning up game objects");
    for entity in go_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Debug, Copy, Clone)]
struct CachedInput {
    wish_dir: Vec3,
    seq: u8,
}

#[derive(Default, Resource)]
struct InputCache(VecDeque<CachedInput>);

impl InputCache {
    fn get_next_sequence(&self) -> u8 {
        if let Some(ele) = self.0.front() {
            if ele.seq == u8::MAX {
                0
            } else {
                ele.seq + 1
            }
        } else {
            0
        }
    }

    fn push(&mut self, wish_dir: Vec3) -> u8 {
        let seq = self.get_next_sequence();
        self.0.push_front(CachedInput { wish_dir, seq });
        seq
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

fn sys_clear_input_cache(mut cache: ResMut<InputCache>) {
    if cache.len() > MAX_INPUTS_CACHED {
        log::warn!("input cache too big, clearing");
        cache.clear();
    }
}

fn sys_enqueue_movements(
    mut conn: Option<ResMut<world_connection::Connection>>,
    wish_dir_query: Query<&WishDir, With<PredictedPlayer>>,
    mut cache: ResMut<InputCache>,
) {
    for wish_dir in wish_dir_query.iter() {
        let seq = cache.push(wish_dir.0);
        if let Some(ref mut conn) = conn {
            conn.enqueue_input(seq, wish_dir.0);
        }
    }
}

/// Read the set wish dir on the controlled player and predict a new translation
fn sys_predict_player_pos(
    mut controlled_query: Query<(&mut Transform, &WishDir), With<PredictedPlayer>>,
    time: Res<Time>,
) {
    let (mut controlled_trans, wish_dir) = match controlled_query.get_single_mut() {
        Ok((c, w)) => (c, w),
        Err(_) => return,
    };
    controlled_trans.translation += wish_dir.0.normalize_or_zero() * time.delta_seconds();
}

pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGameEntityMap::default());
        app.insert_resource(InputCache::default());

        app.add_systems(OnExit(GameStates::Game), sys_destroy_gos);

        app.add_systems(
            Update,
            (
                (sys_on_first_world_state).run_if(in_state(GameStates::LoadGame)),
                sys_clear_input_cache,
                (sys_enqueue_movements, sys_predict_player_pos)
                    .chain()
                    .run_if(in_state(GameStates::Game))
                    .before(SystemSets::NetSend)
                    .after(SystemSets::Controls),
                (sys_on_world_state, sys_sync_positions)
                    .chain()
                    .run_if(in_state(GameStates::Game))
                    .after(SystemSets::NetFetch),
            ),
        );
    }
}
