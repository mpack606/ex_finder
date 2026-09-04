use iced::widget::{container, text};
use iced::{Element, Length, Border};
use std::path::PathBuf;

pub fn format_selected_content(selected_items: &[PathBuf]) -> String {
    match selected_items.len() {
        0 => String::new(),
        1 => selected_items[0]
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        count => format!("{} items selected", count),
    }
}

pub fn view<Message: 'static>(selected_items: &[PathBuf]) -> Element<'static, Message> {
    let content = format_selected_content(selected_items);

    container(
        text(content)
            .size(13)
    )
    .width(Length::Fill)
    .padding(8)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color,
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_selected_content_empty() {
        assert_eq!(format_selected_content(&[]), "");
    }

    #[test]
    fn test_format_selected_content_single() {
        let items = vec![PathBuf::from("/path/to/test.txt")];
        assert_eq!(format_selected_content(&items), "test.txt");
    }

    #[test]
    fn test_format_selected_content_multiple() {
        let items = vec![
            PathBuf::from("/path/to/test1.txt"),
            PathBuf::from("/path/to/test2.txt"),
            PathBuf::from("/path/to/test3.txt"),
        ];
        assert_eq!(format_selected_content(&items), "3 items selected");
    }
}
