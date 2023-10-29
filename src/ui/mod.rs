use bevy::prelude::*;
use bevy::window::PrimaryWindow;

mod game_ui;

use crate::components::MouseToWorldCoords;
pub use game_ui::classes;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game_ui::GameUiPlugin);
    }
}

#[derive(Component)]
pub struct OccludeUI;
