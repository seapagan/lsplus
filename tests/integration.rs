use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("lsplus"));
}

#[test]
fn test_invalid_path() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("/path/that/does/not/exist")
        .assert()
        .success() // The program handles errors internally
        .stderr(predicates::str::contains("No such file or directory"));
}

#[test]
fn test_list_current_directory() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.assert().success();
}

#[test]
fn test_config_file() {
    // Create a temporary directory and config file
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.toml");

    // Write an invalid config file
    fs::write(&config_file, "invalid = toml [ content").unwrap();

    // Set the home directory environment variable
    std::env::set_var("HOME", temp_dir.path());

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.assert().success(); // Should use default params when config is invalid
}

#[test]
fn test_long_format() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l")
        .assert()
        .success();
}

#[test]
fn test_multiple_paths() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.args([".", "Cargo.toml"])
        .assert()
        .success();
}
