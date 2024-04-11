// contains common game components
mod alignment;
pub use alignment::*;

use crate::game::assets;
use bevy::prelude::*;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(1000);

/// Entity has a position in the server world
#[derive(Debug, Component, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Entity that can die
#[derive(Debug, Component, Default)]
pub struct Health(pub i64);

/// Represents one aura belonging to the parent of this entity.
#[derive(Component)]
pub struct Aura {
    pub id: assets::AuraID,
    pub duration: Timer,
}

impl Aura {
    pub fn get_remaining_time(&self) -> Duration {
        self.duration.duration() - self.duration.elapsed()
    }
}

/// The parent entity is shielded
#[derive(Component)]
pub struct ShieldAura(pub i64);

impl ShieldAura {
    pub fn new(base_multiplier: i64) -> Self {
        Self(base_multiplier)
    }
}

/// The parent entity is ticking health
#[derive(Component)]
pub struct TickingEffectAura(pub Timer);

impl TickingEffectAura {
    pub fn new() -> Self {
        TickingEffectAura(Timer::new(TICK_RATE, TimerMode::Repeating))
    }
}

/// Unit can cast spells
#[derive(Debug, Component)]
pub struct SpellCaster;

/// Unit is casting a spell
#[derive(Debug, Component)]
pub struct CastingSpell {
    pub spell_id: assets::SpellID,
    pub target: Entity,
    pub cast_timer: Timer,
}

impl CastingSpell {
    pub fn new(spell_id: assets::SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell {
            spell_id,
            target,
            cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once),
        }
    }
}
