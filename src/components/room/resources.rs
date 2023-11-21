use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;

use super::Room;

#[derive(AssetCollection, Resource)]
pub struct RoomAssets {
    #[asset(path = "ldtk/haunted.ldtk")]
    pub ldtk_asset: Handle<LdtkProject>,
}

#[repr(u8)]
pub enum DoorLocation {
    Up = 0b1000,
    Right = 0b0100,
    Down = 0b0010,
    Left = 0b0001,
}

#[derive(Resource, Clone, Default)]
pub struct RoomCounter {
    pub rooms: HashMap<Room, u8>,
    pub filled_tiles: HashMap<(i32, i32, i8), Room>,
    pub spawnable_ground: HashMap<Room, u8>,
    pub spawnable_basement: HashMap<Room, u8>,
    pub spawnable_upper: HashMap<Room, u8>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Reflect)]
#[allow(dead_code)]
pub enum RoomLevel {
    Basement,
    Ground,
    Upper,
}

#[derive(Event, Hash, PartialEq, Eq)]
pub struct RoomBoundsHitEvent {
    pub character_entity: Entity,
    pub room_entity: Entity,
    pub room: Room,
}

#[repr(i32)]
#[allow(dead_code)]
pub enum LayerMask {
    NonWalkable = 1,
    RoomBound = 2,
    Walkable = 3,
}
