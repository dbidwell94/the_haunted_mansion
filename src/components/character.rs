use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;
use std::ops::Mul;

use crate::GameState;

use super::room::setup_first_rooms;

const CHARACTER_MOVE_SPEED: f32 = 30.0;

#[derive(AssetCollection, Resource)]
struct CharacterWalk {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 4))]
    #[asset(path = "sprites/professor_walk.png")]
    walking: Handle<TextureAtlas>,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, Component, PartialEq)]
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
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CharacterWalk>(GameState::Loading)
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
                timer: Timer::from_seconds(0.125, TimerMode::Repeating),
                frame_count: 9,
                walking: false,
                cols: 9,
            },
            Name::new("Character"),
            Player,
            CharacterFacing::Right,
        ))
        .add_child(camera_entity);
}

fn update_character_animation(
    mut sprites: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &CharacterFacing,
        Entity,
    )>,
    updated_facing: Query<Changed<CharacterFacing>>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation, facing, entity) in sprites.iter_mut() {
        let updated_facing_direction = updated_facing.get(entity).ok().unwrap_or(false);

        animation.timer.tick(time.delta());

        if !animation.walking || updated_facing_direction {
            sprite.index = *facing * animation.cols;
            continue;
        }

        if animation.timer.just_finished() {
            sprite.index += 1;

            if sprite.index >= *facing * animation.cols + animation.frame_count {
                sprite.index = (*facing * animation.cols) + 1;
            }
        }
    }
}

fn process_player_input(
    mut animation_query: Query<
        (&mut AnimationTimer, &mut Transform, &mut CharacterFacing),
        With<Player>,
    >,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut animation, mut transform, mut facing)) = animation_query.get_single_mut() else {
        return;
    };

    animation.walking = true;
    let mut walk_transform = Vec2::new(0., 0.);

    if input.pressed(KeyCode::W) {
        walk_transform.y += 1.;
        if facing.as_ref() != &CharacterFacing::Up {
            *facing = CharacterFacing::Up;
        }
    } else if input.pressed(KeyCode::S) {
        walk_transform.y -= 1.;
        if facing.as_ref() != &CharacterFacing::Down {
            *facing = CharacterFacing::Down;
        }
    } else if input.pressed(KeyCode::D) {
        walk_transform.x += 1.;
        if facing.as_ref() != &CharacterFacing::Right {
            *facing = CharacterFacing::Right;
        }
    } else if input.pressed(KeyCode::A) {
        walk_transform.x -= 1.;
        if facing.as_ref() != &CharacterFacing::Left {
            *facing = CharacterFacing::Left;
        }
    } else {
        animation.walking = false;
    }

    transform.translation +=
        walk_transform.extend(0.) * time.delta_seconds() * CHARACTER_MOVE_SPEED;
}
