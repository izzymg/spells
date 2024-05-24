use crate::{controls::cameras::free_cam, input, ui, window};
use bevy::prelude::*;
use lib_spells::shared;

#[derive(Component, Debug)]
struct NamedObject;

fn sys_despawn_on_press(
    mut commands: Commands,
    buttons: Res<input::ActionButtons>,
    has_name: Query<Entity, With<NamedObject>>,
) {
    if buttons.get_button_state(input::Action::Jump) == input::ButtonState::Pressed {
        if let Some(obj) = has_name.iter().next() {
            commands.entity(obj).despawn_recursive();
        }
    }
}

fn sys_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut next_window_ctx: ResMut<NextState<window::WindowContext>>,
) {
    next_window_ctx.set(window::WindowContext::Play);

    let cube_mesh = meshes.add(Cuboid::default());
    let std_mat = mats.add(StandardMaterial {
        base_color: Color::BLUE,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: std_mat.clone(),
            ..default()
        },
        shared::Name("test!".into()),
        NamedObject,
    ));
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: std_mat.clone(),
            transform: Transform::from_translation(Vec3::new(-5., -3., 0.)),
            ..default()
        },
        shared::Name("test 2".into()),
        NamedObject,
    ));
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: std_mat.clone(),
            transform: Transform::from_translation(Vec3::new(5., 1., 0.))
                .with_scale(Vec3::new(2., 2., 2.)),
            ..default()
        },
        shared::Name("test3".into()),
        NamedObject,
    ));

    commands.spawn((Camera3dBundle::default(), free_cam::FreeCamera::default()));
}

pub struct DevGameUIPlugin;

impl Plugin for DevGameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((free_cam::FreeCameraPlugin));
        app.add_systems(Startup, sys_spawn);
        app.add_systems(Update, (sys_despawn_on_press));
    }
}
