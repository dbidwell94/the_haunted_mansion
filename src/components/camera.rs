use super::{character::CharacterInput, spawn_character_player};
use crate::GameState;
use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use leafwing_input_manager::prelude::*;

const CAMERA_MOVE_SPEED: f32 = 200.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitialSpawn),
            spawn_camera.after(spawn_character_player),
        )
        .add_systems(Update, lerp_to_object.run_if(in_state(GameState::Main)))
        .add_systems(Update, move_camera_anchor.run_if(in_state(GameState::Main)));
    }
}

#[derive(Component, Default)]
pub struct PlayerCamera;

#[derive(Component)]
struct CameraAnchor;

fn spawn_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;
    camera_bundle.camera_2d.clear_color = ClearColorConfig::Custom(Color::BLACK);

    commands.spawn((
        TransformBundle::default(),
        Name::new("Camera Anchor"),
        CameraAnchor,
    ));

    commands.spawn(camera_bundle).insert(PlayerCamera);
}

fn move_camera_anchor(
    mut anchor: Query<&mut Transform, With<CameraAnchor>>,
    input: Query<&ActionState<CharacterInput>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = anchor.get_single_mut() else {
        return;
    };
    let Ok(input) = input.get_single() else {
        return;
    };

    let Some(input) = input.clamped_axis_pair(CharacterInput::MoveCamera) else {
        return;
    };

    let vec_input = input.xy().normalize_or_zero();

    transform.translation += vec_input.extend(0.) * CAMERA_MOVE_SPEED * time.delta_seconds();
}

fn lerp_to_object(
    mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<CameraAnchor>)>,
    entities: Query<&Transform, With<CameraAnchor>>,
) {
    let Ok(mut transform) = cam_query.get_single_mut() else {
        return;
    };

    let Ok(anchor_transform) = entities.get_single() else {
        return;
    };

    transform.translation = transform
        .translation
        .lerp(anchor_transform.translation, 0.15);
}
