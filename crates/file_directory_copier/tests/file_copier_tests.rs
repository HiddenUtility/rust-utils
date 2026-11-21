use file_directory_copier::sync::FileCopier;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_file_copier_copy_should_succeed() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("source.txt");
    let dest_path = dir.path().join("dest.txt");

    fs::write(&source_path, b"hello world").unwrap();

    let copier = FileCopier::new();
    copier.copy(&source_path, &dest_path).expect("Copy failed");

    assert!(dest_path.exists());
    assert_eq!(fs::read(&dest_path).unwrap(), b"hello world");
}
