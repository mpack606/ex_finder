use iced::widget::{button, column, row, scrollable, text, container, svg, mouse_area, stack, image};
use iced::{Element, Length, Color, Alignment, Font, font};
use crate::icons;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub app_icon: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum GridMessage {
    ItemClicked(PathBuf, bool),
    ItemRightClicked(PathBuf, bool),
    BackgroundClicked,
    BackgroundRightClicked,
}

pub(crate) const GRID_SCROLLABLE_ID: &str = "grid-view-scrollable";

pub fn read_directory(path: &Path) -> Result<Vec<DirectoryItem>, std::io::Error> {
    let mut items = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        let is_hidden = file_name.starts_with('.');

        let is_dir = entry_path.is_dir();
        items.push(DirectoryItem {
            path: entry_path,
            name: file_name,
            is_dir,
            is_hidden,
            app_icon: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_read_directory_includes_hidden() {
        let mut dir_path = std::env::temp_dir();
        dir_path.push(format!("ex_finder_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&dir_path).unwrap();

        File::create(dir_path.join("visible.txt")).unwrap();
        File::create(dir_path.join(".hidden.txt")).unwrap();
        fs::create_dir(dir_path.join(".hidden_dir")).unwrap();
        fs::create_dir(dir_path.join("visible_dir")).unwrap();

        let result = read_directory(&dir_path);
        
        // Cleanup before asserts to ensure it happens
        let _ = fs::remove_dir_all(&dir_path);

        let items = result.unwrap();
        assert_eq!(items.len(), 4);
        
        let hidden_file = items.iter().find(|i| i.name == ".hidden.txt").unwrap();
        assert!(hidden_file.is_hidden);
        assert!(!hidden_file.is_dir);

        let hidden_dir = items.iter().find(|i| i.name == ".hidden_dir").unwrap();
        assert!(hidden_dir.is_hidden);
        assert!(hidden_dir.is_dir);

        let visible_file = items.iter().find(|i| i.name == "visible.txt").unwrap();
        assert!(!visible_file.is_hidden);
        assert!(!visible_file.is_dir);
    }

    #[test]
    fn grid_scrollable_uses_stable_identity() {
        assert_eq!(GRID_SCROLLABLE_ID, "grid-view-scrollable");
    }
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
                let mut icon_stack = stack![
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
                ];

                if let Some(app_icon_path) = &item.app_icon {
                    icon_stack = icon_stack.push(
                        container(
                            image(app_icon_path.clone())
                                .width(16)
                                .height(16)
                        )
                        .width(48)
                        .height(48)
                        .align_x(Alignment::End)
                        .align_y(Alignment::End)
                    );
                }
                
                icon_stack.into()
            } else {
                svg(svg::Handle::from_memory(icons::FILE_SVG))
                    .width(48)
                    .height(48)
                    .into()
            };

            let is_hidden = item.is_hidden;
            let item_column = column![
                icon,
                text(display_name)
                    .size(12)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
            ]
            .align_x(Alignment::Center)
            .spacing(6);

            let button_content: Element<_> = if is_hidden {
                stack![
                    item_column,
                    container(column![])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(Color {
                                    a: 0.4,
                                    ..palette.background.base.color
                                }.into()),
                                ..Default::default()
                            }
                        })
                ].into()
            } else {
                item_column.into()
            };

            let item_btn = button(button_content)
            .width(Length::Fixed(100.0))
            .padding(10)
            .on_press(GridMessage::ItemClicked(path_clone.clone(), is_dir))
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

                let mut text_color = palette.background.strong.text;
                if is_hidden {
                    text_color.a = 0.5;
                }

                iced::widget::button::Style {
                    background: bg,
                    text_color,
                    border,
                    ..Default::default()
                }
            });

            let item_view = mouse_area(item_btn)
                .on_right_press(GridMessage::ItemRightClicked(path_clone, is_dir));

            grid_row = grid_row.push(item_view);
        }
        grid_col = grid_col.push(grid_row);
    }

    let scrollable_content = scrollable(grid_col)
        .id(GRID_SCROLLABLE_ID)
        .width(Length::Fill)
        .height(Length::Fill);

    mouse_area(
        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
    )
    .on_press(GridMessage::BackgroundClicked)
    .on_right_press(GridMessage::BackgroundRightClicked)
    .into()
}
