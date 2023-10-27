use super::room::{setup_first_rooms, Room, RoomBoundsHitEvent, RoomEnterExit};
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
use std::ops::Mul;

const CHARACTER_MOVE_SPEED: f32 = 45.0;

#[derive(Actionlike, Reflect, Clone)]
enum CharacterInput {
    Move,
    TogglePause,
    RotateRoom,
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

#[derive(Component)]
pub struct Player;

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
                update_character_animation.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                process_player_input.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                listen_for_pause
                    .run_if(in_state(GameState::Paused).or_else(in_state(GameState::Main))),
            )
            .add_systems(Update, mouse_input.run_if(in_state(GameState::Main)))
            .add_systems(Update, update_character_room_coords);
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
            Player,
            CharacterProps {
                knowledge: rand::thread_rng().gen_range(2..11),
                might: rand::thread_rng().gen_range(2..11),
                sanity: rand::thread_rng().gen_range(2..11),
                speed: rand::thread_rng().gen_range(2..11),
            },
            GridCoords { x: 0, y: 0 },
            InputManagerBundle::<CharacterInput> {
                input_map: InputMap::default()
                    .insert(DualAxis::left_stick(), CharacterInput::Move)
                    .insert(
                        VirtualDPad {
                            down: KeyCode::S.into(),
                            left: KeyCode::A.into(),
                            right: KeyCode::D.into(),
                            up: KeyCode::W.into(),
                        },
                        CharacterInput::Move,
                    )
                    .insert(KeyCode::Escape, CharacterInput::TogglePause)
                    .insert(KeyCode::R, CharacterInput::RotateRoom)
                    .build(),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            Collider::compound(vec![(Vec2::new(0., -10.), 0., Collider::cuboid(4., 2.))]),
        ))
        .add_child(camera_entity);
}

fn update_character_animation(
    mut sprites: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &ActionState<CharacterInput>,
    )>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation, input) in &mut sprites {
        let mut temp_facing = Option::<CharacterFacing>::None;

        if let Some(movement) = input.axis_pair(CharacterInput::Move) {
            animation.walking = true;

            if movement.x().abs() > movement.y().abs() {
                if movement.x() > 0. {
                    temp_facing = Some(CharacterFacing::Right);
                } else {
                    temp_facing = Some(CharacterFacing::Left);
                }
            } else if movement.y().abs() > movement.x().abs() {
                if movement.y() > 0. {
                    temp_facing = Some(CharacterFacing::Up);
                } else {
                    temp_facing = Some(CharacterFacing::Down);
                }
            } else {
                animation.walking = false;
            }
        } else {
            animation.walking = false;
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

fn process_player_input(
    mut animation_query: Query<(&mut Velocity, &ActionState<CharacterInput>), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut velocity, input)) = animation_query.get_single_mut() else {
        return;
    };

    let Some(input) = input.clamped_axis_pair(CharacterInput::Move) else {
        return;
    };

    let walk_transform;

    if input.x().abs() > input.y().abs() {
        walk_transform = Vec2::new(input.x(), 0.);
    } else if input.y().abs() > input.x().abs() {
        walk_transform = Vec2::new(0., input.y());
    } else {
        velocity.linvel = Vec2::ZERO;
        return;
    }

    velocity.linvel = walk_transform * time.delta_seconds() * CHARACTER_MOVE_SPEED * 100.;
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
