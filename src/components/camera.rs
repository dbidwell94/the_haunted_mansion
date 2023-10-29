use super::{character::CharacterInput, spawn_character_player, MouseToWorldCoords, Selectable};
use crate::ui::OccludeUI;
use crate::GameState;
use bevy::math::Vec3Swizzles;
use bevy::render::primitives::Aabb;
use bevy::sprite::collide_aabb::collide;
use bevy::window::PrimaryWindow;
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
        .add_systems(Update, move_camera_anchor.run_if(in_state(GameState::Main)))
        .add_systems(
            Update,
            focus_on_selectable.run_if(in_state(GameState::Main)),
        )
        .add_systems(
            Update,
            update_mouse_coords.run_if(in_state(GameState::Main)),
        );
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

    commands.spawn((camera_bundle, PlayerCamera));
}

fn move_camera_anchor(
    mut commands: Commands,
    mut anchor: Query<
        (&mut Transform, Option<&Parent>, Entity, &GlobalTransform),
        With<CameraAnchor>,
    >,
    input: Query<&ActionState<CharacterInput>>,
    time: Res<Time>,
) {
    let Ok((mut transform, parent, entity, global_transform)) = anchor.get_single_mut() else {
        return;
    };
    let Ok(input) = input.get_single() else {
        return;
    };

    let Some(input) = input.clamped_axis_pair(CharacterInput::MoveCamera) else {
        return;
    };

    let vec_input = input.xy().normalize_or_zero();

    if parent.is_some() && (vec_input.x != 0. || vec_input.y != 0.) {
        let new_transform = global_transform.compute_transform();
        transform.translation = new_transform.translation;
        commands.entity(entity).remove_parent();
    }

    transform.translation += vec_input.extend(0.) * CAMERA_MOVE_SPEED * time.delta_seconds();
}

fn lerp_to_object(
    mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<CameraAnchor>)>,
    entities: Query<&GlobalTransform, With<CameraAnchor>>,
) {
    let Ok(mut transform) = cam_query.get_single_mut() else {
        return;
    };
    let Ok(anchor_transform) = entities.get_single() else {
        return;
    };

    transform.translation = transform
        .translation
        .lerp(anchor_transform.translation(), 0.15);
}

fn focus_on_selectable(
    mut commands: Commands,
    mouse_coords: Res<MouseToWorldCoords>,
    player_input: Query<&ActionState<CharacterInput>>,
    selectables: Query<(Entity, &GlobalTransform, &Aabb), With<Selectable>>,
    mut camera_anchor: Query<(Entity, &mut Transform), With<CameraAnchor>>,
) {
    let Ok((camera_anchor, mut camera_transform)) = camera_anchor.get_single_mut() else {
        return;
    };
    let Ok(player_input) = player_input.get_single() else {
        return;
    };
    if !player_input.just_pressed(CharacterInput::SelectObject) {
        return;
    }
    let Some(mouse_coords) = mouse_coords.0 else {
        return;
    };

    for (selectable, transform, aabb) in &selectables {
        if collide(
            transform.translation(),
            aabb.half_extents.truncate() * 2.,
            mouse_coords.extend(0.),
            Vec2::ONE,
        )
        .is_some()
        {
            commands.entity(camera_anchor).set_parent(selectable);
            camera_transform.translation = Vec3::ZERO;
        }
    }
}

fn update_mouse_coords(
    mut mouse_to_world_coords: ResMut<MouseToWorldCoords>,
    occluded_ui: Query<(&GlobalTransform, &Node), With<OccludeUI>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let world_pos = camera
        .viewport_to_world(camera_transform, cursor_position.clone())
        .map(|ray| ray.origin.truncate());

    if let Some(world_pos) = world_pos {
        let mut occluded = false;
        for (global_transform, node) in &occluded_ui {
            let ui_bounds = node.logical_rect(global_transform);

            occluded = ui_bounds.contains(cursor_position);
        }

        if !occluded {
            mouse_to_world_coords.0 = Some(world_pos);
        } else {
            mouse_to_world_coords.0 = None;
        }
    }
}
