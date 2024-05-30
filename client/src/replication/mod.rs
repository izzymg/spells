/*! Replicates world state into the game world */

mod entity_mapping;

use crate::{controls::wish_dir, events, world_connection, SystemSets};
use bevy::{
    ecs::{entity::MapEntities, system::SystemParam},
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

#[derive(Component, Debug, Default)]
struct LastVelocityChange(Option<Duration>);

#[derive(SystemParam)]
struct ReplicationSys<'w, 's> {
    commands: Commands<'w, 's>,
    entity_map: ResMut<'w, entity_mapping::EntityMap>,
    replication_completed_ev: ResMut<'w, Events<events::ReplicationCompleted>>,
    replicated_query: Query<'w, 's, Entity, With<Replicated>>,
}

#[derive(Bundle, Default)]
struct ReplicatedObjectBundle {
    rep: Replicated,
    last_vel: LastVelocityChange,
}

impl<'w, 's> ReplicationSys<'w, 's> {
    /// Clear all replicated objects & the world to game state
    fn destroy(&mut self) {
        for entity in self.replicated_query.iter() {
            self.commands.entity(entity).despawn_recursive();
        }
        self.entity_map.clear();
    }

    fn update_world_entity(
        &mut self,
        world_entity: Entity,
        mut state: lib_spells::net::EntityState,
    ) {
        let game_entity = self.entity_map.get_game_entity(world_entity).unwrap();
        state.map_entities(self.entity_map.world_to_game());
        self.commands.add(lib_spells::net::AddEntityStateCommand {
            entity: game_entity,
            entity_state: state,
        });
    }

    fn spawn_world_entity(&mut self, world_entity: Entity) -> Entity {
        let game_entity = self.commands.spawn(ReplicatedObjectBundle::default()).id();
        self.entity_map.map(world_entity, game_entity);
        game_entity
    }

    fn despawn_world_entity(&mut self, world_entity: Entity) {
        let game_entity = self.entity_map.unmap_from_world(world_entity);
        self.commands.entity(game_entity).despawn_recursive();
    }

    fn has_world_entity(&self, world_entity: Entity) -> bool {
        self.entity_map.world_entity_is_mapped(world_entity)
    }

    fn replicate_state(
        &mut self,
        mut state: lib_spells::net::WorldState,
        server_player_entity: Entity,
    ) {
        // find entities we're tracking that don't exist in this state, and kill them
        let mapped_world_entities = self.entity_map.collect_world();
        let lost = mapped_world_entities.iter().filter(|world| !state.entity_state_map.contains_key(world));
        for entity in lost {
            self.despawn_world_entity(*entity);
        }

        for (world_entity, state) in state.entity_state_map.drain() {
            if !self.has_world_entity(world_entity) {
                let spawned = self.spawn_world_entity(world_entity);
                if world_entity == server_player_entity {
                    log::debug!("marking player server entity {:?}", world_entity);
                    self.commands.entity(spawned).insert(PredictedPlayer);
                }
            }
            self.update_world_entity(world_entity, state);
        }
        self.replication_completed_ev
            .send(events::ReplicationCompleted);
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
fn sys_replicate_world_state(
    mut state_events: ResMut<Events<events::WorldStateEvent>>,
    mut replication: ReplicationSys,
    mut cached: ResMut<InputCache>,
) {
    for state_ev in state_events.drain() {
        replication.replicate_state(state_ev.state, state_ev.client_info.you);
        cached.drop_to_sequence(state_ev.seq);
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

fn sys_mark_velocity_change(
    time: Res<Time>,
    mut query: Query<
        (&mut LastVelocityChange, &shared::Velocity),
        (Without<PredictedPlayer>, Changed<shared::Velocity>),
    >,
) {
    for (mut last_vel, vel) in query.iter_mut() {
        log::debug!("updating velocity: {:?} {}", last_vel.0, vel.0);
        last_vel.0 = Some(time.elapsed());
    }
}
/// Figure out "in-between" positions for travelling server objects
fn sys_extrapolate_positions(
    time: Res<Time>,
    mut pos_query: Query<
        (
            &mut Transform,
            &shared::Position,
            &shared::Velocity,
            &LastVelocityChange,
        ),
        Without<PredictedPlayer>,
    >,
) {
    for (mut transform, server_pos, server_vel, last_vel_change) in pos_query.iter_mut() {
        let elapsed = match last_vel_change.0 {
            Some(t) => (time.elapsed() - t).as_secs_f32(),
            None => 0.,
        };
        transform.translation = server_pos.0 + (server_vel.0 * elapsed);
    }
}

fn sys_cleanup(mut replication: ReplicationSys) {
    log::debug!("cleaning up replicated objects");
    replication.destroy();
}

pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(entity_mapping::EntityMappingPlugin);
        app.insert_resource(InputCache::default());
        app.add_systems(
            Update,
            (
                (sys_mark_velocity_change, sys_extrapolate_positions)
                    .after(sys_replicate_world_state)
                    .chain(),
                ((
                    sys_enqueue_movements.run_if(resource_changed::<wish_dir::WishDir>),
                    sys_enqueue_movements.run_if(resource_added::<wish_dir::WishDir>),
                    sys_predict_player_pos,
                )
                    .chain()),
                ((
                    sys_replicate_world_state.run_if(on_event::<events::WorldStateEvent>()),
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
