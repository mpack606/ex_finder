use crate::address_bar;
use crate::search;
use crate::grid_view;
use crate::bottom_bar;
use crate::settings;
use crate::sidebar;
use crate::app_icons;
use crate::context_menu;
use crate::commands;
use crate::tabs;
use crate::components::{navigation, rename_modal, selection::SelectionState};
use iced::{Element, Task, Size, Length, Alignment, keyboard, Event};
use iced::widget::{column, row, stack};
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
    icon_load_generation: u64,
    pub selection: SelectionState,
    pub modifiers: iced::keyboard::Modifiers,
    pub grid_scroll_y: f32,
    pub cursor_in_grid: iced::Point,
    window_width: f32,
    window_height: f32,
    last_click: Option<(PathBuf, Instant)>,
    cursor_position: iced::Point,
    context_menu: Option<context_menu::ContextMenuState>,
    clipboard: Vec<PathBuf>,
    renaming_path: Option<(PathBuf, String)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::SidebarMessage),
    AddressBar(address_bar::AddressBarMessage),
    Search(search::SearchMessage),
    Grid(grid_view::GridMessage),
    AppIconFound(PathBuf, u64, Option<Vec<u8>>),
    WindowResized(iced::window::Id, Size),
    MouseMoved(iced::Point),
    ModifiersChanged(iced::keyboard::Modifiers),
    GlobalMouseUp,
    ContextMenu(context_menu::ContextMenuMessage),
    Shortcut(commands::CommandKind),
    Tabs(tabs::TabsMessage),
    RenameRequested(PathBuf),
    RenameInputChanged(String),
    RenameSubmitted,
    CancelRename,
    Refresh,
    RefreshAndSelect(Vec<PathBuf>),
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

        let mut app = Self {
            settings: settings.clone(),
            tabs_state: tabs::TabsState::new(initial_path),
            sidebar_paths,
            address_input,
            address_invalid: false,
            search_query: String::new(),
            grid_items,
            icon_load_generation: 0,
            selection: SelectionState::default(),
            modifiers: iced::keyboard::Modifiers::default(),
            grid_scroll_y: 0.0,
            cursor_in_grid: iced::Point::ORIGIN,
            window_width: settings.window_width as f32,
            window_height: settings.window_height as f32,
            last_click: None,
            cursor_position: iced::Point::ORIGIN,
            context_menu: None,
            clipboard: Vec::new(),
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
                        self.selection.select_single(path.clone());
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            paths: vec![path],
                            target_is_dir: true,
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
                        self.selection.cancel_drag();
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
                            if self.modifiers.command() {
                                self.selection.toggle_cmd(path);
                            } else if self.modifiers.shift() {
                                let items = self.filtered_items();
                                self.selection.select_range(path, &items);
                            } else {
                                self.selection.select_single(path);
                            }
                        }
                    }
                    grid_view::GridMessage::ItemRightClicked(path, is_dir) => {
                        self.selection.cancel_drag();
                        if !self.selection.selected.contains(&path) {
                            self.selection.select_single(path.clone());
                        }
                        let paths: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            paths,
                            target_is_dir: is_dir,
                            is_sidebar: false,
                        });
                    }
                    grid_view::GridMessage::BackgroundDown => {
                        self.context_menu = None;
                        self.selection.begin_drag(self.cursor_in_grid);
                    }
                    grid_view::GridMessage::BackgroundUp => {
                        self.selection.finish_drag();
                    }
                    grid_view::GridMessage::PointerMoved(pos) => {
                        self.cursor_in_grid = pos;
                        let items = self.filtered_items();
                        self.selection.update_drag(
                            pos,
                            &items,
                            self.window_width,
                            self.grid_scroll_y,
                            self.modifiers,
                        );
                    }
                    grid_view::GridMessage::Scrolled(y) => {
                        self.grid_scroll_y = y;
                    }
                    grid_view::GridMessage::BackgroundRightClicked => {
                        self.selection.cancel_drag();
                        self.context_menu = Some(context_menu::ContextMenuState {
                            position: self.cursor_position,
                            paths: Vec::new(),
                            target_is_dir: true,
                            is_sidebar: false,
                        });
                    }
                }
            }
            Message::MouseMoved(position) => {
                self.cursor_position = position;
            }
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::GlobalMouseUp => {
                self.selection.finish_drag();
            }
            Message::ContextMenu(context_msg) => {
                match context_msg {
                    context_menu::ContextMenuMessage::Close => {
                        self.context_menu = None;
                    }
                    context_menu::ContextMenuMessage::Action(action) => {
                        return self.execute_command(action);
                    }
                }
            }
            Message::Shortcut(command_kind) => {
                return self.handle_shortcut(command_kind);
            }
            Message::Tabs(tabs_msg) => {
                if let Some(tabs::TabsEvent::NavigationChanged) = self.tabs_state.update(tabs_msg) {
                    return self.on_navigation_changed();
                }
            }
            Message::RenameRequested(path) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let focus_task = rename_modal::focus_input(&name);
                self.renaming_path = Some((path, name));
                return focus_task;
            }
            Message::RenameInputChanged(val) => {
                if let Some((_, input)) = &mut self.renaming_path {
                    *input = val;
                }
            }
            Message::RenameSubmitted => {
                if let Some((old_path, new_name)) = self.renaming_path.take() {
                    if !new_name.is_empty() {
                        return Task::perform(async move {
                            commands::rename(&old_path, &new_name)
                        }, |result| {
                            match result {
                                Ok(new_path) => Message::RefreshAndSelect(vec![new_path]),
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
            Message::AppIconFound(path, generation, icon_bytes) => {
                if generation != self.icon_load_generation {
                    return Task::none();
                }

                if let Some(item) = self.grid_items.iter_mut().find(|item| item.path == path) {
                    item.app_icon = icon_bytes.map(iced::widget::image::Handle::from_bytes);
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
            Message::RefreshAndSelect(paths) => {
                self.grid_items = grid_view::read_directory(self.tabs_state.active_path()).unwrap_or_default();
                let paths = paths
                    .into_iter()
                    .filter(|path| self.grid_items.iter().any(|item| item.path == *path))
                    .collect();
                self.selection.select_paths(paths);
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
        self.selection.clear();
        self.grid_scroll_y = 0.0;
        self.context_menu = None;
        self.grid_items = grid_view::read_directory(&current).unwrap_or_default();
        self.load_app_icons()
    }

    pub fn filtered_items(&self) -> Vec<grid_view::DirectoryItem> {
        if self.search_query.is_empty() {
            self.grid_items.clone()
        } else {
            let query = self.search_query.to_lowercase();
            self.grid_items
                .iter()
                .filter(|item| item.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    }

    fn load_app_icons(&mut self) -> Task<Message> {
        self.icon_load_generation = self.icon_load_generation.wrapping_add(1);
        let generation = self.icon_load_generation;
        let paths = self
            .grid_items
            .iter()
            .filter(|item| !item.is_dir)
            .map(|item| item.path.clone())
            .collect::<Vec<_>>();

        let tasks = paths.into_iter().map(|path| {
            Task::perform(async move {
                let icon = app_icons::get_app_icon_for_file(&path);
                (path, generation, icon)
            }, |(path, generation, icon)| Message::AppIconFound(path, generation, icon))
        });

        Task::batch(tasks)
    }

    fn execute_command(&mut self, command: commands::Command) -> Task<Message> {
        self.context_menu = None;
        context_menu::handle_action(
            command,
            &mut self.clipboard,
            self.tabs_state.active_path().clone(),
        )
        .map(|event| match event {
            Some(context_menu::ContextMenuEvent::Refresh) => Message::Refresh,
            Some(context_menu::ContextMenuEvent::RefreshAndSelect(paths)) => {
                Message::RefreshAndSelect(paths)
            }
            Some(context_menu::ContextMenuEvent::Rename(path)) => {
                Message::RenameRequested(path)
            }
            Some(context_menu::ContextMenuEvent::OpenInNewTab(path)) => {
                Message::Tabs(tabs::TabsMessage::OpenTab(path))
            }
            None => Message::None,
        })
    }

    fn handle_shortcut(&mut self, command_kind: commands::CommandKind) -> Task<Message> {
        if self.renaming_path.is_some() {
            return Task::none();
        }

        match command_kind {
            commands::CommandKind::Search => iced::widget::operation::focus(search::SEARCH_INPUT_ID),
            commands::CommandKind::Copy => {
                let paths: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
                if paths.is_empty() {
                    Task::none()
                } else {
                    self.execute_command(commands::Command::Copy(paths))
                }
            }
            commands::CommandKind::Paste => self.execute_command(commands::Command::Paste),
            commands::CommandKind::MoveToTrash => {
                let paths: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
                if paths.is_empty() {
                    Task::none()
                } else {
                    self.execute_command(commands::Command::MoveToTrash(paths))
                }
            }
            commands::CommandKind::Rename => {
                let paths: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
                if paths.len() == 1 {
                    self.execute_command(commands::Command::Rename(paths[0].clone()))
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }

    fn shortcut_key(key: &keyboard::Key) -> Option<commands::ShortcutKey> {
        match key {
            keyboard::Key::Character(value) => value
                .chars()
                .next()
                .map(commands::ShortcutKey::Character),
            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                Some(commands::ShortcutKey::Enter)
            }
            keyboard::Key::Named(keyboard::key::Named::Delete)
            | keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                Some(commands::ShortcutKey::Delete)
            }
            _ => None,
        }
    }

    fn shortcut_modifiers(modifiers: keyboard::Modifiers) -> commands::ShortcutModifiers {
        commands::ShortcutModifiers {
            control: modifiers.control(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
            command: modifiers.command(),
        }
    }

    pub fn title(&self) -> String {
        format!("ex_finder - {}", self.tabs_state.active_path().to_string_lossy())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let nav_buttons = navigation::view_controls();

        let top_row = row![
            nav_buttons,
            address_bar::view(&self.address_input, self.address_invalid).map(Message::AddressBar),
            search::view(&self.search_query).map(Message::Search),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(8);

        let filtered_items = self.filtered_items();

        let grid_element = grid_view::view(
            &filtered_items,
            &self.selection.selected,
            self.window_width,
            self.selection.drag_rect(),
        )
        .map(Message::Grid);

        let selected_vec: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
        let bottom_bar = bottom_bar::view(&selected_vec);

        let mut main_content = column![];
        if self.tabs_state.list.len() > 1 {
            main_content = main_content.push(tabs::view(&self.tabs_state).map(Message::Tabs));
        }
        main_content = main_content.push(grid_element).push(bottom_bar);

        let body = row![
            sidebar::view(&self.sidebar_paths, self.tabs_state.active_path()).map(Message::Sidebar),
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
                context_menu::view(context_menu, !self.clipboard.is_empty())
                    .map(Message::ContextMenu)
            );
        } else if let Some((_, input)) = &self.renaming_path {
            root = root.push(rename_modal::view(input));
        }

        root.into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
            iced::event::listen_with(|event, status, _window| {
                match event {
                    Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                        Some(Message::MouseMoved(position))
                    }
                    Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                        Some(Message::GlobalMouseUp)
                    }
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        Some(Message::ModifiersChanged(modifiers))
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                        if let Some(command_kind) = Self::shortcut_key(&key)
                            .and_then(|key| commands::resolve_shortcut(
                                key,
                                Self::shortcut_modifiers(modifiers),
                            ))
                            // Text inputs capture editing keys. Do not turn Enter,
                            // Delete, Copy, or Paste into file operations while the
                            // user is editing text. Search remains application-wide.
                            .filter(|command_kind| {
                                status == iced::event::Status::Ignored
                                    || *command_kind == commands::CommandKind::Search
                            })
                        {
                            Some(Message::Shortcut(command_kind))
                        } else {
                            Some(Message::ModifiersChanged(modifiers))
                        }
                    }
                    Event::Keyboard(keyboard::Event::KeyReleased { key, modifiers, .. }) => {
                        if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                            Some(Message::Search(search::SearchMessage::Clear))
                        } else {
                            Some(Message::ModifiersChanged(modifiers))
                        }
                    }
                    _ => None,
                }
            }),
        ])
    }
}
