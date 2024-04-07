mod effect_creation;
mod resource;

use std::{fmt, time::Duration};

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res},
    },
    log,
    time::{Time, Timer},
};

use super::{
    alignment::{self, Faction, FactionMember, Hostility},
    ServerSets,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: SpellID,
}

impl StartCastingEvent {
    pub fn new(entity: Entity, target: Entity, spell_id: SpellID) -> Self {
        Self {
            entity,
            target,
            spell_id,
        }
    }
}

// Unit is casting a spell
#[derive(Debug, Component)]
pub struct CastingSpell {
    pub spell_id: SpellID,
    pub target: Entity,
    pub cast_timer: Timer,
}

impl CastingSpell {
    fn new(spell_id: SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell {
            spell_id,
            target,
            cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once),
        }
    }
}

#[derive(Clone, Copy, Debug, Event)]
pub struct SpellApplicationEvent {
    pub origin: Entity,
    pub target: Entity,
    pub spell_id: SpellID,
}

/// Begin spell casts when event received
fn sys_start_casting_ev(
    mut events: EventReader<StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<resource::SpellList>,
) {
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            log::debug!("{:?} starts casting spell {}", ev.entity, spell.name);
            commands.entity(ev.entity).insert(CastingSpell::new(
                ev.spell_id,
                ev.target,
                spell.cast_time,
            ));
        } else {
            log::error!("no spell {}", ev.spell_id);
        }
    }
}


// Remove invalid targets on casts
fn sys_validate_cast_targets(
    mut query: Query<(Entity, &mut CastingSpell, Option<&alignment::FactionMember>)>,
    spell_list: Res<resource::SpellList>,
    faction_checker: alignment::FactionChecker,
    mut commands: Commands,
) {
    for (entity, casting, faction_member) in query.iter_mut() {
        let spell = spell_list.get_spell_data(casting.spell_id).unwrap();
        let is_selfcast = entity == casting.target;
        // allow self friendly
        if is_selfcast && spell.hostility == Hostility::Friendly {
            continue;
        }

        // check factions (default is OK)
        let caster_faction = match faction_member {
            Some(member) => member.0,
            None => Faction::default(),
        };
        let target_faction = faction_checker
            .get_entity_faction(casting.target)
            .unwrap_or_default();
        if !is_selfcast
            && alignment::is_valid_target(spell.hostility, caster_faction, target_faction)
        {
            continue;
        }
        // disallow all else
        log::info!(
            "{:?} invalid target {:?} for spell {}",
            entity,
            casting.target,
            casting.spell_id
        );
        commands.entity(entity).remove::<CastingSpell>();
    }
}

/// Send spell application events for finished casts
fn sys_finish_casts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CastingSpell)>,
    mut spell_app_ev_w: EventWriter<SpellApplicationEvent>,
) {
    for (entity, casting) in query.iter_mut() {
        if !casting.cast_timer.finished() {
            continue;
        }
        commands.entity(entity).remove::<CastingSpell>();

        // send spell application events
        spell_app_ev_w.send(SpellApplicationEvent {
            origin: entity,
            spell_id: casting.spell_id,
            target: casting.target,
        });
    }
}

// Tick spell casts
fn sys_tick_casts(time: Res<Time>, mut query: Query<&mut CastingSpell>) {
    for mut casting in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
    }
}

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            FixedUpdate,
            (
                (sys_start_casting_ev, sys_tick_casts, sys_validate_cast_targets, sys_finish_casts, ).chain(),
                effect_creation::get_configs().in_set(ServerSets::EffectCreation),
            )
                .chain(),
        );

        app.insert_resource(resource::get_spell_list_resource())
            .add_event::<StartCastingEvent>()
            .add_event::<SpellApplicationEvent>();
    }
}
