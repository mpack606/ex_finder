mod address_bar;
mod app;
mod app_icons;
mod archive_utils;
mod bottom_bar;
mod commands;
mod components;
mod context_menu;
mod file_info;
mod grid_view;
mod icons;
mod list_view;
mod navigation;
mod search;
mod settings;
mod sidebar;
mod sorting;
mod tabs;
mod updater;
mod view_mode;

use app::App;
use iced::Theme;

fn title(state: &App) -> String {
    state.title()
}

fn theme(_state: &App) -> Theme {
    Theme::Dark
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for updates before starting the GUI
    if let Err(e) = updater::handle_updates() {
        eprintln!("Update check failed: {}", e);
    }

    let settings = settings::load_settings();

    let icon = iced::window::icon::from_file_data(include_bytes!("../assets/icons/icon.png"), None)
        .inspect_err(|e| println!("Error while reading icon file:\n {}", e))
        .ok();

    iced::application(App::boot, App::update, App::view)
        .window(iced::window::Settings {
            size: iced::Size::new(settings.window_width as f32, settings.window_height as f32),
            position: iced::window::Position::Centered,
            icon,
            ..Default::default()
        })
        .default_font(iced::Font::with_name("system-ui"))
        .subscription(App::subscription)
        .title(title)
        .theme(theme)
        .run()
        .map_err(|e| e.into())
}
