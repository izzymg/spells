use crate::{world_connection, GameStates};
use bevy::{
    ecs::{
        entity::{EntityHashMap, MapEntities},
        system::SystemParam,
    },
    log,
    prelude::*,
};

/// Marks the player that is being controlled by this client
#[derive(Component, Debug, Default)]
pub struct ControlledPlayer;

/// Maps World entities to Game entities
#[derive(Resource, Debug, Default)]
pub struct WorldGameEntityMap(EntityHashMap<Entity>);

impl EntityMapper for WorldGameEntityMap {
    fn map_entity(&mut self, entity: Entity) -> Entity {
        // todo: this could crash
        self.0.get(&entity).copied().unwrap()
    }
}

#[derive(SystemParam)]
pub struct ReplicationSys<'w, 's> {
    commands: Commands<'w, 's>,
    world_to_game: ResMut<'w, WorldGameEntityMap>,
}

pub fn sys_sync_positions(
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
        self.commands.entity(*game_entity).insert(ControlledPlayer);
        log::debug!("controlled player: {:?} -> {:?}", world_entity, game_entity);
    }
}

/// Build the world and swap the game state.
pub fn sys_on_first_world_state(
    mut state_events: ResMut<Events<world_connection::WorldStateEvent>>,
    world_conn: Res<world_connection::Connection>,
    mut replication: ReplicationSys,
    mut next_game_state: ResMut<NextState<GameStates>>,
) {
    if let Some(state_ev) = state_events.drain().next() {
        log::info!("got initial world state");
        replication.integrate(state_ev.0);
        replication.mark_controlled_player(world_conn.client_info.you);
        next_game_state.set(GameStates::Game);
    }
}

/// Received new world state. Need to generate comparison against current state if it exists.
pub fn sys_on_world_state(
    mut state_events: ResMut<Events<world_connection::WorldStateEvent>>,
    mut replication: ReplicationSys,
) {
    for state_ev in state_events.drain() {
        replication.integrate(state_ev.0);
    }
}

pub fn sys_destroy_gos(mut commands: Commands, go_query: Query<Entity, With<super::GameObject>>) {
    log::info!("cleaning up game world");
    for entity in go_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
