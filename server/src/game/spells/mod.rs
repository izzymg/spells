mod effect_creation;
pub mod resource;

use std::{fmt, time::Duration};

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res},
    },
    log,
    time::{Time, Timer},
};

use crate::game::{
    alignment::{self, Faction, Hostility}, events, ServerSets
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

/// Begin spell casts when event received
fn sys_start_casting_ev(
    mut events: EventReader<events::StartCastingEvent>,
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

/// Dispatch `SpellApplicationEvent` for finished casts
fn sys_finish_casts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CastingSpell)>,
    mut spell_app_ev_w: EventWriter<events::SpellApplicationEvent>,
) {
    for (entity, casting) in query.iter_mut() {
        if !casting.cast_timer.finished() {
            continue;
        }
        commands.entity(entity).remove::<CastingSpell>();

        // send spell application events
        spell_app_ev_w.send(events::SpellApplicationEvent {
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
                (
                    sys_start_casting_ev,
                    sys_tick_casts,
                    sys_validate_cast_targets,
                    sys_finish_casts,
                )
                    .chain(),
                effect_creation::get_configs().in_set(ServerSets::EffectCreation),
            )
                .chain(),
        );

        app.insert_resource(resource::get_spell_list_resource());
    }
}

#[cfg(test)]
mod tests {
    use crate::game::alignment::FactionMember;

    use super::{
        resource::{SpellData, SpellList},
        sys_validate_cast_targets, CastingSpell,
    };
    use bevy::{
        app::{self, Update},
        time::Timer,
    };

    /// test spell target validation
    macro_rules! target_validation {
        ($name:ident, $s:expr, $c:expr, $t:expr, $e:expr) => {
            #[test]
            fn $name() {
                let mut app = app::App::new();
                app.insert_resource(SpellList {
                    0: vec![
                        SpellData::new("hostile".into(), 0),
                        SpellData::new("friendly".into(), 0).mark_friendly(),
                    ],
                });
                app.add_systems(Update, sys_validate_cast_targets);
                let target = app.world.spawn(FactionMember($c)).id();
                // caster
                let caster = app
                    .world
                    .spawn((
                        FactionMember($t),
                        CastingSpell {
                            cast_timer: Timer::from_seconds(1.0, bevy::time::TimerMode::Once),
                            spell_id: $s,
                            target: target,
                        },
                    ))
                    .id();

                // tick our validation system
                app.update();

                let still_casting = app.world.get::<CastingSpell>(caster);
                assert_eq!($e, still_casting.is_some());
            }
        };
    }

    target_validation!(hostile_share_faction, 0.into(), 0b101, 0b011, false);
    target_validation!(hostile_target_no_faction, 0.into(), 0b001, 0b000, true);
    target_validation!(hostile_caster_no_faction, 0.into(), 0b000, 0b001, true);
    target_validation!(hostile_no_factions, 0.into(), 0b000, 0b000, true);

    target_validation!(friendly_share_faction, 1.into(), 0b101, 0b011, true);
    target_validation!(friendly_target_no_faction, 1.into(), 0b001, 0b000, false);
    target_validation!(friendly_caster_no_faction, 1.into(), 0b000, 0b001, false);
    target_validation!(friendly_no_factions, 1.into(), 0b000, 0b000, false);
}
