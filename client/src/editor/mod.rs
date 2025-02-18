use crate::{controls::cameras::free_cam, input, render::terrain, events};
use bevy::prelude::*;

#[derive(Resource, Default)]
struct PlacePreview(terrain::Voxel);

#[derive(Resource, Default)]
struct EditorTerrain(terrain::VoxelTerrain);

fn sys_spawn(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), free_cam::FreeCamera::default()));
}

fn sys_draw_preview_gizmos(
    mut gizmos: Gizmos,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut place_preview: ResMut<PlacePreview>,
) {
    let (camera, camera_trans) = camera_query.single();
    let world_space_coords = camera
        .viewport_to_world(camera_trans, Vec2::ZERO)
        .unwrap()
        .origin;

    place_preview.0 = (world_space_coords + (camera_trans.forward() * 5.0))
        .round()
        .into();
    gizmos.cuboid(
        Transform::from_translation(place_preview.0.into())
            .with_scale(Vec3::ONE * terrain::VOXEL_SIZE as f32),
        Color::rgba(0.0, 0.8, 0.8, 0.9),
    );
}

fn sys_add_terrain(
    place_preview: ResMut<PlacePreview>,
    mut editor_terrain: ResMut<EditorTerrain>,
    mut terrain_event_send: EventWriter<events::GenerateTerrainEvent>,
    mut button_state: ResMut<input::ActionButtons>,
) {
    if button_state.get_button_state(input::Action::Primary) == input::ButtonState::Pressed {
        if let Some(i) = editor_terrain.0.find(place_preview.0) {
            editor_terrain.0.remove(i);
        } else {
            editor_terrain.0.add(place_preview.0);
        }
        terrain_event_send.send(events::GenerateTerrainEvent {
            terrain: editor_terrain.0.clone(), //ew
        });
    }
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(free_cam::FreeCameraPlugin);
        app.insert_resource(EditorTerrain::default());
        app.insert_resource(PlacePreview::default());
        app.add_systems(Startup, sys_spawn);
        app.add_systems(Update, (sys_add_terrain, sys_draw_preview_gizmos));
    }
}
