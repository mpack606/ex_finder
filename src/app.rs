use crate::address_bar;
use crate::search;
use crate::grid_view;
use crate::bottom_bar;
use crate::settings;
use crate::sidebar;
use crate::icons;
use crate::app_icons;
use crate::context_menu;
use crate::tabs;
use iced::{Element, Task, Size, Length, Alignment, Border, Color, keyboard, Event};
use iced::widget::{button, column, row, svg, stack, text_input, container, mouse_area, text};
use std::path::{PathBuf};
use std::time::{Duration, Instant};

pub struct App {
    settings: settings::Settings,
    tabs_state: tabs::TabsState,
    sidebar_paths: Vec<PathBuf>,
    address_input: String,
    address_invalid: bool,
    search_query: String,
    grid_items: Vec<grid_view::DirectoryItem>,
    selected_item: Option<PathBuf>,
    window_width: f32,
    window_height: f32,
    last_click: Option<(PathBuf, Instant)>,
    cursor_position: iced::Point,
    context_menu: Option<context_menu::ContextMenuState>,
    clipboard: Option<PathBuf>,
    renaming_path: Option<(PathBuf, String)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::SidebarMessage),
    AddressBar(address_bar::AddressBarMessage),
    Search(search::SearchMessage),
    Grid(grid_view::GridMessage),
    AppIconFound(String, Option<PathBuf>),
    WindowResized(iced::window::Id, Size),
    MouseMoved(iced::Point),
    ContextMenu(context_menu::ContextMenuMessage),
    Tabs(tabs::TabsMessage),
    RenameRequested(PathBuf),
    RenameInputChanged(String),
    RenameSubmitted,
    CancelRename,
    Refresh,
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

        let app = Self {
            settings: settings.clone(),
            tabs_state: tabs::TabsState::new(initial_path),
            sidebar_paths,
            address_input,
            address_invalid: false,
            search_query: String::new(),
            grid_items,
            selected_item: None,
            window_width: settings.window_width as f32,
            window_height: settings.window_height as f32,
            last_click: None,
            cursor_position: iced::Point::ORIGIN,
            context_menu: None,
            clipboard: None,
            renaming_path: None,
        };

        let task = app.load_app_icons();

        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_msg) => {
                match sidebar_msg {
                    sidebar::SidebarMessage::SelectPath(path) => {
                        return self.navigate_to_path(path);
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
                    sidebar::SidebarMessage::ItemRightClicked(path) => {
                        self.selected_item = Some(path.clone());
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            path: Some((path, true)), // Sidebar items are always folders
                            is_sidebar: true,
                        });
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
                            return self.navigate_to_path(path);
                        } else {
                            self.address_invalid = true;
                        }
                    }
                }
            }
            Message::Search(search_msg) => {
                match search_msg {
                    search::SearchMessage::InputChanged(val) => {
                        self.search_query = val;
                    }
                    search::SearchMessage::Clear => {
                        self.search_query.clear();
                    }
                }
            }
            Message::Grid(grid_msg) => {
                match grid_msg {
                    grid_view::GridMessage::ItemClicked(path, is_dir) => {
                        self.context_menu = None;
                        let now = Instant::now();
                        let is_double_click = if let Some((last_path, last_time)) = &self.last_click {
                            *last_path == path && now.duration_since(*last_time) < Duration::from_millis(300)
                        } else {
                            false
                        };

                        self.last_click = Some((path.clone(), now));

                        if is_double_click {
                            if is_dir {
                                return self.navigate_to_path(path);
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
                    grid_view::GridMessage::ItemRightClicked(path, is_dir) => {
                        self.selected_item = Some(path.clone());
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            path: Some((path, is_dir)),
                            is_sidebar: false,
                        });
                    }
                    grid_view::GridMessage::BackgroundClicked => {
                        self.selected_item = None;
                        self.context_menu = None;
                    }
                    grid_view::GridMessage::BackgroundRightClicked => {
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            path: None,
                            is_sidebar: false,
                        });
                    }
                }
            }
            Message::MouseMoved(position) => {
                self.cursor_position = position;
            }
            Message::ContextMenu(context_msg) => {
                match context_msg {
                    context_menu::ContextMenuMessage::Close => {
                        self.context_menu = None;
                    }
                    context_menu::ContextMenuMessage::Action(action) => {
                        self.context_menu = None;
                        return context_menu::handle_action(
                            action,
                            &mut self.clipboard,
                            self.tabs_state.active_path().clone(),
                        ).map(|event| match event {
                            Some(context_menu::ContextMenuEvent::Refresh) => Message::Refresh,
                            Some(context_menu::ContextMenuEvent::Rename(path)) => {
                                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                                Message::RenameInputChanged(name);
                                Message::RenameRequested(path)
                            }
                            Some(context_menu::ContextMenuEvent::OpenInNewTab(path)) => {
                                Message::Tabs(tabs::TabsMessage::OpenTab(path))
                            }
                            None => Message::None,
                        });
                    }
                }
            }
            Message::Tabs(tabs_msg) => {
                if let Some(tabs::TabsEvent::NavigationChanged) = self.tabs_state.update(tabs_msg) {
                    return self.on_navigation_changed();
                }
            }
            Message::RenameRequested(path) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                self.renaming_path = Some((path, name));
            }
            Message::RenameInputChanged(val) => {
                if let Some((_, input)) = &mut self.renaming_path {
                    *input = val;
                }
            }
            Message::RenameSubmitted => {
                if let Some((old_path, new_name)) = self.renaming_path.take() {
                    if !new_name.is_empty() {
                        let new_path = old_path.parent().unwrap().join(new_name);
                        return Task::perform(async move {
                            std::fs::rename(old_path, new_path)
                        }, |result| {
                            match result {
                                Ok(_) => Message::Refresh,
                                Err(e) => {
                                    eprintln!("Failed to rename: {}", e);
                                    Message::Refresh
                                }
                            }
                        });
                    }
                }
            }
            Message::CancelRename => {
                self.renaming_path = None;
            }
            Message::AppIconFound(ext, icon_path) => {
                for item in self.grid_items.iter_mut() {
                    if let Some(item_ext) = item.path.extension().and_then(|e| e.to_str()) {
                        if item_ext == ext {
                            item.app_icon = icon_path.clone();
                        }
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
                if self.tabs_state.active_tab_mut().navigate_back() {
                    return self.on_navigation_changed();
                }
            }
            Message::NavigateForward => {
                if self.tabs_state.active_tab_mut().navigate_forward() {
                    return self.on_navigation_changed();
                }
            }
            Message::NavigateUp => {
                if let Some(parent) = self.tabs_state.active_path().parent() {
                    let parent = parent.to_path_buf();
                    return self.navigate_to_path(parent);
                }
            }
            Message::Refresh => {
                self.grid_items = grid_view::read_directory(self.tabs_state.active_path()).unwrap_or_default();
                return self.load_app_icons();
            }
            Message::None => {}
        }
        Task::none()
    }

    fn navigate_to_path(&mut self, path: PathBuf) -> Task<Message> {
        self.tabs_state.active_tab_mut().navigate_to(path.clone());
        let task = self.on_navigation_changed();
        
        self.settings.last_directory = Some(path);
        let _ = settings::save_settings(&self.settings);
        task
    }

    fn on_navigation_changed(&mut self) -> Task<Message> {
        let current = self.tabs_state.active_path().clone();
        self.address_input = current.to_string_lossy().into_owned();
        self.address_invalid = false;
        self.selected_item = None;
        self.context_menu = None;
        self.grid_items = grid_view::read_directory(&current).unwrap_or_default();
        self.load_app_icons()
    }

    fn load_app_icons(&self) -> Task<Message> {
        let mut extensions = std::collections::HashSet::new();
        for item in &self.grid_items {
            if !item.is_dir {
                if let Some(ext) = item.path.extension().and_then(|e| e.to_str()) {
                    extensions.insert(ext.to_string());
                }
            }
        }

        let tasks = extensions.into_iter().map(|ext| {
            Task::perform(async move {
                let icon = app_icons::get_app_icon_for_extension(&ext);
                (ext, icon)
            }, |(ext, icon)| Message::AppIconFound(ext, icon))
        });

        Task::batch(tasks)
    }

    pub fn title(&self) -> String {
        format!("ex_finder - {}", self.tabs_state.active_path().to_string_lossy())
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

        let search_box = search::view(&self.search_query).map(Message::Search);

        let top_row = row![
            nav_buttons,
            address_bar_element,
            search_box
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(8);

        let sidebar_element = sidebar::view(&self.sidebar_paths, self.tabs_state.active_path())
            .map(Message::Sidebar);

        let filtered_items: Vec<_> = if self.search_query.is_empty() {
            self.grid_items.clone()
        } else {
            let query = self.search_query.to_lowercase();
            self.grid_items.iter()
                .filter(|item| item.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        };

        let grid_element = grid_view::view(&filtered_items, self.selected_item.as_ref(), self.window_width)
            .map(Message::Grid);

        let bottom_bar = bottom_bar::view(self.selected_item.as_deref());

        let mut main_content = column![];
        if self.tabs_state.list.len() > 1 {
            main_content = main_content.push(tabs::view(&self.tabs_state).map(Message::Tabs));
        }
        main_content = main_content.push(grid_element).push(bottom_bar);

        let body = row![
            sidebar_element,
            main_content
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        let content = column![
            top_row,
            body
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        let mut root = stack![content];

        if let Some(context_menu) = &self.context_menu {
            root = root.push(
                context_menu::view(context_menu, self.clipboard.is_some())
                    .map(Message::ContextMenu)
            );
        } else if let Some((_, input)) = &self.renaming_path {
            let dialog = container(
                container(
                    column![
                        text("Rename")
                            .size(18)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                        text_input("New name", input)
                            .on_input(Message::RenameInputChanged)
                            .on_submit(Message::RenameSubmitted)
                            .padding(10)
                            .size(14),
                        row![
                            button(text("Cancel").align_x(Alignment::Center))
                                .on_press(Message::CancelRename)
                                .padding(8)
                                .width(Length::Fill)
                                .style(|theme: &iced::Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(palette.background.weak.color.into()),
                                        text_color: palette.background.strong.text,
                                        border: Border {
                                            radius: 8.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                }),
                            button(text("Rename").align_x(Alignment::Center))
                                .on_press(Message::RenameSubmitted)
                                .padding(8)
                                .width(Length::Fill)
                                .style(|theme: &iced::Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(palette.primary.base.color.into()),
                                        text_color: palette.primary.base.text,
                                        border: Border {
                                            radius: 8.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                }),
                        ]
                        .spacing(12)
                    ]
                    .spacing(16)
                    .padding(20)
                    .width(Length::Fixed(300.0))
                )
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.base.color.into()),
                        border: Border {
                            color: palette.background.strong.color,
                            width: 1.0,
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    }
                })
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color {
                    a: 0.5,
                    ..Color::BLACK
                }.into()),
                ..Default::default()
            });

            root = root.push(mouse_area(dialog).on_press(Message::CancelRename));
        }

        root.into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
            iced::event::listen().filter_map(|event| {
                match event {
                    Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                        return Some(Message::MouseMoved(position));
                    }
                    Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                        if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                            return Some(Message::Search(search::SearchMessage::Clear));
                        }
                    }
                    _ => {}
                }
                None
            }),
        ])
    }
}
