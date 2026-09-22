use crate::directory::DirectoryItem;
use crate::icons;
use crate::layout::Rect;
use iced::widget::{column, container, image, mouse_area, row, scrollable, stack, svg, text};
use iced::{Alignment, Color, Element, Font, Length, Point, font};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const ITEM_WIDTH: f32 = 100.0;
pub const ITEM_HEIGHT: f32 = 88.0;
pub const SPACING: f32 = 16.0;
pub const PADDING: f32 = 10.0;

pub fn get_columns(window_width: f32) -> usize {
    let sidebar_width = 200.0;
    let grid_padding = 32.0;
    let available_width = (window_width - sidebar_width - grid_padding).max(100.0);

    let cell_width = 110.0;
    ((available_width / cell_width) as usize).max(1)
}

pub fn item_rect(index: usize, columns: usize, scroll_y: f32) -> Rect {
    let col = index % columns;
    let row = index / columns;
    Rect {
        x: PADDING + (col as f32) * (ITEM_WIDTH + SPACING),
        y: PADDING + (row as f32) * (ITEM_HEIGHT + SPACING) - scroll_y,
        width: ITEM_WIDTH,
        height: ITEM_HEIGHT,
    }
}

#[derive(Debug, Clone)]
pub enum GridMessage {
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

pub(crate) const GRID_SCROLLABLE_ID: &str = "grid-view-scrollable";

pub fn directory_at_position(
    items: &[DirectoryItem],
    position: Point,
    window_width: f32,
    scroll_y: f32,
) -> Option<PathBuf> {
    let columns = get_columns(window_width);
    items
        .iter()
        .enumerate()
        .find(|(index, item)| {
            item.is_dir && item_rect(*index, columns, scroll_y).contains(position)
        })
        .map(|(_, item)| item.path.clone())
}

fn display_name(name: &str) -> String {
    const MAX_CHARACTERS: usize = 12;
    const TRUNCATED_CHARACTERS: usize = 9;

    if name.chars().count() > MAX_CHARACTERS {
        format!(
            "{}...",
            name.chars().take(TRUNCATED_CHARACTERS).collect::<String>()
        )
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn display_name_truncates_unicode_on_character_boundaries() {
        assert_eq!(display_name("ééééééééééééé"), "ééééééééé...");
        assert_eq!(display_name("short-é"), "short-é");
    }

    #[test]
    fn grid_scrollable_uses_stable_identity() {
        assert_eq!(GRID_SCROLLABLE_ID, "grid-view-scrollable");
    }

    #[test]
    fn test_rect_from_points_and_intersects() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(50.0, 80.0);
        let rect = Rect::from_points(p1, p2);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 40.0);
        assert_eq!(rect.height, 60.0);

        // Reverse points
        let rect2 = Rect::from_points(p2, p1);
        assert_eq!(rect, rect2);

        let overlapping = Rect {
            x: 30.0,
            y: 40.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(rect.intersects(&overlapping));

        let non_overlapping = Rect {
            x: 100.0,
            y: 100.0,
            width: 10.0,
            height: 10.0,
        };
        assert!(!rect.intersects(&non_overlapping));
    }

    #[test]
    fn test_item_rect_calculation() {
        let columns = 3;
        let r0 = item_rect(0, columns, 0.0);
        assert_eq!(r0.x, PADDING);
        assert_eq!(r0.y, PADDING);
        assert_eq!(r0.width, ITEM_WIDTH);
        assert_eq!(r0.height, ITEM_HEIGHT);

        let r1 = item_rect(1, columns, 0.0);
        assert_eq!(r1.x, PADDING + ITEM_WIDTH + SPACING);
        assert_eq!(r1.y, PADDING);

        let r3 = item_rect(3, columns, 10.0);
        assert_eq!(r3.x, PADDING);
        assert_eq!(r3.y, PADDING + ITEM_HEIGHT + SPACING - 10.0);
    }

    #[test]
    fn directory_at_position_only_returns_folders() {
        let folder = DirectoryItem {
            path: PathBuf::from("/tmp/folder"),
            name: String::from("folder"),
            is_dir: true,
            is_hidden: false,
            size: 0,
            created: None,
            modified: None,
        };
        let file = DirectoryItem {
            path: PathBuf::from("/tmp/file.txt"),
            name: String::from("file.txt"),
            is_dir: false,
            is_hidden: false,
            size: 0,
            created: None,
            modified: None,
        };
        let items = vec![folder.clone(), file];
        let width = 500.0;

        assert_eq!(
            directory_at_position(&items, Point::new(20.0, 20.0), width, 0.0),
            Some(folder.path)
        );
        assert_eq!(
            directory_at_position(&items, Point::new(130.0, 20.0), width, 0.0),
            None
        );
    }
}

pub struct ViewOptions<'a> {
    pub selected_items: &'a HashSet<PathBuf>,
    pub app_icons: &'a HashMap<PathBuf, image::Handle>,
    pub window_width: f32,
    pub drag_rect: Option<Rect>,
    pub drop_target: Option<&'a Path>,
    pub hovered_item: Option<&'a Path>,
    pub cut_items: &'a [PathBuf],
}

pub fn view(items: &[DirectoryItem], options: ViewOptions<'_>) -> Element<'static, GridMessage> {
    let ViewOptions {
        selected_items,
        app_icons,
        window_width,
        drag_rect,
        drop_target,
        hovered_item,
        cut_items,
    } = options;
    let columns = get_columns(window_width);

    let mut grid_col = column![].spacing(SPACING);

    for chunk in items.chunks(columns) {
        let mut grid_row = row![].spacing(SPACING);
        for item in chunk {
            let is_selected = selected_items.contains(&item.path);
            let is_drop_target = drop_target == Some(item.path.as_path());
            let is_hovered = hovered_item == Some(item.path.as_path());
            let is_cut = cut_items.contains(&item.path);
            let content_opacity = if is_cut { 0.5 } else { 1.0 };
            let path_clone = item.path.clone();
            let is_dir = item.is_dir;

            let display_name = display_name(&item.name);

            let icon: Element<_> = if item.is_dir {
                svg(svg::Handle::from_memory(icons::FOLDER_SVG))
                    .width(48)
                    .height(48)
                    .opacity(content_opacity)
                    .into()
            } else if let Some(ext) = item.path.extension().and_then(|e| e.to_str()) {
                let ext_str = ext.to_uppercase();
                let mut icon_stack = stack![
                    svg(svg::Handle::from_memory(icons::FILE_SVG))
                        .width(48)
                        .height(48)
                        .opacity(content_opacity),
                    container(
                        text(ext_str)
                            .size(10)
                            .font(Font {
                                weight: font::Weight::Bold,
                                family: font::Family::Name("system-ui"),
                                ..Default::default()
                            })
                            .color(Color {
                                a: content_opacity,
                                ..Color::WHITE
                            })
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

                if let Some(app_icon_handle) = app_icons.get(&item.path) {
                    icon_stack = icon_stack.push(
                        container(
                            image(app_icon_handle.clone())
                                .width(16)
                                .height(16)
                                .opacity(content_opacity),
                        )
                        .width(48)
                        .height(48)
                        .align_x(Alignment::End)
                        .align_y(Alignment::End),
                    );
                }

                icon_stack.into()
            } else {
                svg(svg::Handle::from_memory(icons::FILE_SVG))
                    .width(48)
                    .height(48)
                    .opacity(content_opacity)
                    .into()
            };

            let is_hidden = item.is_hidden;
            let item_column = column![
                icon,
                text(display_name)
                    .size(12)
                    .style(move |theme: &iced::Theme| {
                        let mut color = theme.extended_palette().background.strong.text;
                        if is_cut {
                            color.a *= 0.5;
                        }
                        iced::widget::text::Style { color: Some(color) }
                    })
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
                                background: Some(
                                    Color {
                                        a: 0.4,
                                        ..palette.background.base.color
                                    }
                                    .into(),
                                ),
                                ..Default::default()
                            }
                        })
                ]
                .into()
            } else {
                item_column.into()
            };

            let item_card = container(button_content)
                .width(Length::Fixed(ITEM_WIDTH))
                .height(Length::Fixed(ITEM_HEIGHT))
                .padding(8)
                .style(move |theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    let bg = if is_drop_target {
                        Some(palette.success.weak.color.into())
                    } else if is_selected {
                        Some(palette.primary.weak.color.into())
                    } else if is_hovered {
                        Some(palette.background.weak.color.into())
                    } else {
                        None
                    };

                    let border = if is_drop_target {
                        iced::Border {
                            color: palette.success.strong.color,
                            width: 2.0,
                            radius: 12.0.into(),
                        }
                    } else if is_selected {
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

                    container::Style {
                        background: bg,
                        text_color: Some(text_color),
                        border,
                        ..Default::default()
                    }
                });

            let hover_path = path_clone.clone();
            let item_view = mouse_area(item_card)
                .on_press(GridMessage::ItemPressed(path_clone.clone()))
                .on_release(GridMessage::ItemClicked(path_clone.clone(), is_dir))
                .on_enter(GridMessage::ItemHovered(Some(hover_path)))
                .on_exit(GridMessage::ItemHovered(None))
                .interaction(iced::mouse::Interaction::Pointer)
                .on_right_press(GridMessage::ItemRightClicked(path_clone, is_dir));

            grid_row = grid_row.push(item_view);
        }
        grid_col = grid_col.push(grid_row);
    }

    let scrollable_content = scrollable(grid_col)
        .id(GRID_SCROLLABLE_ID)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(|vp| GridMessage::Scrolled(vp.absolute_offset().y));

    let mut main_stack = stack![
        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(PADDING)
    ];

    if let Some(rect) = drag_rect
        && (rect.width > 1.0 || rect.height > 1.0)
    {
        let selection_overlay = container(column![])
            .width(Length::Fixed(rect.width))
            .height(Length::Fixed(rect.height))
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(
                        Color {
                            a: 0.2,
                            ..palette.primary.base.color
                        }
                        .into(),
                    ),
                    border: iced::Border {
                        color: palette.primary.base.color,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                }
            });

        let selection_box = container(selection_overlay)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: rect.y.max(0.0),
                left: rect.x.max(0.0),
                ..Default::default()
            });

        main_stack = main_stack.push(selection_box);
    }

    mouse_area(main_stack)
        .on_press(GridMessage::BackgroundDown)
        .on_release(GridMessage::BackgroundUp)
        .on_move(GridMessage::PointerMoved)
        .on_right_press(GridMessage::BackgroundRightClicked)
        .into()
}
