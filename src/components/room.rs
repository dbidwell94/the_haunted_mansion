use super::{card::CardType, character::Player, navmesh::*};
use crate::GameState;
use bevy::math::Vec3A;
use bevy::render::primitives::Aabb;
use bevy::sprite::collide_aabb::collide;
use bevy::sprite::collide_aabb::Collision;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use derivative::Derivative;
use lazy_static::lazy_static;
use ldtk::*;

const ROOM_SIZE: f32 = 96.0;
pub const INT_TILE_SIZE: f32 = 8.;

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

mod ldtk {
    use bevy::prelude::*;
    use bevy_ecs_ldtk::prelude::*;
    #[derive(Component, Default)]
    pub struct NonWalkable;

    #[derive(LdtkIntCell, Bundle)]
    pub struct NonWalkableBundle {
        non_walkable: NonWalkable,
    }

    #[derive(Component, Default)]
    pub struct RoomBound;

    #[derive(Component)]
    pub struct RoomBoundComponent;

    #[derive(LdtkIntCell, Bundle)]
    pub struct RoomBoundBundle {
        room_bound: RoomBound,
    }

    #[derive(Component, Default)]
    pub struct Walkable;

    #[derive(Component)]
    pub struct WalkableComponent;

    #[derive(LdtkIntCell, Bundle)]
    pub struct WalkableBundle {
        walkable: Walkable,
    }
}

#[derive(Event, Hash, PartialEq, Eq)]
pub struct RoomBoundsHitEvent {
    pub character_entity: Entity,
    pub room_entity: Entity,
    pub room: Room,
}

#[derive(AssetCollection, Resource)]
pub struct RoomAssets {
    #[asset(path = "ldtk/haunted.ldtk")]
    ldtk_asset: Handle<LdtkAsset>,
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

#[derive(Derivative, Component, Debug, Clone, Reflect)]
#[derivative(Hash, Eq, PartialEq)]
pub struct Room {
    pub name: String,
    pub iid: String,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    room_level: HashSet<RoomLevel>,
    allowed_copies: u8,
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

fn spawn_room(
    commands: &mut Commands,
    room_assets: &Res<RoomAssets>,
    counter: &mut ResMut<RoomCounter>,
    put_at: GridCoords,
    room: Room,
    z_index: i8,
) {
    let ldtk_handle = &room_assets.ldtk_asset;
    let world_pos = room_location_to_position((put_at.x, put_at.y));

    commands.spawn((
        RoomBundle {
            ldtk: LdtkWorldBundle {
                ldtk_handle: ldtk_handle.clone(),
                level_set: LevelSet::from_iid(room.iid.to_owned()),
                transform: Transform::from_xyz(world_pos.x, world_pos.y, -1.),
                ..default()
            },
            name: Name::new(room.name.to_owned()),
            room: room.to_owned(),
            location: put_at,
        },
        Aabb {
            half_extents: Vec3::new(ROOM_SIZE / 2., ROOM_SIZE / 2., 0.).into(),
            center: Vec3::new(ROOM_SIZE / 2., ROOM_SIZE / 2., 0.).into(),
        },
    ));

    if counter.rooms.contains_key(&room) {
        *counter.rooms.get_mut(&room).unwrap() += 1;
    } else {
        counter.rooms.insert(room.to_owned(), 1);
    }

    counter
        .filled_tiles
        .insert((put_at.x, put_at.y, z_index), room.to_owned());
}

pub fn setup_first_rooms(
    mut commands: Commands,
    room_assets: Res<RoomAssets>,
    mut room_counter: ResMut<RoomCounter>,
) {
    let entryway = LDTK_ROOMS.iter().find(|room| room.name == "Entryway");
    let entryway_location = GridCoords::new(0, 0);
    let hallway = LDTK_ROOMS.iter().find(|room| room.name == "Hallway-2x0y");
    let hallway_location = GridCoords::new(1, 0);
    let hallway_4way = LDTK_ROOMS.iter().find(|room| room.name == "Hallway-2x2y");
    let hallway_4way_location = GridCoords::new(2, 0);

    let (Some(entryway), Some(hallway), Some(hallway_4way)) = (entryway, hallway, hallway_4way)
    else {
        panic!("Cannot find the first rooms: 'entryway' -- 'hallway-2x0y'");
    };

    spawn_room(
        &mut commands,
        &room_assets,
        &mut room_counter,
        entryway_location,
        entryway.clone(),
        0,
    );
    spawn_room(
        &mut commands,
        &room_assets,
        &mut room_counter,
        hallway_location,
        hallway.clone(),
        0,
    );
    spawn_room(
        &mut commands,
        &room_assets,
        &mut room_counter,
        hallway_4way_location,
        hallway_4way.clone(),
        0,
    );
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

            let transform = Vec2::new(
                (coord.x as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.x as f32)),
                (coord.y as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.y as f32)),
            );

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: (transform.x / INT_TILE_SIZE) as i32,
                            y: (transform.y / INT_TILE_SIZE) as i32,
                        },
                        walkable: super::WalkableState::NotWalkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(transform.x, transform.y, 1.),
                            ..default()
                        },
                    },
                    Name::new("NotWalkable"),
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    // AabbGizmo {
                    //     color: Some(Color::RED),
                    // },
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

        for coord in grid_coords {
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
                ))
                .id();

            commands.entity(entity).add_child(collider);

            let Ok(navmesh_entity) = navmesh.get_single() else {
                continue;
            };

            let transform = Vec2::new(
                (coord.x as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.x as f32)),
                (coord.y as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.y as f32)),
            );

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: (transform.x / INT_TILE_SIZE) as i32,
                            y: (transform.y / INT_TILE_SIZE) as i32,
                        },
                        walkable: super::WalkableState::Walkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(transform.x, transform.y, 1.),
                            ..default()
                        },
                    },
                    Name::new("Walkable"),
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    // AabbGizmo {
                    //     color: Some(Color::GREEN),
                    // },
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

            let transform = Vec2::new(
                (coord.x as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.x as f32)),
                (coord.y as f32 * INT_TILE_SIZE) + (ROOM_SIZE * (room_coords.y as f32)),
            );

            let navmesh_tile = commands
                .spawn((
                    NavmeshTileBundle {
                        grid_coord: GridCoords {
                            x: (transform.x / INT_TILE_SIZE) as i32,
                            y: (transform.y / INT_TILE_SIZE) as i32,
                        },
                        walkable: super::WalkableState::Walkable,
                        transform: TransformBundle {
                            local: Transform::from_xyz(transform.x, transform.y, 1.),
                            ..default()
                        },
                    },
                    Aabb {
                        half_extents: Vec3A::new(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 0.5),
                        ..Default::default()
                    },
                    // AabbGizmo {
                    //     color: Some(Color::GREEN),
                    // },
                    Name::new("Walkable"),
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

fn check_room_entry_or_exit(
    mut player_query: Query<(Entity, &GlobalTransform, &mut Player), With<Player>>,
    room_query: Query<(&Room, Entity, &Aabb, &GlobalTransform), With<Room>>,
    mut room_event: EventWriter<RoomBoundsHitEvent>,
) {
    let Ok((player_entity, player_transform, mut player)) = player_query.get_single_mut() else {
        return;
    };

    for (room, room_entity, room_aabb, room_transform) in &room_query {
        let tx = room_transform.translation() + Vec3::new(ROOM_SIZE / 2., ROOM_SIZE / 2., 0.);

        if let Some(Collision::Inside) = collide(
            tx,
            room_aabb.half_extents.truncate() * 2.,
            player_transform.translation(),
            Vec2::ONE,
        ) {
            if room == &player.in_room {
                continue;
            }

            player.in_room = room.clone();

            room_event.send(RoomBoundsHitEvent {
                character_entity: player_entity,
                room_entity,
                room: room.clone(),
            })
        }
    }
}

fn create_navmesh(mut commands: Commands) {
    commands.spawn(NavmeshBundle {
        name: Name::new("Navmesh"),
        transform: Transform::from_xyz(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 1.).into(),
        ..default()
    });
}
