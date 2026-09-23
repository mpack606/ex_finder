use crate::address_bar;
use crate::app_icons;
use crate::bottom_bar;
use crate::commands;
use crate::components::{navigation, rename_modal, selection::SelectionState};
use crate::context_menu;
use crate::directory::{self, DirectoryItem};
use crate::file_actions;
use crate::file_info;
use crate::grid_view;
use crate::list_view;
use crate::macos_drag;
use crate::search;
use crate::settings;
use crate::sidebar;
use crate::sorting;
use crate::tabs;
use crate::view_mode;
use iced::widget::{column, row, stack};
use iced::{Alignment, Element, Event, Length, Size, Task, keyboard};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct App {
    settings: settings::Settings,
    tabs_state: tabs::TabsState,
    recent_locations_expanded: bool,
    address_input: String,
    address_invalid: bool,
    address_editing: bool,
    search_query: String,
    sort_order: sorting::SortOrder,
    view_mode: view_mode::ViewMode,
    directory: directory::State,
    loaded_icons: HashMap<PathBuf, iced::widget::image::Handle>,
    operation_error: Option<String>,
    settings_save_generation: u64,
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
    clipboard: file_actions::Clipboard,
    item_drag: Option<ItemDragState>,
    external_drag: Option<ExternalDragState>,
    hovered_grid_item: Option<PathBuf>,
    suppress_next_item_click: bool,
    renaming_path: Option<(PathBuf, String)>,
    file_info: Option<file_info::State>,
}

#[derive(Debug, Clone)]
struct ItemDragState {
    paths: Vec<PathBuf>,
    start: iced::Point,
    is_dragging: bool,
    drop_target: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ExternalDragState {
    paths: Vec<PathBuf>,
    grid_origin: iced::Point,
    is_inside: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::SidebarMessage),
    AddressBar(address_bar::AddressBarMessage),
    Search(search::SearchMessage),
    Sorting(sorting::SortingMessage),
    Grid(grid_view::GridMessage),
    List(list_view::ListMessage),
    ViewMode(view_mode::Message),
    AppIconsFound(Vec<PathBuf>, u64, Option<Vec<u8>>),
    DirectoryLoaded(directory::LoadResult),
    OperationCompleted(file_actions::Event),
    SaveSettings(u64),
    SettingsSaved(Result<(), String>),
    ExitRequested(iced::window::Id),
    SettingsSavedAndClose(iced::window::Id, Result<(), String>),
    WindowResized(iced::window::Id, Size),
    MouseMoved(iced::Point),
    CursorLeft(iced::window::Id),
    ExternalDragStarted(Result<(), String>),
    ExternalDragEnded(Option<macos_drag::Outcome>),
    ExternalFileHovered(PathBuf),
    ExternalFilesHoveredLeft,
    ModifiersChanged(iced::keyboard::Modifiers),
    GlobalMouseUp,
    ContextMenu(context_menu::ContextMenuMessage),
    Navigation(navigation::Message),
    Shortcut(commands::CommandKind),
    Tabs(tabs::TabsMessage),
    RenameRequested(PathBuf),
    RenameInputChanged(String),
    RenameSubmitted,
    CancelRename,
    FileInfo(file_info::Message),
    FileInfoRequested(PathBuf),
    FileInfoLoaded(PathBuf, Result<file_info::Details, String>),
    Refresh,
    RefreshAndSelect(Vec<PathBuf>),
    CutCompleted(Vec<PathBuf>),
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let settings = settings::load_settings();
        let initial_path = settings
            .last_directory
            .clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));
        let address_input = initial_path.to_string_lossy().into_owned();
        let window_width = settings.window_width as f32;
        let window_height = settings.window_height as f32;

        let mut app = Self {
            settings,
            tabs_state: tabs::TabsState::new(initial_path.clone()),
            recent_locations_expanded: false,
            address_input,
            address_invalid: false,
            address_editing: false,
            search_query: String::new(),
            sort_order: sorting::SortOrder::default(),
            view_mode: view_mode::ViewMode::default(),
            directory: directory::State::default(),
            loaded_icons: HashMap::new(),
            operation_error: None,
            settings_save_generation: 0,
            icon_load_generation: 0,
            selection: SelectionState::default(),
            modifiers: iced::keyboard::Modifiers::default(),
            grid_scroll_y: 0.0,
            cursor_in_grid: iced::Point::ORIGIN,
            window_width,
            window_height,
            last_click: None,
            cursor_position: iced::Point::ORIGIN,
            context_menu: None,
            clipboard: file_actions::Clipboard::default(),
            item_drag: None,
            external_drag: None,
            hovered_grid_item: None,
            suppress_next_item_click: false,
            renaming_path: None,
            file_info: None,
        };

        let task = app.request_directory_load(
            initial_path,
            directory::LoadAction::Current {
                select: Some(Vec::new()),
                record_location: true,
            },
        );

        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_msg) => match sidebar_msg {
                sidebar::SidebarMessage::SelectPath(path) => {
                    return self.navigate_to_path(path);
                }
                sidebar::SidebarMessage::AddCurrentPath(path) => {
                    if !self.settings.quick_access_paths.contains(&path) {
                        self.settings.quick_access_paths.push(path);
                        return self.schedule_settings_save();
                    }
                }
                sidebar::SidebarMessage::RemovePath(path) => {
                    self.settings.quick_access_paths.retain(|p| p != &path);
                    return self.schedule_settings_save();
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
                sidebar::SidebarMessage::ToggleRecentLocations => {
                    self.recent_locations_expanded = !self.recent_locations_expanded;
                }
            },
            Message::AddressBar(address_msg) => match address_msg {
                address_bar::AddressBarMessage::Edit => {
                    self.address_editing = true;
                    return iced::widget::operation::focus(address_bar::ADDRESS_INPUT_ID);
                }
                address_bar::AddressBarMessage::InputChanged(val) => {
                    self.address_input = val;
                }
                address_bar::AddressBarMessage::Navigate(path) => {
                    return self.navigate_to_path(path);
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
            },
            Message::Search(search_msg) => match search_msg {
                search::SearchMessage::InputChanged(val) => {
                    self.search_query = val;
                }
                search::SearchMessage::Clear => {
                    self.search_query.clear();
                }
            },
            Message::Sorting(sorting::SortingMessage::Selected(order)) => {
                self.sort_order = order;
                self.grid_scroll_y = 0.0;
            }
            Message::ViewMode(view_mode::Message::Selected(mode)) => {
                self.view_mode = mode;
                self.grid_scroll_y = 0.0;
                self.selection.cancel_drag();
                self.item_drag = None;
                self.hovered_grid_item = None;
            }
            Message::List(list_msg) => {
                let grid_msg = match list_msg {
                    list_view::ListMessage::ItemPressed(path) => {
                        grid_view::GridMessage::ItemPressed(path)
                    }
                    list_view::ListMessage::ItemClicked(path, is_dir) => {
                        grid_view::GridMessage::ItemClicked(path, is_dir)
                    }
                    list_view::ListMessage::ItemHovered(path) => {
                        grid_view::GridMessage::ItemHovered(path)
                    }
                    list_view::ListMessage::ItemRightClicked(path, is_dir) => {
                        grid_view::GridMessage::ItemRightClicked(path, is_dir)
                    }
                    list_view::ListMessage::BackgroundDown => {
                        grid_view::GridMessage::BackgroundDown
                    }
                    list_view::ListMessage::BackgroundUp => grid_view::GridMessage::BackgroundUp,
                    list_view::ListMessage::PointerMoved(position) => {
                        grid_view::GridMessage::PointerMoved(position)
                    }
                    list_view::ListMessage::Scrolled(offset) => {
                        grid_view::GridMessage::Scrolled(offset)
                    }
                    list_view::ListMessage::BackgroundRightClicked => {
                        grid_view::GridMessage::BackgroundRightClicked
                    }
                };
                return self.update(Message::Grid(grid_msg));
            }
            Message::Grid(grid_msg) => match grid_msg {
                grid_view::GridMessage::ItemPressed(path) => {
                    self.context_menu = None;
                    self.selection.cancel_drag();
                    self.suppress_next_item_click = false;
                    let paths = if self.selection.selected.contains(&path) {
                        self.selection.selected.iter().cloned().collect()
                    } else {
                        vec![path]
                    };
                    self.item_drag = Some(ItemDragState {
                        paths,
                        start: self.cursor_in_grid,
                        is_dragging: false,
                        drop_target: None,
                    });
                }
                grid_view::GridMessage::ItemClicked(path, is_dir) => {
                    if self.item_drag.is_none() && !self.suppress_next_item_click {
                        return Task::none();
                    }
                    if self.item_drag.as_ref().is_some_and(|drag| drag.is_dragging)
                        || self.suppress_next_item_click
                    {
                        self.suppress_next_item_click = false;
                        return Task::none();
                    }
                    self.item_drag = None;
                    self.context_menu = None;
                    self.selection.cancel_drag();
                    let now = Instant::now();
                    let is_double_click = if let Some((last_path, last_time)) = &self.last_click {
                        *last_path == path
                            && now.duration_since(*last_time) < Duration::from_millis(300)
                    } else {
                        false
                    };

                    self.last_click = Some((path.clone(), now));

                    if is_double_click {
                        if is_dir {
                            return self.navigate_to_path(path);
                        } else {
                            return file_actions::open(path).map(Message::OperationCompleted);
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
                grid_view::GridMessage::ItemHovered(path) => {
                    self.hovered_grid_item = path;
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
                    self.item_drag = None;
                    self.selection.begin_drag(self.cursor_in_grid);
                }
                grid_view::GridMessage::BackgroundUp => {
                    self.selection.finish_drag();
                }
                grid_view::GridMessage::PointerMoved(pos) => {
                    self.cursor_in_grid = pos;
                    if self.item_drag.is_some() {
                        let items = self.filtered_items();
                        let (dragging_paths, is_dragging, became_dragging) = {
                            let drag = self.item_drag.as_mut().unwrap();
                            let dx = (pos.x - drag.start.x).abs();
                            let dy = (pos.y - drag.start.y).abs();
                            let was_dragging = drag.is_dragging;
                            if dx >= 4.0 || dy >= 4.0 {
                                drag.is_dragging = true;
                            }
                            (
                                drag.paths.clone(),
                                drag.is_dragging,
                                !was_dragging && drag.is_dragging,
                            )
                        };
                        if became_dragging {
                            self.selection.select_paths(dragging_paths.clone());
                        }
                        let target = is_dragging
                            .then(|| match self.view_mode {
                                view_mode::ViewMode::Grid => grid_view::directory_at_position(
                                    &items,
                                    pos,
                                    self.window_width,
                                    self.grid_scroll_y,
                                ),
                                view_mode::ViewMode::List => list_view::directory_at_position(
                                    &items,
                                    pos,
                                    self.window_width,
                                    self.grid_scroll_y,
                                ),
                            })
                            .flatten()
                            .filter(|target| !dragging_paths.contains(target));
                        if let Some(drag) = &mut self.item_drag {
                            drag.drop_target = target;
                        }
                        return Task::none();
                    }
                    let items = self.filtered_items();
                    self.selection.update_drag(
                        pos,
                        &items,
                        self.window_width,
                        self.grid_scroll_y,
                        self.modifiers,
                        self.view_mode,
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
            },
            Message::MouseMoved(position) => {
                self.cursor_position = position;
            }
            Message::CursorLeft(window_id) => {
                if let Some(external_drag) = &mut self.external_drag {
                    external_drag.is_inside = false;
                    self.item_drag = None;
                    return Task::none();
                }
                if let Some(paths) = take_external_drag(&mut self.item_drag) {
                    self.suppress_next_item_click = true;
                    self.external_drag = Some(ExternalDragState {
                        paths: paths.clone(),
                        grid_origin: iced::Point::new(
                            self.cursor_position.x - self.cursor_in_grid.x,
                            self.cursor_position.y - self.cursor_in_grid.y,
                        ),
                        is_inside: false,
                    });
                    return iced::window::run(window_id, move |window| {
                        macos_drag::start(window, &paths)
                    })
                    .then(|result| match result {
                        Ok(receiver) => Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || receiver.recv())
                                    .await
                                    .ok()
                                    .and_then(Result::ok)
                            },
                            Message::ExternalDragEnded,
                        ),
                        Err(error) => Task::done(Message::ExternalDragStarted(Err(error))),
                    });
                }
            }
            Message::ExternalDragStarted(result) => {
                if let Err(error) = result {
                    self.operation_error = Some(error);
                    self.external_drag = None;
                }
            }
            Message::ExternalDragEnded(outcome) => {
                let external_drag = self.external_drag.take();
                self.item_drag = None;
                let Some((external_drag, position)) = external_drag.zip(
                    outcome
                        .filter(|outcome| outcome.dropped)
                        .and_then(|outcome| outcome.position),
                ) else {
                    return Task::none();
                };
                if position.x < 0.0
                    || position.y < 0.0
                    || position.x > self.window_width
                    || position.y > self.window_height
                {
                    return Task::none();
                }
                return self.finish_external_drop(external_drag, position);
            }
            Message::ExternalFileHovered(path) => {
                if let Some(external_drag) = &mut self.external_drag
                    && external_drag.paths.contains(&path)
                {
                    external_drag.is_inside = true;
                    if self.item_drag.is_none() {
                        self.item_drag = Some(ItemDragState {
                            paths: external_drag.paths.clone(),
                            start: self.cursor_in_grid,
                            is_dragging: true,
                            drop_target: None,
                        });
                    }
                }
            }
            Message::ExternalFilesHoveredLeft => {
                if let Some(external_drag) = &mut self.external_drag {
                    self.item_drag = None;
                    external_drag.is_inside = false;
                }
            }
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::GlobalMouseUp => {
                self.selection.finish_drag();
                if self.external_drag.is_some() {
                    return Task::none();
                }
                if self.item_drag.as_ref().is_some_and(|drag| drag.is_dragging) {
                    let drag = self.item_drag.take().unwrap();
                    self.suppress_next_item_click = true;
                    if let Some(destination) = drag.drop_target {
                        return self
                            .execute_command(commands::Command::Move(drag.paths, destination));
                    }
                }
            }
            Message::ContextMenu(context_msg) => match context_msg {
                context_menu::ContextMenuMessage::Close => {
                    self.context_menu = None;
                }
                context_menu::ContextMenuMessage::Action(action) => {
                    return self.execute_command(action);
                }
            },
            Message::Shortcut(command_kind) => {
                return self.handle_shortcut(command_kind);
            }
            Message::Tabs(tabs_msg) => {
                if let Some(tabs::TabsEvent::NavigationChanged) = self.tabs_state.update(tabs_msg) {
                    let path = self.tabs_state.active_path().clone();
                    return self.request_directory_load(
                        path,
                        directory::LoadAction::Current {
                            select: Some(Vec::new()),
                            record_location: true,
                        },
                    );
                }
            }
            Message::RenameRequested(path) => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
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
                if let Some((old_path, new_name)) = self.renaming_path.take()
                    && !new_name.is_empty()
                {
                    return file_actions::rename(old_path, new_name)
                        .map(Message::OperationCompleted);
                }
            }
            Message::CancelRename => {
                self.renaming_path = None;
            }
            Message::FileInfo(file_info::Message::Close) => {
                self.file_info = None;
            }
            Message::FileInfoRequested(path) => {
                self.file_info = Some(file_info::State::Loading(path.clone()));
                return Task::perform(
                    async move {
                        let load_path = path.clone();
                        let result =
                            tokio::task::spawn_blocking(move || file_info::load(load_path))
                                .await
                                .map_err(|error| format!("background task failed: {error}"))
                                .and_then(|result| result);
                        (path, result)
                    },
                    |(path, result)| Message::FileInfoLoaded(path, result),
                );
            }
            Message::FileInfoLoaded(path, result) => {
                if matches!(&self.file_info, Some(file_info::State::Loading(loading)) if loading == &path)
                {
                    self.file_info = Some(match result {
                        Ok(details) => file_info::State::Loaded(details),
                        Err(message) => file_info::State::Error { path, message },
                    });
                }
            }
            Message::AppIconsFound(paths, generation, icon_bytes) => {
                if generation != self.icon_load_generation {
                    return Task::none();
                }

                if let Some(icon) = icon_bytes.map(iced::widget::image::Handle::from_bytes) {
                    for path in paths {
                        if self.directory.items.iter().any(|item| item.path == path) {
                            self.loaded_icons.insert(path, icon.clone());
                        }
                    }
                }
            }
            Message::WindowResized(_id, size) => {
                self.window_width = size.width;
                self.window_height = size.height;
                self.settings.window_width = size.width as u32;
                self.settings.window_height = size.height as u32;
                return self.schedule_settings_save();
            }
            Message::Navigation(message) => match message {
                navigation::Message::Back => {
                    if let Some(path) = self.tabs_state.active_tab().history_back.last().cloned() {
                        return self.request_directory_load(path, directory::LoadAction::Back);
                    }
                }
                navigation::Message::Forward => {
                    if let Some(path) = self.tabs_state.active_tab().history_forward.last().cloned()
                    {
                        return self.request_directory_load(path, directory::LoadAction::Forward);
                    }
                }
                navigation::Message::Up => {
                    if let Some(parent) = self.tabs_state.active_path().parent() {
                        let parent = parent.to_path_buf();
                        return self.navigate_to_path(parent);
                    }
                }
            },
            Message::Refresh => {
                return self.request_directory_load(
                    self.tabs_state.active_path().clone(),
                    directory::LoadAction::Current {
                        select: None,
                        record_location: false,
                    },
                );
            }
            Message::RefreshAndSelect(paths) => {
                return self.request_directory_load(
                    self.tabs_state.active_path().clone(),
                    directory::LoadAction::Current {
                        select: Some(paths),
                        record_location: false,
                    },
                );
            }
            Message::CutCompleted(paths) => {
                self.clipboard.clear();
                return self.update(Message::RefreshAndSelect(paths));
            }
            Message::DirectoryLoaded(load) => {
                if !self.directory.accepts(load.generation) {
                    return Task::none();
                }
                match load.result {
                    Ok(items) => {
                        return self.finish_directory_load(load.path, load.action, items);
                    }
                    Err(error) => self.operation_error = Some(error),
                }
            }
            Message::OperationCompleted(event) => return self.handle_operation_event(event),
            Message::SaveSettings(generation) => {
                if generation != self.settings_save_generation {
                    return Task::none();
                }
                let settings = self.settings.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            settings::save_settings(&settings).map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| format!("background task failed: {error}"))
                        .and_then(|result| result)
                    },
                    Message::SettingsSaved,
                );
            }
            Message::SettingsSaved(result) => {
                if let Err(error) = result {
                    self.operation_error = Some(format!("Could not save settings: {error}"));
                }
            }
            Message::ExitRequested(id) => {
                let settings = self.settings.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            settings::save_settings(&settings).map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| format!("background task failed: {error}"))
                        .and_then(|result| result)
                    },
                    move |result| Message::SettingsSavedAndClose(id, result),
                );
            }
            Message::SettingsSavedAndClose(id, result) => {
                if let Err(error) = result {
                    eprintln!("Could not save settings before closing: {error}");
                }
                return iced::window::close(id);
            }
        }
        Task::none()
    }

    fn navigate_to_path(&mut self, path: PathBuf) -> Task<Message> {
        self.request_directory_load(path, directory::LoadAction::Navigate)
    }

    fn finish_external_drop(
        &mut self,
        external_drag: ExternalDragState,
        window_position: iced::Point,
    ) -> Task<Message> {
        let position = grid_position(window_position, external_drag.grid_origin);
        let items = self.filtered_items();
        let destination = match self.view_mode {
            view_mode::ViewMode::Grid => grid_view::directory_at_position(
                &items,
                position,
                self.window_width,
                self.grid_scroll_y,
            ),
            view_mode::ViewMode::List => list_view::directory_at_position(
                &items,
                position,
                self.window_width,
                self.grid_scroll_y,
            ),
        }
        .filter(|target| !external_drag.paths.contains(target));

        if let Some(destination) = destination {
            self.suppress_next_item_click = true;
            self.execute_command(commands::Command::Move(external_drag.paths, destination))
        } else {
            Task::none()
        }
    }

    fn request_directory_load(
        &mut self,
        path: PathBuf,
        action: directory::LoadAction,
    ) -> Task<Message> {
        if matches!(
            &action,
            directory::LoadAction::Current {
                record_location: true,
                ..
            }
        ) {
            self.directory.items.clear();
            self.selection.clear();
        }
        self.operation_error = None;
        self.directory
            .request(path, action)
            .map(Message::DirectoryLoaded)
    }

    fn finish_directory_load(
        &mut self,
        path: PathBuf,
        action: directory::LoadAction,
        items: Vec<DirectoryItem>,
    ) -> Task<Message> {
        let (selection, record_location) = match action {
            directory::LoadAction::Current {
                select,
                record_location,
            } => (select, record_location),
            directory::LoadAction::Navigate => {
                self.tabs_state.active_tab_mut().navigate_to(path.clone());
                (Some(Vec::new()), true)
            }
            directory::LoadAction::Back => {
                self.tabs_state.active_tab_mut().navigate_back();
                (Some(Vec::new()), true)
            }
            directory::LoadAction::Forward => {
                self.tabs_state.active_tab_mut().navigate_forward();
                (Some(Vec::new()), true)
            }
        };

        self.directory.items = items;
        self.loaded_icons.clear();
        if let Some(paths) = selection {
            let paths = paths
                .into_iter()
                .filter(|path| self.directory.items.iter().any(|item| item.path == *path))
                .collect();
            self.selection.select_paths(paths);
        }
        self.operation_error = None;
        self.address_input = path.to_string_lossy().into_owned();
        self.address_invalid = false;
        self.address_editing = false;
        self.grid_scroll_y = 0.0;
        self.context_menu = None;
        self.file_info = None;

        let icon_task = self.load_app_icons();
        if record_location {
            self.settings.last_directory = Some(path.clone());
            self.settings.record_recent_location(path);
            Task::batch([icon_task, self.schedule_settings_save()])
        } else {
            icon_task
        }
    }

    pub fn filtered_items(&self) -> Vec<DirectoryItem> {
        let mut items = if self.search_query.is_empty() {
            self.directory.items.clone()
        } else {
            let query = self.search_query.to_lowercase();
            self.directory
                .items
                .iter()
                .filter(|item| item.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        };
        sorting::sort_items(&mut items, self.sort_order);
        items
    }

    fn load_app_icons(&mut self) -> Task<Message> {
        self.icon_load_generation = self.icon_load_generation.wrapping_add(1);
        let generation = self.icon_load_generation;
        let mut paths_by_icon = HashMap::<app_icons::IconKey, Vec<PathBuf>>::new();
        for item in self.directory.items.iter().filter(|item| !item.is_dir) {
            paths_by_icon
                .entry(app_icons::cache_key(&item.path))
                .or_default()
                .push(item.path.clone());
        }

        let tasks = paths_by_icon.into_values().map(|paths| {
            let sample_path = paths[0].clone();
            Task::perform(
                async move {
                    let icon = tokio::task::spawn_blocking(move || {
                        app_icons::get_app_icon_for_file(&sample_path)
                    })
                    .await
                    .ok()
                    .flatten();
                    (paths, generation, icon)
                },
                |(paths, generation, icon)| Message::AppIconsFound(paths, generation, icon),
            )
        });

        Task::batch(tasks)
    }

    fn execute_command(&mut self, command: commands::Command) -> Task<Message> {
        self.context_menu = None;
        self.operation_error = None;
        file_actions::execute(
            command,
            &mut self.clipboard,
            self.tabs_state.active_path().clone(),
        )
        .map(Message::OperationCompleted)
    }

    fn handle_operation_event(&mut self, event: file_actions::Event) -> Task<Message> {
        match event {
            file_actions::Event::Done => Task::none(),
            file_actions::Event::Refresh => self.update(Message::Refresh),
            file_actions::Event::RefreshAndSelect(paths) => {
                self.update(Message::RefreshAndSelect(paths))
            }
            file_actions::Event::CutCompleted(paths) => self.update(Message::CutCompleted(paths)),
            file_actions::Event::Rename(path) => self.update(Message::RenameRequested(path)),
            file_actions::Event::OpenInNewTab(path) => {
                self.update(Message::Tabs(tabs::TabsMessage::Open(path)))
            }
            file_actions::Event::GetInfo(path) => self.update(Message::FileInfoRequested(path)),
            file_actions::Event::Failed(error) => {
                self.operation_error = Some(error);
                Task::none()
            }
        }
    }

    fn schedule_settings_save(&mut self) -> Task<Message> {
        self.settings_save_generation = self.settings_save_generation.wrapping_add(1);
        let generation = self.settings_save_generation;
        Task::perform(
            async move { tokio::time::sleep(Duration::from_millis(500)).await },
            move |_| Message::SaveSettings(generation),
        )
    }

    fn handle_shortcut(&mut self, command_kind: commands::CommandKind) -> Task<Message> {
        if self.renaming_path.is_some() || self.file_info.is_some() {
            return Task::none();
        }

        match command_kind {
            commands::CommandKind::Search => {
                iced::widget::operation::focus(search::SEARCH_INPUT_ID)
            }
            commands::CommandKind::Copy => {
                let paths: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
                if paths.is_empty() {
                    Task::none()
                } else {
                    self.execute_command(commands::Command::Copy(paths))
                }
            }
            commands::CommandKind::Cut => {
                let paths: Vec<PathBuf> = self
                    .selection
                    .selected
                    .iter()
                    .filter(|path| self.directory.items.iter().any(|item| &item.path == *path))
                    .cloned()
                    .collect();
                if paths.is_empty() {
                    Task::none()
                } else {
                    self.execute_command(commands::Command::Cut(paths))
                }
            }
            commands::CommandKind::Paste => self.execute_command(commands::Command::Paste(
                self.tabs_state.active_path().clone(),
            )),
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
            keyboard::Key::Character(value) => {
                value.chars().next().map(commands::ShortcutKey::Character)
            }
            keyboard::Key::Named(keyboard::key::Named::Enter) => Some(commands::ShortcutKey::Enter),
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
        format!(
            "ex_finder - {}",
            self.tabs_state.active_path().to_string_lossy()
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let nav_buttons = navigation::view_controls().map(Message::Navigation);

        let top_row = row![
            nav_buttons,
            address_bar::view(
                self.tabs_state.active_path(),
                &self.address_input,
                self.address_invalid,
                self.address_editing,
            )
            .map(Message::AddressBar),
            sorting::view(self.sort_order).map(Message::Sorting),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(8);

        let filtered_items = self.filtered_items();

        let items_element: Element<'_, Message> = match self.view_mode {
            view_mode::ViewMode::Grid => grid_view::view(
                &filtered_items,
                grid_view::ViewOptions {
                    selected_items: &self.selection.selected,
                    app_icons: &self.loaded_icons,
                    window_width: self.window_width,
                    drag_rect: self.selection.drag_rect(),
                    drop_target: self
                        .item_drag
                        .as_ref()
                        .and_then(|drag| drag.drop_target.as_deref()),
                    hovered_item: self.hovered_grid_item.as_deref(),
                    cut_items: self.clipboard.cut_paths(),
                },
            )
            .map(Message::Grid),
            view_mode::ViewMode::List => list_view::view(
                &filtered_items,
                &self.selection.selected,
                &self.loaded_icons,
                self.item_drag
                    .as_ref()
                    .and_then(|drag| drag.drop_target.as_deref()),
                self.hovered_grid_item.as_deref(),
                self.clipboard.cut_paths(),
            )
            .map(Message::List),
        };

        let selected_vec: Vec<PathBuf> = self.selection.selected.iter().cloned().collect();
        let status = self.status_text();
        let bottom_bar = bottom_bar::view(
            &selected_vec,
            status.as_deref(),
            view_mode::view(self.view_mode).map(Message::ViewMode),
            search::view(&self.search_query).map(Message::Search),
        );

        let mut main_content = column![];
        if self.tabs_state.list.len() > 1 {
            main_content = main_content.push(tabs::view(&self.tabs_state).map(Message::Tabs));
        }
        main_content = main_content.push(items_element).push(bottom_bar);

        let body = row![
            sidebar::view(
                &self.settings.quick_access_paths,
                &self.settings.recent_locations,
                self.recent_locations_expanded,
                self.tabs_state.active_path(),
            )
            .map(Message::Sidebar),
            main_content
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        let content = column![top_row, body]
            .width(Length::Fill)
            .height(Length::Fill);

        let mut root = stack![content];

        if let Some(context_menu) = &self.context_menu {
            root = root.push(
                context_menu::view(
                    context_menu,
                    self.clipboard.has_items(),
                    self.tabs_state.active_path(),
                )
                .map(Message::ContextMenu),
            );
        } else if let Some(file_info) = &self.file_info {
            root = root.push(file_info::view(file_info).map(Message::FileInfo));
        } else if let Some((_, input)) = &self.renaming_path {
            root = root.push(rename_modal::view(input));
        }

        root.into()
    }

    fn status_text(&self) -> Option<String> {
        self.directory
            .loading
            .as_ref()
            .map(|path| format!("Loading {}…", path.display()))
            .or_else(|| self.operation_error.clone())
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
            iced::window::close_requests().map(Message::ExitRequested),
            iced::event::listen_with(|event, status, window| {
                match event {
                    Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                        Some(Message::MouseMoved(position))
                    }
                    Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                        Some(Message::GlobalMouseUp)
                    }
                    Event::Mouse(iced::mouse::Event::CursorLeft) => {
                        Some(Message::CursorLeft(window))
                    }
                    Event::Window(iced::window::Event::FileHovered(path)) => {
                        Some(Message::ExternalFileHovered(path))
                    }
                    Event::Window(iced::window::Event::FilesHoveredLeft) => {
                        Some(Message::ExternalFilesHoveredLeft)
                    }
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        Some(Message::ModifiersChanged(modifiers))
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                        if let Some(command_kind) = Self::shortcut_key(&key)
                            .and_then(|key| {
                                commands::resolve_shortcut(key, Self::shortcut_modifiers(modifiers))
                            })
                            // Text inputs capture editing keys. Do not turn Enter,
                            // Delete, Copy, Cut, or Paste into file operations while the
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

fn take_external_drag(item_drag: &mut Option<ItemDragState>) -> Option<Vec<PathBuf>> {
    if item_drag.as_ref().is_some_and(|drag| drag.is_dragging) {
        item_drag.take().map(|drag| drag.paths)
    } else {
        None
    }
}

fn grid_position(window_position: iced::Point, grid_origin: iced::Point) -> iced::Point {
    iced::Point::new(
        window_position.x - grid_origin.x,
        window_position.y - grid_origin.y,
    )
}

#[cfg(test)]
mod tests {
    use super::{ItemDragState, grid_position, take_external_drag};
    use iced::Point;
    use std::path::PathBuf;

    fn drag(is_dragging: bool) -> Option<ItemDragState> {
        Some(ItemDragState {
            paths: vec![PathBuf::from("report.pdf")],
            start: Point::ORIGIN,
            is_dragging,
            drop_target: None,
        })
    }

    #[test]
    fn leaving_window_exports_an_active_item_drag() {
        let mut state = drag(true);

        assert_eq!(
            take_external_drag(&mut state),
            Some(vec![PathBuf::from("report.pdf")])
        );
        assert!(state.is_none());
    }

    #[test]
    fn leaving_window_does_not_steal_a_click_before_drag_threshold() {
        let mut state = drag(false);

        assert_eq!(take_external_drag(&mut state), None);
        assert!(state.is_some());
    }

    #[test]
    fn native_drop_position_is_translated_back_to_grid_coordinates() {
        assert_eq!(
            grid_position(Point::new(450.0, 320.0), Point::new(200.0, 52.0)),
            Point::new(250.0, 268.0)
        );
    }
}
