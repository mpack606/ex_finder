use iced::widget::{button, column, container, mouse_area, row, text, text_input};
use iced::{Alignment, Border, Element, Length};

use crate::app::Message;

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