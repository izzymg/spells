use bevy::{
    ecs::{
        entity::Entity, event::EventReader, schedule::IntoSystemConfigs, system::{Commands, Query, Res}
    }, hierarchy::{Children, DespawnRecursiveExt}, prelude::BuildChildren, time::{Time, Timer, TimerMode}
};
use super::{resource, AddAuraEvent, Aura, AuraType, RemoveAuraEvent, ShieldAura, TickingEffectAura};

/// Tick auras & remove expired
fn sys_tick_clean_auras(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Aura)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.duration.tick(time.delta());
        if effect.duration.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


/// Process an add aura event 
fn sys_add_aura_ev(
    mut ev_r: EventReader<AddAuraEvent>,
    mut commands: Commands, 
    auras_db: resource::AuraSysResource,
) {
    for ev in ev_r.read() {
        // look up status
        if let Some(aura_data) = auras_db.get_status_effect_data(ev.aura_id) {
            // spawn base aura
            let base_entity = commands
                .spawn((Aura {
                    id: ev.aura_id,
                    duration: Timer::new(aura_data.duration, TimerMode::Once),
                },))
                .id();

            // add aura types
            match aura_data.status_type {
                AuraType::TickingHP => {
                    commands.entity(base_entity).insert(TickingEffectAura::new())
                }
                AuraType::Shield => commands
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
    mut ev_r: EventReader<RemoveAuraEvent>,
    mut commands: Commands,
    child_query: Query<&Children>,
    status_effect_query: Query<&Aura>,
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

pub fn get_configs() -> impl IntoSystemConfigs<()> {
    (sys_tick_clean_auras, sys_remove_aura_ev, sys_add_aura_ev).into_configs()
}