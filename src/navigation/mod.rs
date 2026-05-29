use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_path: PathBuf,
    pub history_back: Vec<PathBuf>,
    pub history_forward: Vec<PathBuf>,
}

impl NavigationState {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            current_path: initial_path,
            history_back: Vec::new(),
            history_forward: Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, new_path: PathBuf) {
        if new_path == self.current_path {
            return;
        }
        self.history_back.push(self.current_path.clone());
        self.history_forward.clear();
        self.current_path = new_path;
    }

    pub fn navigate_back(&mut self) -> bool {
        if let Some(prev) = self.history_back.pop() {
            self.history_forward.push(self.current_path.clone());
            self.current_path = prev;
            true
        } else {
            false
        }
    }

    pub fn navigate_forward(&mut self) -> bool {
        if let Some(next) = self.history_forward.pop() {
            self.history_back.push(self.current_path.clone());
            self.current_path = next;
            true
        } else {
            false
        }
    }
}
