use super::{card::CardType, character::Player, navmesh::*};
use crate::prelude::*;
use crate::GameState;
use bevy::math::Vec3A;
use bevy::render::primitives::Aabb;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use derivative::Derivative;
use lazy_static::lazy_static;

const ROOM_SIZE: f32 = 96.0;
const INT_TILE_SIZE: f32 = 8.;
const INT_GRID_TILE_COUNT_PER_AXIS: i32 = 12;

fn room_location_to_position(location: (i32, i32)) -> Vec2 {
    Vec2::new(location.0 as f32 * ROOM_SIZE, location.1 as f32 * ROOM_SIZE)
}

lazy_static! {
    pub static ref LDTK_ROOMS: [Room; 3] = [
        Room {
            name: "Entryway".into(),
            allowed_copies: 1,
            iid: "ac9b33c2-6280-11ee-baef-b119038a937a".into(),
            room_level: HashSet::from([RoomLevel::Ground]),
            card: None
        },
        Room {
            name: "Hallway-2x0y".into(),
            allowed_copies: 4,
            iid: "078ebb40-6280-11ee-81c9-dd1f0b0b06bd".into(),
            room_level: HashSet::from([RoomLevel::Ground, RoomLevel::Upper]),
            card: None
        },
        Room {
            name: "Hallway-2x2y".into(),
            allowed_copies: 2,
            iid: "f2a4fac0-6280-11ee-8d3e-0d30e91a8fca".into(),
            room_level: HashSet::from([]),
            card: None
        }
    ];
}

#[derive(PartialEq, Hash, Eq)]
pub enum RoomEnterExit {
    Enter,
    Exit,
}

#[derive(Event, Hash, PartialEq, Eq)]
pub struct RoomBoundsHitEvent {
    pub character_entity: Entity,
    pub room_entity: Entity,
    pub room: Room,
    pub door_location: DoorLocation,
    pub movement_type: RoomEnterExit,
}

#[derive(AssetCollection, Resource)]
pub struct RoomAssets {
    #[asset(path = "ldtk/haunted.ldtk")]
    ldtk_asset: Handle<LdtkAsset>,
}

#[derive(Component, Default)]
pub struct NonWalkable;

#[derive(LdtkIntCell, Bundle)]
pub struct NonWalkableBundle {
    non_walkable: NonWalkable,
}

#[derive(Component, Default)]
pub struct RoomBound;

#[derive(Component)]
struct RoomBoundComponent;

#[derive(LdtkIntCell, Bundle)]
pub struct RoomBoundBundle {
    room_bound: RoomBound,
}

#[derive(Component, Default)]
struct Walkable;

#[derive(Component)]
struct WalkableComponent;

#[derive(LdtkIntCell, Bundle)]
struct WalkableBundle {
    walkable: Walkable,
}

#[repr(i32)]
#[allow(dead_code)]
enum LayerMask {
    NonWalkable = 1,
    RoomBound = 2,
    Walkable = 3,
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

#[derive(Component, Reflect, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DoorLocation {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Derivative, Component, Debug, Clone, Reflect)]
#[derivative(Hash, Eq, PartialEq)]
pub struct Room {
    name: String,
    iid: String,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    room_level: HashSet<RoomLevel>,
    allowed_copies: u8,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    card: Option<CardType>,
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
            .register_type::<DoorLocation>()
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

#[derive(Component, Default, Debug)]
pub struct Wall;

#[derive(Bundle, LdtkIntCell, Default, Debug)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Bundle)]
struct RoomBundle {
    ldtk: LdtkWorldBundle,
    name: Name,
    room: Room,
    location: GridCoords,
}

pub fn setup_first_rooms(
    mut commands: Commands,
    room_assets: Res<RoomAssets>,
    mut room_counter: ResMut<RoomCounter>,
) {
    let entryway = LDTK_ROOMS.iter().find(|room| room.name == "Entryway");
    let entryway_location = room_location_to_position((0, 0));
    let hallway = LDTK_ROOMS.iter().find(|room| room.name == "Hallway-2x0y");
    let hallway_location = room_location_to_position((1, 0));
    let hallway_4way = LDTK_ROOMS.iter().find(|room| room.name == "Hallway-2x2y");
    let hallway_4way_location = room_location_to_position((1, 1));

    let (Some(entryway), Some(hallway), Some(hallway_4way)) = (entryway, hallway, hallway_4way)
    else {
        panic!("Cannot find the first rooms: 'entryway' -- 'hallway-2x0y'");
    };

    let ldtk_handle = &room_assets.ldtk_asset;

    commands.spawn(RoomBundle {
        ldtk: LdtkWorldBundle {
            ldtk_handle: ldtk_handle.clone(),
            level_set: LevelSet::from_iid(entryway.iid.to_owned()),
            transform: Transform::from_xyz(entryway_location.x, entryway_location.y, -1.),
            ..default()
        },
        name: Name::new(entryway.name.to_owned()),
        room: entryway.to_owned(),
        location: GridCoords { x: 0, y: 0 },
    });

    commands.spawn(RoomBundle {
        ldtk: LdtkWorldBundle {
            ldtk_handle: ldtk_handle.clone(),
            level_set: LevelSet::from_iid(hallway.iid.to_owned()),
            transform: Transform::from_xyz(hallway_location.x, hallway_location.y, -1.),
            ..default()
        },
        name: Name::new(hallway.name.to_owned()),
        room: hallway.to_owned(),
        location: GridCoords { x: 1, y: 0 },
    });

    commands.spawn(RoomBundle {
        ldtk: LdtkWorldBundle {
            ldtk_handle: ldtk_handle.clone(),
            level_set: LevelSet::from_iid(hallway_4way.iid.to_owned()),
            transform: Transform::from_xyz(hallway_4way_location.x, hallway_4way_location.y, -1.),
            ..default()
        },
        name: Name::new(hallway_4way.name.to_owned()),
        room: hallway_4way.to_owned(),
        location: GridCoords { x: 1, y: 1 },
    });

    room_counter.rooms.insert(entryway.to_owned(), 1);
    room_counter.rooms.insert(hallway.to_owned(), 1);
    room_counter.rooms.insert(hallway_4way.to_owned(), 1);

    room_counter
        .filled_tiles
        .insert((0, 0, 0), entryway.to_owned());
    room_counter
        .filled_tiles
        .insert((1, 0, 0), hallway.to_owned());
    room_counter
        .filled_tiles
        .insert((1, 1, 0), hallway_4way.to_owned());
}

fn spawn_wall_colliders(
    mut commands: Commands,
    non_walkable_query: Query<(&GridCoords, &Parent), Added<NonWalkable>>,
    parent_query: Query<&Parent, Without<NonWalkable>>,
    grandparent_query: Query<&Parent, With<Handle<LdtkLevel>>>,
    room_query: Query<(Entity, &GridCoords), With<Room>>,
    navmesh: Query<Entity, With<NavmeshParent>>,
) {
    let mut level_to_non_walkable_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    non_walkable_query.for_each(|(&grid_coords, parent)| {
        let Ok((room_entity, _)) = parent_query
            .get(parent.get())
            .and_then(|grandparent| grandparent_query.get(grandparent.get()))
            .and_then(|parent| room_query.get(parent.get()))
        else {
            return;
        };

        level_to_non_walkable_locations
            .entry(room_entity)
            .or_default()
            .insert(grid_coords);
    });

    for (entity, room_coords) in &room_query {
        let Some(grid_coords) = level_to_non_walkable_locations.get(&entity) else {
            continue;
        };

        for coord in grid_coords {
            let collider = commands
                .spawn((
                    Collider::cuboid(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2.),
                    CollisionGroups {
                        memberships: Group::GROUP_1,
                        ..default()
                    },
                    RigidBody::Fixed,
                    ActiveEvents::COLLISION_EVENTS,
                    Name::new("Non-Walkable Collider"),
                    TransformBundle {
                        local: Transform::from_xyz(
                            coord.x as f32 * INT_TILE_SIZE + INT_TILE_SIZE / 2.,
                            coord.y as f32 * INT_TILE_SIZE + INT_TILE_SIZE / 2.,
                            0.,
                        ),
                        ..Default::default()
                    },
                ))
                .id();

            commands.entity(entity).add_child(collider);

            let Ok(navmesh_entity) = navmesh.get_single() else {
                continue;
            };

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: room_coords.x + coord.x,
                            y: room_coords.y + coord.y,
                        },
                        walkable: super::Walkable::NotWalkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(
                                (coord.x as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.x as f32)),
                                (coord.y as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.y as f32)),
                                1.,
                            ),
                            ..default()
                        },
                    },
                    Name::new("NotWalkable"),
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    AabbGizmo {
                        color: Some(Color::RED),
                    },
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

fn spawn_room_bounds(
    mut commands: Commands,
    non_walkable_query: Query<(&GridCoords, &Parent), Added<RoomBound>>,
    parent_query: Query<&Parent, Without<RoomBound>>,
    grandparent_query: Query<&Parent, With<Handle<LdtkLevel>>>,
    room_query: Query<(Entity, &GridCoords), With<Room>>,
    navmesh: Query<Entity, With<NavmeshParent>>,
) {
    let mut level_to_room_bound_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    non_walkable_query.for_each(|(&grid_coords, parent)| {
        let Ok((room_entity, _)) = parent_query
            .get(parent.get())
            .and_then(|grandparent| grandparent_query.get(grandparent.get()))
            .and_then(|parent| room_query.get(parent.get()))
        else {
            return;
        };

        level_to_room_bound_locations
            .entry(room_entity)
            .or_default()
            .insert(grid_coords);
    });

    for (entity, room_coords) in &room_query {
        let Some(grid_coords) = level_to_room_bound_locations.get(&entity) else {
            continue;
        };
        let int_grid_bounds_max: i32 = INT_GRID_TILE_COUNT_PER_AXIS - 1;
        let int_grid_bounds_min: i32 = 0;

        for coord in grid_coords {
            let door_location: DoorLocation;
            if coord.x == int_grid_bounds_max {
                door_location = DoorLocation::Right;
            } else if coord.x == int_grid_bounds_min {
                door_location = DoorLocation::Left;
            } else if coord.y == int_grid_bounds_max {
                door_location = DoorLocation::Top;
            } else {
                door_location = DoorLocation::Bottom;
            }

            let collider = commands
                .spawn((
                    Collider::cuboid(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2.),
                    CollisionGroups {
                        memberships: Group::GROUP_2,
                        ..default()
                    },
                    ActiveEvents::COLLISION_EVENTS,
                    RigidBody::Fixed,
                    Sensor,
                    Name::new("Room Bound Sensor"),
                    TransformBundle {
                        local: Transform::from_xyz(
                            coord.x as f32 * INT_TILE_SIZE + INT_TILE_SIZE / 2.,
                            coord.y as f32 * INT_TILE_SIZE + INT_TILE_SIZE / 2.,
                            0.,
                        ),
                        ..Default::default()
                    },
                    RoomBoundComponent,
                    door_location,
                ))
                .id();

            commands.entity(entity).add_child(collider);

            let Ok(navmesh_entity) = navmesh.get_single() else {
                continue;
            };

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: room_coords.x + coord.x,
                            y: room_coords.y + coord.y,
                        },
                        walkable: super::Walkable::Walkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(
                                (coord.x as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.x as f32)),
                                (coord.y as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.y as f32)),
                                1.,
                            ),
                            ..default()
                        },
                    },
                    Name::new("Walkable"),
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    AabbGizmo {
                        color: Some(Color::GREEN),
                    },
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

fn spawn_walkable_navtiles(
    mut commands: Commands,
    walkable_query: Query<(&GridCoords, &Parent), Added<Walkable>>,
    parent_query: Query<&Parent, Without<Walkable>>,
    grandparent_query: Query<&Parent, With<Handle<LdtkLevel>>>,
    room_query: Query<(Entity, &GridCoords), With<Room>>,
    navmesh: Query<Entity, With<NavmeshParent>>,
) {
    let mut level_to_room_bound_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    walkable_query.for_each(|(&grid_coords, parent)| {
        let Ok((room_entity, _)) = parent_query
            .get(parent.get())
            .and_then(|grandparent| grandparent_query.get(grandparent.get()))
            .and_then(|parent| room_query.get(parent.get()))
        else {
            return;
        };

        level_to_room_bound_locations
            .entry(room_entity)
            .or_default()
            .insert(grid_coords);
    });

    for (entity, room_coords) in &room_query {
        let Some(grid_coords) = level_to_room_bound_locations.get(&entity) else {
            continue;
        };

        for coord in grid_coords {
            let Ok(navmesh_entity) = navmesh.get_single() else {
                continue;
            };

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: room_coords.x + coord.x,
                            y: room_coords.y + coord.y,
                        },
                        walkable: super::Walkable::Walkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(
                                (coord.x as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.x as f32)),
                                (coord.y as f32 * INT_TILE_SIZE)
                                    + (ROOM_SIZE * (room_coords.y as f32)),
                                1.,
                            ),
                            ..default()
                        },
                    },
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    AabbGizmo {
                        color: Some(Color::GREEN),
                    },
                    Name::new("Walkable"),
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

fn check_room_entry_or_exit(
    mut collision_event: EventReader<CollisionEvent>,
    player_query: Query<(Entity, &GridCoords), With<Player>>,
    room_bound_query: Query<(&Parent, &DoorLocation), With<RoomBoundComponent>>,
    room_query: Query<(&Room, &GridCoords, Entity), With<Room>>,
    mut room_event: EventWriter<RoomBoundsHitEvent>,
) {
    let mut events_to_send = HashSet::<RoomBoundsHitEvent>::new();
    for collision in &mut collision_event {
        let CollisionEvent::Started(ent1, ent2, _) = collision else {
            continue;
        };

        let Some(((player_entity, player_coords), collision_entity)) =
            player_query.get_either_returning_other(*ent1, *ent2)
        else {
            continue;
        };

        let Ok((room_bound, door_location)) = room_bound_query.get(collision_entity) else {
            continue;
        };
        let Ok((room, coords, room_entity)) = room_query.get(room_bound.get()) else {
            continue;
        };

        let movement_type: RoomEnterExit;

        if coords == player_coords {
            movement_type = RoomEnterExit::Exit;
        } else {
            movement_type = RoomEnterExit::Enter;
        };

        events_to_send.insert(RoomBoundsHitEvent {
            character_entity: player_entity,
            room_entity,
            room: room.clone(),
            door_location: *door_location,
            movement_type,
        });
    }

    for evt in events_to_send {
        room_event.send(evt);
    }
}

fn create_navmesh(mut commands: Commands) {
    commands.spawn(NavmeshBundle {
        name: Name::new("Navmesh"),
        transform: Transform::from_xyz(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 1.).into(),
        ..default()
    });
}
