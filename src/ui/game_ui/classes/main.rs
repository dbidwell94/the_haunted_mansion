use bevy::prelude::*;

pub fn c_root(b: &mut NodeBundle) {
    b.style = Style {
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        height: Val::Vh(100.),
        width: Val::Vw(100.),
        position_type: PositionType::Relative,
        ..default()
    };
}

pub fn c_button_with_text(_: &AssetServer, b: &mut ButtonBundle) {
    b.background_color = BackgroundColor(Color::NONE);
    b.style = Style {
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
}

pub fn c_card_holder(b: &mut NodeBundle) {
    b.border_color = BorderColor(Color::WHITE);
    b.background_color = BackgroundColor(Color::BLACK);
    b.style = Style {
        border: UiRect::all(Val::Px(2.)),
        display: Display::Flex,
        padding: UiRect::axes(Val::Px(6.), Val::Px(30.)),
        bottom: Val::Percent(0.),
        left: Val::Percent(0.),
        position_type: PositionType::Absolute,
        margin: UiRect::all(Val::Px(10.)),
        ..default()
    };
}

pub fn c_button_text(assets: &AssetServer, b: &mut TextStyle) {
    b.font = assets.load("fonts/pixel.ttf");
    b.font_size = 21.;
}
