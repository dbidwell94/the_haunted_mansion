use bevy::gizmos::prelude::*;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_ecs_ldtk::prelude::*;
use futures_lite::future;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

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
struct MeshGrid {
    grids_and_weights: Arc<RwLock<HashMap<GridCoords, Walkable>>>,
}

#[derive(Component, Clone, Eq, PartialEq)]
pub enum Walkable {
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
    pub walkable: Walkable,
    pub grid_coord: GridCoords,
    pub transform: TransformBundle,
}

#[derive(Event)]
pub struct NavmeshAnswerEvent {
    pub requesting_entity: Entity,
    pub path: Result<VecDeque<GridCoords>, ()>,
}

#[derive(Component)]
struct PathfindingTask(Task<NavmeshAnswerEvent>);

#[derive(Event, Copy, Clone)]
pub struct MoveRequest {
    pub requesting_entity: Entity,
    pub move_from: GridCoords,
    pub move_to: GridCoords,
}

#[derive(Event)]
pub struct RebuildNavmesh;

fn update_navmesh(
    changed_walkables: Query<(&GridCoords, &Walkable), Or<(Added<Walkable>, Changed<Walkable>)>>,
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

fn rebuild_navmesh(
    mut reset_request: EventReader<RebuildNavmesh>,
    walkables: Query<(&GridCoords, &Walkable), With<Walkable>>,
    navmesh_grid: ResMut<MeshGrid>,
) {
    let mut reset = false;
    for _ in &mut reset_request {
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

fn listen_for_navmesh_requests(
    mut commands: Commands,
    mut request_listener: EventReader<MoveRequest>,
    navmesh_grid: Res<MeshGrid>,
) {
    if request_listener.is_empty() {
        return;
    }
    let thread_pool = AsyncComputeTaskPool::get();
    for move_request in &mut request_listener {
        let move_request = move_request.clone();
        let arc_grid = navmesh_grid.grids_and_weights.clone();
        let task = thread_pool.spawn(async move { pathfind(move_request, arc_grid) });
        commands.spawn(PathfindingTask(task));
    }
}

fn poll_for_pathfinding_completion(
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
fn pathfind(
    request: MoveRequest,
    grid: Arc<RwLock<HashMap<GridCoords, Walkable>>>,
) -> NavmeshAnswerEvent {
    use derivative::Derivative;
    let Ok(grid) = grid.as_ref().read().map_err(|_| ()) else {
        return NavmeshAnswerEvent {
            path: Err(()),
            requesting_entity: request.requesting_entity,
        };
    };

    #[derive(Derivative, Default, Debug, Clone)]
    #[derivative(Hash, PartialEq, Eq)]
    struct AStarNode {
        /// distance from end node
        #[derivative(Hash = "ignore", PartialEq = "ignore", Ord = "ignore")]
        h_cost: f32,
        /// distance from starting node
        #[derivative(Hash = "ignore", PartialEq = "ignore", Ord = "ignore")]
        g_cost: f32,
        tile: GridCoords,
        #[derivative(Ord = "ignore")]
        parent: Option<Box<AStarNode>>,
    }

    impl AStarNode {
        pub fn f_cost(&self) -> f32 {
            self.g_cost + self.h_cost
        }
    }

    impl PartialOrd for AStarNode {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.f_cost().partial_cmp(&other.f_cost())
        }
    }

    impl Ord for AStarNode {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.f_cost().total_cmp(&other.f_cost())
        }
    }

    fn get_distance_between(coord1: &GridCoords, coord2: &GridCoords) -> f32 {
        (Vec2::new(coord1.x as f32, coord1.y as f32) - Vec2::new(coord2.x as f32, coord2.y as f32))
            .length()
    }

    let mut open_routes = Vec::<AStarNode>::new();
    let mut closed_routes = HashSet::<AStarNode>::new();

    fn get_item_reference<'a>(
        item: &AStarNode,
        array: &'a mut Vec<AStarNode>,
    ) -> Option<&'a mut AStarNode> {
        for reference in array {
            if reference.tile == item.tile {
                return Some(reference);
            }
        }
        None
    }

    let Some(_) = grid.get(&request.move_from) else {
        return NavmeshAnswerEvent {
            path: Err(()),
            requesting_entity: request.requesting_entity,
        };
    };

    open_routes.push(AStarNode {
        tile: request.move_from,
        g_cost: 0.,
        h_cost: get_distance_between(&request.move_from, &request.move_to),
        parent: None,
    });

    let found_tile: Option<AStarNode>;

    loop {
        open_routes.sort();
        let Some(current) = open_routes.pop() else {
            return NavmeshAnswerEvent {
                path: Err(()),
                requesting_entity: request.requesting_entity,
            };
        };
        let top = GridCoords::new(current.tile.x, current.tile.y + 1);
        let right = GridCoords::new(current.tile.x + 1, current.tile.y);
        let bottom = GridCoords::new(current.tile.x, current.tile.y - 1);
        let left = GridCoords::new(current.tile.x - 1, current.tile.y);

        if current.tile == request.move_to {
            found_tile = Some(current);
            break;
        }

        for neighbor in [top, right, bottom, left] {
            let mut neighbor_node = AStarNode {
                tile: neighbor,
                ..default()
            };
            if closed_routes.contains(&neighbor_node) {
                continue;
            }
            let Some(walkable) = grid.get(&neighbor) else {
                closed_routes.insert(neighbor_node);
                continue;
            };
            if walkable == &Walkable::NotWalkable {
                closed_routes.insert(neighbor_node);
                continue;
            }

            neighbor_node.g_cost = get_distance_between(&request.move_from, &neighbor);
            neighbor_node.h_cost = get_distance_between(&neighbor, &request.move_to);

            // todo! Get neighbor from open_routes and change it's parent to be current_node

            if let Some(neighbor_array_ref) = get_item_reference(&neighbor_node, &mut open_routes) {
                neighbor_array_ref.parent = Some(Box::new(current.clone()));
            }

            if neighbor_node.f_cost() < current.f_cost() {
                neighbor_node.parent = Some(Box::new(current.clone()));
            }
            open_routes.push(neighbor_node);
        }
    }

    let mut to_return = VecDeque::new();

    let mut current = found_tile.map(|v| Box::new(v));

    while let Some(current_node) = current {
        to_return.push_front(current_node.tile.clone());
        current = current_node.parent;
    }

    return NavmeshAnswerEvent {
        requesting_entity: request.requesting_entity,
        path: Ok(to_return),
    };
}

fn debug_tiles(
    mut gizmos: Gizmos,
    tiles: Query<(&Walkable, &GridCoords)>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Insert) {
        for (walkable, coords) in &tiles {
            let color = match walkable {
                Walkable::NotWalkable => Color::RED,
                Walkable::Walkable => Color::GREEN,
            };
            gizmos.rect_2d(
                Vec2::new(9. + coords.x as f32, 9. + coords.y as f32),
                0.,
                Vec2::new(9., 9.),
                color,
            );
        }
    }
}
