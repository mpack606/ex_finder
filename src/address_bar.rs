use iced::widget::{button, container, mouse_area, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length};
use std::path::{Path, PathBuf};

pub const ADDRESS_INPUT_ID: &str = "address-input";

#[derive(Debug, Clone)]
pub enum AddressBarMessage {
    Edit,
    InputChanged(String),
    Navigate(PathBuf),
    Submit,
}

fn breadcrumbs(path: &Path) -> Vec<(String, PathBuf)> {
    let mut current = PathBuf::new();

    path.components()
        .map(|component| {
            current.push(component.as_os_str());
            let label = match component {
                std::path::Component::RootDir => String::from("/"),
                _ => component.as_os_str().to_string_lossy().into_owned(),
            };
            (label, current.clone())
        })
        .collect()
}

fn breadcrumb_view(path: &Path) -> Element<'static, AddressBarMessage> {
    let parts = breadcrumbs(path);
    let last_index = parts.len().saturating_sub(1);
    let mut content = row![].spacing(2).align_y(Alignment::Center);

    for (index, (label, target)) in parts.into_iter().enumerate() {
        if index > 0 {
            content = content.push(text("›").size(18).style(|theme: &iced::Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().background.strong.color),
                }
            }));
        }

        let is_current = index == last_index;
        let segment =
            container(text(label).size(14))
                .padding([5, 8])
                .style(move |theme: &iced::Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        background: is_current.then_some(palette.background.strong.color.into()),
                        text_color: Some(palette.background.strong.text),
                        border: Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

        content = content.push(
            mouse_area(segment)
                .on_press(AddressBarMessage::Navigate(target))
                .on_double_click(AddressBarMessage::Edit)
                .interaction(iced::mouse::Interaction::Pointer),
        );
    }

    let trail = scrollable(content)
        .horizontal()
        .anchor_right()
        .width(Length::Fill);

    let bar = container(trail)
        .padding([2, 4])
        .width(Length::Fill)
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

    mouse_area(bar)
        .on_double_click(AddressBarMessage::Edit)
        .into()
}

pub fn view(
    current_path: &Path,
    input_value: &str,
    is_invalid: bool,
    is_editing: bool,
) -> Element<'static, AddressBarMessage> {
    if !is_editing {
        return breadcrumb_view(current_path);
    }

    let address_input = text_input("Enter path here...", input_value)
        .id(ADDRESS_INPUT_ID)
        .on_input(AddressBarMessage::InputChanged)
        .on_submit(AddressBarMessage::Submit)
        .padding(8)
        .width(Length::Fill)
        .style(move |theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::text_input::Style {
                background: palette.background.base.color.into(),
                border: Border {
                    color: if is_invalid {
                        palette.danger.base.color
                    } else {
                        palette.background.strong.color
                    },
                    width: if is_invalid { 2.0 } else { 1.0 },
                    radius: 12.0.into(),
                },
                icon: palette.background.strong.color,
                placeholder: palette.background.strong.color,
                value: palette.background.strong.text,
                selection: palette.primary.weak.color,
            }
        });

    let mut content = row![].spacing(8).align_y(Alignment::Center);

    if is_invalid {
        content = content.push(text("⚠").size(16).style(|theme: &iced::Theme| {
            iced::widget::text::Style {
                color: Some(theme.extended_palette().danger.base.color),
            }
        }));
    }

    content
        .push(address_input)
        .push(
            button("Go")
                .padding(8)
                .on_press(AddressBarMessage::Submit)
                .style(action_button_style),
        )
        .into()
}

fn action_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let is_hovered = status == button::Status::Hovered;

    button::Style {
        background: Some(if is_hovered {
            palette.background.base.color.into()
        } else {
            palette.background.strong.color.into()
        }),
        text_color: palette.background.strong.text,
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
}

#[cfg(test)]
mod tests {
    use super::breadcrumbs;
    use std::path::{Path, PathBuf};

    #[test]
    fn builds_a_target_for_each_absolute_path_segment() {
        assert_eq!(
            breadcrumbs(Path::new("/path/to/folder/foo/bar")),
            vec![
                (String::from("/"), PathBuf::from("/")),
                (String::from("path"), PathBuf::from("/path")),
                (String::from("to"), PathBuf::from("/path/to")),
                (String::from("folder"), PathBuf::from("/path/to/folder")),
                (String::from("foo"), PathBuf::from("/path/to/folder/foo")),
                (
                    String::from("bar"),
                    PathBuf::from("/path/to/folder/foo/bar")
                ),
            ]
        );
    }

    #[test]
    fn root_is_a_clickable_breadcrumb() {
        assert_eq!(
            breadcrumbs(Path::new("/")),
            vec![(String::from("/"), PathBuf::from("/"))]
        );
    }
}
