use bevy::{prelude::*, tasks::Task};
use bevy_ecs_ldtk::prelude::*;

use super::NavmeshAnswerEvent;

#[derive(Component, Clone, Eq, PartialEq)]
pub enum WalkableState {
    NotWalkable,
    Walkable,
}

#[derive(Component, Default)]
pub struct NavmeshParent;

#[derive(Bundle, Default)]
pub struct NavmeshBundle {
    pub transform: TransformBundle,
    pub name: Name,
    pub navmesh: NavmeshParent,
}

#[derive(Bundle)]
pub struct NavmeshTileBundle {
    pub walkable: WalkableState,
    pub grid_coord: GridCoords,
    pub transform: TransformBundle,
}

#[derive(Component)]
pub struct PathfindingTask(pub Task<NavmeshAnswerEvent>);
