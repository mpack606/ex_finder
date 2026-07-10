use iced::{Element, Length, Alignment, Border};
use iced::widget::{button, row, text, svg};
use std::path::{PathBuf};
use crate::icons;
use crate::navigation;

#[derive(Debug, Clone)]
pub enum TabsMessage {
    SelectTab(usize),
    CloseTab(usize),
    OpenTab(PathBuf),
}

pub enum TabsEvent {
    NavigationChanged,
}

pub struct TabsState {
    pub list: Vec<navigation::NavigationState>,
    pub active_index: usize,
}

impl TabsState {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            list: vec![navigation::NavigationState::new(initial_path)],
            active_index: 0,
        }
    }

    pub fn update(&mut self, message: TabsMessage) -> Option<TabsEvent> {
        match message {
            TabsMessage::SelectTab(index) => {
                if index < self.list.len() {
                    self.active_index = index;
                    Some(TabsEvent::NavigationChanged)
                } else {
                    None
                }
            }
            TabsMessage::CloseTab(index) => {
                if self.list.len() > 1 && index < self.list.len() {
                    self.list.remove(index);
                    if index < self.active_index {
                        self.active_index -= 1;
                    }
                    if self.active_index >= self.list.len() {
                        self.active_index = self.list.len() - 1;
                    }
                    Some(TabsEvent::NavigationChanged)
                } else {
                    None
                }
            }
            TabsMessage::OpenTab(path) => {
                self.list.push(navigation::NavigationState::new(path));
                self.active_index = self.list.len() - 1;
                Some(TabsEvent::NavigationChanged)
            }
        }
    }

    pub fn active_tab_mut(&mut self) -> &mut navigation::NavigationState {
        &mut self.list[self.active_index]
    }

    pub fn active_path(&self) -> &PathBuf {
        &self.list[self.active_index].current_path
    }
}

pub fn view(
    state: &TabsState,
) -> Element<'static, TabsMessage> {
    let mut tab_elements = Vec::new();

    for (i, nav) in state.list.iter().enumerate() {
        let path = &nav.current_path;
        let is_active = i == state.active_index;
        let folder_name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        let close_btn = button(
            svg(svg::Handle::from_memory(icons::CLOSE_SVG))
                .width(10)
                .height(10)
        )
        .padding(4)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;
            
            button::Style {
                background: if is_hovered {
                    Some(palette.background.weak.color.into())
                } else {
                    None
                },
                text_color: palette.background.strong.text,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .on_press(TabsMessage::CloseTab(i));

        let tab_btn = button(
            row![
                text(folder_name).size(13).width(Length::Fill),
                close_btn
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .padding(8)
        .width(Length::Fill)
        .style(move |theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = status == button::Status::Hovered;
            
            let bg = if is_active {
                Some(palette.primary.weak.color.into())
            } else if is_hovered {
                Some(palette.background.weak.color.into())
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color: if is_active {
                    palette.primary.weak.text
                } else {
                    palette.background.strong.text
                },
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .on_press(TabsMessage::SelectTab(i));

        tab_elements.push(tab_btn.into());
    }

    row(tab_elements)
        .spacing(4)
        .padding(4)
        .width(Length::Fill)
        .into()
}
