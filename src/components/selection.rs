use std::collections::HashSet;
use std::path::PathBuf;

use iced::{Point, keyboard::Modifiers};

use crate::directory::DirectoryItem;
use crate::grid_view;
use crate::layout::Rect;
use crate::{list_view, view_mode::ViewMode};

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected: HashSet<PathBuf>,
    pub last_selected: Option<PathBuf>,
    pub drag: Option<DragState>,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub start: Point,
    pub current: Point,
    pub initial_selection: HashSet<PathBuf>,
    pub is_dragging: bool,
}

impl SelectionState {
    pub fn clear(&mut self) {
        self.selected.clear();
        self.last_selected = None;
        self.drag = None;
    }

    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

    pub fn select_single(&mut self, path: PathBuf) {
        self.selected.clear();
        self.selected.insert(path.clone());
        self.last_selected = Some(path);
    }

    pub fn select_paths(&mut self, paths: Vec<PathBuf>) {
        self.clear();
        self.selected.extend(paths);
        self.last_selected = self.selected.iter().next().cloned();
    }

    pub fn toggle_cmd(&mut self, path: PathBuf) {
        if self.selected.contains(&path) {
            self.selected.remove(&path);
            if self.last_selected.as_ref() == Some(&path) {
                self.last_selected = self.selected.iter().next().cloned();
            }
        } else {
            self.selected.insert(path.clone());
            self.last_selected = Some(path);
        }
    }

    pub fn select_range(&mut self, path: PathBuf, items: &[DirectoryItem]) {
        if let Some(target_idx) = items.iter().position(|item| item.path == path) {
            let start_idx = self
                .last_selected
                .as_ref()
                .and_then(|anchor| items.iter().position(|item| item.path == *anchor))
                .unwrap_or(0);

            let range = start_idx.min(target_idx)..=start_idx.max(target_idx);
            self.selected.clear();
            for index in range {
                self.selected.insert(items[index].path.clone());
            }

            if self.last_selected.is_none() {
                self.last_selected = items.first().map(|item| item.path.clone());
            }
        }
    }

    pub fn begin_drag(&mut self, position: Point) {
        self.drag = Some(DragState {
            start: position,
            current: position,
            initial_selection: self.selected.clone(),
            is_dragging: false,
        });
    }

    pub fn finish_drag(&mut self) {
        if let Some(drag) = self.drag.take()
            && !drag.is_dragging
        {
            self.selected.clear();
            self.last_selected = None;
        }
    }

    pub fn update_drag(
        &mut self,
        position: Point,
        items: &[DirectoryItem],
        window_width: f32,
        scroll_y: f32,
        modifiers: Modifiers,
        view_mode: ViewMode,
    ) {
        let (start, initial_selection) = {
            let Some(drag) = &mut self.drag else {
                return;
            };

            drag.current = position;
            let dx = (drag.current.x - drag.start.x).abs();
            let dy = (drag.current.y - drag.start.y).abs();
            if dx >= 4.0 || dy >= 4.0 {
                drag.is_dragging = true;
            }

            if !drag.is_dragging {
                return;
            }

            (drag.start, drag.initial_selection.clone())
        };

        let selection_rect = Rect::from_points(start, position);
        let newly_selected: HashSet<PathBuf> = items
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                let item_rect = match view_mode {
                    ViewMode::Grid => {
                        grid_view::item_rect(*index, grid_view::get_columns(window_width), scroll_y)
                    }
                    ViewMode::List => list_view::item_rect(*index, window_width, scroll_y),
                };
                selection_rect.intersects(&item_rect)
            })
            .map(|(_, item)| item.path.clone())
            .collect();

        if modifiers.command() {
            self.selected = initial_selection.union(&newly_selected).cloned().collect();
        } else {
            self.selected = newly_selected;
        }

        if self.selected.len() == 1 {
            self.last_selected = self.selected.iter().next().cloned();
        }
    }

    pub fn drag_rect(&self) -> Option<Rect> {
        self.drag
            .as_ref()
            .filter(|drag| drag.is_dragging)
            .map(|drag| Rect::from_points(drag.start, drag.current))
    }
}
