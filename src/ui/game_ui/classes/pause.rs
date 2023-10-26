use bevy::prelude::*;

pub fn c_root(b: &mut NodeBundle) {
    super::main::c_root(b);
    b.background_color = BackgroundColor(Color::Rgba {
        red: 0.,
        green: 0.,
        blue: 0.,
        alpha: 0.5,
    });
}

pub fn c_center(b: &mut NodeBundle) {
    b.background_color = BackgroundColor(Color::BLACK);
    b.style = Style {
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(12.)),
        ..default()
    };
}

pub fn c_pause_text(assets: &AssetServer, b: &mut TextStyle) {
    super::main::c_button_text(assets, b);

    b.font_size = 34.0;
}
