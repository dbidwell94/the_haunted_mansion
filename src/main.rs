#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod components;
mod events;
mod prelude;
mod ui;
mod utils;

#[derive(Default, Eq, PartialEq, Debug, Hash, Clone, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    InitialSpawn,
    Main,
    Paused,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MainMenu),
        )
        .add_plugins((
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin {
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "The Haunted Mansion".into(),
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        #[cfg(target_os = "windows")]
                        backends: Some(bevy::render::settings::Backends::DX12),
                        ..default()
                    }),
                })
                .set(ImagePlugin::default_nearest()),
            LdtkPlugin,
            #[cfg(debug_assertions)]
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // #[cfg(debug_assertions)]
            // RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((components::ComponentPlugin, crate::ui::UiPlugin))
        .add_systems(
            Update,
            start_game
                .run_if(in_state(GameState::InitialSpawn))
                .after(components::setup_first_rooms),
        )
        .add_systems(OnEnter(GameState::Main), grab_cursor)
        .add_systems(OnExit(GameState::Main), release_cursor)
        .run();
}

fn start_game(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Main);
}

fn grab_cursor(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    window.cursor.grab_mode = CursorGrabMode::Confined;
}

fn release_cursor(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    window.cursor.grab_mode = CursorGrabMode::None;
}
