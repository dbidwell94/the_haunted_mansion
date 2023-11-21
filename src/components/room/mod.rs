mod components;
mod resources;
mod systems;

use super::navmesh::*;
use crate::GameState;

use bevy::{prelude::*, utils::HashSet};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use components::ldtk::*;
pub use components::*;
use lazy_static::lazy_static;
pub use resources::*;
pub use systems::*;

pub const ROOM_SIZE: f32 = 96.0;
pub const INT_TILE_SIZE: f32 = 8.;

lazy_static! {
    pub static ref LDTK_ROOMS: [Room; 3] = [
        Room {
            name: "Entryway".into(),
            allowed_copies: 1,
            iid: "ac9b33c2-6280-11ee-baef-b119038a937a".into(),
            room_level: HashSet::from([RoomLevel::Ground]),
            card: None,
            door_connections: 0b0100
        },
        Room {
            name: "Hallway-2x0y".into(),
            allowed_copies: 4,
            iid: "078ebb40-6280-11ee-81c9-dd1f0b0b06bd".into(),
            room_level: HashSet::from([RoomLevel::Ground, RoomLevel::Upper]),
            card: None,
            door_connections: 0b0101
        },
        Room {
            name: "Hallway-2x2y".into(),
            allowed_copies: 2,
            iid: "f2a4fac0-6280-11ee-8d3e-0d30e91a8fca".into(),
            room_level: HashSet::from([RoomLevel::Ground, RoomLevel::Upper]),
            card: None,
            door_connections: 0b1111
        }
    ];
}

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, RoomAssets>(GameState::Loading)
            .init_resource::<RoomCounter>()
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseZeroTranslation,
                int_grid_rendering: IntGridRendering::Invisible,
                ..default()
            })
            .register_type::<Room>()
            .register_type::<HashSet<RoomLevel>>()
            .add_event::<RoomBoundsHitEvent>()
            .register_ldtk_int_cell::<NonWalkableBundle>(LayerMask::NonWalkable as i32)
            .register_ldtk_int_cell::<RoomBoundBundle>(LayerMask::RoomBound as i32)
            .register_ldtk_int_cell::<WalkableBundle>(LayerMask::Walkable as i32)
            .add_systems(OnEnter(GameState::InitialSpawn), setup_first_rooms)
            .add_systems(OnEnter(GameState::InitialSpawn), create_navmesh)
            .add_systems(Update, spawn_wall_colliders)
            .add_systems(Update, spawn_room_bounds)
            .add_systems(Update, spawn_walkable_navtiles)
            .add_systems(Update, check_room_entry_or_exit);
    }
}
