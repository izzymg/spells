#[cfg(test)]
mod tests {
    use bevy::{app::App, ecs::event::Events, hierarchy::Children, time::TimePlugin};
    use crate::game::status_effect::{self, AddStatusEffectEvent, RemoveStatusEffectEvent, StatusEffect, StatusEffectPlugin};

    #[test]
    fn add_auras() {

        let mut app = App::new();
        app.add_plugins(StatusEffectPlugin);
        app.add_plugins(TimePlugin);
        
        let test_entity = app.world.spawn(()).id();
        let test_status_id = 0;

        let num_auras = 5;
        for _ in 0..num_auras {
            let mut event_writer = app.world.resource_mut::<Events<status_effect::AddStatusEffectEvent>>();
            event_writer.send(AddStatusEffectEvent {
                status_id: test_status_id,
                target_entity: test_entity,
            });
            
            app.update();
        }

        // was entity created?
        let children = app.world.get::<Children>(test_entity).unwrap();
        assert_eq!(children.iter().len(), num_auras);

        for &child in children.iter() {
            let effect = app.world.get::<StatusEffect>(child).unwrap();
            assert_eq!(effect.id, 0)
        }
    }

    #[test]
    fn remove_auras() {
        
        let mut app = App::new();
        app.add_plugins(StatusEffectPlugin);
        app.add_plugins(TimePlugin);
        
        let test_entity = app.world.spawn(()).id();
        let test_status_id = 0;

        // add n auras
        let num_auras = 5;
        for _ in 0..num_auras {
            let mut event_writer = app.world.resource_mut::<Events<status_effect::AddStatusEffectEvent>>();
            event_writer.send(AddStatusEffectEvent {
                status_id: test_status_id,
                target_entity: test_entity,
            });
            
            app.update();
        }

        // add secondary auras that shouldn't be removed
        let num_secondary = 3;
        for _ in 0..num_secondary {
            let mut event_writer = app.world.resource_mut::<Events<status_effect::AddStatusEffectEvent>>();
            event_writer.send(AddStatusEffectEvent {
                status_id: 1,
                target_entity: test_entity,
            });
            app.update();
        }

        // drop m auras
        let num_removed = 3;
        for _ in 0..num_removed {
            let mut event_writer = app.world.resource_mut::<Events<status_effect::RemoveStatusEffectEvent>>();
            event_writer.send(RemoveStatusEffectEvent {
                status_id: test_status_id,
                target_entity: test_entity,
            });
            
            app.update();
        }

        let children = app.world.get::<Children>(test_entity).unwrap();
        assert_eq!(children.iter().len(), num_auras - num_removed + num_secondary);
    }
}