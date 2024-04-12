use std::time::Duration;

use crate::game::{assets, events};
use bevy::prelude::*;

use super::ServerSets;
use lib_spells::shared;

// todo: rewrite this shit
// why are we even having auras as child entities? well you know why but
// just use a hashmap to represent slots: Aura: HashMap<i8, Option<(AuraID, Timer)>>
// use a trait like Aura { get_aura(slot), remove_aura(slot), tick_aura(slot) }
// then write a proc macro to derive(aura) that implements the hashmap and functions
// badabing badaboom
// then you can write individual queries over all TickingHP auras and so on

const TICK_RATE: Duration = Duration::from_millis(1000);

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
/// Process an add aura event
fn sys_add_aura_ev(
    mut ev_r: EventReader<events::AddAuraEvent>,
    mut commands: Commands,
    auras_asset: Res<assets::AurasAsset>,
) {
    for ev in ev_r.read() {
        // look up status
        if let Some(aura_data) = auras_asset.lookup(ev.aura_id) {
            // spawn base aura
            let base_entity = commands
                .spawn(shared::Aura {
                    id: ev.aura_id,
                    duration: Timer::new(aura_data.duration, TimerMode::Once),
                    owner: ev.target_entity,
                })
                .id();

            // add aura types
            match aura_data.status_type {
                shared::AuraType::TickingHP => commands
                    .entity(base_entity)
                    .insert(TickingEffectAura::new()),
                shared::AuraType::Shield => commands
                    .entity(base_entity)
                    .insert(ShieldAura::new(aura_data.base_multiplier)),
            };

            // parent
            commands.entity(ev.target_entity).add_child(base_entity);
        }
    }
}

/// Process a remove aura event
fn sys_remove_aura_ev(
    mut ev_r: EventReader<events::RemoveAuraEvent>,
    mut commands: Commands,
    child_query: Query<&Children>,
    status_effect_query: Query<&shared::Aura>,
) {
    'event_processing: for ev in ev_r.read() {
        // find children of entity
        if let Ok(children) = child_query.get(ev.target_entity) {
            for &child in children.iter() {
                // for each child grab status
                if let Ok(status) = status_effect_query.get(child) {
                    if status.id == ev.aura_id {
                        // drop one instance of this status
                        commands.entity(child).despawn_recursive();
                        continue 'event_processing;
                    }
                }
            }
        }
    }
}

pub struct EffectApplicationPlugin;

impl Plugin for EffectApplicationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            FixedUpdate,
            (sys_add_aura_ev, sys_remove_aura_ev).in_set(ServerSets::EffectApplication),
        );
    }
}
