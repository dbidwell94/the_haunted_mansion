use std::collections::VecDeque;

use crate::components::MouseToWorldCoords;
use crate::components::MoveRequest;
use crate::components::NavmeshAnswerEvent;
use crate::components::Selectable;
use crate::components::INT_TILE_SIZE;
use crate::GameState;

use super::components::*;
use super::resources::*;
use super::CHARACTER_MOVE_SPEED;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::GridCoords;
use bevy_matchbox::matchbox_socket::PeerId;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::prelude::*;

pub fn spawn_character_player(mut commands: Commands, asset: Res<CharacterWalk>) {
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::splat(25.)),
        index: CharacterFacing::Right * 9usize,
        anchor: Anchor::BottomCenter,
        ..default()
    };

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;
    camera_bundle.camera_2d.clear_color = ClearColorConfig::Custom(Color::BLACK);

    commands
        .spawn((
            SpriteSheetBundle {
                texture_atlas: asset.professor.clone(),
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
                    .insert(MouseButton::Right, CharacterInput::WalkSelect)
                    .insert(MouseButton::Left, CharacterInput::SelectObject)
                    .insert(
                        VirtualDPad {
                            up: KeyCode::W.into(),
                            down: KeyCode::S.into(),
                            left: KeyCode::A.into(),
                            right: KeyCode::D.into(),
                        },
                        CharacterInput::MoveCamera,
                    )
                    .build(),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Velocity::default(),
            Sensor,
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            Collider::compound(vec![(Vec2::new(0., 2.), 0., Collider::cuboid(4., 2.))]),
        ))
        .insert((Selectable,));
}

pub fn spawn_network_player(
    commands: &mut Commands,
    asset: &Res<CharacterWalk>,
    player_id: PeerId,
) {
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::splat(25.)),
        index: CharacterFacing::Right * 9usize,
        anchor: Anchor::BottomCenter,
        ..default()
    };

    commands
        .spawn((
            SpriteSheetBundle {
                texture_atlas: asset.fbi.clone(),
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
            CharacterProps::default(),
            Name::new("Character"),
            GravityScale(0.),
            GridCoords { x: 0, y: 0 },
            RigidBody::Dynamic,
            Velocity::default(),
            Sensor,
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            Collider::compound(vec![(Vec2::new(0., 2.), 0., Collider::cuboid(4., 2.))]),
            NetworkPlayer { player_id },
            NetworkTransform {
                ..Default::default()
            },
        ))
        .insert(Selectable);
}

pub fn on_main_exit(mut player_velocity: Query<&mut Velocity, With<Player>>) {
    let Ok(mut player_velocity) = player_velocity.get_single_mut() else {
        return;
    };

    player_velocity.linvel = Vec2::ZERO;
}

pub fn update_character_animation(
    mut sprites: Query<(&mut TextureAtlasSprite, &mut AnimationTimer, Entity)>,
    movement: Query<
        (&Velocity, Option<&Player>, Option<&NetworkTransform>),
        Or<(With<Player>, With<NetworkTransform>)>,
    >,
    time: Res<Time>,
) {
    for (mut sprite, mut animation, player_entity) in &mut sprites {
        let Ok((velocity, player, net_transform)) = movement.get(player_entity) else {
            continue;
        };

        let move_path = if player.is_some() {
            &player.unwrap().move_path
        } else {
            &net_transform.unwrap().move_path
        };

        let mut temp_facing = Option::<CharacterFacing>::None;

        let velocity = velocity.linvel.normalize();

        animation.walking = move_path.len() > 0;

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

pub fn listen_for_pause(
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

pub fn move_player(
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

    let Some(path) = player.move_to else {
        velocity.linvel = Vec2::ZERO;
        return;
    };

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

pub fn move_network_player(
    mut player_query: Query<
        (&mut Velocity, &mut NetworkTransform, &Transform),
        With<NetworkTransform>,
    >,
    time: Res<Time>,
) {
    for (mut velocity, mut player, player_transform) in &mut player_query {
        let current_grid = GridCoords::new(
            (player_transform.translation.x / INT_TILE_SIZE).round() as i32,
            (player_transform.translation.y / INT_TILE_SIZE).round() as i32,
        );

        if player.move_to.is_none() && player.move_path.len() > 0 {
            player.move_to = player.move_path.pop_front();
        }

        let Some(path) = player.move_to else {
            velocity.linvel = Vec2::ZERO;
            return;
        };

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
}

pub fn request_pathfinding(
    mouse: Res<MouseToWorldCoords>,
    player_input: Query<(&ActionState<CharacterInput>, &Transform, Entity), With<Player>>,
    mut pathfinding_request: EventWriter<MoveRequest>,
) {
    let Ok((character_input, character_position, player_entity)) = player_input.get_single() else {
        return;
    };

    let Some(mouse_pos) = mouse.0 else {
        return;
    };

    if character_input.just_pressed(CharacterInput::WalkSelect) {
        let pos = character_position.translation.truncate();
        let mouse_tile_pos = GridCoords::new(
            (mouse_pos.x.round() / INT_TILE_SIZE) as i32,
            (mouse_pos.y.round() / INT_TILE_SIZE) as i32,
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

pub fn check_pathfinding_answer(
    mut gizmos: Gizmos,
    mut pathfinding_event_received: EventReader<NavmeshAnswerEvent>,
    mut player: Query<(Entity, &mut Player), With<Player>>,
) {
    let Ok((player_entity, mut player)) = player.get_single_mut() else {
        return;
    };

    for pathfinding_event in &mut pathfinding_event_received.read() {
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
