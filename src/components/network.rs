use std::collections::VecDeque;

use super::{
    character::{
        spawn_network_player, CharacterProps, CharacterWalk, NetworkPlayer, NetworkTransform,
        Player,
    },
    NavmeshAnswerEvent,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use bevy_matchbox::prelude::*;
use serde::{Deserialize, Serialize};

const ROOM_URL: &'static str = "ws://192.168.50.130:3536/haunted_mansion";

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum NetworkState {
    #[default]
    WaitingForPlayers,
    Playing,
    Disconnected,
}

#[derive(Serialize, Deserialize, Debug)]
enum NetworkEvent {
    PlayerPathing(VecDeque<(i32, i32)>),
    PropsFor(CharacterProps),
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<NetworkState>()
            .add_systems(
                Update,
                (
                    init_networked_players
                        .run_if(resource_exists::<CharacterWalk>())
                        .run_if(in_state(NetworkState::WaitingForPlayers)),
                    listen_for_start_multiplayer,
                ),
            )
            .add_systems(
                Update,
                recieve_remote_state.run_if(in_state(NetworkState::Playing)),
            )
            .add_systems(
                Update,
                broadcast_player_pathfinding.run_if(in_state(NetworkState::Playing)),
            )
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
            requested_players: 1,
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
        commands.insert_resource(MatchboxSocket::new_reliable(ROOM_URL));
        return;
    }
}

fn init_networked_players(
    mut commands: Commands,
    asset: Res<CharacterWalk>,
    local_player: Query<(Entity, &CharacterProps), With<Player>>,
    socket: Option<ResMut<MatchboxSocket<SingleChannel>>>,
    lobby_config: Res<LobbyConfig>,
    mut network_state: ResMut<NextState<NetworkState>>,
) {
    let Some(mut socket) = socket else {
        return;
    };

    if socket.get_channel(0).is_err() {
        return; // multiplayer session has already started
    }

    socket.update_peers();

    let Some(my_local_id) = socket.id() else {
        return;
    };

    let Ok((local_player_entity, character_props)) = local_player.get_single() else {
        return;
    };

    if socket.connected_peers().count() != lobby_config.requested_players {
        return;
    }

    info!("Player count reached. Starting match");

    let peers = socket.connected_peers().collect::<Vec<_>>();

    let character_props = bincode::serialize(&NetworkEvent::PropsFor(character_props.clone()))
        .ok()
        .map(|d| d.into_boxed_slice());

    for peer in peers {
        if peer == my_local_id {
            let network_player = NetworkPlayer { player_id: peer };
            commands
                .entity(local_player_entity)
                .insert((network_player, NetworkTransform::default()));
            continue;
        }
        spawn_network_player(&mut commands, &asset, peer);

        if let Some(props) = &character_props {
            socket.send(props.clone(), peer);
        }
    }

    network_state.set(NetworkState::Playing);
}

fn recieve_remote_state(
    mut players: Query<
        (&mut NetworkTransform, &NetworkPlayer, &mut CharacterProps),
        Without<Player>,
    >,
    socket: Option<ResMut<MatchboxSocket<SingleChannel>>>,
) {
    let Some(mut socket) = socket else {
        return;
    };

    for (peer, data) in socket.receive() {
        let Some((mut transform, _, mut props)) = players
            .iter_mut()
            .find(|(_, ref player, _)| player.player_id == peer)
        else {
            continue;
        };

        let Ok(network_event) = bincode::deserialize::<NetworkEvent>(&data) else {
            continue;
        };

        match network_event {
            NetworkEvent::PlayerPathing(vecs) => {
                transform.move_path = vecs
                    .into_iter()
                    .map(|(x, y)| GridCoords::new(x, y))
                    .collect()
            }
            NetworkEvent::PropsFor(recv_props) => {
                *props = recv_props;
            }
        }
    }
}

fn broadcast_player_pathfinding(
    socket: Option<ResMut<MatchboxSocket<SingleChannel>>>,
    mut pathfinding_event: EventReader<NavmeshAnswerEvent>,
) {
    let Some(mut socket) = socket else {
        return;
    };

    let Some(self_id) = socket.id() else {
        return;
    };

    let peers = socket
        .connected_peers()
        .filter(|p| p != &self_id)
        .collect::<Vec<_>>();

    for NavmeshAnswerEvent { path, .. } in &mut pathfinding_event {
        let path = path.clone().ok().unwrap_or(Vec::new());

        let Ok(data_to_send) = bincode::serialize(&NetworkEvent::PlayerPathing(
            path.into_iter().map(|i| (i.x, i.y)).collect(),
        )) else {
            continue;
        };

        let boxed = data_to_send.into_boxed_slice();

        for peer in &peers {
            socket.send(boxed.clone(), *peer);
        }
    }
}
