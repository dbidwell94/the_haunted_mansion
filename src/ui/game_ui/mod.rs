use crate::{
    ui::classes::main::{c_button_text, c_button_with_text},
    GameState,
};
use bevy::{app::AppExit, prelude::*};
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
            .add_systems(OnExit(GameState::Paused), destroy_pause_ui_layout)
            .add_systems(
                Update,
                pause_button_system.run_if(in_state(GameState::Paused)),
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
        .insert(pause_components::ButtonType::Quit);

    commands
        .entity(entity)
        .insert((Name::new("Pause Ui Layout"), PauseUiParent));
}

fn pause_button_system(
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &pause_components::ButtonType,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for (interaction, mut bg_color, button_type) in &mut interactions {
        match *interaction {
            Interaction::Pressed => match *button_type {
                pause_components::ButtonType::Quit => {
                    app_exit_event.send(AppExit);
                }
            },
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
