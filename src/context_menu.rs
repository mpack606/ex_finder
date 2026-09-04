use iced::{Element, Length, Border, Point, Padding, Task};
use iced::widget::{button, column, container, mouse_area, text};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    OpenInNewTab(PathBuf),
    Copy(Vec<PathBuf>),
    Paste,
    Rename(PathBuf),
    MoveToTrash(Vec<PathBuf>),
    Zip(Vec<PathBuf>),
    Unzip(PathBuf),
    CreateNewFolder,
    Refresh,
}

#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    Action(ContextMenuAction),
    Close,
}

#[derive(Debug, Clone)]
pub enum ContextMenuEvent {
    Refresh,
    Rename(PathBuf),
    OpenInNewTab(PathBuf),
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
            menu_items.push(menu_item("Open in new tab".to_string(), Some(ContextMenuAction::OpenInNewTab(state.paths[0].clone())), true));
        }
        menu_items.push(menu_item("Copy".to_string(), Some(ContextMenuAction::Copy(state.paths.clone())), true));
        
        if !state.is_sidebar {
            if state.paths.len() == 1 {
                menu_items.push(menu_item("Rename".to_string(), Some(ContextMenuAction::Rename(state.paths[0].clone())), true));
            }
            menu_items.push(menu_item("Move to trash".to_string(), Some(ContextMenuAction::MoveToTrash(state.paths.clone())), true));
            menu_items.push(menu_item("Zip".to_string(), Some(ContextMenuAction::Zip(state.paths.clone())), true));
            
            if state.paths.len() == 1 && !state.target_is_dir && state.paths[0].extension().map_or(false, |ext| ext == "zip") {
                menu_items.push(menu_item("Unzip".to_string(), Some(ContextMenuAction::Unzip(state.paths[0].clone())), true));
            }
        }
    } else {
        menu_items.push(menu_item(
            "Create new folder".to_string(),
            Some(ContextMenuAction::CreateNewFolder),
            true,
        ));
    }

    if is_folder {
        let action = if clipboard_has_item {
            Some(ContextMenuAction::Paste)
        } else {
            None
        };
        menu_items.push(menu_item("Paste".to_string(), action, clipboard_has_item));
    }

    menu_items.push(menu_item("Refresh".to_string(), Some(ContextMenuAction::Refresh), true));

    if menu_items.is_empty() {
        return column![].into();
    }

    let menu = container(
        column(menu_items)
            .width(Length::Fixed(120.0))
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
    clipboard: &mut Vec<PathBuf>,
    current_path: PathBuf,
) -> Task<Option<ContextMenuEvent>> {
    match action {
        ContextMenuAction::OpenInNewTab(path) => {
            Task::done(Some(ContextMenuEvent::OpenInNewTab(path)))
        }
        ContextMenuAction::Copy(paths) => {
            *clipboard = paths;
            Task::done(Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Paste => {
            if !clipboard.is_empty() {
                let src_paths = clipboard.clone();
                let dest_dir = current_path;
                Task::perform(async move {
                    for src_path in src_paths {
                        if let Some(file_name) = src_path.file_name() {
                            let dest_path = dest_dir.join(file_name);
                            if src_path.is_dir() {
                                let _ = copy_dir_all(&src_path, &dest_path);
                            } else {
                                let _ = std::fs::copy(&src_path, &dest_path);
                            }
                        }
                    }
                }, |_| Some(ContextMenuEvent::Refresh))
            } else {
                Task::none()
            }
        }
        ContextMenuAction::MoveToTrash(paths) => {
            Task::perform(async move {
                for path in paths {
                    let _ = trash::delete(path);
                }
            }, |_| Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Rename(path) => {
            Task::done(Some(ContextMenuEvent::Rename(path)))
        }
        ContextMenuAction::Zip(paths) => {
            Task::perform(async move {
                crate::archive_utils::zip_items(&paths);
            }, |_| Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Unzip(path) => {
            Task::perform(async move {
                crate::archive_utils::unzip_item(&path);
            }, |_| Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::CreateNewFolder => {
            Task::perform(async move {
                let _ = create_new_folder(current_path);
            }, |_| Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Refresh => {
            Task::done(Some(ContextMenuEvent::Refresh))
        }
    }
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn create_new_folder(current_path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::create_dir(current_path.as_ref().join("New Folder"))
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
        let mut clipboard = Vec::new();
        let paths = vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")];
        let _ = handle_action(
            ContextMenuAction::Copy(paths.clone()),
            &mut clipboard,
            PathBuf::from("/tmp"),
        );
        assert_eq!(clipboard, paths);
    }
}

fn menu_item(label: String, action: Option<ContextMenuAction>, enabled: bool) -> Element<'static, ContextMenuMessage> {
    let mut btn = button(
        text(label)
            .size(13)
            .width(Length::Fill)
    )
    .padding(8)
    .width(Length::Fill);

    if let Some(act) = action {
        btn = btn.on_press(ContextMenuMessage::Action(act));
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
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
