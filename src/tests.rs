#[cfg(test)]
mod tests {
    use bevy::{app::App, ecs::event::Events, hierarchy::Children, time::TimePlugin};
    use crate::game::status_effect::{self, AddStatusEffectEvent, StatusEffect, StatusEffectPlugin};

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
}