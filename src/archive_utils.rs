use std::path::Path;
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
