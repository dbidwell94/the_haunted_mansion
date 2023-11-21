use crate::GameState;
use bevy::prelude::*;
mod components;
mod systems;
use systems::*;

pub const CAMERA_MOVE_SPEED: f32 = 200.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_camera)
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
