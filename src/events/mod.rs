use crate::components::Room;
use bevy::prelude::*;

#[derive(Event)]
pub enum GameEvent {
    Move(i32),
    RoomEnter(Room),
    RoomLeave(Room),
    Damaged(i32),
    Death,
}
