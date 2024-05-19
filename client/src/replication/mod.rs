/*! Replicates world state into the game world */

use crate::{controls::wish_dir, events, world_connection, SystemSets};
use bevy::{
    ecs::{
        entity::{EntityHashMap, MapEntities},
        system::SystemParam,
    },
    log,
    prelude::*,
};
use lib_spells::shared;
use std::collections::VecDeque;
use std::time::Duration;

const MAX_INPUTS_CACHED: usize = 200;

/// Marks this entity as being a replicated entity
#[derive(Component, Debug, Default)]
pub struct Replicated;

/// Marks the player that is being predicted by this client
#[derive(Component, Debug, Default)]
pub struct PredictedPlayer;

/// Maps World entities to Game entities
#[derive(Debug, Default)]
struct WorldToGameMapper(EntityHashMap<Entity>);

impl EntityMapper for WorldToGameMapper {
    fn map_entity(&mut self, entity: Entity) -> Entity {
        // todo: this could crash
        self.0.get(&entity).copied().unwrap()
    }
}

/// Need this indirection because we can't access Local inner value *rage*
#[derive(Debug, Default)]
struct ReplicationSysWorldToGame(WorldToGameMapper);

#[derive(SystemParam)]
struct ReplicationSys<'w, 's> {
    commands: Commands<'w, 's>,
    world_to_game: Local<'s, ReplicationSysWorldToGame>,
    replication_completed_ev: ResMut<'w, Events<events::ReplicationCompleted>>,
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
             .0
            .get(&world_entity)
            .expect("should be mapped");

        state.map_entities(&mut self.world_to_game.0);
        self.commands.add(lib_spells::net::AddEntityStateCommand {
            entity: game_entity,
            entity_state: state,
        });
    }

    fn spawn_world_entity(&mut self, world_entity: Entity) {
        let game_entity = self.commands.spawn(Replicated).id();
        self.world_to_game.0 .0.insert(world_entity, game_entity);
        log::debug!(
            "spawned world entity {:?} -> {:?}",
            world_entity,
            game_entity
        );
    }

    fn despawn_world_entity(&mut self, world_entity: Entity) {
        let game_entity = self.world_to_game.0 .0.remove(&world_entity).unwrap();
        self.commands.entity(game_entity).despawn_recursive();
        log::debug!("despawned world entity {:?}", world_entity);
    }

    fn has_world_entity(&self, world_entity: Entity) -> bool {
        self.world_to_game.0 .0.contains_key(&world_entity)
    }

    fn replicate_state(&mut self, mut state: lib_spells::net::WorldState) {
        // find entities we're tracking that don't exist in this state, and kill them
        let lost = self
            .world_to_game
            .0
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
        self.replication_completed_ev
            .send(events::ReplicationCompleted);
    }

    /// Marks the given world entity as being predicted by this client.
    fn mark_predicted_player(&mut self, world_entity: Entity) {
        let game_entity = self.world_to_game.0 .0.get(&world_entity).unwrap();
        self.commands.entity(*game_entity).insert(PredictedPlayer);
    }
}

#[derive(Debug, Copy, Clone)]
struct CachedInput {
    wish_dir: Vec3,
    seq: u8,
    time: Duration,
}

#[derive(Default, Resource)]
struct InputCache(VecDeque<CachedInput>);

impl InputCache {
    fn get_next_sequence(&self) -> u8 {
        if let Some(ele) = self.0.back() {
            if ele.seq == u8::MAX {
                0
            } else {
                ele.seq + 1
            }
        } else {
            0
        }
    }

    /// Drops all entries up to but not including `seq`, returning dropped count
    fn drop_to_sequence(&mut self, seq: u8) -> usize {
        let len = self.0.len();
        while let Some(ic) = self.front() {
            if ic.seq < seq {
                self.pop();
            } else {
                break;
            }
        }
        len - self.0.len()
    }

    fn pop(&mut self) -> Option<CachedInput> {
        self.0.pop_front()
    }

    fn get(&self, index: usize) -> Option<&CachedInput> {
        self.0.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &CachedInput> {
        self.0.iter()
    }

    fn front(&self) -> Option<&CachedInput> {
        self.0.front()
    }

    fn push(&mut self, wish_dir: Vec3, time: Duration) -> u8 {
        let seq = self.get_next_sequence();
        self.0.push_back(CachedInput {
            wish_dir,
            seq,
            time,
        });
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

/// Received new world state.
fn sys_on_world_state(
    mut state_events: ResMut<Events<events::WorldStateEvent>>,
    mut replication: ReplicationSys,
    mut cached: ResMut<InputCache>,
) {
    for state_ev in state_events.drain() {
        replication.replicate_state(state_ev.state);
        // TODO: this only needs to happen once really
        replication.mark_predicted_player(state_ev.client_info.you);

        log::debug!(
            "dropped {} read inputs",
            cached.drop_to_sequence(state_ev.seq)
        );
    }
}

fn sys_reconcile_player(
    time: Res<Time>,
    cached: ResMut<InputCache>,
    mut predicted_pos_query: Query<(&mut Transform, &shared::Position), With<PredictedPlayer>>,
) {
    let (mut player_actual_pos, player_server_pos) = match predicted_pos_query.get_single_mut() {
        Ok(v) => v,
        _ => return,
    };

    let mut replayed_pos = player_server_pos.0;
    for (i, input) in cached.iter().enumerate() {
        // predict to next input, or to current time
        let t = match cached.get(i + 1) {
            Some(i) => i.time,
            None => time.elapsed(),
        };
        let t_passed = (t - input.time).as_secs_f32();
        replayed_pos += input.wish_dir * t_passed;
    }
    let replay_err = replayed_pos - player_actual_pos.translation;
    log::debug!(
        "player predicted pos: {}, error: {}",
        replayed_pos,
        replay_err
    );
    player_actual_pos.translation = replayed_pos;
}

fn sys_sync_server_positions(
    mut pos_query: Query<(&mut Transform, &shared::Position), Without<PredictedPlayer>>,
) {
    for (mut transform, pos) in pos_query.iter_mut() {
        transform.translation = pos.0;
    }
}

/// Cache & enqueue new wish direction inputs
fn sys_enqueue_movements(
    mut conn: Option<ResMut<world_connection::Connection>>,
    wish_dir: Res<wish_dir::WishDir>,
    mut cache: ResMut<InputCache>,
    time: Res<Time>,
) {
    let current_time = time.elapsed();
    let seq = cache.push(wish_dir.0, current_time);
    if let Some(ref mut conn) = conn {
        conn.enqueue_input(current_time, seq, wish_dir.0);
    }
}

/// Read the set wish dir on the predicted player and predict a new translation
fn sys_predict_player_pos(
    mut predicted_query: Query<&mut Transform, With<PredictedPlayer>>,
    wish_dir: Res<wish_dir::WishDir>,
    time: Res<Time>,
) {
    let mut predicted_trans = match predicted_query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };
    predicted_trans.translation += wish_dir.0 * time.delta_seconds();
}

/// Figure out "in-between" positions for travelling server objects
fn sys_interpolate_positions(
    time: Res<Time>,
    mut pos_query: Query<(&mut Transform, &shared::Velocity), Without<PredictedPlayer>>,
) {
    for (mut trans, vel) in pos_query.iter_mut() {
        trans.translation += vel.0 * time.delta_seconds();
    }
}

fn sys_cleanup(mut commands: Commands, replication_query: Query<Entity, With<Replicated>>) {
    log::debug!("cleaning up replicated objects");
    for entity in replication_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputCache::default());
        app.add_systems(
            Update,
            (
                sys_interpolate_positions,
                ((
                    sys_enqueue_movements.run_if(resource_changed::<wish_dir::WishDir>),
                    sys_enqueue_movements.run_if(resource_added::<wish_dir::WishDir>),
                    sys_predict_player_pos,
                )
                    .chain()),
                ((
                    sys_on_world_state.run_if(on_event::<events::WorldStateEvent>()),
                    (sys_sync_server_positions, sys_reconcile_player)
                        .run_if(on_event::<events::ReplicationCompleted>()),
                )
                    .chain()),
                sys_clear_input_cache,
                sys_cleanup.run_if(on_event::<events::DisconnectedEvent>()),
            )
                .in_set(SystemSets::Replication),
        );
    }
}
