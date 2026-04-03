use lsplus::utils::icons::{Icon, get_item_icon, has_extension};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(unix)]
use std::path::PathBuf;

#[test]
fn test_get_item_icon_uses_known_file_types_and_names() {
    let metadata = fs::metadata("Cargo.toml").unwrap();

    assert_eq!(
        get_item_icon(&metadata, Path::new("test.unknown")),
        Icon::GenericFile
    );
    assert_eq!(
        get_item_icon(&metadata, Path::new("test.rs")),
        Icon::RustFile
    );
    assert_eq!(
        get_item_icon(&metadata, Path::new("Cargo.toml")),
        Icon::TomlFile
    );
    assert_eq!(
        get_item_icon(&metadata, Path::new("nested/config/.gitignore")),
        Icon::GitFile
    );
}

#[test]
fn test_get_item_icon_uses_known_directory_names() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    let other_dir = temp_dir.path().join("plain-dir");
    fs::create_dir(&git_dir).unwrap();
    fs::create_dir(&other_dir).unwrap();

    let git_metadata = fs::metadata(&git_dir).unwrap();
    let other_metadata = fs::metadata(&other_dir).unwrap();

    assert_eq!(get_item_icon(&git_metadata, &git_dir), Icon::GitFile);
    assert_eq!(get_item_icon(&other_metadata, &other_dir), Icon::Folder);
}

#[cfg(unix)]
#[test]
fn test_get_item_icon_handles_symlinks_and_non_utf8_extensions() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("test_link");
    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let link_metadata = fs::symlink_metadata(&link).unwrap();
    assert_eq!(get_item_icon(&link_metadata, &link), Icon::Symlink);

    let file_name = OsString::from_vec(b"bad-\xff.rs".to_vec());
    let file_path = PathBuf::from(&file_name);
    let full_path = temp_dir.path().join(&file_path);
    fs::write(&full_path, "fn main() {}").unwrap();
    let metadata = fs::metadata(&full_path).unwrap();

    assert_eq!(get_item_icon(&metadata, &full_path), Icon::RustFile);
}

#[test]
fn test_has_extension_rejects_empty_extension_and_dot_entries() {
    assert!(!has_extension("file.txt", ""));
    assert!(!has_extension(".", "txt"));
    assert!(!has_extension("..", "txt"));
    assert!(has_extension("file.txt", "txt"));
}
