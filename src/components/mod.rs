use bevy::prelude::*;

#[allow(dead_code)]
mod card;
mod character;
mod navmesh;
mod room;

pub use navmesh::{
    MoveRequest, NavmeshAnswerEvent, NavmeshBundle, NavmeshTileBundle, RebuildNavmesh,
    Walkable,
};
pub use room::{Room, INT_TILE_SIZE};

pub use character::spawn_character_player;
pub use room::setup_first_rooms;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            room::RoomPlugin,
            character::CharacterPlugin,
            navmesh::NavmeshPlugin,
        ));
    }
}
