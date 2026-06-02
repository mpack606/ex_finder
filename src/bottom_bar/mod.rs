use iced::widget::{container, text};
use iced::{Element, Length, Border};
use std::path::Path;

pub fn view<Message: 'static>(selected_item: Option<&Path>) -> Element<'static, Message> {
    let content = if let Some(selected) = selected_item {
        selected.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };

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
