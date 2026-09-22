use crate::commands::{self, Command, CommandKind};
use iced::widget::{button, column, container, mouse_area, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Point};
use std::path::PathBuf;

pub use crate::commands::Command as ContextMenuAction;

#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    Action(ContextMenuAction),
    Close,
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
    current_path: &std::path::Path,
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

            if state.paths.len() == 1
                && !state.target_is_dir
                && state.paths[0].extension().is_some_and(|ext| ext == "zip")
            {
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
            Some(Command::Paste(paste_destination(state, current_path)))
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

    let menu = container(column(menu_items).width(Length::Fixed(220.0)))
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
            }),
    )
    .on_press(ContextMenuMessage::Close)
    .on_right_press(ContextMenuMessage::Close)
    .into()
}

fn paste_destination(state: &ContextMenuState, current_path: &std::path::Path) -> PathBuf {
    if state.paths.len() == 1 && state.target_is_dir {
        state.paths[0].clone()
    } else {
        current_path.to_path_buf()
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn paste_destination_uses_the_context_folder() {
        let state = ContextMenuState {
            position: Point::ORIGIN,
            paths: vec![PathBuf::from("/tmp/target")],
            target_is_dir: true,
            is_sidebar: false,
        };

        assert_eq!(
            paste_destination(&state, std::path::Path::new("/tmp/current")),
            PathBuf::from("/tmp/target")
        );
    }

    #[test]
    fn paste_destination_uses_current_path_for_the_background() {
        let state = ContextMenuState {
            position: Point::ORIGIN,
            paths: Vec::new(),
            target_is_dir: true,
            is_sidebar: false,
        };

        assert_eq!(
            paste_destination(&state, std::path::Path::new("/tmp/current")),
            PathBuf::from("/tmp/current")
        );
    }
}

fn menu_item(
    label: &str,
    command_kind: CommandKind,
    action: Option<Command>,
    enabled: bool,
) -> Element<'static, ContextMenuMessage> {
    let command_kind = action.as_ref().map(Command::kind).unwrap_or(command_kind);
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

    let mut btn = button(content).padding(8).width(Length::Fill);

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
