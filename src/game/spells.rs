mod resource;
use std::time::Duration;
use bevy::{
    app::{FixedUpdate, Plugin}, ecs::{
        component::Component, entity::Entity, event::{Event, EventReader, EventWriter}, query::With, system::{Commands, Query, Res}
    }, log::*, time::{Time, Timer}
};
use super::health;


/// Marks as having spells which can be cast.
/// Contains a list of valid spell IDs in Spellbook.
#[derive(Debug, Component)]
pub struct Spellcaster;


#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: usize,
}

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(resource::get_spell_list_resource())
            .add_event::<StartCastingEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spellcast_tick_system,
                    on_start_casting_system,
                )
            );
    }
}

struct SpellData {
    name: String,
    cast_time: Duration,
    target_health_effect: Option<i64>,
    self_health_effect: Option<i64>,
}

impl SpellData {
    fn new(name: String, cast_ms: u64) -> SpellData {
        SpellData {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            self_health_effect: None,
            target_health_effect: None,
        }
    }
    fn new_target_hp(name: String, cast_ms: u64, damage: i64) -> SpellData {
        SpellData {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            self_health_effect: None,
            target_health_effect: Some(damage),
        }
    }
}

// Unit is casting a spell
#[derive(Debug, Component)]
struct CastingSpell {
    spell_id: usize,
    target: Entity,
    cast_timer: Timer,
}

impl CastingSpell {
    fn new(spell_id: usize, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell { spell_id, target, cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once) }
    }
}

// Tick spell casts and handle finished casts
fn spellcast_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    spell_list: Res<resource::SpellList>,
    mut ev_w: EventWriter<health::HealthTickEvent>,
    mut query: Query<(Entity, &mut CastingSpell), With<Spellcaster>>,
) {
    for (entity, mut casting) in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
        debug!(
            "casting spell {} at {} ({}s)",
            casting.spell_id,
            casting.target.index(),
            casting.cast_timer.elapsed_secs()
        );

        if casting.cast_timer.finished() {
            commands.entity(entity).remove::<CastingSpell>();
            cast_spell(
                &mut ev_w,
                &spell_list,
                SpellCastData {
                    caster: entity,
                    target: casting.target,
                    spell_id: casting.spell_id,
                }
            )
        }
    }
}


struct SpellCastData {
    spell_id: usize,
    target: Entity,
    caster: Entity,
}

fn cast_spell(ev_w: &mut EventWriter<health::HealthTickEvent>, spell_list: &Res<resource::SpellList>, data: SpellCastData) {
    if let Some(spell_data) = spell_list.get_spell_data(data.spell_id) {

        // apply target hp
        if let Some(hp) = spell_data.target_health_effect {
            ev_w.send(health::HealthTickEvent {
                entity: data.target,
                hp,
            });
        }

        // apply self hp
        if let Some(hp) = spell_data.self_health_effect {
            ev_w.send(health::HealthTickEvent {
                entity: data.caster,
                hp,
            });
        }
    } else {
        error!("no spell at id {}", data.spell_id);
    }
}

// begin spell casts when event received
fn on_start_casting_system(
    mut events: EventReader<StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<resource::SpellList>
)
{
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            debug!("e{} starts casting spell {}", ev.entity.index(), spell.name);
            commands.entity(ev.entity).insert(CastingSpell::new(ev.spell_id, ev.target, spell.cast_time));
        } else {
            error!("no spell at id {}", ev.spell_id);
        }
    }
}