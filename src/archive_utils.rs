use std::path::{Path, PathBuf};
use std::process::Command;

pub fn zip_item(path: &Path) {
    let mut dest = path.to_path_buf();
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".zip");
    dest.set_file_name(name);
    
    let _ = Command::new("/usr/bin/ditto")
        .arg("-c")
        .arg("-k")
        .arg("--sequesterRsrc")
        .arg(path)
        .arg(&dest)
        .status();
}

pub fn zip_items(paths: &[PathBuf]) {
    if paths.is_empty() {
        return;
    }
    if paths.len() == 1 {
        zip_item(&paths[0]);
        return;
    }

    let parent = paths[0].parent().unwrap_or_else(|| Path::new("."));
    let mut dest = parent.join("Archive.zip");
    let mut counter = 2;
    while dest.exists() {
        dest = parent.join(format!("Archive {}.zip", counter));
        counter += 1;
    }

    let mut cmd = Command::new("/usr/bin/zip");
    cmd.arg("-r").arg(&dest);
    for p in paths {
        if let Some(file_name) = p.file_name() {
            cmd.arg(file_name);
        }
    }
    cmd.current_dir(parent);
    let _ = cmd.status();
}

pub fn unzip_item(path: &Path) {
    let stem = path.file_stem().unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));
    let dest = parent.join(stem);
    
    let _ = Command::new("/usr/bin/ditto")
        .arg("-x")
        .arg("-k")
        .arg(path)
        .arg(&dest)
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_zip_items_multiple_files() {
        let temp_dir = std::env::temp_dir().join(format!(
            "ex_finder_zip_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let file1 = temp_dir.join("file1.txt");
        let file2 = temp_dir.join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        zip_items(&[file1.clone(), file2.clone()]);

        let archive_path = temp_dir.join("Archive.zip");
        assert!(archive_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
