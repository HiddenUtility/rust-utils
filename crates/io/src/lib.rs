use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// ファイルをコピーします。
pub fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

/// ディレクトリをコピーします。
pub fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(src).map_err(io::Error::other)?;
        let dest_path = dst.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_copy_file_should_work_successfully() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src.txt");
        let dst_path = dir.path().join("dst.txt");

        let mut file = File::create(&src_path).unwrap();
        writeln!(file, "hello world").unwrap();

        copy_file(&src_path, &dst_path).expect("copy_file failed");

        let content = fs::read_to_string(&dst_path).expect("failed to read dst file");
        assert_eq!(content, "hello world\n");
    }

    #[test]
    fn test_copy_dir_should_work_successfully() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src_dir");
        let dst_dir = dir.path().join("dst_dir");

        fs::create_dir(&src_dir).unwrap();
        let file1_path = src_dir.join("file1.txt");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, "file1 content").unwrap();

        let sub_dir = src_dir.join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();
        let file2_path = sub_dir.join("file2.txt");
        let mut file2 = File::create(&file2_path).unwrap();
        writeln!(file2, "file2 content").unwrap();

        copy_dir(&src_dir, &dst_dir).expect("copy_dir failed");

        assert!(dst_dir.exists());
        assert!(dst_dir.is_dir());

        let dst_file1_path = dst_dir.join("file1.txt");
        assert!(dst_file1_path.exists());
        let content1 = fs::read_to_string(&dst_file1_path).unwrap();
        assert_eq!(content1, "file1 content\n");

        let dst_sub_dir = dst_dir.join("sub_dir");
        assert!(dst_sub_dir.exists());
        assert!(dst_sub_dir.is_dir());

        let dst_file2_path = dst_sub_dir.join("file2.txt");
        assert!(dst_file2_path.exists());
        let content2 = fs::read_to_string(&dst_file2_path).unwrap();
        assert_eq!(content2, "file2 content\n");
    }
}
