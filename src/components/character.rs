use super::room::setup_first_rooms;
use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::ops::Mul;

const CHARACTER_MOVE_SPEED: f32 = 45.0;

#[derive(Actionlike, Reflect, Clone)]
enum CharacterInput {
    Move,
}

#[derive(AssetCollection, Resource)]
struct CharacterWalk {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 4))]
    #[asset(path = "sprites/professor_walk.png")]
    walking: Handle<TextureAtlas>,
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
struct Player;

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

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CharacterWalk>(GameState::Loading)
            .add_plugins(InputManagerPlugin::<CharacterInput>::default())
            .add_systems(
                OnEnter(GameState::Main),
                spawn_character.after(setup_first_rooms),
            )
            .add_systems(
                Update,
                update_character_animation.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                process_player_input.run_if(in_state(GameState::Main)),
            );
    }
}

fn spawn_character(mut commands: Commands, asset: Res<CharacterWalk>) {
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::splat(25.)),
        index: CharacterFacing::Right * 9usize,
        ..default()
    };

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;

    let camera_entity = commands.spawn(camera_bundle).id();

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
            Player,
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
                    .build(),
                ..Default::default()
            },
            KinematicCharacterController::default(),
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
    mut animation_query: Query<
        (
            &mut KinematicCharacterController,
            &ActionState<CharacterInput>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    let Ok((mut char_controller, input)) = animation_query.get_single_mut() else {
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
        return;
    }

    char_controller.translation =
        Some(walk_transform * time.delta_seconds() * CHARACTER_MOVE_SPEED);
}
