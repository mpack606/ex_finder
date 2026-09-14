use crate::app_icons;
use chrono::{DateTime, Local};
use iced::widget::{button, column, container, image, mouse_area, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

#[derive(Debug, Clone)]
pub enum State {
    Loading(PathBuf),
    Loaded(Details),
    Error { path: PathBuf, message: String },
}

#[derive(Debug, Clone)]
pub struct Details {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub modified: String,
    pub path: String,
    pub default_app: String,
    pub permissions: String,
    pub icon: Option<iced::widget::image::Handle>,
}

pub fn load(path: PathBuf) -> Result<Details, String> {
    let metadata =
        fs::metadata(&path).map_err(|error| format!("Could not read file information: {error}"))?;
    let byte_size = if metadata.is_dir() {
        directory_size(&path)
    } else {
        metadata.len()
    };

    Ok(Details {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned()),
        kind: file_type(&path, metadata.is_dir()),
        size: format_size(byte_size),
        modified: metadata
            .modified()
            .map(format_date)
            .unwrap_or_else(|_| "Unavailable".to_owned()),
        path: path.to_string_lossy().into_owned(),
        default_app: if metadata.is_file() {
            default_application(&path).unwrap_or_else(|| "Unavailable".to_owned())
        } else {
            "Not applicable".to_owned()
        },
        permissions: format_permissions(metadata.permissions().mode()),
        icon: app_icons::get_app_icon_for_file(&path).map(iced::widget::image::Handle::from_bytes),
    })
}

pub fn view(state: &State) -> Element<'static, Message> {
    let body: Element<'static, Message> = match state {
        State::Loading(path) => column![
            heading("File information"),
            text(format!("Loading information for {}…", path.display())).size(14),
        ]
        .spacing(18)
        .into(),
        State::Error { path, message } => column![
            heading("File information"),
            detail_row("Path", path.to_string_lossy().into_owned()),
            detail_row("Error", message.clone()),
        ]
        .spacing(14)
        .into(),
        State::Loaded(details) => {
            let icon: Element<'static, Message> = details
                .icon
                .as_ref()
                .map(|handle| image(handle.clone()).width(64).height(64).into())
                .unwrap_or_else(|| {
                    container(text("No icon").size(12))
                        .width(64)
                        .height(64)
                        .center(64)
                        .into()
                });

            column![
                row![icon, heading(details.name.clone())]
                    .spacing(16)
                    .align_y(Alignment::Center),
                detail_row("Name", details.name.clone()),
                detail_row("Type", details.kind.clone()),
                detail_row("Size", details.size.clone()),
                detail_row("Modified", details.modified.clone()),
                detail_row("Full path", details.path.clone()),
                detail_row("Default app", details.default_app.clone()),
                detail_row("Permissions", details.permissions.clone()),
            ]
            .spacing(12)
            .into()
        }
    };

    let close_button = button(text("Close").align_x(Alignment::Center))
        .on_press(Message::Close)
        .padding(8)
        .width(Length::Fill)
        .style(|theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            button::Style {
                background: Some(palette.primary.base.color.into()),
                text_color: palette.primary.base.text,
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    let dialog = container(column![scrollable(body), close_button].spacing(18))
        .padding(20)
        .width(Length::Fixed(520.0))
        .max_height(560.0)
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

    mouse_area(
        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(
                    iced::Color {
                        a: 0.5,
                        ..theme.extended_palette().background.base.color
                    }
                    .into(),
                ),
                ..Default::default()
            }),
    )
    .on_press(Message::Close)
    .on_right_press(Message::Close)
    .into()
}

fn heading(value: impl text::IntoFragment<'static>) -> iced::widget::Text<'static> {
    text(value).size(18).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    })
}

fn detail_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label)
            .size(13)
            .width(Length::Fixed(100.0))
            .style(|theme: &iced::Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().background.strong.text),
                }
            }),
        text(value).size(13).width(Length::Fill),
    ]
    .spacing(14)
    .align_y(Alignment::Start)
    .into()
}

fn directory_size(path: &Path) -> u64 {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| {
            entry.file_type().map_or(0, |file_type| {
                if file_type.is_symlink() {
                    fs::symlink_metadata(entry.path()).map_or(0, |metadata| metadata.len())
                } else if file_type.is_dir() {
                    directory_size(&entry.path())
                } else {
                    entry.metadata().map_or(0, |metadata| metadata.len())
                }
            })
        })
        .sum()
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    let readable = if unit == 0 {
        format!("{bytes} bytes")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    };
    if unit == 0 {
        readable
    } else {
        format!("{readable} ({bytes} bytes)")
    }
}

fn format_date(time: SystemTime) -> String {
    let local: DateTime<Local> = time.into();
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn format_permissions(mode: u32) -> String {
    format!(
        "Owner: {}  Group: {}  Everyone: {} ({:03o})",
        permission_triplet(mode, 6),
        permission_triplet(mode, 3),
        permission_triplet(mode, 0),
        mode & 0o777
    )
}

fn permission_triplet(mode: u32, shift: u32) -> String {
    let bits = (mode >> shift) & 0o7;
    format!(
        "{}{}{}",
        if bits & 0o4 != 0 { 'r' } else { '-' },
        if bits & 0o2 != 0 { 'w' } else { '-' },
        if bits & 0o1 != 0 { 'x' } else { '-' },
    )
}

fn file_type(path: &Path, is_directory: bool) -> String {
    if is_directory {
        return "Folder".to_owned();
    }
    macos_file_type(path).unwrap_or_else(|| {
        path.extension()
            .filter(|extension| !extension.is_empty())
            .map(|extension| format!("{} file", extension.to_string_lossy().to_uppercase()))
            .unwrap_or_else(|| "File".to_owned())
    })
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn macos_file_type(path: &Path) -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;

    let path = NSString::from_str(&path.to_string_lossy());
    let workspace = NSWorkspace::sharedWorkspace();
    let kind = workspace.typeOfFile_error(&path).ok()?;
    workspace
        .localizedDescriptionForType(&kind)
        .map(|description| description.to_string())
}

#[cfg(not(target_os = "macos"))]
fn macos_file_type(_path: &Path) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn default_application(path: &Path) -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    let path = NSString::from_str(&path.to_string_lossy());
    let file_url = NSURL::fileURLWithPath(&path);
    let app_url = NSWorkspace::sharedWorkspace().URLForApplicationToOpenURL(&file_url)?;
    let app_name = app_url.lastPathComponent()?.to_string();
    Some(
        app_name
            .strip_suffix(".app")
            .unwrap_or(&app_name)
            .to_owned(),
    )
}

#[cfg(not(target_os = "macos"))]
fn default_application(_path: &Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_sizes_for_people_and_keeps_exact_bytes() {
        assert_eq!(format_size(42), "42 bytes");
        assert_eq!(format_size(1536), "1.5 KB (1536 bytes)");
    }

    #[test]
    fn formats_all_unix_permission_groups() {
        assert_eq!(
            format_permissions(0o100754),
            "Owner: rwx  Group: r-x  Everyone: r-- (754)"
        );
    }

    #[test]
    fn directory_size_includes_nested_files() {
        let root = std::env::temp_dir().join(format!(
            "ex_finder_info_{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("one"), b"1234").unwrap();
        fs::write(root.join("nested/two"), b"123456").unwrap();

        assert_eq!(directory_size(&root), 10);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn directory_size_does_not_follow_symlink_cycles() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "ex_finder_info_symlink_{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("nested")).unwrap();
        symlink(&root, root.join("nested/back-to-root")).unwrap();

        let size = directory_size(&root);

        assert!(size > 0);
        fs::remove_dir_all(root).unwrap();
    }
}
