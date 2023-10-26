use bevy::prelude::*;
mod game_ui;

pub use game_ui::classes;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game_ui::GameUiPlugin);
    }
}
