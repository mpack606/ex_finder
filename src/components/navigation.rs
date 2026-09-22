use iced::widget::{button, row, svg};
use iced::{Border, Element};

use crate::icons;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Back,
    Forward,
    Up,
}

fn nav_button(svg_data: &'static [u8], message: Message) -> button::Button<'static, Message> {
    button(svg(svg::Handle::from_memory(svg_data)).width(16).height(16))
        .padding(6)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;

            button::Style {
                background: is_hovered.then_some(palette.background.base.color.into()),
                border: Border {
                    color: if is_hovered {
                        palette.primary.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .on_press(message)
}

pub fn view_controls() -> Element<'static, Message> {
    row![
        nav_button(icons::BACK_SVG, Message::Back),
        nav_button(icons::FORWARD_SVG, Message::Forward),
        nav_button(icons::UP_SVG, Message::Up),
    ]
    .spacing(6)
    .into()
}
