use std::ops::Mul;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CharacterType {
    Professor,
    Fbi,
}

#[derive(Actionlike, Reflect, Clone)]
pub enum CharacterInput {
    TogglePause,
    RotateRoom,
    WalkSelect,
    SelectObject,
    MoveCamera,
}

#[derive(AssetCollection, Resource)]
pub struct CharacterWalk {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 4))]
    #[asset(path = "sprites/sheets/professor_walk.png")]
    pub professor: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 4))]
    #[asset(path = "sprites/sheets/fbi_walk.png")]
    pub fbi: Handle<TextureAtlas>,
}

#[derive(AssetCollection, Resource)]
pub struct Headshots {
    #[asset(path = "sprites/professor_headshot.png")]
    pub professor_headshot: Handle<Image>,
    #[asset(path = "sprites/fbi_headshot.png")]
    pub fbi_headshot: Handle<Image>,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(dead_code)]
pub enum CharacterFacing {
    Up = 0,
    Left = 1,
    Down = 2,
    #[default]
    Right = 3,
}

impl Mul<usize> for CharacterFacing {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        (self as usize) * rhs
    }
}
