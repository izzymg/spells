use std::{fmt, time::Duration};

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component, entity::Entity, event::Event, schedule::IntoSystemConfigs
    },
    time::{Timer, TimerMode},
};

use super::ServerSets;
const TICK_RATE: Duration = Duration::from_millis(1000);

mod resource;
mod effect_application;
mod effect_creation;

/// Used to look up an aura in the aura list resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuraID(usize);

impl AuraID {
    fn get(self) -> usize {
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
enum AuraType {
    TickingHP,
    Shield,
}

/// Represents one aura belonging to the parent of this entity.
#[derive(Component)]
pub struct Aura {
    pub id: AuraID,
    pub duration: Timer,
}

impl Aura {
    pub fn get_remaining_time(&self) -> Duration {
        self.duration.duration() - self.duration.elapsed()
    }
}

/// Request to add an aura child to the given entity
#[derive(Event, Debug)]
pub struct AddAuraEvent {
    pub aura_id: AuraID,
    pub target_entity: Entity,
}

/// Request to drop an aura child from the given entity
#[derive(Event, Debug)]
pub struct RemoveAuraEvent {
    pub aura_id: AuraID,
    pub target_entity: Entity,
}

/// The parent entity is shielded
#[derive(Component)]
pub struct ShieldAura {
    pub value: i64,
}

impl ShieldAura {
    pub fn new(base_multiplier: i64) -> ShieldAura {
        ShieldAura {
            value: base_multiplier,
        }
    }
}

/// Damage the shields of a given entity by damage
#[derive(Event)]
pub struct ShieldDamageEvent {
    pub damage: i64,
    pub entity: Entity,
}


/// Ticking aura that causes queues an effect on the parent each tick.
#[derive(Component)]
pub struct TickingEffectAura {
    ticker: Timer,
}

impl TickingEffectAura {
    pub fn new() -> TickingEffectAura {
        TickingEffectAura {
            ticker: Timer::new(TICK_RATE, TimerMode::Repeating)
        }
    }
}

pub struct AuraPlugin;

impl Plugin for AuraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_event::<AddAuraEvent>()
            .add_event::<RemoveAuraEvent>()
            .add_event::<ShieldDamageEvent>()
            .insert_resource(resource::get_resource())
            .add_systems(
                FixedUpdate,
                (
                    effect_creation::get_configs().in_set(ServerSets::EffectCreation),
                    effect_application::get_configs().in_set(ServerSets::EffectApplication),
                ),
            );
    }
}
