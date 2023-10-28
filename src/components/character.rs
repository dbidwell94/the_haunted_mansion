use super::room::{setup_first_rooms, Room, RoomBoundsHitEvent, RoomEnterExit};
use super::{MoveRequest, NavmeshAnswerEvent, INT_TILE_SIZE};
use crate::GameState;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::prelude::*;
use std::collections::VecDeque;
use std::ops::Mul;

const CHARACTER_MOVE_SPEED: f32 = 45.0;

#[derive(Actionlike, Reflect, Clone)]
enum CharacterInput {
    TogglePause,
    RotateRoom,
    WalkSelect,
}

#[derive(Resource, Default)]
struct MouseToWorldCoords(Vec2);

#[derive(AssetCollection, Resource)]
pub struct CharacterWalk {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 4))]
    #[asset(path = "sprites/professor_walk.png")]
    walking: Handle<TextureAtlas>,
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct CharacterProps {
    pub speed: u8,
    pub might: u8,
    pub sanity: u8,
    pub knowledge: u8,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(dead_code)]
enum CharacterFacing {
    Up = 0,
    Left = 1,
    Down = 2,
    #[default]
    Right = 3,
}

#[derive(Component, Default)]
pub struct Player {
    move_path: VecDeque<GridCoords>,
    move_to: Option<GridCoords>,
}

impl Mul<usize> for CharacterFacing {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        (self as usize) * rhs
    }
}

#[derive(Component)]
struct AnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub walking: bool,
    pub cols: usize,
    pub facing: CharacterFacing,
}

#[derive(Component)]
struct PlayerCamera;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CharacterWalk>(GameState::Loading)
            .init_resource::<MouseToWorldCoords>()
            .register_type::<CharacterProps>()
            .add_plugins(InputManagerPlugin::<CharacterInput>::default())
            .add_systems(
                OnEnter(GameState::InitialSpawn),
                spawn_character_player.after(setup_first_rooms),
            )
            .add_systems(
                Update,
                update_character_animation
                    .after(move_player)
                    .run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                listen_for_pause
                    .run_if(in_state(GameState::Paused).or_else(in_state(GameState::Main))),
            )
            .add_systems(Update, mouse_input.run_if(in_state(GameState::Main)))
            .add_systems(Update, update_character_room_coords)
            .add_systems(
                Update,
                request_pathfinding.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                check_pathfinding_answer.run_if(in_state(GameState::Main)),
            )
            .add_systems(Update, move_player.run_if(in_state(GameState::Main)));
    }
}

pub fn spawn_character_player(mut commands: Commands, asset: Res<CharacterWalk>) {
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::splat(25.)),
        index: CharacterFacing::Right * 9usize,
        ..default()
    };

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;
    camera_bundle.camera_2d.clear_color = ClearColorConfig::Custom(Color::BLACK);

    let camera_entity = commands.spawn(camera_bundle).insert(PlayerCamera).id();
    commands
        .spawn((
            SpriteSheetBundle {
                texture_atlas: asset.walking.clone(),
                sprite,
                transform: Transform::from_xyz(48., 48., 2.),
                ..default()
            },
            AnimationTimer {
                timer: Timer::from_seconds(0.125 * 0.5, TimerMode::Repeating),
                frame_count: 9,
                walking: false,
                cols: 9,
                facing: CharacterFacing::Right,
            },
            Name::new("Character"),
            GravityScale(0.),
            Player::default(),
            CharacterProps {
                knowledge: rand::thread_rng().gen_range(2..11),
                might: rand::thread_rng().gen_range(2..11),
                sanity: rand::thread_rng().gen_range(2..11),
                speed: rand::thread_rng().gen_range(2..11),
            },
            GridCoords { x: 0, y: 0 },
            InputManagerBundle::<CharacterInput> {
                input_map: InputMap::default()
                    .insert(KeyCode::Escape, CharacterInput::TogglePause)
                    .insert(KeyCode::R, CharacterInput::RotateRoom)
                    .insert(MouseButton::Left, CharacterInput::WalkSelect)
                    .build(),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            // Collider::compound(vec![(Vec2::new(0., -10.), 0., Collider::cuboid(4., 2.))]),
        ))
        .add_child(camera_entity);
}

fn update_character_animation(
    mut sprites: Query<(&mut TextureAtlasSprite, &mut AnimationTimer, Entity)>,
    movement: Query<(&Velocity, &Player), With<Player>>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation, player_entity) in &mut sprites {
        let Ok((velocity, player)) = movement.get(player_entity) else {
            continue;
        };

        let mut temp_facing = Option::<CharacterFacing>::None;

        let velocity = velocity.linvel.normalize();

        animation.walking = player.move_path.len() > 0;

        if velocity.x.abs() > velocity.y.abs() {
            if velocity.x > 0. {
                temp_facing = Some(CharacterFacing::Right);
            } else {
                temp_facing = Some(CharacterFacing::Left);
            }
        } else if velocity.y.abs() > velocity.x.abs() {
            if velocity.y > 0. {
                temp_facing = Some(CharacterFacing::Up);
            } else {
                temp_facing = Some(CharacterFacing::Down);
            }
        }

        let facing_changed = if let Some(temp_facing) = temp_facing {
            let changed = temp_facing != animation.facing;
            animation.facing = temp_facing;
            changed
        } else {
            false
        };

        animation.timer.tick(time.delta());

        if !animation.walking || facing_changed {
            sprite.index = animation.facing * animation.cols;
            continue;
        }

        if animation.timer.just_finished() {
            sprite.index += 1;

            if sprite.index >= animation.facing * animation.cols + animation.frame_count {
                sprite.index = (animation.facing * animation.cols) + 1;
            }
        }
    }
}

fn listen_for_pause(
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    input_query: Query<&ActionState<CharacterInput>, With<Player>>,
) {
    let Ok(input) = input_query.get_single() else {
        return;
    };

    let pause_pressed = input.just_pressed(CharacterInput::TogglePause);
    if !pause_pressed {
        return;
    }

    if game_state.get() == &GameState::Main {
        next_game_state.set(GameState::Paused);
    } else {
        next_game_state.set(GameState::Main);
    }
}

fn mouse_input(
    mut mouse_coords: ResMut<MouseToWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_coords.0 = world_pos;
    }
}

fn move_player(
    mut player_query: Query<(&mut Velocity, &mut Player, &Transform), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut velocity, mut player, player_transform)) = player_query.get_single_mut() else {
        return;
    };

    let current_grid = GridCoords::new(
        (player_transform.translation.x / INT_TILE_SIZE).round() as i32,
        (player_transform.translation.y / INT_TILE_SIZE).round() as i32,
    );

    if player.move_to.is_none() && player.move_path.len() > 0 {
        player.move_to = player.move_path.pop_front();
    }

    let Some(mut path) = player.move_to else {
        velocity.linvel = Vec2::ZERO;
        return;
    };

    // todo! Offset the pathing to account for the feet being on the bottom of the character, not the center
    path.x += 1;
    path.y += 1;

    if path == current_grid {
        player.move_to = None;
        velocity.linvel = Vec2::ZERO;
        return;
    }

    let direction = (Vec2::new(path.x as f32 * INT_TILE_SIZE, path.y as f32 * INT_TILE_SIZE)
        - player_transform.translation.truncate())
    .normalize();

    velocity.linvel = direction * time.delta_seconds() * CHARACTER_MOVE_SPEED * 100.;
}

fn update_character_room_coords(
    mut room_changed_event: EventReader<RoomBoundsHitEvent>,
    mut character_query: Query<&mut GridCoords, (With<Player>, Without<Room>)>,
    room_query: Query<&GridCoords, (With<Room>, Without<Player>)>,
) {
    for evt in &mut room_changed_event {
        let Ok(mut player_grid_coords) = character_query.get_mut(evt.character_entity) else {
            continue;
        };
        let Ok(room_grid_coords) = room_query.get(evt.room_entity) else {
            continue;
        };
        if evt.movement_type == RoomEnterExit::Enter {
            player_grid_coords.x = room_grid_coords.x;
            player_grid_coords.y = room_grid_coords.y;
            debug!("Entered new room!");
        } else {
            debug!("Exited a room!");
        }
    }
}

fn request_pathfinding(
    mouse: Res<MouseToWorldCoords>,
    player_input: Query<(&ActionState<CharacterInput>, &Transform, Entity), With<Player>>,
    mut pathfinding_request: EventWriter<MoveRequest>,
) {
    let Ok((character_input, character_position, player_entity)) = player_input.get_single() else {
        return;
    };

    if character_input.just_pressed(CharacterInput::WalkSelect) {
        let pos = character_position.translation.truncate();
        let mouse_tile_pos = GridCoords::new(
            (mouse.0.x.round() / INT_TILE_SIZE) as i32,
            (mouse.0.y.round() / INT_TILE_SIZE) as i32,
        );

        let char_position = GridCoords::new(
            (pos.x.round() / INT_TILE_SIZE) as i32,
            (pos.y.round() / INT_TILE_SIZE) as i32,
        );

        pathfinding_request.send(MoveRequest {
            requesting_entity: player_entity,
            move_from: char_position,
            move_to: mouse_tile_pos,
        });
    }
}

fn check_pathfinding_answer(
    mut gizmos: Gizmos,
    mut pathfinding_event_received: EventReader<NavmeshAnswerEvent>,
    mut player: Query<(Entity, &mut Player), With<Player>>,
) {
    let Ok((player_entity, mut player)) = player.get_single_mut() else {
        return;
    };

    for pathfinding_event in &mut pathfinding_event_received {
        if pathfinding_event.requesting_entity != player_entity {
            continue;
        }
        if let Ok(path) = &pathfinding_event.path {
            let mut path: VecDeque<_> = path.clone().into();
            player.move_path.clear();
            player.move_path.append(&mut path);
        } else {
            player.move_path.clear();
        }
    }

    let mut start_option = player
        .move_path
        .get(0)
        .map(|g| Vec2::new(g.x as f32, g.y as f32));
    let mut end_option = player
        .move_path
        .get(1)
        .map(|g| Vec2::new(g.x as f32, g.y as f32));
    let mut end_index = 1usize;

    while start_option.is_some() && end_option.is_some() {
        let start = start_option.unwrap();
        let end = end_option.unwrap();

        gizmos.line_2d(
            start * INT_TILE_SIZE,
            end * INT_TILE_SIZE,
            Color::Rgba {
                red: 1.,
                green: 1.,
                blue: 1.,
                alpha: 0.5,
            },
        );

        start_option = player
            .move_path
            .get(end_index)
            .map(|g| Vec2::new(g.x as f32, g.y as f32));
        end_index += 1;
        end_option = player
            .move_path
            .get(end_index)
            .map(|g| Vec2::new(g.x as f32, g.y as f32));
    }
}
