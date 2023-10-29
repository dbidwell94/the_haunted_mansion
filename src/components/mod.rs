use bevy::prelude::*;

mod camera;
#[allow(dead_code)]
mod card;
mod character;
mod navmesh;
mod network;
mod room;

#[derive(Component, Default)]
pub struct Selectable;

#[derive(Resource, Default)]
pub struct MouseToWorldCoords(Option<Vec2>);

pub use character::spawn_character_player;
pub use navmesh::{
    MoveRequest, NavmeshAnswerEvent, NavmeshBundle, NavmeshTileBundle, RebuildNavmesh,
    WalkableState,
};
pub use network::{GgrsConfig, LobbyConfig, NetworkCharacterInput, StartMultiplayer};
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
            network::NetworkPlugin,
        ))
        .init_resource::<MouseToWorldCoords>();
    }
}
