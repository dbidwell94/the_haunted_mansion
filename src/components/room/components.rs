use super::RoomLevel;
use crate::components::card::CardType;
use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use derivative::Derivative;

pub mod ldtk {
    use bevy::prelude::*;
    use bevy_ecs_ldtk::prelude::*;
    #[derive(Component, Default)]
    pub struct NonWalkable;

    #[derive(LdtkIntCell, Bundle, Default)]
    pub struct NonWalkableBundle {
        non_walkable: NonWalkable,
    }

    #[derive(Component, Default)]
    pub struct RoomBound;

    #[derive(Component)]
    pub struct RoomBoundComponent;

    #[derive(LdtkIntCell, Bundle, Default)]
    pub struct RoomBoundBundle {
        room_bound: RoomBound,
    }

    #[derive(Component, Default)]
    pub struct Walkable;

    #[derive(Component)]
    pub struct WalkableComponent;

    #[derive(LdtkIntCell, Bundle, Default)]
    pub struct WalkableBundle {
        walkable: Walkable,
    }
}


#[derive(Component, Default, Debug)]
pub struct Wall;

#[derive(Bundle, LdtkIntCell, Default, Debug)]
pub struct WallBundle {
    pub wall: Wall,
}

#[derive(Bundle)]
pub struct RoomBundle {
    pub ldtk: LdtkWorldBundle,
    pub name: Name,
    pub room: Room,
    pub location: GridCoords,
}

#[derive(Derivative, Component, Debug, Clone, Reflect)]
#[derivative(Hash, Eq, PartialEq)]
pub struct Room {
    pub name: String,
    pub iid: String,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    pub room_level: HashSet<RoomLevel>,
    pub allowed_copies: u8,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    pub card: Option<CardType>,

    /// ```rust
    /// // up    = 0b1000
    /// // right = 0b0100
    /// // down  = 0b0010
    /// // left  = 0b0001
    /// ```
    pub door_connections: u8,
}
