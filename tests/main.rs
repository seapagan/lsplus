use assert_cmd::Command;
#[cfg(unix)]
use nix::unistd::Uid;
use predicates::str::contains;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn test_main_exits_with_error_when_app_returns_err() {
    if Uid::effective().is_root() {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let blocked_dir = temp_dir.path().join("blocked");
    fs::create_dir(&blocked_dir).unwrap();
    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o000))
        .unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg(&blocked_dir)
        .assert()
        .failure()
        .stderr(contains("Error:"));

    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o755))
        .unwrap();
}
