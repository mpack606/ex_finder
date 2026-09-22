use crate::icons;
use iced::widget::{button, row, svg};
use iced::{Border, Element};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone)]
pub enum Message {
    Selected(ViewMode),
}

pub fn view(selected: ViewMode) -> Element<'static, Message> {
    row![
        mode_button(icons::GRID_VIEW_SVG, ViewMode::Grid, selected),
        mode_button(icons::LIST_VIEW_SVG, ViewMode::List, selected),
    ]
    .spacing(4)
    .into()
}

fn mode_button(
    icon: &'static [u8],
    mode: ViewMode,
    selected: ViewMode,
) -> button::Button<'static, Message> {
    let is_selected = mode == selected;
    button(svg(svg::Handle::from_memory(icon)).width(16).height(16))
        .padding(6)
        .on_press(Message::Selected(mode))
        .style(move |theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;
            button::Style {
                background: if is_selected {
                    Some(palette.primary.weak.color.into())
                } else if is_hovered {
                    Some(palette.background.base.color.into())
                } else {
                    None
                },
                border: Border {
                    color: if is_selected || is_hovered {
                        palette.primary.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: if is_selected { 1.5 } else { 1.0 },
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
}
