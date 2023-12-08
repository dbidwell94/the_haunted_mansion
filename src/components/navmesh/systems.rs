use std::sync::{Arc, RwLock};

use crate::components::INT_TILE_SIZE;

use super::{components::*, MeshGrid, MoveRequest, NavmeshAnswerEvent, RebuildNavmesh};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use futures_lite::future;

pub fn update_navmesh(
    changed_walkables: Query<
        (&GridCoords, &WalkableState),
        Or<(Added<WalkableState>, Changed<WalkableState>)>,
    >,
    navmesh_grid: ResMut<MeshGrid>,
) {
    for (coords, walkable) in &changed_walkables {
        navmesh_grid
            .grids_and_weights
            .write()
            .unwrap()
            .insert(*coords, walkable.clone());
    }
}

pub fn rebuild_navmesh(
    mut reset_request: EventReader<RebuildNavmesh>,
    walkables: Query<(&GridCoords, &WalkableState), With<WalkableState>>,
    navmesh_grid: ResMut<MeshGrid>,
) {
    let mut reset = false;
    for _ in &mut reset_request.read() {
        reset = true;
    }

    if !reset {
        return;
    }

    let mut grid = navmesh_grid.grids_and_weights.write().unwrap();

    grid.clear();
    for (coords, walkable) in &walkables {
        grid.insert(*coords, walkable.clone());
    }
}

pub fn listen_for_navmesh_requests(
    mut commands: Commands,
    mut request_listener: EventReader<MoveRequest>,
    navmesh_grid: Res<MeshGrid>,
) {
    if request_listener.is_empty() {
        return;
    }
    let thread_pool = AsyncComputeTaskPool::get();
    for move_request in &mut request_listener.read() {
        let move_request = move_request.clone();
        let arc_grid = navmesh_grid.grids_and_weights.clone();
        let task = thread_pool.spawn(async move { pathfind(move_request, arc_grid) });
        commands.spawn(PathfindingTask(task));
    }
}

pub fn poll_for_pathfinding_completion(
    mut commands: Commands,
    mut pathfinding_tasks: Query<(&mut PathfindingTask, Entity)>,
    mut event_sender: EventWriter<NavmeshAnswerEvent>,
) {
    for (mut pathfinding_task, entity) in &mut pathfinding_tasks {
        if let Some(item) = future::block_on(future::poll_once(&mut pathfinding_task.0)) {
            commands.entity(entity).despawn();
            event_sender.send(item);
        }
    }
}

/// A* implementation
pub fn pathfind(
    request: MoveRequest,
    grid: Arc<RwLock<HashMap<GridCoords, WalkableState>>>,
) -> NavmeshAnswerEvent {
    use pathfinding::prelude::*;

    let Ok(grid) = grid.read() else {
        return NavmeshAnswerEvent {
            path: Err(()),
            requesting_entity: request.requesting_entity,
        };
    };

    let result = astar(
        &request.move_from,
        |&coord| {
            let up = GridCoords::new(coord.x, coord.y + 1);
            let right = GridCoords::new(coord.x + 1, coord.y);
            let down = GridCoords::new(coord.x, coord.y - 1);
            let left = GridCoords::new(coord.x - 1, coord.y);

            let neighbors = [up, down, left, right]
                .iter()
                .filter(|&coord| {
                    grid.contains_key(coord)
                        && grid.get(coord).unwrap() != &WalkableState::NotWalkable
                })
                .map(|coord| (coord.clone(), 1))
                .collect::<Vec<_>>();
            neighbors
        },
        |&a| {
            (Vec2::new(a.x as f32, a.y as f32)
                - Vec2::new(request.move_to.x as f32, request.move_to.y as f32))
            .length() as i32
        },
        |&p| p == request.move_to,
    );

    let Some((path, _)) = result else {
        return NavmeshAnswerEvent {
            path: Err(()),
            requesting_entity: request.requesting_entity,
        };
    };

    return NavmeshAnswerEvent {
        requesting_entity: request.requesting_entity,
        path: Ok(path),
    };
}

pub fn debug_tiles(
    mut gizmos: Gizmos,
    tiles: Query<(&WalkableState, &GridCoords)>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Insert) {
        for (walkable, coords) in &tiles {
            let color = match walkable {
                WalkableState::NotWalkable => Color::RED,
                WalkableState::Walkable => Color::GREEN,
            };
            gizmos.rect_2d(
                Vec2::new(
                    INT_TILE_SIZE * coords.x as f32 + INT_TILE_SIZE / 2.,
                    INT_TILE_SIZE * coords.y as f32 + INT_TILE_SIZE / 2.,
                ),
                0.,
                Vec2::new(INT_TILE_SIZE, INT_TILE_SIZE),
                color,
            );
        }
    }
}
