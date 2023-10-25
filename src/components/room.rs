use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use derivative::Derivative;
use lazy_static::lazy_static;

use crate::GameState;

const ROOM_SIZE: f32 = 96.0;
const INT_TILE_SIZE: f32 = 8.;

fn room_location_to_position(location: (i32, i32)) -> Vec2 {
    Vec2::new(location.0 as f32 * ROOM_SIZE, location.1 as f32 * ROOM_SIZE)
}

lazy_static! {
    pub static ref LDTK_ROOMS: [Room; 2] = [
        Room {
            name: "Entryway".into(),
            allowed_copies: 1,
            iid: "ac9b33c2-6280-11ee-baef-b119038a937a".into(),
            room_level: HashSet::from([RoomLevel::Ground])
        },
        Room {
            name: "Hallway-2x0y".into(),
            allowed_copies: 4,
            iid: "078ebb40-6280-11ee-81c9-dd1f0b0b06bd".into(),
            room_level: HashSet::from([RoomLevel::Ground, RoomLevel::Upper])
        }
    ];
}

const LDTK_LOCATION: &'static str = "ldtk/haunted.ldtk";

#[derive(Component, Default)]
pub struct NonWalkable;

#[derive(LdtkIntCell, Bundle)]
pub struct NonWalkableBundle {
    non_walkable: NonWalkable,
}

#[repr(i32)]
#[allow(dead_code)]
enum LayerMask {
    NonWalkable = 1,
    RoomBound = 2,
}

#[derive(Resource, Clone, Default)]
pub struct RoomCounter {
    pub rooms: HashMap<Room, u8>,
    pub filled_tiles: HashMap<(i32, i32, i8), Room>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Reflect)]
#[allow(dead_code)]
pub enum RoomLevel {
    Basement,
    Ground,
    Upper,
}

pub struct RoomPlugin;

#[derive(Derivative, Component, Debug, Clone, Reflect)]
#[derivative(Hash, Eq, PartialEq)]
pub struct Room {
    name: String,
    iid: String,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    room_level: HashSet<RoomLevel>,
    allowed_copies: u8,
}

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomCounter>()
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseZeroTranslation,
                int_grid_rendering: IntGridRendering::Invisible,
                ..default()
            })
            .register_type::<Room>()
            .register_type::<HashSet<RoomLevel>>()
            .register_ldtk_int_cell::<NonWalkableBundle>(LayerMask::NonWalkable as i32)
            .add_systems(OnEnter(GameState::Main), setup_first_rooms)
            .add_systems(Update, spawn_wall_colliders);
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
}

pub fn setup_first_rooms(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut room_counter: ResMut<RoomCounter>,
) {
    let entryway = LDTK_ROOMS.iter().find(|room| room.name == "Entryway");
    let entryway_location = room_location_to_position((0, 0));
    let hallway = LDTK_ROOMS.iter().find(|room| room.name == "Hallway-2x0y");
    let hallway_location = room_location_to_position((1, 0));

    let (Some(entryway), Some(hallway)) = (entryway, hallway) else {
        panic!("Cannot find the first rooms: 'entryway' -- 'hallway-2x0y'");
    };

    let ldtk_handle = asset_server.load(LDTK_LOCATION);

    commands.spawn(RoomBundle {
        ldtk: LdtkWorldBundle {
            ldtk_handle: ldtk_handle.clone(),
            level_set: LevelSet::from_iid(entryway.iid.to_owned()),
            transform: Transform::from_xyz(entryway_location.x, entryway_location.y, -1.),
            ..default()
        },
        name: Name::new(entryway.name.to_owned()),
        room: entryway.to_owned(),
    });

    commands.spawn(RoomBundle {
        ldtk: LdtkWorldBundle {
            ldtk_handle,
            level_set: LevelSet::from_iid(hallway.iid.to_owned()),
            transform: Transform::from_xyz(hallway_location.x, hallway_location.y, -1.),
            ..default()
        },
        name: Name::new(hallway.name.to_owned()),
        room: hallway.to_owned(),
    });

    room_counter.rooms.insert(entryway.to_owned(), 1);
    room_counter.rooms.insert(hallway.to_owned(), 1);

    room_counter
        .filled_tiles
        .insert((0, 0, 0), entryway.to_owned());
    room_counter
        .filled_tiles
        .insert((1, 0, 0), hallway.to_owned());
}

fn spawn_wall_colliders(
    mut commands: Commands,
    non_walkable_query: Query<(&GridCoords, &Parent), Added<NonWalkable>>,
    parent_query: Query<&Parent, Without<NonWalkable>>,
    grandparent_query: Query<&Parent, With<Handle<LdtkLevel>>>,
    room_query: Query<Entity, With<Room>>,
) {
    let mut level_to_non_walkable_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    non_walkable_query.for_each(|(&grid_coords, parent)| {
        let Ok(room_entity) = parent_query
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

    for entity in &room_query {
        let Some(grid_coords) = level_to_non_walkable_locations.get(&entity) else {
            continue;
        };

        for coord in grid_coords {
            let collider = commands
                .spawn((
                    Collider::cuboid(INT_TILE_SIZE / 2., INT_TILE_SIZE / 2.),
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
        }
    }
}
