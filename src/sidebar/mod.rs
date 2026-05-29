use iced::widget::{button, column, text, container, row};
use iced::{Element, Length};
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
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
    ]
    .spacing(10);

    for path in quick_access_paths {
        let display_name = if Some(path.as_path()) == dirs::home_dir().as_deref() {
            "Home".to_string()
        } else {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned())
        };

        let path_clone = path.clone();
        let btn = button(
            row![
                text("📁 ").size(14),
                text(display_name).size(14)
            ].spacing(6)
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(SidebarMessage::SelectPath(path_clone));

        let remove_btn = button(text("✕").size(10))
            .padding(6)
            .on_press(SidebarMessage::RemovePath(path.clone()));

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
                text("➕ ").size(12),
                text("Pin Current").size(12)
            ].spacing(4)
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(SidebarMessage::AddCurrentPath(current_path.to_path_buf()));
        
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
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
