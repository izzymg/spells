use bevy::{log, prelude::*};

use crate::game::{assets, components, events};

/// Begin spell casts when event received
pub(super) fn sys_start_casting_ev(
    mut events: EventReader<events::StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<assets::SpellList>,
) {
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            log::debug!("{:?} starts casting spell {}", ev.entity, spell.name);
            commands
                .entity(ev.entity)
                .insert(components::CastingSpell::new(
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
pub(super) fn sys_validate_cast_targets(
    mut query: Query<(
        Entity,
        &mut components::CastingSpell,
        Option<&components::FactionMember>,
    )>,
    spell_list: Res<assets::SpellList>,
    faction_checker: components::FactionChecker,
    mut commands: Commands,
) {
    for (entity, casting, faction_member) in query.iter_mut() {
        let spell = spell_list.get_spell_data(casting.spell_id).unwrap();
        let is_selfcast = entity == casting.target;
        // allow self friendly
        if is_selfcast && spell.hostility == components::Hostility::Friendly {
            continue;
        }

        // check factions (default is OK)
        let caster_faction = match faction_member {
            Some(member) => member.0,
            None => components::Faction::default(),
        };
        let target_faction = faction_checker
            .get_entity_faction(casting.target)
            .unwrap_or_default();
        if !is_selfcast
            && components::is_valid_target(spell.hostility, caster_faction, target_faction)
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
        commands.entity(entity).remove::<components::CastingSpell>();
    }
}

/// Dispatch `SpellApplicationEvent` for finished casts
pub(super) fn sys_finish_casts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut components::CastingSpell)>,
    mut spell_app_ev_w: EventWriter<events::SpellApplicationEvent>,
) {
    for (entity, casting) in query.iter_mut() {
        if !casting.cast_timer.finished() {
            continue;
        }
        commands.entity(entity).remove::<components::CastingSpell>();

        // send spell application events
        spell_app_ev_w.send(events::SpellApplicationEvent {
            origin: entity,
            spell_id: casting.spell_id,
            target: casting.target,
        });
    }
}

// Tick spell casts
pub(super) fn sys_tick_casts(time: Res<Time>, mut query: Query<&mut components::CastingSpell>) {
    for mut casting in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
    }
}

/// Read spell application events and create effects
pub(super) fn sys_spell_application_ev(
    spell_list: Res<assets::SpellList>,
    mut effect_ev_w: EventWriter<events::EffectQueueEvent>,
    mut spell_ev_r: EventReader<events::SpellApplicationEvent>,
) {
    for ev in spell_ev_r.read() {
        if let Some(spell_data) = spell_list.get_spell_data(ev.spell_id) {
            effect_ev_w.send(events::EffectQueueEvent {
                target: ev.target,
                health_effect: spell_data.target_health_effect,
                aura_effect: spell_data.target_aura_effect,
            });
        } else {
            log::warn!("no spell {}", ev.spell_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::components::FactionMember;

    use super::{
        assets::{SpellData, SpellList},
        components, sys_validate_cast_targets,
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
                        components::CastingSpell {
                            cast_timer: Timer::from_seconds(1.0, bevy::time::TimerMode::Once),
                            spell_id: $s,
                            target: target,
                        },
                    ))
                    .id();

                // tick our validation system
                app.update();

                let still_casting = app.world.get::<components::CastingSpell>(caster);
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
