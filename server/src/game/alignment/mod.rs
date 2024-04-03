/// For friendlyness/hostility determinations

use bevy::ecs::{component::Component, entity::Entity, system::{Query, SystemParam}};

type Faction = u8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Hostility {
    Hostile,
    Friendly,
}

impl Default for Hostility {
    fn default() -> Self {
        Hostility::Hostile
    }
}

#[derive(Component, Debug)]
pub struct FactionMember(pub Faction);

pub fn shares_faction(a: Faction, b: Faction) -> bool {
    (a & b) != 0
}

pub fn is_valid_target(hostility: Hostility, origin: Faction, target: Faction) -> bool {
    let shares_faction = shares_faction(origin, target);
    if hostility == Hostility::Hostile {
        !shares_faction
    } else {
        shares_faction
    }
}

#[derive(SystemParam)]
pub struct FactionChecker<'w, 's> {
    factions: Query<'w, 's, &'static FactionMember>,
}

impl<'w, 's> FactionChecker<'w, 's> {
    pub fn get_entity_faction(&self, entity: Entity) -> Option<Faction> {
        if let Ok(f) = self.factions.get(entity) {
            return Some(f.0)
        }
        None
    }
}
