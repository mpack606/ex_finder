use crate::commands::{self, Command};
use iced::Task;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Event {
    Done,
    Refresh,
    RefreshAndSelect(Vec<PathBuf>),
    CutCompleted(Vec<PathBuf>),
    Rename(PathBuf),
    OpenInNewTab(PathBuf),
    GetInfo(PathBuf),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    paths: Vec<PathBuf>,
    operation: Option<ClipboardOperation>,
}

impl Clipboard {
    pub fn has_items(&self) -> bool {
        !self.paths.is_empty()
    }

    pub fn cut_paths(&self) -> &[PathBuf] {
        if self.operation == Some(ClipboardOperation::Cut) {
            &self.paths
        } else {
            &[]
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn execute(action: Command, clipboard: &mut Clipboard, current_path: PathBuf) -> Task<Event> {
    match action {
        Command::OpenInNewTab(path) => Task::done(Event::OpenInNewTab(path)),
        Command::GetInfo(path) => Task::done(Event::GetInfo(path)),
        Command::Copy(paths) => {
            clipboard.paths = paths;
            clipboard.operation = Some(ClipboardOperation::Copy);
            Task::done(Event::Done)
        }
        Command::Cut(paths) => {
            clipboard.paths = paths;
            clipboard.operation = Some(ClipboardOperation::Cut);
            Task::done(Event::Done)
        }
        Command::Paste(destination) => {
            if !clipboard.has_items() {
                return Task::none();
            }

            let paths = clipboard.paths.clone();
            let operation = clipboard.operation;
            Task::perform(
                run_io(move || match operation {
                    Some(ClipboardOperation::Cut) => commands::move_items(&paths, &destination),
                    _ => commands::copy(&paths, &destination),
                }),
                move |result| match result {
                    Ok(paths) if operation == Some(ClipboardOperation::Cut) => {
                        Event::CutCompleted(paths)
                    }
                    Ok(paths) => Event::RefreshAndSelect(paths),
                    Err(error) => Event::Failed(format!("Paste failed: {error}")),
                },
            )
        }
        Command::Move(paths, destination) => Task::perform(
            run_io(move || commands::move_items(&paths, &destination)),
            |result| match result {
                Ok(_) => Event::Refresh,
                Err(error) => Event::Failed(format!("Move failed: {error}")),
            },
        ),
        Command::MoveToTrash(paths) => Task::perform(
            run_io(move || commands::move_to_trash(&paths)),
            |result| match result {
                Ok(()) => Event::Refresh,
                Err(error) => Event::Failed(format!("Move to trash failed: {error}")),
            },
        ),
        Command::Rename(path) => Task::done(Event::Rename(path)),
        Command::Zip(paths) => {
            Task::perform(
                run_io(move || Ok(commands::zip(&paths))),
                |result| match result {
                    Ok(paths) if !paths.is_empty() => Event::RefreshAndSelect(paths),
                    Ok(_) => Event::Failed("Could not create the archive".to_owned()),
                    Err(error) => Event::Failed(format!("Archive failed: {error}")),
                },
            )
        }
        Command::Unzip(path) => {
            Task::perform(
                run_io(move || commands::unzip(&path)),
                |result| match result {
                    Ok(()) => Event::Refresh,
                    Err(error) => Event::Failed(format!("Unzip failed: {error}")),
                },
            )
        }
        Command::CreateNewFolder => Task::perform(
            run_io(move || commands::create_new_folder(&current_path)),
            |result| match result {
                Ok(path) => Event::RefreshAndSelect(vec![path]),
                Err(error) => Event::Failed(format!("Create folder failed: {error}")),
            },
        ),
        Command::Refresh => Task::done(Event::Refresh),
    }
}

pub fn rename(path: PathBuf, new_name: String) -> Task<Event> {
    Task::perform(
        run_io(move || commands::rename(&path, &new_name)),
        |result| match result {
            Ok(path) => Event::RefreshAndSelect(vec![path]),
            Err(error) => Event::Failed(format!("Rename failed: {error}")),
        },
    )
}

pub fn open(path: PathBuf) -> Task<Event> {
    Task::perform(
        run_io(move || open::that(path).map_err(io::Error::other)),
        |result| match result {
            Ok(()) => Event::Done,
            Err(error) => Event::Failed(format!("Open failed: {error}")),
        },
    )
}

async fn run_io<T, Operation>(operation: Operation) -> Result<T, String>
where
    T: Send + 'static,
    Operation: FnOnce() -> io::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| format!("background task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_tracks_copy_and_cut_operations() {
        let paths = vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")];
        let mut clipboard = Clipboard::default();

        let _ = execute(
            Command::Copy(paths.clone()),
            &mut clipboard,
            PathBuf::from("/tmp"),
        );
        assert!(clipboard.has_items());
        assert!(clipboard.cut_paths().is_empty());

        let _ = execute(
            Command::Cut(paths.clone()),
            &mut clipboard,
            PathBuf::from("/tmp"),
        );
        assert_eq!(clipboard.cut_paths(), paths);

        clipboard.clear();
        assert!(!clipboard.has_items());
    }
}
