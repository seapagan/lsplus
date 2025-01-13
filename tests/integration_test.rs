use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_lsplus_version() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_lsplus_help() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lsplus"));
}

#[test]
fn test_lsplus_list_current_directory() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("testfile");
    fs::write(&file_path, "test content").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testfile"));
}

#[test]
fn test_lsplus_list_specific_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("testfile");
    fs::write(&file_path, "test content").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg(file_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("testfile"));
}

#[test]
fn test_lsplus_long_format() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("testfile");
    fs::write(&file_path, "test content").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testfile"));
}
