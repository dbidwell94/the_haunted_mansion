mod components;
mod resources;
mod systems;

use super::room::setup_first_rooms;
use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
pub use components::*;
use leafwing_input_manager::prelude::*;
use resources::*;
pub use resources::{CharacterInput, CharacterType, CharacterWalk};
pub use systems::*;

const CHARACTER_MOVE_SPEED: f32 = 45.0;
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CharacterWalk>(GameState::Loading)
            .add_collection_to_loading_state::<_, Headshots>(GameState::Loading)
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
                    .run_if(in_state(GameState::Main).or_else(in_state(GameState::Paused))),
            )
            .add_systems(
                Update,
                listen_for_pause
                    .run_if(in_state(GameState::Paused).or_else(in_state(GameState::Main))),
            )
            .add_systems(
                Update,
                request_pathfinding.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                check_pathfinding_answer.run_if(in_state(GameState::Main)),
            )
            .add_systems(
                Update,
                (move_player, move_network_player)
                    .run_if(in_state(GameState::Main).or_else(in_state(GameState::Paused))),
            )
            .add_systems(OnExit(GameState::Main), on_main_exit);
    }
}
