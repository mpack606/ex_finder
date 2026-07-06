use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;

use std::sync::OnceLock;

static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<PathBuf>>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, Option<PathBuf>>> {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_app_icon_for_extension(extension: &str) -> Option<PathBuf> {
    let mut cache = get_cache().lock().unwrap();
    if let Some(path) = cache.get(extension) {
        return path.clone();
    }

    let icon_path = fetch_app_icon_for_extension(extension);
    cache.insert(extension.to_string(), icon_path.clone());
    icon_path
}

fn fetch_app_icon_for_extension(extension: &str) -> Option<PathBuf> {
    // Create a dummy file with the extension to find the default app
    let temp_dir = env::temp_dir();
    let dummy_file = temp_dir.join(format!("dummy.{}", extension));
    if !dummy_file.exists() {
        let _ = std::fs::File::create(&dummy_file);
    }

    // Get default app path using swift
    let output = Command::new("swift")
        .arg("-e")
        .arg("import AppKit; let url = URL(fileURLWithPath: CommandLine.arguments[1]); if let appURL = NSWorkspace.shared.urlForApplication(toOpen: url) { print(appURL.path) }")
        .arg(&dummy_file)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let app_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if app_path.is_empty() {
        return None;
    }

    // Get icon filename from Info.plist
    let info_plist = Path::new(&app_path).join("Contents/Info.plist");
    if !info_plist.exists() {
        return None;
    }

    let output = Command::new("defaults")
        .arg("read")
        .arg(info_plist.to_str()?)
        .arg("CFBundleIconFile")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut icon_file = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !icon_file.ends_with(".icns") {
        icon_file.push_str(".icns");
    }

    let icns_path = Path::new(&app_path).join("Contents/Resources").join(icon_file);
    if !icns_path.exists() {
        return None;
    }

    // Convert to PNG using sips
    let png_name = format!("icon_{}.png", extension);
    let png_path = temp_dir.join(png_name);

    let output = Command::new("sips")
        .arg("-s")
        .arg("format")
        .arg("png")
        .arg("-z")
        .arg("32")
        .arg("32")
        .arg(icns_path.to_str()?)
        .arg("--out")
        .arg(png_path.to_str()?)
        .output()
        .ok()?;

    if output.status.success() {
        Some(png_path)
    } else {
        None
    }
}
