#[cfg(test)]
pub use crate::commands::zip as zip_items;

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
