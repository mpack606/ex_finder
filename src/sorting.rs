use crate::grid_view::DirectoryItem;
use iced::widget::{pick_list, text};
use iced::{Border, Element, Length};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Default,
    Name,
    Created,
    Edited,
    Size,
}

impl SortOrder {
    const ALL: [Self; 5] = [
        Self::Default,
        Self::Name,
        Self::Created,
        Self::Edited,
        Self::Size,
    ];
}

impl fmt::Display for SortOrder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Default => "Default",
            Self::Name => "Name",
            Self::Created => "Create date",
            Self::Edited => "Edit date",
            Self::Size => "Size",
        })
    }
}

#[derive(Debug, Clone)]
pub enum SortingMessage {
    Selected(SortOrder),
}

pub fn view(selected: SortOrder) -> Element<'static, SortingMessage> {
    let selector = pick_list(SortOrder::ALL, Some(selected), SortingMessage::Selected)
        .width(Length::Fixed(120.0))
        .padding(8)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let active = iced::widget::pick_list::Style {
                text_color: palette.background.weak.text,
                placeholder_color: palette.background.weak.text,
                handle_color: palette.background.weak.text,
                background: palette.background.weak.color.into(),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
            };

            match status {
                iced::widget::pick_list::Status::Active => active,
                iced::widget::pick_list::Status::Hovered
                | iced::widget::pick_list::Status::Opened { .. } => {
                    iced::widget::pick_list::Style {
                        border: Border {
                            color: palette.primary.base.color,
                            ..active.border
                        },
                        ..active
                    }
                }
            }
        });

    iced::widget::row![text("Sort:"), selector]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
}

pub fn sort_items(items: &mut [DirectoryItem], order: SortOrder) {
    items.sort_by(|a, b| compare_items(a, b, order));
}

fn compare_items(a: &DirectoryItem, b: &DirectoryItem, order: SortOrder) -> Ordering {
    let by_name = || {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    };

    match order {
        SortOrder::Default => match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => by_name(),
        },
        SortOrder::Name => by_name(),
        SortOrder::Created => newest_first(a.created, b.created).then_with(by_name),
        SortOrder::Edited => newest_first(a.modified, b.modified).then_with(by_name),
        SortOrder::Size => b.size.cmp(&a.size).then_with(by_name),
    }
}

fn newest_first(a: Option<std::time::SystemTime>, b: Option<std::time::SystemTime>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => b.cmp(&a),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    fn item(name: &str, is_dir: bool, seconds: u64, size: u64) -> DirectoryItem {
        DirectoryItem {
            path: PathBuf::from(name),
            name: name.into(),
            is_dir,
            is_hidden: false,
            size,
            created: Some(UNIX_EPOCH + Duration::from_secs(seconds)),
            modified: Some(UNIX_EPOCH + Duration::from_secs(seconds)),
            app_icon: None,
        }
    }

    fn names(items: &[DirectoryItem]) -> Vec<&str> {
        items.iter().map(|item| item.name.as_str()).collect()
    }

    #[test]
    fn default_keeps_folders_first_and_sorts_each_group_by_name() {
        let mut items = vec![
            item("a-file", false, 1, 1),
            item("z-folder", true, 1, 1),
            item("a-folder", true, 1, 1),
            item("z-file", false, 1, 1),
        ];

        sort_items(&mut items, SortOrder::Default);

        assert_eq!(names(&items), ["a-folder", "z-folder", "a-file", "z-file"]);
    }

    #[test]
    fn name_sort_does_not_separate_folders_and_files() {
        let mut items = vec![
            item("a-file", false, 1, 1),
            item("z-folder", true, 1, 1),
            item("b-folder", true, 1, 1),
        ];

        sort_items(&mut items, SortOrder::Name);

        assert_eq!(names(&items), ["a-file", "b-folder", "z-folder"]);
    }

    #[test]
    fn dates_and_size_sort_largest_values_first() {
        let mut items = vec![
            item("old-small", false, 1, 10),
            item("new-medium", false, 3, 20),
            item("middle-large", false, 2, 30),
        ];

        sort_items(&mut items, SortOrder::Created);
        assert_eq!(names(&items), ["new-medium", "middle-large", "old-small"]);

        sort_items(&mut items, SortOrder::Edited);
        assert_eq!(names(&items), ["new-medium", "middle-large", "old-small"]);

        sort_items(&mut items, SortOrder::Size);
        assert_eq!(names(&items), ["middle-large", "new-medium", "old-small"]);
    }
}
