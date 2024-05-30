use bevy::{
    ecs::entity::{EntityHashMap, EntityMapper},
    prelude::*,
};

use bevy::log;

#[derive(Debug, Default)]
pub struct MappedEntities(EntityHashMap<Entity>);

impl EntityMapper for MappedEntities {
    fn map_entity(&mut self, entity: Entity) -> Entity {
        // todo: this could crash
        self.0.get(&entity).copied().unwrap()
    }
}

#[derive(Resource, Debug, Default)]
pub struct EntityMap {
    world_to_game: MappedEntities,
    game_to_world: MappedEntities,
}

impl EntityMap {

    pub fn collect_world(&self) -> Vec<Entity> {
        self.world_to_game.0.keys().copied().collect()
    }

    pub fn game_to_world(&mut self) -> &mut impl EntityMapper {
        &mut self.game_to_world
    }

    pub fn world_to_game(&mut self) -> &mut impl EntityMapper {
        &mut self.world_to_game
    }

    pub fn get_world_entity(&self, game: Entity) -> Option<Entity> {
        self.game_to_world.0.get(&game).copied()
    }

    pub fn get_game_entity(&self, world: Entity) -> Option<Entity> {
        self.world_to_game.0.get(&world).copied()
    }

    /// Clears the mapping between World <-> Game and returns the World entity. Crashy.
    pub fn unmap_from_game(&mut self, game: Entity) -> Entity {
        let world = self.game_to_world.0.remove(&game).unwrap();
        self.world_to_game.0.remove(&world).unwrap();
        log::debug!("unmapped world {:?} <-> game {:?}", world, game);
        world
    }

    /// Clears the mapping between World <-> Game and returns the Game entity. Crashy.
    pub fn unmap_from_world(&mut self, world: Entity) -> Entity {
        let game = self.world_to_game.0.remove(&world).unwrap();
        self.game_to_world.0.remove(&game).unwrap();
        log::debug!("unmapped world {:?} <-> game {:?}", world, game);
        game
    }

    pub fn world_entity_is_mapped(&self, world: Entity) -> bool {
        self.world_to_game.0.contains_key(&world)
    }
    pub fn game_entity_is_mapped(&self, game: Entity) -> bool {
        self.game_to_world.0.contains_key(&game)
    }

    pub fn clear(&mut self) {
        self.world_to_game.0.clear();
        self.game_to_world.0.clear();
        log::debug!("clearing entity mappings");
    }

    pub fn map(&mut self, world: Entity, game: Entity) {
        self.world_to_game.0.insert(world, game);
        self.game_to_world.0.insert(game, world);
        log::debug!("mapped world {:?} <-> game {:?}", world, game);
    }
}

pub struct EntityMappingPlugin;
impl Plugin for EntityMappingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EntityMap::default());
    }
}
