use iced::widget::{button, column, row, scrollable, text, container, svg, mouse_area, stack};
use iced::{Element, Length, Color, Alignment, Font, font};
use crate::icons;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub enum GridMessage {
    ItemClicked(PathBuf, bool),
    BackgroundClicked,
}

pub fn read_directory(path: &Path) -> Result<Vec<DirectoryItem>, std::io::Error> {
    let mut items = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        if file_name.starts_with('.') {
            continue;
        }

        let is_dir = entry_path.is_dir();
        items.push(DirectoryItem {
            path: entry_path,
            name: file_name,
            is_dir,
        });
    }

    items.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(items)
}

pub fn view(
    items: &[DirectoryItem],
    selected_item: Option<&PathBuf>,
    window_width: f32,
) -> Element<'static, GridMessage> {
    let sidebar_width = 200.0;
    let grid_padding = 32.0;
    let available_width = (window_width - sidebar_width - grid_padding).max(100.0);

    let cell_width = 110.0;
    let columns = ((available_width / cell_width) as usize).max(1);

    let mut grid_col = column![].spacing(16);

    for chunk in items.chunks(columns) {
        let mut grid_row = row![].spacing(16);
        for item in chunk {
            let is_selected = selected_item == Some(&item.path);
            let path_clone = item.path.clone();
            let is_dir = item.is_dir;

            let display_name = if item.name.len() > 12 {
                format!("{}...", &item.name[0..9])
            } else {
                item.name.clone()
            };

            let icon: Element<_> = if item.is_dir {
                svg(svg::Handle::from_memory(icons::FOLDER_SVG))
                    .width(48)
                    .height(48)
                    .into()
            } else if let Some(ext) = item.path.extension().and_then(|e| e.to_str()) {
                let ext_str = ext.to_uppercase();
                stack![
                    svg(svg::Handle::from_memory(icons::FILE_SVG))
                        .width(48)
                        .height(48),
                    container(
                        text(ext_str)
                            .size(10)
                            .font(Font {
                                weight: font::Weight::Bold,
                                family: font::Family::Name("system-ui"),
                                ..Default::default()
                            })
                            .color(Color::WHITE)
                    )
                    .width(48)
                    .height(48)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .padding(iced::Padding {
                        top: 12.0,
                        ..Default::default()
                    })
                ]
                .into()
            } else {
                svg(svg::Handle::from_memory(icons::FILE_SVG))
                    .width(48)
                    .height(48)
                    .into()
            };

            let item_btn = button(
                column![
                    icon,
                    text(display_name)
                        .size(12)
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                ]
                .align_x(Alignment::Center)
                .spacing(6)
            )
            .width(Length::Fixed(100.0))
            .padding(10)
            .on_press(GridMessage::ItemClicked(path_clone, is_dir))
            .style(move |theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let bg = if is_selected {
                    Some(palette.primary.weak.color.into())
                } else if status == iced::widget::button::Status::Hovered {
                    Some(palette.background.weak.color.into())
                } else {
                    None
                };

                let border = if is_selected {
                    iced::Border {
                        color: palette.primary.strong.color,
                        width: 1.5,
                        radius: 12.0.into(),
                    }
                } else {
                    iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 12.0.into(),
                    }
                };

                iced::widget::button::Style {
                    background: bg,
                    text_color: palette.background.strong.text,
                    border,
                    ..Default::default()
                }
            });

            grid_row = grid_row.push(item_btn);
        }
        grid_col = grid_col.push(grid_row);
    }

    let scrollable_content = scrollable(grid_col)
        .width(Length::Fill)
        .height(Length::Fill);

    mouse_area(
        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
    )
    .on_press(GridMessage::BackgroundClicked)
    .into()
}
