use crate::components::{room::LDTK_ROOMS, Room};
use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use bevy_matchbox::matchbox_socket::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::resources::CharacterFacing;

#[derive(Component)]
pub struct Player {
    pub move_path: VecDeque<GridCoords>,
    pub move_to: Option<GridCoords>,
    pub in_room: Room,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            move_path: Default::default(),
            move_to: Default::default(),
            in_room: LDTK_ROOMS
                .iter()
                .find(|r| r.name.to_lowercase().contains("entry"))
                .unwrap()
                .clone(),
        }
    }
}

#[derive(Component)]
pub struct NetworkPlayer {
    pub player_id: PeerId,
}

#[derive(Component, Default, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct CharacterProps {
    pub speed: u8,
    pub might: u8,
    pub sanity: u8,
    pub knowledge: u8,
}

#[derive(Component, Default)]
pub struct NetworkTransform {
    pub move_path: VecDeque<GridCoords>,
    pub move_to: Option<GridCoords>,
}

#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub walking: bool,
    pub cols: usize,
    pub facing: CharacterFacing,
}
