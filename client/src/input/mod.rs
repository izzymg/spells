use bevy::{input::mouse::MouseMotion, prelude::*};
use std::collections::HashMap;

#[derive(Copy, PartialEq, Debug, Clone)]
pub enum Action {
    Jump,
    Activate,
    Primary,
    Secondary,
}

#[derive(Copy, PartialEq, Debug, Clone)]
pub enum Axis {
    Look(Vec2),
    Movement(Vec2),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ButtonAction {
    Pressed,
    Released,
    Held,
}

/// Hardware type agnostic input event
#[derive(Event)]
pub struct ActionEvent<T> {
    pub action: Action,
    pub data: T,
}

impl<T> ActionEvent<T> {
    fn create(action: Action, data: T) -> Self {
        Self { action, data }
    }
}

/// Hardware agnostic input axis state
#[derive(Resource, Default, Copy, Clone, Debug)]
pub struct InputAxes {
    pub movement: Vec2,
    pub look: Vec2,
}

impl InputAxes {
    pub fn get_movement_3d(&self) -> Vec3 {
        // -z goes forward in bevy
        Vec3::new(self.movement.x, 0.0, -self.movement.y)
    }
}

/// Maps keyboard codes to axes
#[derive(Resource)]
pub struct KeyAxisMap {
    map: HashMap<KeyCode, Axis>,
}

impl Default for KeyAxisMap {
    fn default() -> Self {
        Self {
            map: HashMap::from([
                // wasd move
                (KeyCode::KeyW, Axis::Movement(Vec2::Y)),
                (KeyCode::KeyS, Axis::Movement(Vec2::NEG_Y)),
                (KeyCode::KeyD, Axis::Movement(Vec2::X)),
                (KeyCode::KeyA, Axis::Movement(Vec2::NEG_X)),
                // arrow look
                (KeyCode::ArrowUp, Axis::Look(Vec2::Y)),
                (KeyCode::ArrowDown, Axis::Look(Vec2::NEG_Y)),
                (KeyCode::ArrowRight, Axis::Look(Vec2::X)),
                (KeyCode::ArrowLeft, Axis::Look(Vec2::NEG_X)),
            ]),
        }
    }
}

/// Maps mouse buttons to actions
#[derive(Resource)]
pub struct MouseButtonActionMap {
    map: HashMap<MouseButton, Action>,
}

impl Default for MouseButtonActionMap {
    fn default() -> Self {
        Self {
            map: HashMap::from([
                (MouseButton::Left, Action::Primary),
                (MouseButton::Right, Action::Secondary),
            ]),
        }
    }
}

/// Maps keyboard codes to actions
#[derive(Resource)]
pub struct KeyActionMap {
    map: HashMap<KeyCode, Action>,
}

impl Default for KeyActionMap {
    fn default() -> Self {
        Self {
            map: HashMap::from([
                (KeyCode::Space, Action::Jump),
                (KeyCode::KeyE, Action::Activate),
            ]),
        }
    }
}

fn sys_process_keycode_inputs(
    key_actions: Res<KeyActionMap>,
    key_events: Res<ButtonInput<KeyCode>>,
    mut button_event_writer: EventWriter<ActionEvent<ButtonAction>>,
) {
    let pressed = key_events.get_just_pressed().filter_map(|code| {
        key_actions
            .map
            .get(code)
            .map(|action| ActionEvent::<ButtonAction>::create(*action, ButtonAction::Pressed))
    });
    let released = key_events.get_just_released().filter_map(|code| {
        key_actions
            .map
            .get(code)
            .map(|action| ActionEvent::<ButtonAction>::create(*action, ButtonAction::Released))
    });
    let held = key_events.get_pressed().filter_map(|code| {
        key_actions
            .map
            .get(code)
            .map(|action| ActionEvent::<ButtonAction>::create(*action, ButtonAction::Held))
    });

    button_event_writer.send_batch(held.chain(released.chain(pressed)));
}

fn sys_process_keyboard_axes(
    key_inputs: Res<ButtonInput<KeyCode>>,
    key_axis_map: Res<KeyAxisMap>,
    mut input_axes: ResMut<InputAxes>,
) {

    for ev in key_inputs.get_pressed() {
        if let Some(action) = key_axis_map.map.get(ev) {
            match action {
                Axis::Movement(dir) => {
                    input_axes.movement += *dir;
                }
                Axis::Look(dir) => {
                    input_axes.look += *dir;
                }
            }
        }
    }
}

fn sys_process_mouse_inputs(
    mouse_button_action_map: Res<MouseButtonActionMap>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut button_event_writer: EventWriter<ActionEvent<ButtonAction>>,
) {
    let mut event_buffer = vec![];

    for ev in mouse_buttons.get_just_pressed() {
        if let Some(action) = mouse_button_action_map.map.get(ev) {
            event_buffer.push((action, ButtonAction::Pressed));
        }
    }

    for ev in mouse_buttons.get_just_released() {
        if let Some(action) = mouse_button_action_map.map.get(ev) {
            event_buffer.push((action, ButtonAction::Released));
        }
    }

    for ev in mouse_buttons.get_pressed() {
        if let Some(action) = mouse_button_action_map.map.get(ev) {
            event_buffer.push((action, ButtonAction::Held));
        }
    }

    let events: Vec<ActionEvent<ButtonAction>> = event_buffer
        .iter()
        .map(|v| ActionEvent::<ButtonAction> {
            action: *v.0,
            data: v.1,
        })
        .collect();
    button_event_writer.send_batch(events);
}

// can't see a reason to support remapping mouselook lol
fn sys_get_mouselook(mut mouse_evr: EventReader<MouseMotion>, mut input_axes: ResMut<InputAxes>) {
    for ev in mouse_evr.read() {
        input_axes.look = ev.delta;
    }
}

fn sys_clear_axes(mut input_axes: ResMut<InputAxes>) {
    input_axes.look = Vec2::ZERO;
    input_axes.movement = Vec2::ZERO;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSystemSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionEvent<ButtonAction>>();
        app.insert_resource(KeyActionMap::default());
        app.insert_resource(KeyAxisMap::default());
        app.insert_resource(MouseButtonActionMap::default());
        app.insert_resource(InputAxes::default());
        app.add_systems(
            Update,
            (sys_clear_axes, (
                sys_process_mouse_inputs,
                sys_process_keycode_inputs,
                sys_process_keyboard_axes,
                sys_get_mouselook,
            )).chain()
                .in_set(InputSystemSet),
        );
    }
}

#[cfg(test)]
mod testing {

    use super::*;

    #[test]
    fn test_button_events() {
        let mut app = App::new();
        app.add_plugins((InputPlugin, bevy::input::InputPlugin));
        let mut button_input = app
            .world
            .get_resource_mut::<ButtonInput<KeyCode>>()
            .unwrap();
        button_input.press(KeyCode::Space);
        app.update();
        let events: Vec<ActionEvent<ButtonAction>> = app
            .world
            .get_resource_mut::<Events<ActionEvent<ButtonAction>>>()
            .unwrap()
            .drain()
            .collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].data == ButtonAction::Held);
        assert!(events[0].action == Action::Jump);
    }

    #[test]
    fn text_axis_events() {
        let mut app = App::new();
        app.add_plugins((InputPlugin, bevy::input::InputPlugin));
        let mut button_input = app
            .world
            .get_resource_mut::<ButtonInput<KeyCode>>()
            .unwrap();
        button_input.press(KeyCode::KeyW);
        button_input.press(KeyCode::ArrowRight);
        app.update();
        let axes = app.world.get_resource::<InputAxes>().unwrap();
        assert_eq!(axes.movement, Vec2::Y);
        assert_eq!(axes.look, Vec2::X);
    }
}
