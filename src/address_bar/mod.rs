use iced::widget::{row, text, text_input, button};
use iced::{Element, Length, Color, Border};

#[derive(Debug, Clone)]
pub enum AddressBarMessage {
    InputChanged(String),
    Submit,
}

pub fn view(input_value: &str, is_invalid: bool) -> Element<'static, AddressBarMessage> {
    let mut address_input = text_input("Enter path here...", input_value)
        .on_input(AddressBarMessage::InputChanged)
        .on_submit(AddressBarMessage::Submit)
        .padding(8)
        .width(Length::Fill);

    if is_invalid {
        address_input = address_input.style(|theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: Border {
                    color: Color::from_rgb(0.9, 0.2, 0.2),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                icon: palette.background.strong.color,
                placeholder: palette.background.strong.color,
                value: palette.background.strong.text,
                selection: palette.primary.weak.color,
            }
        });
    } else {
        address_input = address_input.style(|theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                icon: palette.background.strong.color,
                placeholder: palette.background.strong.color,
                value: palette.background.strong.text,
                selection: palette.primary.weak.color,
            }
        });
    }

    let mut content = row![].spacing(8).align_y(iced::Alignment::Center);

    if is_invalid {
        content = content.push(
            text("⚠️")
                .size(16)
                .color(Color::from_rgb(0.9, 0.2, 0.2))
        );
    }

    content = content
        .push(address_input)
        .push(
            button("Go")
                .padding(8)
                .on_press(AddressBarMessage::Submit)
        );

    content.into()
}
