use crate::components::ROOM_SIZE;
use bevy::{prelude::*, render::primitives::Aabb};
use rand::prelude::*;

const NUMBER_OF_SIDES: u32 = 6;

pub fn roll_die(number_of_die: u8) -> u32 {
    let mut total: u32 = 0;
    let mut rng = rand::thread_rng();

    for _ in 0..number_of_die {
        total += rng.gen_range(1..NUMBER_OF_SIDES);
    }

    total
}

/// Get character pos within the room, where 0, 0 is the center of the room
pub fn character_transform_to_pos_in_room(
    player_transform: &GlobalTransform,
    room_transform: &GlobalTransform,
    room_aabb: &Aabb,
) -> Vec2 {
    // let tx =
    //     (room_transform.translation() + Vec3::new(ROOM_SIZE / 2., ROOM_SIZE / 2., 0.)).truncate();
    // let (min_x, max_x, min_y, max_y) = (
    //     tx.x - room_aabb.half_extents.x,
    //     tx.x + room_aabb.half_extents.x,
    //     tx.y - room_aabb.half_extents.y,
    //     tx.y + room_aabb.half_extents.y,
    // );

    (player_transform.translation()
        - (room_transform.translation() + Vec2::new(ROOM_SIZE / 2., ROOM_SIZE / 2.).extend(0.)))
    .truncate()
}
