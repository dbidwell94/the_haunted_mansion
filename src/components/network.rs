use bevy::prelude::*;
use bevy_ggrs::ggrs::{self, PlayerHandle};
use bevy_matchbox::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;

const ROOM_URL: &'static str = "ws://192.168.50.130:3536/haunted_mansion";

#[derive(Copy, Clone, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct NetworkCharacterInput {
    pub move_vector: Vec2,
}

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = NetworkCharacterInput;
    type State = NetworkingState;
    type Address = PeerId;
}

#[derive(Clone)]
pub struct NetworkingState {
    pub character_config: HashMap<PlayerHandle, i32>,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (wait_for_players, listen_for_start_multiplayer))
            .add_event::<StartMultiplayer>()
            .init_resource::<LobbyConfig>();
    }
}

#[derive(Resource)]
pub struct LobbyConfig {
    pub requested_players: usize,
    pub force_start: bool,
}

#[derive(Event)]
pub struct StartMultiplayer;

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            force_start: false,
            requested_players: 2,
        }
    }
}

fn listen_for_start_multiplayer(
    mut commands: Commands,
    mut evt: EventReader<StartMultiplayer>,
    maybe_started: Option<Res<MatchboxSocket<SingleChannel>>>,
) {
    for _ in &mut evt {
        if maybe_started.is_some() {
            return;
        }
        info!("Connecting to matchmaking server at {}", ROOM_URL);
        commands.insert_resource(MatchboxSocket::new_ggrs(ROOM_URL));
        return;
    }
}

fn wait_for_players(
    mut commands: Commands,
    socket: Option<ResMut<MatchboxSocket<SingleChannel>>>,
    lobby_config: Res<LobbyConfig>,
) {
    let Some(mut socket) = socket else {
        return;
    };
    if socket.get_channel(0).is_err() {
        return; // multiplayer session has already started
    }

    socket.update_peers();
    let players = socket.players();
    if players.len() < lobby_config.requested_players && !lobby_config.force_start {
        return; // need more players
    }
    info!("Requested number of players connected: starting game");

    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(lobby_config.requested_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("Failed to add player");
    }

    let channel = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("Failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
}
