use crate::game::{assets, components, events};
use bevy::prelude::*;

use super::ServerSets;

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
                .spawn((components::Aura {
                    id: ev.aura_id,
                    duration: Timer::new(aura_data.duration, TimerMode::Once),
                },))
                .id();

            // add aura types
            match aura_data.status_type {
                assets::AuraType::TickingHP => commands
                    .entity(base_entity)
                    .insert(components::TickingEffectAura::new()),
                assets::AuraType::Shield => commands
                    .entity(base_entity)
                    .insert(components::ShieldAura::new(aura_data.base_multiplier)),
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
    status_effect_query: Query<&components::Aura>,
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
