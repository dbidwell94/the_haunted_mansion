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

pub fn c_character_list(b: &mut NodeBundle) {
    c_inventory_container(b);

    b.style.bottom = Val::Auto;
    b.style.top = Val::Percent(0.);
    b.background_color = BackgroundColor(Color::NONE);
}

pub fn c_inventory_box(b: &mut NodeBundle) {
    b.border_color = BorderColor(Color::rgb(0.75, 0.75, 0.75));
    b.style = Style {
        width: Val::Px(50.),
        height: Val::Px(50.),
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.)),
        ..default()
    };
}

pub fn c_inventory_container(b: &mut NodeBundle) {
    b.background_color = BackgroundColor(Color::rgba(0., 0., 0., 0.95));
    b.style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        bottom: Val::Percent(0.),
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(5.)),
        margin: UiRect::axes(Val::Px(0.), Val::Px(10.)),
        ..default()
    };
}

pub fn c_button_with_text(_: &AssetServer, b: &mut ButtonBundle) {
    b.background_color = BackgroundColor(Color::NONE);
    b.border_color = BorderColor(Color::WHITE);
    b.style = Style {
        border: UiRect::all(Val::Px(2.)),
        padding: UiRect::axes(Val::Px(8.), Val::Px(4.)),
        margin: UiRect::axes(Val::Px(0.), Val::Px(4.)),
        ..default()
    };
}

pub fn c_button_text(assets: &AssetServer, b: &mut TextStyle) {
    b.font = assets.load("fonts/pixel.ttf");
    b.font_size = 21.;
}
