use crate::grid_view::{DirectoryItem, Rect};
use crate::icons;
use chrono::{DateTime, Local};
use iced::widget::{column, container, image, mouse_area, row, scrollable, svg, text};
use iced::{Alignment, Color, Element, Length, Point};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const HEADER_HEIGHT: f32 = 32.0;
pub const ROW_HEIGHT: f32 = 42.0;
pub const PADDING: f32 = 10.0;
pub(crate) const LIST_SCROLLABLE_ID: &str = "list-view-scrollable";

#[derive(Debug, Clone)]
pub enum ListMessage {
    ItemPressed(PathBuf),
    ItemClicked(PathBuf, bool),
    ItemHovered(Option<PathBuf>),
    ItemRightClicked(PathBuf, bool),
    BackgroundDown,
    BackgroundUp,
    PointerMoved(Point),
    Scrolled(f32),
    BackgroundRightClicked,
}

pub fn item_rect(index: usize, window_width: f32, scroll_y: f32) -> Rect {
    Rect {
        x: PADDING,
        y: PADDING + HEADER_HEIGHT + index as f32 * ROW_HEIGHT - scroll_y,
        width: (window_width - 220.0).max(100.0),
        height: ROW_HEIGHT,
    }
}

pub fn directory_at_position(
    items: &[DirectoryItem],
    position: Point,
    window_width: f32,
    scroll_y: f32,
) -> Option<PathBuf> {
    items
        .iter()
        .enumerate()
        .find(|(index, item)| {
            item.is_dir && item_rect(*index, window_width, scroll_y).contains(position)
        })
        .map(|(_, item)| item.path.clone())
}

pub fn format_type(item: &DirectoryItem) -> String {
    if item.is_dir {
        "Folder".to_owned()
    } else {
        item.path
            .extension()
            .filter(|extension| !extension.is_empty())
            .map(|extension| format!("{} file", extension.to_string_lossy().to_uppercase()))
            .unwrap_or_else(|| "File".to_owned())
    }
}

pub fn format_date(created: Option<SystemTime>) -> String {
    created
        .map(|time| {
            let local: DateTime<Local> = time.into();
            local.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| "—".to_owned())
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} bytes")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn view(
    items: &[DirectoryItem],
    selected_items: &HashSet<PathBuf>,
    drop_target: Option<&Path>,
    hovered_item: Option<&Path>,
    cut_items: &[PathBuf],
) -> Element<'static, ListMessage> {
    let header = row![
        text("Name").width(Length::Fill),
        text("Type").width(Length::Fixed(140.0)),
        text("Create date").width(Length::Fixed(170.0)),
        text("Size").width(Length::Fixed(100.0)),
    ]
    .height(Length::Fixed(HEADER_HEIGHT))
    .padding([6, 10])
    .align_y(Alignment::Center);

    let mut rows = column![header];
    for item in items {
        let is_selected = selected_items.contains(&item.path);
        let is_drop_target = drop_target == Some(item.path.as_path());
        let is_hovered = hovered_item == Some(item.path.as_path());
        let is_cut = cut_items.contains(&item.path);
        let opacity = if item.is_hidden || is_cut { 0.5 } else { 1.0 };
        let icon: Element<'static, ListMessage> = if item.is_dir {
            svg(svg::Handle::from_memory(icons::FOLDER_SVG))
                .width(24)
                .height(24)
                .opacity(opacity)
                .into()
        } else if let Some(handle) = &item.app_icon {
            image(handle.clone())
                .width(24)
                .height(24)
                .opacity(opacity)
                .into()
        } else {
            svg(svg::Handle::from_memory(icons::FILE_SVG))
                .width(24)
                .height(24)
                .opacity(opacity)
                .into()
        };

        let text_style = move |theme: &iced::Theme| {
            let mut color = theme.extended_palette().background.strong.text;
            color.a *= opacity;
            iced::widget::text::Style { color: Some(color) }
        };
        let size = if item.is_dir {
            String::new()
        } else {
            format_size(item.size)
        };
        let row_content = row![
            row![icon, text(item.name.clone()).size(13).style(text_style)]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            text(format_type(item))
                .size(13)
                .style(text_style)
                .width(Length::Fixed(140.0)),
            text(format_date(item.created))
                .size(13)
                .style(text_style)
                .width(Length::Fixed(170.0)),
            text(size)
                .size(13)
                .style(text_style)
                .width(Length::Fixed(100.0)),
        ]
        .height(Length::Fixed(ROW_HEIGHT))
        .padding([6, 10])
        .align_y(Alignment::Center);

        let row_card =
            container(row_content)
                .width(Length::Fill)
                .style(move |theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    let background = if is_drop_target {
                        Some(palette.success.weak.color.into())
                    } else if is_selected {
                        Some(palette.primary.weak.color.into())
                    } else if is_hovered {
                        Some(palette.background.weak.color.into())
                    } else {
                        None
                    };
                    container::Style {
                        background,
                        border: iced::Border {
                            color: if is_drop_target {
                                palette.success.strong.color
                            } else if is_selected {
                                palette.primary.strong.color
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if is_drop_target {
                                2.0
                            } else if is_selected {
                                1.0
                            } else {
                                0.0
                            },
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    }
                });

        let path = item.path.clone();
        let hover_path = path.clone();
        let is_dir = item.is_dir;
        rows = rows.push(
            mouse_area(row_card)
                .on_press(ListMessage::ItemPressed(path.clone()))
                .on_release(ListMessage::ItemClicked(path.clone(), is_dir))
                .on_enter(ListMessage::ItemHovered(Some(hover_path)))
                .on_exit(ListMessage::ItemHovered(None))
                .on_right_press(ListMessage::ItemRightClicked(path, is_dir))
                .interaction(iced::mouse::Interaction::Pointer),
        );
    }

    let content = scrollable(rows)
        .id(LIST_SCROLLABLE_ID)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(|viewport| ListMessage::Scrolled(viewport.absolute_offset().y));

    mouse_area(
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(PADDING),
    )
    .on_press(ListMessage::BackgroundDown)
    .on_release(ListMessage::BackgroundUp)
    .on_move(ListMessage::PointerMoved)
    .on_right_press(ListMessage::BackgroundRightClicked)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn item(name: &str, is_dir: bool, size: u64) -> DirectoryItem {
        DirectoryItem {
            path: PathBuf::from(name),
            name: name.to_owned(),
            is_dir,
            is_hidden: false,
            size,
            created: Some(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            modified: None,
            app_icon: None,
        }
    }

    #[test]
    fn formats_folder_and_file_details() {
        assert_eq!(format_type(&item("folder", true, 0)), "Folder");
        assert_eq!(format_type(&item("photo.png", false, 1536)), "PNG file");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn list_position_only_targets_directories() {
        let items = vec![item("folder", true, 0), item("file.txt", false, 1)];
        assert_eq!(
            directory_at_position(&items, Point::new(20.0, 50.0), 800.0, 0.0),
            Some(PathBuf::from("folder"))
        );
        assert_eq!(
            directory_at_position(&items, Point::new(20.0, 92.0), 800.0, 0.0),
            None
        );
    }

    #[test]
    fn list_scrollable_uses_stable_identity() {
        assert_eq!(LIST_SCROLLABLE_ID, "list-view-scrollable");
    }
}
