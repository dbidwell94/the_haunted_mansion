use crate::GameState;
use bevy::prelude::*;
use bevy_ui_dsl::*;
pub mod classes;

const PLAYER_INVENTORY_COUNT: u8 = 6;

pub struct GameUiPlugin;

#[derive(Component, Reflect)]
struct GameUiParent;

#[derive(Component, Reflect)]
struct PauseUiParent;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Main), build_main_ui_layout)
            .add_systems(OnExit(GameState::Main), destroy_main_ui)
            .add_systems(OnEnter(GameState::Paused), build_pause_ui_layout)
            .add_systems(OnExit(GameState::Paused), destroy_pause_ui_layout);
    }
}

fn destroy_main_ui(mut commands: Commands, ui: Query<Entity, With<GameUiParent>>) {
    let Ok(ui) = ui.get_single() else {
        return;
    };
    commands.entity(ui).despawn_recursive();
}

fn build_main_ui_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    use classes::main::*;
    let root_entity = root(c_root, &asset_server, &mut commands, |p| {
        node(c_inventory_container, p, |p| {
            for _ in 0..PLAYER_INVENTORY_COUNT {
                node(c_inventory_box, p, |_| {});
            }
        });
    });

    commands
        .entity(root_entity)
        .insert((GameUiParent, Name::new("Main UI Layout")));
}

fn destroy_pause_ui_layout(mut commands: Commands, query: Query<Entity, With<PauseUiParent>>) {
    let Ok(entity) = query.get_single() else {
        return;
    };

    commands.entity(entity).despawn_recursive();
}

fn build_pause_ui_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    use classes::pause::*;

    let entity = root(c_root, &asset_server, &mut commands, |p| {
        node(c_center, p, |p| {
            text("PAUSED", (), c_pause_text, p);
        });
    });

    commands
        .entity(entity)
        .insert((Name::new("Pause Ui Layout"), PauseUiParent));
}
