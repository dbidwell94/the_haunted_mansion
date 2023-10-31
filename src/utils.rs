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
