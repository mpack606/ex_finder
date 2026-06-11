mod app;
mod settings;
mod navigation;
mod sidebar;
mod address_bar;
mod search;
mod grid_view;
mod bottom_bar;
mod icons;

use app::App;
use iced::Theme;

fn title(state: &App) -> String {
    state.title()
}

fn theme(_state: &App) -> Theme {
    Theme::Dark
}

fn main() -> iced::Result {
    let settings = settings::load_settings();

    iced::application(App::boot, App::update, App::view)
        .window(iced::window::Settings {
            size: iced::Size::new(settings.window_width as f32, settings.window_height as f32),
            position: iced::window::Position::Centered,
            ..Default::default()
        })
        .default_font(iced::Font::with_name("system-ui"))
        .subscription(App::subscription)
        .title(title)
        .theme(theme)
        .run()
}
