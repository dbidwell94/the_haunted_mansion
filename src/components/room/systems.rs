use super::components::{ldtk::*, *};
use super::LDTK_ROOMS;
use super::ROOM_SIZE;
use super::{resources::*, INT_TILE_SIZE};
use crate::components::character::Player;
use crate::components::navmesh::NavmeshParent;
use crate::components::{NavmeshBundle, NavmeshTileBundle};
use crate::prelude::*;
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::utils::{HashMap, HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

fn room_location_to_position(location: (i32, i32)) -> Vec2 {
    Vec2::new(location.0 as f32 * ROOM_SIZE, location.1 as f32 * ROOM_SIZE)
}

pub fn spawn_room(
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
                level_set: LevelSet::from_iids([room.iid.to_owned()]),
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

pub fn spawn_wall_colliders(
    mut commands: Commands,
    non_walkable_query: Query<(&GridCoords, &Parent), Added<NonWalkable>>,
    parent_query: Query<&Parent, Without<NonWalkable>>,
    grandparent_query: Query<&Parent, With<LevelIid>>,
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
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

pub fn spawn_room_bounds(
    mut commands: Commands,
    non_walkable_query: Query<(&GridCoords, &Parent), Added<RoomBound>>,
    parent_query: Query<&Parent, Without<RoomBound>>,
    grandparent_query: Query<&Parent, With<LevelIid>>,
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
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

pub fn spawn_walkable_navtiles(
    mut commands: Commands,
    walkable_query: Query<(&GridCoords, &Parent), Added<Walkable>>,
    parent_query: Query<&Parent, Without<Walkable>>,
    grandparent_query: Query<&Parent, With<LevelIid>>,
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
                    Name::new("Walkable"),
                ))
                .id();

            commands.entity(navmesh_entity).add_child(navmesh_tile);
        }
    }
}

pub fn check_room_entry_or_exit(
    mut player_query: Query<(Entity, &GlobalTransform, &mut Player), With<Player>>,
    room_query: Query<(&Room, Entity, &Aabb, &GlobalTransform), With<Room>>,
    mut room_event: EventWriter<RoomBoundsHitEvent>,
) {
    let Ok((player_entity, player_global_transform, mut player)) = player_query.get_single_mut()
    else {
        return;
    };

    for (room, room_entity, room_aabb, room_transform) in &room_query {
        let tx = room_transform.translation() + Vec3::new(ROOM_SIZE / 2., ROOM_SIZE / 2., 0.);

        if let Some(Collision::Inside) = collide(
            tx,
            room_aabb.half_extents.truncate() * 2.,
            player_global_transform.translation(),
            Vec2::ONE,
        ) {
            if room == &player.in_room {
                continue;
            }

            player.in_room = room.clone();

            let player_pos_in_room = character_transform_to_pos_in_room(
                player_global_transform,
                room_transform,
                room_aabb,
            );

            room_event.send(RoomBoundsHitEvent {
                character_entity: player_entity,
                room_entity,
                room: room.clone(),
            })
        }
    }
}

pub fn create_navmesh(mut commands: Commands) {
    commands.spawn(NavmeshBundle {
        name: Name::new("Navmesh"),
        transform: Transform::from_xyz(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2., 1.).into(),
        ..default()
    });
}
