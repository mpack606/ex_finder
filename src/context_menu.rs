use iced::{Element, Length, Border, Point, Padding, Task};
use iced::widget::{button, column, container, mouse_area, text};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    OpenInNewTab(PathBuf),
    Copy(PathBuf),
    Paste,
    Rename(PathBuf),
    MoveToTrash(PathBuf),
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
    pub path: Option<(PathBuf, bool)>,
    pub is_sidebar: bool,
}

pub fn view(
    state: &ContextMenuState,
    clipboard_has_item: bool,
) -> Element<'static, ContextMenuMessage> {
    let mut menu_items = Vec::new();
    let mut is_folder = true;

    if let Some((path, is_dir)) = &state.path {
        is_folder = *is_dir;
        if is_folder {
            menu_items.push(menu_item("Open in new tab".to_string(), Some(ContextMenuAction::OpenInNewTab(path.clone())), true));
        }
        menu_items.push(menu_item("Copy".to_string(), Some(ContextMenuAction::Copy(path.clone())), true));
        
        if !state.is_sidebar {
            menu_items.push(menu_item("Rename".to_string(), Some(ContextMenuAction::Rename(path.clone())), true));
            menu_items.push(menu_item("Move to trash".to_string(), Some(ContextMenuAction::MoveToTrash(path.clone())), true));
        }
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
    clipboard: &mut Option<PathBuf>,
    current_path: PathBuf,
) -> Task<Option<ContextMenuEvent>> {
    match action {
        ContextMenuAction::OpenInNewTab(path) => {
            Task::done(Some(ContextMenuEvent::OpenInNewTab(path)))
        }
        ContextMenuAction::Copy(path) => {
            *clipboard = Some(path);
            Task::done(Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Paste => {
            if let Some(src_path) = clipboard.clone() {
                let dest_dir = current_path;
                Task::perform(async move {
                    let file_name = src_path.file_name().unwrap();
                    let dest_path = dest_dir.join(file_name);
                    
                    if src_path.is_dir() {
                        let _ = copy_dir_all(&src_path, &dest_path);
                    } else {
                        let _ = std::fs::copy(&src_path, &dest_path);
                    }
                }, |_| Some(ContextMenuEvent::Refresh))
            } else {
                Task::none()
            }
        }
        ContextMenuAction::MoveToTrash(path) => {
            Task::perform(async move {
                let _ = trash::delete(path);
            }, |_| Some(ContextMenuEvent::Refresh))
        }
        ContextMenuAction::Rename(path) => {
            Task::done(Some(ContextMenuEvent::Rename(path)))
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
