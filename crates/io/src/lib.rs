use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// ファイルをコピーします。
pub fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    let mut temp_dst = dst.to_path_buf();
    temp_dst.set_extension("temp");
    fs::copy(src, &temp_dst)?;
    fs::rename(&temp_dst, dst)?;
    Ok(())
}

/// ディレクトリをコピーします。
pub fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    let mut temp_dst = dst.to_path_buf();
    let mut s = dst.to_string_lossy().into_owned();
    s.push_str("_temp");
    let temp_dst_path = Path::new(&s);

    if let Some(parent) = temp_dst_path.parent() {
        fs::create_dir_all(parent)?;
    }

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(src).map_err(io::Error::other)?;
        let dest_path = temp_dst_path.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest_path)?;
        }
    }
    fs::rename(temp_dst_path, dst)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

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

    #[test]
    fn test_copy_file_uses_temp_extension() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src.txt");
        let dst_path = dir.path().join("dst.txt");
        let temp_dst_path = dst_path.with_extension("temp");

        let mut file = File::create(&src_path).unwrap();
        writeln!(file, "hello world").unwrap();

        let src_path = Arc::new(src_path);
        let dst_path = Arc::new(dst_path);
        let temp_dst_path = Arc::new(temp_dst_path);

        let src_clone = Arc::clone(&src_path);
        let dst_clone = Arc::clone(&dst_path);
        let temp_dst_clone = Arc::clone(&temp_dst_path);
        let handle = thread::spawn(move || {
            copy_file(&src_clone, &dst_clone).expect("copy_file failed");
        });

        let mut found_temp = false;
        for _ in 0..5000 {
            if temp_dst_clone.exists() {
                found_temp = true;
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }

        assert!(found_temp, "Temporary file was not found during copy");
        handle.join().unwrap();

        assert!(dst_path.exists());
        assert!(!temp_dst_path.exists());
    }

    #[test]
    fn test_copy_dir_uses_temp_suffix() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src_dir");
        let dst_dir = dir.path().join("dst_dir");
        let temp_dst_path = dir.path().join("dst_dir_temp");

        fs::create_dir(&src_dir).unwrap();
        let file1_path = src_dir.join("file1.txt");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, "file1 content").unwrap();

        let src_dir = Arc::new(src_dir);
        let dst_dir = Arc::new(dst_dir);
        let temp_dst_path = Arc::new(temp_dst_path);

        let src_clone = Arc::clone(&src_dir);
        let dst_clone = Arc::clone(&dst_dir);
        let temp_dst_clone = Arc::clone(&temp_dst_path);
        let handle = thread::spawn(move || {
            copy_dir(&src_clone, &dst_clone).expect("copy_dir failed");
        });

        let mut found_temp = false;
        for _ in 0..5000 {
            if temp_dst_clone.exists() {
                found_temp = true;
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }

        assert!(found_temp, "Temporary directory was not found during copy");
        handle.join().unwrap();

        assert!(dst_dir.exists());
        assert!(!temp_dst_path.exists());
    }
}
