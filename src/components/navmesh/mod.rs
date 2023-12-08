mod components;
mod systems;

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::prelude::*;
use std::sync::{Arc, RwLock};
use systems::*;

pub use components::{NavmeshBundle, NavmeshParent, NavmeshTileBundle, WalkableState};

pub struct NavmeshPlugin;

impl Plugin for NavmeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NavmeshAnswerEvent>()
            .add_event::<MoveRequest>()
            .add_event::<RebuildNavmesh>()
            .init_resource::<MeshGrid>()
            .add_systems(Update, update_navmesh)
            .add_systems(Update, listen_for_navmesh_requests)
            .add_systems(Update, rebuild_navmesh)
            .add_systems(Update, debug_tiles)
            .add_systems(Update, poll_for_pathfinding_completion);
    }
}

#[derive(Resource, Default)]
pub struct MeshGrid {
    grids_and_weights: Arc<RwLock<HashMap<GridCoords, WalkableState>>>,
}

#[derive(Event, Debug)]
pub struct NavmeshAnswerEvent {
    pub requesting_entity: Entity,
    pub path: Result<Vec<GridCoords>, ()>,
}

#[derive(Event, Copy, Clone)]
pub struct MoveRequest {
    pub requesting_entity: Entity,
    pub move_from: GridCoords,
    pub move_to: GridCoords,
}

#[derive(Event)]
pub struct RebuildNavmesh;
