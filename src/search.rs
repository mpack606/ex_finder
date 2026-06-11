use iced::widget::{button, container, stack, svg, text_input};
use iced::{Alignment, Border, Element, Length, Padding};
use crate::icons;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    InputChanged(String),
    Clear,
}

pub fn view(query: &str) -> Element<'_, SearchMessage> {
    let search_input = text_input("Search...", query)
        .on_input(SearchMessage::InputChanged)
        .padding(Padding {
            top: 8.0,
            right: if query.is_empty() { 8.0 } else { 30.0 },
            bottom: 8.0,
            left: 8.0,
        })
        .width(Length::Fixed(200.0))
        .style(|theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                icon: palette.background.strong.color,
                placeholder: palette.background.strong.color,
                value: palette.background.strong.text,
                selection: palette.primary.weak.color,
            }
        });

    if query.is_empty() {
        Element::from(search_input)
    } else {
        let clear_button = button(
            svg(svg::Handle::from_memory(icons::CLOSE_SVG))
                .width(10)
                .height(10)
        )
        .padding(4)
        .on_press(SearchMessage::Clear)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;
            
            button::Style {
                background: if is_hovered {
                    Some(palette.background.weak.color.into())
                } else {
                    None
                },
                border: Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        stack![
            search_input,
            container(clear_button)
                .width(Length::Fixed(200.0))
                .align_x(Alignment::End)
                .align_y(Alignment::Center)
                .padding(Padding {
                    right: 6.0,
                    ..Padding::ZERO
                })
        ].into()
    }
}
