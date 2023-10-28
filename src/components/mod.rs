use bevy::prelude::*;

mod camera;
#[allow(dead_code)]
mod card;
mod character;
mod navmesh;
mod room;

pub use character::spawn_character_player;
pub use navmesh::{
    MoveRequest, NavmeshAnswerEvent, NavmeshBundle, NavmeshTileBundle, RebuildNavmesh,
    WalkableState,
};
pub use room::setup_first_rooms;
pub use room::{Room, INT_TILE_SIZE};

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            room::RoomPlugin,
            character::CharacterPlugin,
            navmesh::NavmeshPlugin,
            camera::CameraPlugin,
        ));
    }
}
