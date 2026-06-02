use iced::widget::{button, column, text, container, row, svg};
use iced::{Element, Length};
use crate::icons;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    SelectPath(PathBuf),
    AddCurrentPath(PathBuf),
    RemovePath(PathBuf),
}

pub fn view(quick_access_paths: &[PathBuf], current_path: &Path) -> Element<'static, SidebarMessage> {
    let mut sidebar_col = column![
        text("Quick Access")
            .size(16)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                family: iced::font::Family::Name("system-ui"),
                ..Default::default()
            })
    ]
    .spacing(10);

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

        sidebar_col = sidebar_col.push(
            row![btn, remove_btn]
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
        
        sidebar_col = sidebar_col.push(add_btn);
    }

    container(sidebar_col)
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
