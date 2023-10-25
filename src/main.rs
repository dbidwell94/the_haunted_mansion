use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod components;
mod events;

#[derive(Default, Eq, PartialEq, Debug, Hash, Clone, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Main,
    UI,
    Paused,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(GameState::Main))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "The Haunted Mansion".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            LdtkPlugin,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(components::ComponentPlugin)
        .run();
}
