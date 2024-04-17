use bevy::{input::mouse::MouseMotion, prelude::*};
use std::collections::HashMap;

#[derive(Hash, Eq, Copy, PartialEq, Debug, Clone)]
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

#[derive(Default, PartialEq, Eq, Copy, Clone, Debug)]
pub enum ButtonState {
    #[default]
    None,
    Pressed,
    Released,
    Held,
}

/// Hardware-type agnostic input axis state
#[derive(Resource, Default, Copy, Clone, Debug)]
pub struct ActionAxes {
    pub movement: Vec2,
    pub look: Vec2,
}

impl ActionAxes {
    pub fn get_movement_3d(&self) -> Vec3 {
        // -z goes forward in bevy
        Vec3::new(self.movement.x, 0.0, -self.movement.y)
    }
}

/// Hardware-type agnostic input button state
#[derive(Resource, Default, Clone, Debug)]
pub struct ActionButtons {
    map: HashMap<Action, ButtonState>,
}

impl ActionButtons {
    fn set_state(&mut self, action: Action, state: ButtonState) {
        self.map.insert(action, state);
    }

    // lazily creates state for actions
    pub fn get_button_state(&mut self, action: Action) -> ButtonState {
        if let Some(state) = self.map.get(&action) {
            *state
        } else {
            self.map.insert(action, ButtonState::default());
            ButtonState::default()
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Reflect)]
enum Input {
    MouseButton(MouseButton),
    KeyCode(KeyCode),
}

/// Maps buttons to actions
#[derive(Resource)]
struct InputButtonActionMap(HashMap<Input, Action>);

impl Default for InputButtonActionMap {
    fn default() -> Self {
        Self(HashMap::from([
            (Input::KeyCode(KeyCode::Space), Action::Jump),
            (Input::KeyCode(KeyCode::KeyE), Action::Activate),
            (Input::MouseButton(MouseButton::Left), Action::Primary),
            (Input::MouseButton(MouseButton::Right), Action::Secondary),
        ]))
    }
}

/// Maps key codes to axes
#[derive(Resource)]
struct KeyAxisMap {
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

fn input_to_button_state<T: Send + Copy + std::hash::Hash + Eq + Sync>(code: T, input: &ButtonInput<T>) -> ButtonState {
   if input.just_pressed(code) {
       ButtonState::Pressed
   } else if input.just_released(code) {
       ButtonState::Released
   } else if input.pressed(code) {
       ButtonState::Held
   } else {
       ButtonState::None
   }
}

fn sys_process_buttons(
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    key_inputs: Res<ButtonInput<KeyCode>>,
    input_buttons: Res<InputButtonActionMap>,
    mut action_state: ResMut<ActionButtons>,
) {
    let key_inputs = key_inputs.into_inner();
    let mouse_inputs = mouse_inputs.into_inner();
    for (input, action) in input_buttons.0.iter() {
        let state = match input {
            Input::KeyCode(key) =>
                input_to_button_state(*key, key_inputs),
            Input::MouseButton(btn) =>
                input_to_button_state(*btn, mouse_inputs)
        };
        
        action_state.set_state(*action, state);
    }
}

fn sys_process_keyboard_axes(
    key_inputs: Res<ButtonInput<KeyCode>>,
    key_axis_map: Res<KeyAxisMap>,
    mut input_axes: ResMut<ActionAxes>,
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

// can't see a reason to support remapping mouselook lol
fn sys_get_mouselook(mut mouse_evr: EventReader<MouseMotion>, mut input_axes: ResMut<ActionAxes>) {
    for ev in mouse_evr.read() {
        input_axes.look = ev.delta;
    }
}

fn sys_clear_axes(mut input_axes: ResMut<ActionAxes>) {
    input_axes.look = Vec2::ZERO;
    input_axes.movement = Vec2::ZERO;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSystemSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyAxisMap::default());
        app.insert_resource(InputButtonActionMap::default());
        app.insert_resource(ActionAxes::default());
        app.insert_resource(ActionButtons::default());
        app.add_systems(
            Update,
            (
                sys_clear_axes,
                (
                    sys_process_buttons,
                    sys_process_keyboard_axes,
                    sys_get_mouselook,
                ),
            )
                .chain()
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
        let mut button_state = app.world.get_resource_mut::<ActionButtons>().unwrap();
        assert!(button_state.get_button_state(Action::Jump) == ButtonState::Held);
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
        let axes = app.world.get_resource::<ActionAxes>().unwrap();
        assert_eq!(axes.movement, Vec2::Y);
        assert_eq!(axes.look, Vec2::X);
    }
}
