use bevy::{
    app::{FixedUpdate, Plugin,}, ecs::{
        component::Component, entity::Entity, event::{Event, EventReader, EventWriter}, system::{Commands, Query, Res}
    }, hierarchy::{Children, Parent, BuildChildren}, log, time::{self, Time, Timer}
};

mod resource;
mod ticking;

pub mod aura_types {
    /// Health Point ticks
    pub const TICKING_HP: usize = 5;
}

pub struct AurasPlugin;
impl Plugin for AurasPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(resource::get_aura_list_resource())
            .add_event::<AddAuraEvent::<{aura_types::TICKING_HP}>>()
            .add_event::<RemoveAuraEvent::<{aura_types::TICKING_HP}>>()
            .add_systems(
                FixedUpdate,
                (
                    on_add_aura_event_system::<{aura_types::TICKING_HP}>,
                    on_remove_aura_event_system::<{aura_types::TICKING_HP}>,
                    tick_aura_system::<{aura_types::TICKING_HP}>,
                    ticking::ticking_damage_system,
                )
            )
        ;
    }
}

#[derive(Event, Debug)]
pub struct AddAuraEvent<const AURA_TYPE: usize> {
    pub target: Entity,
    pub aura_data_id: usize,
}

#[derive(Event, Debug)]
pub struct RemoveAuraEvent<const AURA_TYPE: usize> {
    pub target: Entity,
    pub aura_data_id: usize,
}


#[derive(Component)]
struct Aura<const AURA_TYPE: usize> {
    timer: Timer,
    aura_data_id: usize,
}

// tick all auras of type AURA_TYPE, trigger remove event if expired
fn tick_aura_system<const AURA_TYPE: usize>(
    time: Res<Time>,
    mut ev_w: EventWriter<RemoveAuraEvent<AURA_TYPE>>,
    mut query: Query<(&Parent, &mut Aura<AURA_TYPE>)>
) {
    for (parent, mut aura) in query.iter_mut() {

        aura.timer.tick(time.delta());
        log::debug!("aura {} ticks on {:?} ({}s)", aura.aura_data_id, parent, aura.timer.elapsed_secs());

        if aura.timer.finished() {
            ev_w.send(RemoveAuraEvent::<AURA_TYPE> {
                target: parent.get(),
                aura_data_id: aura.aura_data_id,
            });
        }
    }
}

// spawn auras as children of given entity
fn on_add_aura_event_system<const AURA_TYPE: usize>(
    mut commands: Commands,
    mut ev_r: EventReader<AddAuraEvent<AURA_TYPE>>,
    aura_list: Res<resource::AuraList>,
) {
    for ev in ev_r.read() {
        // find relevant aura
        if let Some(aura_data) = aura_list.get_aura_data(ev.aura_data_id) {
            // create aura entity
            let aura = commands
                .spawn(Aura::<AURA_TYPE> {
                    aura_data_id: ev.aura_data_id,
                    timer: Timer::new(aura_data.duration, time::TimerMode::Once),
                })
                .id();
                log::debug!("added aura {} to {:?}", ev.aura_data_id, ev.target);

            // aura is child of parent entity
            commands.entity(ev.target).push_children(&[aura]);
        } else {
            log::error!("no aura at id {}", ev.target.index())
        }
    }
}

// drop aura by ID on given entity
fn on_remove_aura_event_system<const AURA_TYPE: usize>(
    mut commands: Commands,
    mut ev_r: EventReader<RemoveAuraEvent<AURA_TYPE>>,
    query: Query<&Children>,
    query_auras: Query<&Aura<AURA_TYPE>>,
) {
    for ev in ev_r.read() {
        // get all child entities of our parent
        for &child in query.get(ev.target).unwrap().iter() {
            // for every aura child
            if let Ok(aura) = query_auras.get(child) {
                // find aura and remove match
                if aura.aura_data_id == ev.aura_data_id {
                    commands.entity(child).despawn();
                    log::debug!("removed aura {} from {:?}", ev.aura_data_id, ev.target)
                }
            }
        }
    }
}

