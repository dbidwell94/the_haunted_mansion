use bevy::prelude::*;

mod character;
mod room;
#[allow(dead_code)]
mod card;

pub use room::Room;

pub use room::setup_first_rooms;
pub use character::spawn_character;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((room::RoomPlugin, character::CharacterPlugin));
    }
}
