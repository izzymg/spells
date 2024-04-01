// shields a target from damage

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::Query,
    },
    hierarchy::Children,
};


#[derive(Component)]
pub struct StatusShield {
    pub value: i64,
}

impl StatusShield {
    pub fn new(base_multiplier: i64) -> StatusShield {
        StatusShield {
            value: base_multiplier,
        }
    }
}

#[derive(Event)]
pub struct ShieldDamageEvent {
    pub damage: i64,
    pub entity: Entity,
}

pub(super) fn shield_damage_system(
    mut ev_r: EventReader<ShieldDamageEvent>,
    mut shield_query: Query<&mut StatusShield>,
    child_query: Query<&Children>,
) {
    for ev in ev_r.read() {
        let mut damage = ev.damage;
        if let Ok(children) = child_query.get(ev.entity) {
            let mut iter = shield_query.iter_many_mut(children);
            // apply n damage to shields
            while let Some(mut shield) = iter.fetch_next() {
                let applied_dmg = shield.value.min(damage);
                shield.value -= applied_dmg;
                damage -= applied_dmg;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::{app::{self, Update}, ecs::event::Events, hierarchy::BuildWorldChildren};

    use super::{shield_damage_system, ShieldDamageEvent, StatusShield};

    #[test]
    pub fn test_shield_damage_system() {
        let mut app = app::App::new();
        app.add_event::<ShieldDamageEvent>();
        app.add_systems(Update, shield_damage_system);

        let test_shield_values = vec![100, 200];

        // spawn entity to be shielded
        let shielded_parent = app.world.spawn(()).id();
        // apply shields
        {
            for value in test_shield_values.iter() {
                let shield = app.world.spawn(StatusShield {
                    value: *value,
                }).id();

                app.world.entity_mut(shielded_parent).add_child(shield);
            }
            app.update();
        }

        // apply n damage to shield
        {
            let damage = 105;
            let mut event_writer = app.world.resource_mut::<Events<ShieldDamageEvent>>();
            event_writer.send(ShieldDamageEvent {
                damage,
                entity: shielded_parent,
            });
            app.update();

            // get total shield value
            let shield_value = app.world.query::<&StatusShield>().iter(&app.world).map(|s| s.value).reduce(|f, v| f + v).unwrap();
            let test_shield_value = test_shield_values.iter().copied().reduce(|a, b| a + b).unwrap();
            assert_eq!(shield_value, test_shield_value - damage);
        }
    }
}