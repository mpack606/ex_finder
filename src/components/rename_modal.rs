use iced::widget::{button, column, container, mouse_area, row, text, text_input};
use iced::{Alignment, Border, Element, Length};

use crate::app::Message;

pub const RENAME_INPUT_ID: &str = "rename-input";

pub fn focus_input(file_name: &str) -> iced::Task<Message> {
    let cursor_position = stem_cursor_position(file_name);

    iced::widget::operation::focus(RENAME_INPUT_ID).chain(iced::widget::operation::move_cursor_to(
        RENAME_INPUT_ID,
        cursor_position,
    ))
}

fn stem_cursor_position(file_name: &str) -> usize {
    let stem_end = file_name
        .rfind('.')
        .filter(|&index| index > 0)
        .unwrap_or(file_name.len());
    text_input::Value::new(&file_name[..stem_end]).len()
}

pub fn view<'a>(input_value: &'a str) -> Element<'a, Message> {
    let dialog = container(
        column![
            text("Rename")
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            text_input("New name", input_value)
                .id(RENAME_INPUT_ID)
                .on_input(Message::RenameInputChanged)
                .on_submit(Message::RenameSubmitted)
                .padding(10)
                .size(14),
            row![
                button(text("Cancel").align_x(Alignment::Center))
                    .on_press(Message::CancelRename)
                    .padding(8)
                    .width(Length::Fill)
                    .style(|theme: &iced::Theme, _status| {
                        let palette = theme.extended_palette();
                        button::Style {
                            background: Some(palette.background.weak.color.into()),
                            text_color: palette.background.strong.text,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                button(text("Rename").align_x(Alignment::Center))
                    .on_press(Message::RenameSubmitted)
                    .padding(8)
                    .width(Length::Fill)
                    .style(|theme: &iced::Theme, _status| {
                        let palette = theme.extended_palette();
                        button::Style {
                            background: Some(palette.primary.base.color.into()),
                            text_color: palette.primary.base.text,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(12)
        ]
        .spacing(16)
        .padding(20)
        .width(Length::Fixed(300.0))
    )
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.base.color.into()),
            border: Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    });

    mouse_area(
        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(iced::Color {
                    a: 0.5,
                    ..theme.extended_palette().background.base.color
                }
                .into()),
                ..Default::default()
            }),
    )
    .on_press(Message::CancelRename)
    .into()
}

#[cfg(test)]
mod tests {
    use super::stem_cursor_position;

    #[test]
    fn rename_cursor_is_placed_before_the_final_extension() {
        assert_eq!(stem_cursor_position("report.txt"), 6);
        assert_eq!(stem_cursor_position("archive.tar.gz"), 11);
        assert_eq!(stem_cursor_position("README"), 6);
        assert_eq!(stem_cursor_position(".gitignore"), 10);
    }

    #[test]
    fn rename_cursor_position_counts_unicode_graphemes() {
        assert_eq!(stem_cursor_position("re\u{301}sume\u{301}.pdf"), 6);
    }
}
