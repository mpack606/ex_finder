use iced::widget::{button, column, text, container, row, svg, scrollable, mouse_area, tooltip};
use iced::{Element, Length};
use crate::icons;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    SelectPath(PathBuf),
    AddCurrentPath(PathBuf),
    RemovePath(PathBuf),
    ItemRightClicked(PathBuf),
    ToggleRecentLocations,
}

pub fn view(
    quick_access_paths: &[PathBuf],
    recent_locations: &[PathBuf],
    recent_locations_expanded: bool,
    current_path: &Path,
) -> Element<'static, SidebarMessage> {
    let title = text("Quick Access")
        .size(16)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            family: iced::font::Family::Name("system-ui"),
            ..Default::default()
        });

    let mut list_col = column![].spacing(10);

    for path in quick_access_paths {
        let is_home = Some(path.as_path()) == dirs::home_dir().as_deref();
        let display_name = if is_home {
            "Home".to_string()
        } else {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned())
        };

        let icon = if is_home {
            svg(svg::Handle::from_memory(icons::HOME_SVG))
                .width(18)
                .height(18)
        } else {
            svg(svg::Handle::from_memory(icons::FOLDER_SVG))
                .width(18)
                .height(18)
        };

        let is_current = path == current_path;
        let path_clone = path.clone();
        let btn = button(
            row![
                icon,
                text(display_name).size(14)
            ].spacing(6).align_y(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(SidebarMessage::SelectPath(path_clone))
        .style(move |theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            
            let bg = if is_current {
                Some(palette.background.strong.color.into())
            } else if status == button::Status::Hovered {
                Some(palette.background.base.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: palette.background.strong.text,
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        let remove_btn = button(text("✕").size(10))
            .padding(6)
            .on_press(SidebarMessage::RemovePath(path.clone()))
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let bg = if status == button::Status::Hovered {
                    Some(palette.background.base.color.into())
                } else {
                    None
                };

                button::Style {
                    background: bg,
                    text_color: palette.background.strong.text,
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        let btn_with_right_click = mouse_area(btn)
            .on_right_press(SidebarMessage::ItemRightClicked(path.clone()));

        list_col = list_col.push(
            row![btn_with_right_click, remove_btn]
                .align_y(iced::Alignment::Center)
                .spacing(5)
        );
    }

    // Add current path button
    let is_already_bookmarked = quick_access_paths.contains(&current_path.to_path_buf());
    if !is_already_bookmarked {
        let add_btn = button(
            row![
                svg(svg::Handle::from_memory(icons::PIN_SVG))
                    .width(16)
                    .height(16),
                text("Pin Current").size(12)
            ].spacing(6).align_y(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(SidebarMessage::AddCurrentPath(current_path.to_path_buf()))
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let bg = if status == button::Status::Hovered {
                Some(palette.background.base.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: palette.background.strong.text,
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });
        
        list_col = list_col.push(add_btn);
    }

    let disclosure = if recent_locations_expanded { "▼" } else { "▶" };
    let recent_header = button(
        row![
            text(disclosure).size(11),
            text("Recent locations")
                .size(16)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    family: iced::font::Family::Name("system-ui"),
                    ..Default::default()
                })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .padding(4)
    .on_press(SidebarMessage::ToggleRecentLocations)
    .style(|theme: &iced::Theme, status| {
        let palette = theme.extended_palette();
        button::Style {
            background: (status == button::Status::Hovered)
                .then(|| palette.background.base.color.into()),
            text_color: palette.background.strong.text,
            border: iced::Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let mut sidebar_content = column![title, list_col, recent_header].spacing(10);

    if recent_locations_expanded {
        let mut recent_list = column![].spacing(5);
        for path in recent_locations {
            let is_current = path == current_path;
            let path_clone = path.clone();
            let full_path = path.to_string_lossy().into_owned();
            let display_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| full_path.clone());
            let recent_button = button(
                row![
                    svg(svg::Handle::from_memory(icons::FOLDER_SVG))
                        .width(16)
                        .height(16),
                    text(display_name).size(12)
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
            )
            .width(Length::Fill)
            .padding(8)
            .on_press(SidebarMessage::SelectPath(path_clone))
            .style(move |theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let background = if is_current {
                    Some(palette.background.strong.color.into())
                } else if status == button::Status::Hovered {
                    Some(palette.background.base.color.into())
                } else {
                    None
                };

                button::Style {
                    background,
                    text_color: palette.background.strong.text,
                    border: iced::Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });
            let recent_with_tooltip = tooltip(
                recent_button,
                text(full_path).size(12),
                tooltip::Position::Right,
            )
            .gap(6)
            .padding(8)
            .delay(Duration::from_millis(400))
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.strong.color.into()),
                    text_color: Some(palette.background.strong.text),
                    border: iced::Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });
            recent_list = recent_list.push(recent_with_tooltip);
        }
        sidebar_content = sidebar_content.push(recent_list);
    }

    let sidebar_layout = scrollable(sidebar_content).height(Length::Fill);

    container(sidebar_layout)
    .width(Length::Fixed(200.0))
    .height(Length::Fill)
    .padding(10)
        .style(move |theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: iced::border::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
