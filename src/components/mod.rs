use bevy::prelude::*;

#[allow(dead_code)]
mod card;
mod character;
mod room;

pub use room::Room;

pub use character::spawn_character;
pub use room::setup_first_rooms;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((room::RoomPlugin, character::CharacterPlugin));
    }
}
