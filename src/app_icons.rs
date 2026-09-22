use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum IconKey {
    Extension(String),
    Path(PathBuf),
}

static ICON_CACHE: OnceLock<Mutex<HashMap<IconKey, Option<Vec<u8>>>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<IconKey, Option<Vec<u8>>>> {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn cache_key(path: &Path) -> IconKey {
    path.extension()
        .map(|extension| IconKey::Extension(extension.to_string_lossy().to_lowercase()))
        .unwrap_or_else(|| IconKey::Path(path.to_path_buf()))
}

pub fn get_app_icon_for_file(path: &Path) -> Option<Vec<u8>> {
    let key = cache_key(path);
    {
        let cache = get_cache().lock().unwrap();
        if let Some(icon) = cache.get(&key) {
            return icon.clone();
        }
    }

    let icon = fetch_app_icon(&key);
    get_cache().lock().unwrap().insert(key, icon.clone());
    icon
}

fn fetch_app_icon(key: &IconKey) -> Option<Vec<u8>> {
    use objc2::AnyThread;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSRect, NSSize, NSString};

    let workspace = NSWorkspace::sharedWorkspace();
    let image = match key {
        IconKey::Extension(extension) => {
            let extension = NSString::from_str(extension);
            #[allow(deprecated)]
            workspace.iconForFileType(&extension)
        }
        IconKey::Path(path) => {
            let path = NSString::from_str(&path.to_string_lossy());
            workspace.iconForFile(&path)
        }
    };

    let mut rect = NSRect::new(Default::default(), NSSize::new(32.0, 32.0));
    let cg_image = unsafe { image.CGImageForProposedRect_context_hints(&mut rect, None, None)? };
    let bitmap = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &cg_image);
    let data = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Some(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, ImageFormat};

    #[test]
    fn cache_key_reuses_icons_for_matching_extensions() {
        assert_eq!(
            cache_key(Path::new("/one/report.PDF")),
            cache_key(Path::new("/two/another.pdf"))
        );
    }

    #[test]
    fn cache_key_keeps_extensionless_files_separate() {
        assert_ne!(
            cache_key(Path::new("/one/Makefile")),
            cache_key(Path::new("/two/Makefile"))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn get_app_icon_for_file_returns_png_for_existing_file() {
        let path = std::env::temp_dir().join("ex_finder_icon_test.txt");
        std::fs::write(&path, b"icon test").unwrap();

        let icon = get_app_icon_for_file(&path).unwrap();
        let decoded = image::load_from_memory_with_format(&icon, ImageFormat::Png).unwrap();

        assert_eq!(decoded.dimensions(), (32, 32));
        assert!(decoded.to_rgba8().pixels().any(|pixel| pixel[3] > 0));
        let _ = std::fs::remove_file(path);
    }
}
