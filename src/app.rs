use crate::address_bar;
use crate::grid_view;
use crate::bottom_bar;
use crate::navigation;
use crate::settings;
use crate::sidebar;
use crate::icons;
use iced::{Element, Task, Size, Length, Alignment, Border};
use iced::widget::{button, column, row, svg};
use std::path::{PathBuf};
use std::time::{Duration, Instant};

pub struct App {
    settings: settings::Settings,
    navigation: navigation::NavigationState,
    sidebar_paths: Vec<PathBuf>,
    address_input: String,
    address_invalid: bool,
    grid_items: Vec<grid_view::DirectoryItem>,
    selected_item: Option<PathBuf>,
    window_width: f32,
    window_height: f32,
    last_click: Option<(PathBuf, Instant)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::SidebarMessage),
    AddressBar(address_bar::AddressBarMessage),
    Grid(grid_view::GridMessage),
    WindowResized(iced::window::Id, Size),
    NavigateBack,
    NavigateForward,
    NavigateUp,
    None,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let settings = settings::load_settings();
        let initial_path = settings.last_directory
            .clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

        let grid_items = grid_view::read_directory(&initial_path).unwrap_or_default();
        let sidebar_paths = settings.quick_access_paths.clone();
        let address_input = initial_path.to_string_lossy().into_owned();

        (
            Self {
                settings: settings.clone(),
                navigation: navigation::NavigationState::new(initial_path),
                sidebar_paths,
                address_input,
                address_invalid: false,
                grid_items,
                selected_item: None,
                window_width: settings.window_width as f32,
                window_height: settings.window_height as f32,
                last_click: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_msg) => {
                match sidebar_msg {
                    sidebar::SidebarMessage::SelectPath(path) => {
                        self.navigate_to_path(path);
                    }
                    sidebar::SidebarMessage::AddCurrentPath(path) => {
                        if !self.settings.quick_access_paths.contains(&path) {
                            self.settings.quick_access_paths.push(path);
                            self.sidebar_paths = self.settings.quick_access_paths.clone();
                            let _ = settings::save_settings(&self.settings);
                        }
                    }
                    sidebar::SidebarMessage::RemovePath(path) => {
                        self.settings.quick_access_paths.retain(|p| p != &path);
                        self.sidebar_paths = self.settings.quick_access_paths.clone();
                        let _ = settings::save_settings(&self.settings);
                    }
                }
            }
            Message::AddressBar(address_msg) => {
                match address_msg {
                    address_bar::AddressBarMessage::InputChanged(val) => {
                        self.address_input = val;
                    }
                    address_bar::AddressBarMessage::Submit => {
                        let path = PathBuf::from(&self.address_input);
                        if path.exists() && path.is_dir() {
                            self.address_invalid = false;
                            self.navigate_to_path(path);
                        } else {
                            self.address_invalid = true;
                        }
                    }
                }
            }
            Message::Grid(grid_msg) => {
                match grid_msg {
                    grid_view::GridMessage::ItemClicked(path, is_dir) => {
                        let now = Instant::now();
                        let is_double_click = if let Some((last_path, last_time)) = &self.last_click {
                            *last_path == path && now.duration_since(*last_time) < Duration::from_millis(300)
                        } else {
                            false
                        };

                        self.last_click = Some((path.clone(), now));

                        if is_double_click {
                            if is_dir {
                                self.navigate_to_path(path);
                            } else {
                                let path_clone = path.clone();
                                return Task::perform(async move {
                                    let _ = open::that(path_clone);
                                }, |_| Message::None);
                            }
                        } else {
                            self.selected_item = Some(path);
                        }
                    }
                    grid_view::GridMessage::BackgroundClicked => {
                        self.selected_item = None;
                    }
                }
            }
            Message::WindowResized(_id, size) => {
                self.window_width = size.width;
                self.window_height = size.height;
                self.settings.window_width = size.width as u32;
                self.settings.window_height = size.height as u32;
                let _ = settings::save_settings(&self.settings);
            }
            Message::NavigateBack => {
                if self.navigation.navigate_back() {
                    self.on_navigation_changed();
                }
            }
            Message::NavigateForward => {
                if self.navigation.navigate_forward() {
                    self.on_navigation_changed();
                }
            }
            Message::NavigateUp => {
                if let Some(parent) = self.navigation.current_path.parent() {
                    let parent = parent.to_path_buf();
                    self.navigate_to_path(parent);
                }
            }
            Message::None => {}
        }
        Task::none()
    }

    fn navigate_to_path(&mut self, path: PathBuf) {
        self.navigation.navigate_to(path.clone());
        self.on_navigation_changed();
        
        self.settings.last_directory = Some(path);
        let _ = settings::save_settings(&self.settings);
    }

    fn on_navigation_changed(&mut self) {
        let current = &self.navigation.current_path;
        self.address_input = current.to_string_lossy().into_owned();
        self.address_invalid = false;
        self.selected_item = None;
        self.grid_items = grid_view::read_directory(current).unwrap_or_default();
    }

    pub fn title(&self) -> String {
        format!("ex_finder - {}", self.navigation.current_path.to_string_lossy())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let back_btn = button(
            svg(svg::Handle::from_memory(icons::BACK_SVG))
                .width(16)
                .height(16)
        ).padding(6)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;

            let bg = if is_hovered {
                Some(palette.background.base.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: palette.background.strong.text,
                border: Border {
                    color: if is_hovered {
                        palette.primary.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::NavigateBack);
        let forward_btn = button(
            svg(svg::Handle::from_memory(icons::FORWARD_SVG))
                .width(16)
                .height(16)
        ).padding(6)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;

            let bg = if is_hovered {
                Some(palette.background.base.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: palette.background.strong.text,
                border: Border {
                    color: if is_hovered {
                        palette.primary.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::NavigateForward);
        let up_btn = button(
            svg(svg::Handle::from_memory(icons::UP_SVG))
                .width(16)
                .height(16)
        ).padding(6)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;

            let bg = if is_hovered {
                Some(palette.background.base.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: palette.background.strong.text,
                border: Border {
                    color: if is_hovered {
                        palette.primary.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::NavigateUp);
        let nav_buttons = row![back_btn, forward_btn, up_btn].spacing(6);

        let address_bar_element = address_bar::view(&self.address_input, self.address_invalid)
            .map(Message::AddressBar);

        let top_row = row![
            nav_buttons,
            address_bar_element
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(8);

        let sidebar_element = sidebar::view(&self.sidebar_paths, &self.navigation.current_path)
            .map(Message::Sidebar);

        let grid_element = grid_view::view(&self.grid_items, self.selected_item.as_ref(), self.window_width)
            .map(Message::Grid);

        let bottom_bar = bottom_bar::view(self.selected_item.as_deref());

        let body = row![
            sidebar_element,
            column![
                grid_element,
                bottom_bar
            ]
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        column![
            top_row,
            body
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
    }
}
