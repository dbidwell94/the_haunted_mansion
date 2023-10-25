use bevy::prelude::*;

mod character;
mod room;
mod card;

pub use room::Room;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((room::RoomPlugin, character::CharacterPlugin));
    }
}
