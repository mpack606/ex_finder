use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

#[derive(Debug, Clone)]
pub enum Command {
    OpenInNewTab(PathBuf),
    GetInfo(PathBuf),
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
    Paste,
    Move(Vec<PathBuf>, PathBuf),
    Rename(PathBuf),
    MoveToTrash(Vec<PathBuf>),
    Zip(Vec<PathBuf>),
    Unzip(PathBuf),
    CreateNewFolder,
    Refresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Search,
    Copy,
    Cut,
    Move,
    Paste,
    MoveToTrash,
    Rename,
    OpenInNewTab,
    GetInfo,
    Zip,
    Unzip,
    CreateNewFolder,
    Refresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutKey {
    Character(char),
    Enter,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutModifiers {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Debug, Clone, Copy)]
struct Shortcut {
    command: CommandKind,
    key: ShortcutKey,
    modifiers: ShortcutModifiers,
    display: &'static str,
}

const NO_MODIFIERS: ShortcutModifiers = ShortcutModifiers {
    control: false,
    shift: false,
    alt: false,
    command: false,
};

const CONTROL: ShortcutModifiers = ShortcutModifiers {
    control: true,
    ..NO_MODIFIERS
};

const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        command: CommandKind::Search,
        key: ShortcutKey::Character('f'),
        modifiers: CONTROL,
        display: "Ctrl+F",
    },
    Shortcut {
        command: CommandKind::Copy,
        key: ShortcutKey::Character('c'),
        modifiers: CONTROL,
        display: "Ctrl+C",
    },
    Shortcut {
        command: CommandKind::Cut,
        key: ShortcutKey::Character('x'),
        modifiers: CONTROL,
        display: "Ctrl+X",
    },
    Shortcut {
        command: CommandKind::Paste,
        key: ShortcutKey::Character('v'),
        modifiers: CONTROL,
        display: "Ctrl+V",
    },
    Shortcut {
        command: CommandKind::MoveToTrash,
        key: ShortcutKey::Delete,
        modifiers: NO_MODIFIERS,
        display: "Delete",
    },
    Shortcut {
        command: CommandKind::Rename,
        key: ShortcutKey::Enter,
        modifiers: NO_MODIFIERS,
        display: "Enter",
    },
];

impl Command {
    pub fn kind(&self) -> CommandKind {
        match self {
            Self::OpenInNewTab(_) => CommandKind::OpenInNewTab,
            Self::GetInfo(_) => CommandKind::GetInfo,
            Self::Copy(_) => CommandKind::Copy,
            Self::Cut(_) => CommandKind::Cut,
            Self::Paste => CommandKind::Paste,
            Self::Move(_, _) => CommandKind::Move,
            Self::Rename(_) => CommandKind::Rename,
            Self::MoveToTrash(_) => CommandKind::MoveToTrash,
            Self::Zip(_) => CommandKind::Zip,
            Self::Unzip(_) => CommandKind::Unzip,
            Self::CreateNewFolder => CommandKind::CreateNewFolder,
            Self::Refresh => CommandKind::Refresh,
        }
    }
}

pub fn shortcut_for(command: CommandKind) -> Option<&'static str> {
    SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.command == command)
        .map(|shortcut| shortcut.display)
}

pub fn resolve_shortcut(
    key: ShortcutKey,
    modifiers: ShortcutModifiers,
) -> Option<CommandKind> {
    // `iced` exposes the platform command key separately from Control on macOS.
    // Accept both so the documented Ctrl shortcuts also behave like native macOS
    // shortcuts when the Command key is used.
    let modifiers = ShortcutModifiers {
        control: modifiers.control || modifiers.command,
        command: false,
        ..modifiers
    };
    let key = match key {
        ShortcutKey::Character(character) => {
            ShortcutKey::Character(character.to_ascii_lowercase())
        }
        key => key,
    };

    SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.key == key && shortcut.modifiers == modifiers)
        .map(|shortcut| shortcut.command)
}

pub fn rename(old_path: &Path, new_name: &str) -> io::Result<PathBuf> {
    let new_path = old_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(new_name);
    fs::rename(old_path, &new_path).map(|_| new_path)
}

pub fn copy(paths: &[PathBuf], destination: &Path) -> io::Result<Vec<PathBuf>> {
    let mut destination_paths = Vec::new();

    for source in paths {
        if let Some(file_name) = source.file_name() {
            let destination_path = available_copy_path(source, destination, file_name);
            if source.is_dir() {
                copy_directory(source, &destination_path)?;
            } else {
                fs::copy(source, &destination_path)?;
            }
            destination_paths.push(destination_path);
        }
    }

    Ok(destination_paths)
}

pub fn move_items(paths: &[PathBuf], destination: &Path) -> io::Result<Vec<PathBuf>> {
    if !destination.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "move destination is not a directory",
        ));
    }

    let moves = paths
        .iter()
        .filter_map(|source| {
            source
                .file_name()
                .map(|file_name| (source.clone(), destination.join(file_name)))
        })
        .filter(|(source, target)| source != target)
        .collect::<Vec<_>>();

    for (source, target) in &moves {
        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} no longer exists", source.display()),
            ));
        }
        if target.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} already exists", target.display()),
            ));
        }
        if source.is_dir() && destination.starts_with(source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot move a folder into itself",
            ));
        }
    }

    for (source, target) in &moves {
        fs::rename(source, target)?;
    }

    Ok(moves.into_iter().map(|(_, target)| target).collect())
}

fn available_copy_path(source: &Path, destination: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
    let original = destination.join(file_name);
    if original != source && !original.exists() {
        return original;
    }

    let stem = source
        .file_stem()
        .unwrap_or(file_name)
        .to_string_lossy();
    let extension = source
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let suffix = if counter == 1 {
            " copy".to_owned()
        } else {
            format!(" copy {counter}")
        };
        let candidate = destination.join(format!("{stem}{suffix}{extension}"));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub fn move_to_trash(paths: &[PathBuf]) -> io::Result<()> {
    for path in paths {
        trash::delete(path)
            .map_err(|error| io::Error::other(error.to_string()))?;
    }
    Ok(())
}

pub fn zip(paths: &[PathBuf]) -> Vec<PathBuf> {
    if paths.is_empty() {
        return Vec::new();
    }

    if paths.len() == 1 {
        return zip_item(&paths[0]).into_iter().collect();
    }

    let parent = paths[0].parent().unwrap_or_else(|| Path::new("."));
    let mut destination = parent.join("Archive.zip");
    let mut counter = 2;
    while destination.exists() {
        destination = parent.join(format!("Archive {counter}.zip"));
        counter += 1;
    }

    let mut command = ProcessCommand::new("/usr/bin/zip");
    command.arg("-r").arg(&destination);
    for path in paths {
        if let Some(file_name) = path.file_name() {
            command.arg(file_name);
        }
    }
    command.current_dir(parent);

    command
        .status()
        .ok()
        .filter(|status| status.success())
        .map(|_| vec![destination])
        .unwrap_or_default()
}

pub fn zip_item(path: &Path) -> Option<PathBuf> {
    let mut destination = path.to_path_buf();
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".zip");
    destination.set_file_name(name);

    let status = ProcessCommand::new("/usr/bin/ditto")
        .arg("-c")
        .arg("-k")
        .arg("--sequesterRsrc")
        .arg(path)
        .arg(&destination)
        .status();

    status
        .ok()
        .filter(|status| status.success())
        .map(|_| destination)
}

pub fn unzip(path: &Path) -> io::Result<()> {
    let stem = path.file_stem().unwrap_or_default();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let destination = parent.join(stem);

    ProcessCommand::new("/usr/bin/ditto")
        .arg("-x")
        .arg("-k")
        .arg(path)
        .arg(destination)
        .status()
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::other("unzip command failed"))
            }
        })
}

pub fn create_new_folder(current_path: &Path) -> io::Result<PathBuf> {
    let path = current_path.join("New Folder");
    fs::create_dir(&path).map(|_| path)
}

fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_destination = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &entry_destination)?;
        } else {
            fs::copy(entry.path(), entry_destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: ShortcutModifiers = ShortcutModifiers {
        command: true,
        ..NO_MODIFIERS
    };

    #[test]
    fn resolves_all_documented_shortcuts() {
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('f'), CONTROL),
            Some(CommandKind::Search)
        );
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('c'), CONTROL),
            Some(CommandKind::Copy)
        );
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('x'), CONTROL),
            Some(CommandKind::Cut)
        );
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('v'), CONTROL),
            Some(CommandKind::Paste)
        );
        assert_eq!(
            resolve_shortcut(ShortcutKey::Delete, NO_MODIFIERS),
            Some(CommandKind::MoveToTrash)
        );
        assert_eq!(
            resolve_shortcut(ShortcutKey::Enter, NO_MODIFIERS),
            Some(CommandKind::Rename)
        );
    }

    #[test]
    fn character_shortcuts_are_case_insensitive() {
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('C'), CONTROL),
            Some(CommandKind::Copy)
        );
    }

    #[test]
    fn accepts_the_native_macos_command_modifier() {
        assert_eq!(
            resolve_shortcut(ShortcutKey::Character('c'), COMMAND),
            Some(CommandKind::Copy)
        );
    }

    #[test]
    fn rejects_extra_modifiers() {
        assert_eq!(
            resolve_shortcut(
                ShortcutKey::Character('c'),
                ShortcutModifiers {
                    shift: true,
                    ..CONTROL
                }
            ),
            None
        );
    }

    #[test]
    fn get_info_has_no_keyboard_shortcut() {
        assert_eq!(shortcut_for(CommandKind::GetInfo), None);
    }

    #[test]
    fn copy_in_the_same_directory_creates_a_numbered_duplicate() {
        let directory = std::env::temp_dir().join(format!(
            "ex_finder_copy_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let source = directory.join("notes.txt");
        fs::write(&source, "hello").unwrap();

        let first = copy(std::slice::from_ref(&source), &directory).unwrap();
        let second = copy(std::slice::from_ref(&source), &directory).unwrap();

        assert_eq!(first, vec![directory.join("notes copy.txt")]);
        assert_eq!(second, vec![directory.join("notes copy 2.txt")]);
        assert_eq!(fs::read_to_string(&first[0]).unwrap(), "hello");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn move_items_moves_files_and_folders_to_destination() {
        let root = std::env::temp_dir().join(format!(
            "ex_finder_move_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let destination = root.join("destination");
        let file = root.join("notes.txt");
        let folder = root.join("photos");
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir(&folder).unwrap();
        fs::write(&file, "hello").unwrap();

        let moved = move_items(&[file.clone(), folder.clone()], &destination).unwrap();

        assert_eq!(
            moved,
            vec![destination.join("notes.txt"), destination.join("photos")]
        );
        assert!(!file.exists());
        assert!(!folder.exists());
        assert_eq!(fs::read_to_string(&moved[0]).unwrap(), "hello");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn move_items_does_not_overwrite_an_existing_item() {
        let root = std::env::temp_dir().join(format!(
            "ex_finder_move_conflict_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let destination = root.join("destination");
        let source = root.join("notes.txt");
        fs::create_dir_all(&destination).unwrap();
        fs::write(&source, "source").unwrap();
        fs::write(destination.join("notes.txt"), "destination").unwrap();

        let result = move_items(std::slice::from_ref(&source), &destination);

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(source).unwrap(), "source");
        assert_eq!(
            fs::read_to_string(destination.join("notes.txt")).unwrap(),
            "destination"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
