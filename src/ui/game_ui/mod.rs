use crate::components::StartMultiplayer;
use crate::{ui::OccludeUI, GameState};
use bevy::{app::AppExit, prelude::*};
use bevy_ui_dsl::*;

pub mod classes;

const PLAYER_INVENTORY_COUNT: u8 = 6;

#[derive(Component)]
struct TextInput;

pub struct GameUiPlugin;

#[derive(Component, Reflect)]
struct GameUiParent;

#[derive(Component, Reflect)]
struct PauseUiParent;

#[derive(Component, Reflect)]
struct MainMenuUiParent;

#[derive(Component)]
struct AnimateTransition;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_button_interactions)
            .add_systems(OnEnter(GameState::Main), build_main_ui_layout)
            .add_systems(OnExit(GameState::Main), destroy_main_ui)
            .add_systems(OnEnter(GameState::Paused), build_pause_ui_layout)
            .add_systems(OnExit(GameState::Paused), destroy_pause_ui_layout)
            .add_systems(OnEnter(GameState::MainMenu), build_main_menu_ui)
            .add_systems(OnExit(GameState::MainMenu), destroy_main_menu_ui)
            .add_systems(
                Update,
                pause_button_system.run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                handle_main_menu_buttons.run_if(in_state(GameState::MainMenu)),
            );
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

    let mut inventory_bounding_box = None;

    let root_entity = root(c_root, &asset_server, &mut commands, |p| {
        node(c_inventory_container, p, |p| {
            for _ in 0..PLAYER_INVENTORY_COUNT {
                node(c_inventory_box, p, |_| {});
            }
        })
        .set(&mut inventory_bounding_box);
    });

    commands
        .entity(root_entity)
        .insert((GameUiParent, Name::new("Main UI Layout")));

    commands
        .entity(inventory_bounding_box.unwrap())
        .insert(OccludeUI);
}

mod pause_components {
    use bevy::prelude::*;

    #[derive(Component)]
    pub enum ButtonType {
        Quit,
    }
}

fn destroy_pause_ui_layout(mut commands: Commands, query: Query<Entity, With<PauseUiParent>>) {
    let Ok(entity) = query.get_single() else {
        return;
    };

    commands.entity(entity).despawn_recursive();
}

fn build_pause_ui_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    use classes::main::{c_button_text, c_button_with_text};
    use classes::pause::*;

    let mut quit_button = None;

    let entity = root(c_root, &asset_server, &mut commands, |p| {
        node(c_center, p, |p| {
            text("PAUSED", (), c_pause_text, p);
            node(pad_below, p, |_| {});
            text_button("Quit", c_button_with_text, c_button_text, p).set(&mut quit_button);
        });
    });

    commands
        .entity(quit_button.unwrap())
        .insert((pause_components::ButtonType::Quit, AnimateTransition));

    commands
        .entity(entity)
        .insert((Name::new("Pause Ui Layout"), PauseUiParent));
}

fn animate_button_interactions(
    mut interactions: Query<
        (&mut BackgroundColor, &Interaction),
        (With<AnimateTransition>, With<Button>),
    >,
) {
    for (mut bg_color, interaction) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {}
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Rgba {
                    red: 1.,
                    green: 1.,
                    blue: 1.,
                    alpha: 0.125,
                });
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::NONE);
            }
        }
    }
}

fn pause_button_system(
    mut interactions: Query<
        (&Interaction, &pause_components::ButtonType),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for (interaction, button_type) in &mut interactions {
        match *interaction {
            Interaction::Pressed => match *button_type {
                pause_components::ButtonType::Quit => {
                    app_exit_event.send(AppExit);
                }
            },
            _ => {}
        }
    }
}

mod main_menu_components {
    use bevy::prelude::*;

    #[derive(Component)]
    pub enum ButtonType {
        Singleplayer,
        Multiplayer,
        Quit,
    }
}

fn build_main_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    use classes::main::{c_button_with_text, c_root};
    use classes::pause::{c_center, c_pause_text};
    use main_menu_components::*;

    let mut singleplayer = None;
    let mut multiplayer = None;
    let mut quit = None;

    let main_menu_entity = root(c_root, &asset_server, &mut commands, |p| {
        node(c_center, p, |p| {
            text_button("Start Singleplayer", c_button_with_text, c_pause_text, p)
                .set(&mut singleplayer);
            text_button("Start Multiplayer", c_button_with_text, c_pause_text, p)
                .set(&mut multiplayer);
            text_button("Quit", c_button_with_text, c_pause_text, p).set(&mut quit);
        });
    });

    commands
        .entity(singleplayer.unwrap())
        .insert((ButtonType::Singleplayer, AnimateTransition));
    commands
        .entity(multiplayer.unwrap())
        .insert((ButtonType::Multiplayer, AnimateTransition));
    commands
        .entity(quit.unwrap())
        .insert((ButtonType::Quit, AnimateTransition));

    commands
        .entity(main_menu_entity)
        .insert((Name::new("Main Menu UI Layout"), MainMenuUiParent));
}

fn destroy_main_menu_ui(
    mut commands: Commands,
    main_menu_ui: Query<Entity, With<MainMenuUiParent>>,
) {
    let Ok(entity) = main_menu_ui.get_single() else {
        return;
    };

    commands.entity(entity).despawn_recursive();
}

fn handle_main_menu_buttons(
    main_menu_buttons: Query<
        (&Interaction, &main_menu_components::ButtonType),
        With<main_menu_components::ButtonType>,
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
    mut start_multiplayer: EventWriter<StartMultiplayer>,
) {
    for (interaction, button_type) in &main_menu_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *button_type {
            main_menu_components::ButtonType::Singleplayer => {
                game_state.set(GameState::InitialSpawn);
            }
            main_menu_components::ButtonType::Multiplayer => {
                game_state.set(GameState::InitialSpawn);
                start_multiplayer.send(StartMultiplayer);
            }
            main_menu_components::ButtonType::Quit => {
                app_exit.send(AppExit);
            }
        }
    }
}
