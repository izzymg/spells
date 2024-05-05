use core::fmt;
use std::time::Duration;
use bincode;
use serde::{self, Deserialize, Serialize};
use bevy_ecs::{prelude::*, entity::MapEntities};
use bevy_time::Timer;
use bevy_math::prelude::*;

pub type SerializationError = bincode::ErrorKind;

/// Entity can be harmed and healed
#[derive(Deserialize, Serialize, Component, Debug, Copy, Clone)]
pub struct Health(pub i64);

/// Represents one aura belonging to the parent of this entity
#[derive(Deserialize, Serialize, Component, Debug, Clone)]
pub struct Aura {
    pub id: AuraID,
    pub duration: Timer,
    pub owner: Entity,
}

impl MapEntities for Aura {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        let new_entity = entity_mapper.map_entity(self.owner);
        self.owner = new_entity;
    }
}

impl Aura {
    pub fn get_remaining_time(&self) -> Duration {
        self.duration.duration() - self.duration.elapsed()
    }
}

/// Used to look up an aura in the aura list resource
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuraID(usize);

impl AuraID {
    pub fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for AuraID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for AuraID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(AURA:{})", self.0)
    }
}

/// Possible aura types
pub enum AuraType {
    TickingHP,
    Shield,
}

/// We can use this to look up complex data about a spell
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellID(usize);

impl SpellID {
    pub fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for SpellID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for SpellID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(SPELL:{})", self.0)
    }
}

/// Unit can cast spells
#[derive(Debug, Component, Copy, Clone, Serialize, Deserialize)]
pub struct SpellCaster;

/// Unit is casting a spell
#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct CastingSpell {
    pub spell_id: SpellID,
    pub target: Entity,
    pub cast_timer: Timer,
}

impl CastingSpell {
    pub fn new(spell_id: SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell {
            spell_id,
            target,
            cast_timer: Timer::new(cast_time, bevy_time::TimerMode::Once),
        }
    }
}

/// Unit exists in world space.
#[derive(Debug, Copy, Component, Clone, Serialize, Deserialize)]
pub struct Position(pub Vec3);

/// Unit position delta over time.
#[derive(Debug, Copy, Component, Clone, Serialize, Deserialize)]
pub struct Velocity(pub Vec3);

#[derive(Deserialize, Serialize, Copy, Clone, Component, Debug)]
pub struct Player;

#[derive(Deserialize, Serialize, Clone, Component, Debug)]
pub struct Name(pub String);


