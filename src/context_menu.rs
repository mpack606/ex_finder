use crate::commands::{self, Command, CommandKind};
use iced::{Alignment, Border, Element, Length, Padding, Point, Task};
use iced::widget::{button, column, container, mouse_area, row, text};
use std::path::PathBuf;

pub use crate::commands::Command as ContextMenuAction;

#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    Action(ContextMenuAction),
    Close,
}

#[derive(Debug, Clone)]
pub enum ContextMenuEvent {
    Refresh,
    RefreshAndSelect(Vec<PathBuf>),
    CutCompleted(Vec<PathBuf>),
    Rename(PathBuf),
    OpenInNewTab(PathBuf),
    GetInfo(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    pub paths: Vec<PathBuf>,
    pub operation: Option<ClipboardOperation>,
}

impl Clipboard {
    pub fn has_items(&self) -> bool {
        !self.paths.is_empty()
    }

    pub fn cut_paths(&self) -> &[PathBuf] {
        if self.operation == Some(ClipboardOperation::Cut) {
            &self.paths
        } else {
            &[]
        }
    }
}

pub struct ContextMenuState {
    pub position: Point,
    pub paths: Vec<PathBuf>,
    pub target_is_dir: bool,
    pub is_sidebar: bool,
}

pub fn view(
    state: &ContextMenuState,
    clipboard_has_item: bool,
) -> Element<'static, ContextMenuMessage> {
    let mut menu_items = Vec::new();
    let is_folder = if state.paths.is_empty() {
        true
    } else if state.paths.len() == 1 {
        state.target_is_dir
    } else {
        false
    };

    if !state.paths.is_empty() {
        if state.paths.len() == 1 && state.target_is_dir {
            menu_items.push(menu_item(
                "Open in new tab",
                CommandKind::OpenInNewTab,
                Some(Command::OpenInNewTab(state.paths[0].clone())),
                true,
            ));
        }
        menu_items.push(menu_item(
            "Copy",
            CommandKind::Copy,
            Some(Command::Copy(state.paths.clone())),
            true,
        ));
        if !state.is_sidebar {
            menu_items.push(menu_item(
                "Cut",
                CommandKind::Cut,
                Some(Command::Cut(state.paths.clone())),
                true,
            ));
        }
        
        if !state.is_sidebar {
            if state.paths.len() == 1 {
                menu_items.push(menu_item(
                    "Rename",
                    CommandKind::Rename,
                    Some(Command::Rename(state.paths[0].clone())),
                    true,
                ));
            }
            menu_items.push(menu_item(
                "Move to trash",
                CommandKind::MoveToTrash,
                Some(Command::MoveToTrash(state.paths.clone())),
                true,
            ));
            menu_items.push(menu_item(
                "Zip",
                CommandKind::Zip,
                Some(Command::Zip(state.paths.clone())),
                true,
            ));
            
            if state.paths.len() == 1 && !state.target_is_dir && state.paths[0].extension().map_or(false, |ext| ext == "zip") {
                menu_items.push(menu_item(
                    "Unzip",
                    CommandKind::Unzip,
                    Some(Command::Unzip(state.paths[0].clone())),
                    true,
                ));
            }
        }
    } else {
        menu_items.push(menu_item(
            "Create new folder",
            CommandKind::CreateNewFolder,
            Some(Command::CreateNewFolder),
            true,
        ));
    }

    if is_folder {
        let action = if clipboard_has_item {
            Some(Command::Paste)
        } else {
            None
        };
        menu_items.push(menu_item(
            "Paste",
            CommandKind::Paste,
            action,
            clipboard_has_item,
        ));
    }

    menu_items.push(menu_item(
        "Refresh",
        CommandKind::Refresh,
        Some(Command::Refresh),
        true,
    ));

    if state.paths.len() == 1 {
        menu_items.push(menu_item(
            "Get Info",
            CommandKind::GetInfo,
            Some(Command::GetInfo(state.paths[0].clone())),
            true,
        ));
    }

    if menu_items.is_empty() {
        return column![].into();
    }

    let menu = container(
        column(menu_items)
            .width(Length::Fixed(220.0))
    )
    .padding(4)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    });

    mouse_area(
        container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding {
                top: state.position.y,
                left: state.position.x,
                ..Default::default()
            })
    )
    .on_press(ContextMenuMessage::Close)
    .on_right_press(ContextMenuMessage::Close)
    .into()
}

pub fn handle_action(
    action: ContextMenuAction,
    clipboard: &mut Clipboard,
    current_path: PathBuf,
) -> Task<Option<ContextMenuEvent>> {
    match action {
        ContextMenuAction::OpenInNewTab(path) => {
            Task::done(Some(ContextMenuEvent::OpenInNewTab(path)))
        }
        ContextMenuAction::GetInfo(path) => {
            Task::done(Some(ContextMenuEvent::GetInfo(path)))
        }
        ContextMenuAction::Copy(paths) => {
            clipboard.paths = paths;
            clipboard.operation = Some(ClipboardOperation::Copy);
            Task::done(Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Cut(paths) => {
            clipboard.paths = paths;
            clipboard.operation = Some(ClipboardOperation::Cut);
            Task::done(Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Paste => {
            if clipboard.has_items() {
                let paths = clipboard.paths.clone();
                let operation = clipboard.operation;
                Task::perform(async move {
                    match operation {
                        Some(ClipboardOperation::Cut) => commands::move_items(&paths, &current_path),
                        _ => commands::copy(&paths, &current_path),
                    }
                }, move |result| match result {
                    Ok(paths) if operation == Some(ClipboardOperation::Cut) => {
                        Some(ContextMenuEvent::CutCompleted(paths))
                    }
                    Ok(paths) => Some(ContextMenuEvent::RefreshAndSelect(paths)),
                    Err(error) => {
                        eprintln!("Failed to paste: {}", error);
                        Some(ContextMenuEvent::Refresh)
                    }
                })
            } else {
                Task::none()
            }
        }
        ContextMenuAction::Move(paths, destination) => {
            Task::perform(async move {
                commands::move_items(&paths, &destination)
            }, |result| match result {
                Ok(_) => Some(ContextMenuEvent::Refresh),
                Err(error) => {
                    eprintln!("Failed to move: {}", error);
                    Some(ContextMenuEvent::Refresh)
                }
            })
        }
        ContextMenuAction::MoveToTrash(paths) => {
            Task::perform(async move {
                commands::move_to_trash(&paths)
            }, |result| {
                if let Err(error) = result {
                    eprintln!("Failed to move to trash: {}", error);
                }
                Some(ContextMenuEvent::Refresh)
            })
        }
        ContextMenuAction::Rename(path) => {
            Task::done(Some(ContextMenuEvent::Rename(path)))
        }
        ContextMenuAction::Zip(paths) => {
            Task::perform(async move {
                commands::zip(&paths)
            }, |path| Some(ContextMenuEvent::RefreshAndSelect(path.into_iter().collect())))
        }
        ContextMenuAction::Unzip(path) => {
            Task::perform(async move {
                commands::unzip(&path)
            }, |result| {
                if let Err(error) = result {
                    eprintln!("Failed to unzip: {}", error);
                }
                Some(ContextMenuEvent::Refresh)
            })
        }
        ContextMenuAction::CreateNewFolder => {
            Task::perform(async move {
                commands::create_new_folder(&current_path)
            }, |result| {
                if let Err(error) = result {
                    eprintln!("Failed to create folder: {}", error);
                }
                Some(ContextMenuEvent::Refresh)
            })
        }
        ContextMenuAction::Refresh => {
            Task::done(Some(ContextMenuEvent::Refresh))
        }
    }
}

#[cfg(test)]
fn create_new_folder(current_path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    commands::create_new_folder(current_path.as_ref()).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn create_new_folder_creates_new_folder_in_current_directory() {
        let current_path = std::env::temp_dir().join(format!(
            "ex_finder_create_folder_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&current_path).unwrap();

        let result = create_new_folder(&current_path);

        assert!(result.is_ok());
        assert!(current_path.join("New Folder").is_dir());
        fs::remove_dir_all(current_path).unwrap();
    }

    #[test]
    fn create_new_folder_returns_error_when_folder_already_exists() {
        let current_path = std::env::temp_dir().join(format!(
            "ex_finder_create_folder_existing_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(current_path.join("New Folder")).unwrap();

        let result = create_new_folder(&current_path);

        assert!(result.is_err());
        fs::remove_dir_all(current_path).unwrap();
    }

    #[test]
    fn handle_action_copy_sets_multiple_paths_in_clipboard() {
        let mut clipboard = Clipboard::default();
        let paths = vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")];
        let _ = handle_action(
            ContextMenuAction::Copy(paths.clone()),
            &mut clipboard,
            PathBuf::from("/tmp"),
        );
        assert_eq!(clipboard.paths, paths);
        assert_eq!(clipboard.operation, Some(ClipboardOperation::Copy));
    }

    #[test]
    fn handle_action_cut_marks_clipboard_for_move() {
        let mut clipboard = Clipboard::default();
        let paths = vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")];
        let _ = handle_action(
            ContextMenuAction::Cut(paths.clone()),
            &mut clipboard,
            PathBuf::from("/tmp"),
        );

        assert_eq!(clipboard.paths, paths);
        assert_eq!(clipboard.operation, Some(ClipboardOperation::Cut));
        assert_eq!(clipboard.cut_paths(), clipboard.paths);
    }
}

fn menu_item(
    label: &str,
    command_kind: CommandKind,
    action: Option<Command>,
    enabled: bool,
) -> Element<'static, ContextMenuMessage> {
    let command_kind = action
        .as_ref()
        .map(Command::kind)
        .unwrap_or(command_kind);
    let mut content = row![text(label.to_owned()).size(13).width(Length::Fill)]
        .spacing(12)
        .align_y(Alignment::Center);

    if let Some(shortcut) = commands::shortcut_for(command_kind) {
        let shortcut_hint = container(
            text(format!("[{shortcut}]"))
                .size(12)
                .font(iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..Default::default()
                })
                .style(move |theme: &iced::Theme| {
                    let mut color = theme.extended_palette().background.strong.text;
                    color.a = if enabled { 0.8 } else { 0.4 };
                    iced::widget::text::Style { color: Some(color) }
                }),
        )
        .padding(Padding {
            top: 2.0,
            right: 6.0,
            bottom: 2.0,
            left: 6.0,
        })
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        });
        content = content.push(shortcut_hint);
    }

    let mut btn = button(content)
    .padding(8)
    .width(Length::Fill);

    if let Some(command) = action {
        btn = btn.on_press(ContextMenuMessage::Action(command));
    }

    btn.style(move |theme: &iced::Theme, status| {
        let palette = theme.extended_palette();
        let is_hovered = status == button::Status::Hovered;
        
        let text_color = if enabled {
            palette.background.strong.text
        } else {
            let mut color = palette.background.strong.text;
            color.a = 0.4;
            color
        };

        button::Style {
            background: if is_hovered && enabled {
                Some(palette.primary.weak.color.into())
            } else {
                None
            },
            text_color,
            border: Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
