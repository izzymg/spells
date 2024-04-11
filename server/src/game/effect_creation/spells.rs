use bevy::{log, prelude::*};
use lib_spells::{alignment, serialization};

use crate::game::{assets, events};

// Remove invalid targets on casts
pub(super) fn sys_validate_cast_targets(
    mut query: Query<(
        Entity,
        &mut serialization::CastingSpell,
        Option<&alignment::FactionMember>,
    )>,
    spell_list: Res<assets::SpellsAsset>,
    faction_checker: alignment::FactionChecker,
    mut commands: Commands,
) {
    for (entity, casting, faction_member) in query.iter_mut() {
        let spell = spell_list.get_spell_data(casting.spell_id).unwrap();
        let is_selfcast = entity == casting.target;
        // allow self friendly
        if is_selfcast && spell.hostility == alignment::Hostility::Friendly {
            continue;
        }

        // check factions (default is OK)
        let caster_faction = match faction_member {
            Some(member) => member.0,
            None => alignment::Faction::default(),
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
        commands
            .entity(entity)
            .remove::<serialization::CastingSpell>();
    }
}

/// Dispatch `SpellApplicationEvent` for finished casts
pub(super) fn sys_dispatch_finished_casts(
    query: Query<(Entity, &serialization::CastingSpell)>,
    mut spell_app_ev_w: EventWriter<events::SpellApplicationEvent>,
) {
    let events: Vec<events::SpellApplicationEvent> = query
        .iter()
        .filter_map(|(caster, cast)| {
            cast.cast_timer
                .finished()
                .then_some(events::SpellApplicationEvent {
                    origin: caster,
                    spell_id: cast.spell_id,
                    target: cast.target,
                })
        })
        .collect();

    spell_app_ev_w.send_batch(events);
}

pub(super) fn sys_remove_finished_casts(
    mut commands: Commands,
    query: Query<(Entity, &serialization::CastingSpell)>,
) {
    for (entity, _) in query.iter().filter(|(_, cast)| cast.cast_timer.finished()) {
        commands
            .entity(entity)
            .remove::<serialization::CastingSpell>();
    }
}

// Tick spell casts
pub(super) fn sys_tick_casts(time: Res<Time>, mut query: Query<&mut serialization::CastingSpell>) {
    for mut casting in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
    }
}

/// Read spell application events and create effects
pub(super) fn sys_spell_application_ev(
    spell_list: Res<assets::SpellsAsset>,
    mut effect_ev_w: EventWriter<events::EffectQueueEvent>,
    mut spell_ev_r: EventReader<events::SpellApplicationEvent>,
) {
    for ev in spell_ev_r.read() {
        if let Some(spell_data) = spell_list.get_spell_data(ev.spell_id) {
            let ev = events::EffectQueueEvent {
                target: ev.target,
                health_effect: spell_data.target_health_effect,
                aura_effect: spell_data.target_aura_effect,
            };
            effect_ev_w.send(ev);
        } else {
            log::warn!("no spell {}", ev.spell_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{assets, sys_validate_cast_targets};
    use bevy::{
        app::{self, Update},
        time::Timer,
    };
    use lib_spells::{alignment, serialization};

    /// test spell target validation
    macro_rules! target_validation {
        ($name:ident, $s:expr, $c:expr, $t:expr, $e:expr) => {
            #[test]
            fn $name() {
                let mut app = app::App::new();
                app.insert_resource(assets::SpellsAsset {
                    0: vec![
                        assets::SpellData::new("hostile".into(), 0),
                        assets::SpellData::new("friendly".into(), 0).mark_friendly(),
                    ],
                });
                app.add_systems(Update, sys_validate_cast_targets);
                let target = app.world.spawn(alignment::FactionMember($c)).id();
                // caster
                let caster = app
                    .world
                    .spawn((
                        alignment::FactionMember($t),
                        serialization::CastingSpell {
                            cast_timer: Timer::from_seconds(1.0, bevy::time::TimerMode::Once),
                            spell_id: $s,
                            target,
                        },
                    ))
                    .id();

                // tick our validation system
                app.update();

                let still_casting = app.world.get::<serialization::CastingSpell>(caster);
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
