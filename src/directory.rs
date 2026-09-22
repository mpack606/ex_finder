use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use iced::Task;

#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum LoadAction {
    Current {
        select: Option<Vec<PathBuf>>,
        record_location: bool,
    },
    Navigate,
    Back,
    Forward,
}

#[derive(Debug, Clone)]
pub struct LoadResult {
    pub path: PathBuf,
    pub generation: u64,
    pub action: LoadAction,
    pub result: Result<Vec<DirectoryItem>, String>,
}

#[derive(Debug, Default)]
pub struct State {
    pub items: Vec<DirectoryItem>,
    pub loading: Option<PathBuf>,
    generation: u64,
}

impl State {
    pub fn request(&mut self, path: PathBuf, action: LoadAction) -> Task<LoadResult> {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.loading = Some(path.clone());
        let read_path = path.clone();

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    read(&read_path)
                        .map_err(|error| format!("Could not open {}: {error}", read_path.display()))
                })
                .await
                .map_err(|error| format!("background task failed: {error}"))
                .and_then(|result| result)
            },
            move |result| LoadResult {
                path,
                generation,
                action,
                result,
            },
        )
    }

    pub fn accepts(&mut self, generation: u64) -> bool {
        if generation != self.generation {
            return false;
        }
        self.loading = None;
        true
    }
}

pub fn read(path: &Path) -> Result<Vec<DirectoryItem>, std::io::Error> {
    let mut items = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let metadata = entry.metadata().ok();

        items.push(DirectoryItem {
            path: entry_path.clone(),
            name: file_name.clone(),
            is_dir: metadata
                .as_ref()
                .map(|metadata| metadata.is_dir())
                .unwrap_or_else(|| entry_path.is_dir()),
            is_hidden: file_name.starts_with('.'),
            size: metadata.as_ref().map_or(0, |metadata| metadata.len()),
            created: metadata
                .as_ref()
                .and_then(|metadata| metadata.created().ok()),
            modified: metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok()),
        });
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_includes_hidden_entries() {
        let directory = std::env::temp_dir().join(format!(
            "ex_finder_directory_{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(directory.join(".hidden_dir")).unwrap();
        fs::write(directory.join("visible.txt"), b"").unwrap();
        fs::write(directory.join(".hidden.txt"), b"").unwrap();

        let items = read(&directory).unwrap();

        assert_eq!(items.len(), 3);
        assert!(
            items
                .iter()
                .any(|item| item.name == ".hidden.txt" && item.is_hidden)
        );
        assert!(
            items
                .iter()
                .any(|item| item.name == ".hidden_dir" && item.is_dir)
        );
        assert!(
            items
                .iter()
                .any(|item| item.name == "visible.txt" && !item.is_hidden)
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn state_rejects_stale_load_results() {
        let mut state = State::default();
        let _first = state.request(
            PathBuf::from("/first"),
            LoadAction::Current {
                select: None,
                record_location: false,
            },
        );
        let _second = state.request(
            PathBuf::from("/second"),
            LoadAction::Current {
                select: None,
                record_location: false,
            },
        );

        assert!(!state.accepts(1));
        assert!(state.accepts(2));
        assert!(state.loading.is_none());
    }
}
