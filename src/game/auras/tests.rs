#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{
        app::{App, Update},
        ecs::event::Events,
        hierarchy::{BuildWorldChildren, Children},
        utils::tracing::Event,
    };

    use crate::game::{
        auras::{
            aura_types, on_add_aura_event_system, on_remove_aura_event_system, resource::{AuraData, AuraList}, ticking_hp::ticking_hp_system, AddAuraEvent, Aura, RemoveAuraEvent, AURA_TICK_RATE
        },
        health::{self, Health, HealthTickEvent},
    };

    fn get_test_aura_data() -> AuraList {
        AuraList(vec![
            AuraData {
                duration: Duration::from_secs(1),
                hp_per_tick: Some(-1),
                name: "test_1".into(),
            },
            AuraData {
                duration: Duration::from_secs(5),
                hp_per_tick: Some(20),
                name: "test_2".into(),
            },
            AuraData {
                duration: Duration::from_secs(15),
                hp_per_tick: Some(15),
                name: "test_3".into(),
            },
            AuraData {
                duration: Duration::from_secs(0),
                hp_per_tick: Some(-50),
                name: "test_4".into(),
            },
        ])
    }

    struct Test {
        hp_per_tick: i64,
        duration: Duration,
    }

    #[test]
    fn did_add_aura() {
        const AURA_TYPE: usize = 50;
        let mut app = App::new();
        app.add_systems(Update, on_add_aura_event_system::<{ AURA_TYPE }>);
        app.insert_resource(get_test_aura_data());
        app.add_event::<AddAuraEvent<{ AURA_TYPE }>>();

        let some_guy = app.world.spawn(Health(50)).id();

        let mut res = app.world.resource_mut::<Events<AddAuraEvent<AURA_TYPE>>>();

        let event_sends = 5;
        for _ in 0..event_sends {
            res.send(AddAuraEvent::<AURA_TYPE> {
                aura_data_id: 0,
                target: some_guy,
            });
        }


        app.update();

        let q_children = app.world.get::<Children>(some_guy);
        // get children of some_guy
        for &children in q_children.iter() {
            assert_eq!(children.len(), event_sends);
            for &child in children.iter() {
                let q_1 = app.world.get::<Aura<AURA_TYPE>>(child);
                assert_eq!(q_1.unwrap().aura_data_id, 0);
            }
        }

    }

    #[test]
    fn did_remove_aura() {
        const AURA_TYPE: usize = 50;
        let mut app = App::new();
        app.add_systems(Update, (on_add_aura_event_system::<{ AURA_TYPE }>, on_remove_aura_event_system::<{ AURA_TYPE }>));
        app.insert_resource(get_test_aura_data());
        app.add_event::<AddAuraEvent<{ AURA_TYPE }>>();
        app.add_event::<RemoveAuraEvent<{ AURA_TYPE }>>();

        let some_guy = app.world.spawn(Health(50)).id();

        let mut add_aura = app.world.resource_mut::<Events<AddAuraEvent<AURA_TYPE>>>();
        let event_sends = 5;
        for _ in 0..event_sends {
            add_aura.send(AddAuraEvent::<AURA_TYPE> {
                aura_data_id: 0,
                target: some_guy,
            });
        }
        app.update();

        let mut remove_aura = app.world.resource_mut::<Events<RemoveAuraEvent<AURA_TYPE>>>();
        for _ in 0..event_sends {
            remove_aura.send(RemoveAuraEvent::<AURA_TYPE> {
                aura_data_id: 0,
                target: some_guy,
            });
        }
        app.update();

        let q_children = app.world.get::<Children>(some_guy);
        // get children of some_guy
        for &children in q_children.iter() {
            assert_eq!(children.len(), 0);
        }
    }

    // our hp tick system should trigger an HP tick event each time the aura's ticker advances by AURA_TICK_RATE
    #[test]
    fn did_trigger_hp_tick() {
        let test_aura_data = get_test_aura_data();

        let tests: Vec<Test> = test_aura_data
            .0
            .iter()
            .map(|v| Test {
                duration: v.duration,
                hp_per_tick: v.hp_per_tick.unwrap(),
            })
            .collect();

        const STARTING_HP: i64 = 50;
        for (i, test) in tests.iter().enumerate() {
            let mut app = App::new();
            app.add_event::<health::HealthTickEvent>();
            app.add_systems(Update, ticking_hp_system);
            app.insert_resource(get_test_aura_data());

            // get aura data from resource
            // spawn mob with ticking HP aura
            let auras = app
                .world
                .spawn(Aura::<{ aura_types::TICKING_HP }>::new(i, test.duration))
                .id();
            let mob = app.world.spawn(Health(STARTING_HP)).id();
            app.world.entity_mut(mob).add_child(auras);

            // tick aura ticker once & step system
            for mut hp in app
                .world
                .query::<&mut Aura<{ aura_types::TICKING_HP }>>()
                .iter_mut(&mut app.world)
            {
                hp.ticker.tick(AURA_TICK_RATE);
            }
            app.update();

            // get HealthTick event reader
            let health_tick_events = app.world.resource::<Events<HealthTickEvent>>();
            let mut health_tick_ev_r = health_tick_events.get_reader();
            let health_tick_ev = health_tick_ev_r.read(health_tick_events).next().unwrap();

            assert_eq!(health_tick_ev.hp, test.hp_per_tick);

            app.world.despawn(mob);
        }
    }
}
