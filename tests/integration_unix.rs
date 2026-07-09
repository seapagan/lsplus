#![cfg(unix)]

use assert_cmd::Command;
use lsplus::utils::icons::Icon;
use nix::unistd::Uid;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use strip_ansi_escapes::strip_str;
use tempfile::tempdir;

mod common;

use common::{
    command_with_home, has_ansi, run_and_capture, run_and_capture_raw,
};

struct PermissionGuard {
    path: std::path::PathBuf,
}

impl Drop for PermissionGuard {
    fn drop(&mut self) {
        let _ =
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o700));
    }
}

fn create_indicator_fixture() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let child_dir = temp_dir.path().join("child");
    let exec_path = temp_dir.path().join("run.sh");
    let target_path = temp_dir.path().join("target.txt");
    let link_path = temp_dir.path().join("link");

    fs::create_dir(&child_dir).unwrap();
    fs::write(&exec_path, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&exec_path, fs::Permissions::from_mode(0o755))
        .unwrap();
    fs::write(&target_path, "target").unwrap();
    std::os::unix::fs::symlink(&target_path, &link_path).unwrap();

    temp_dir
}

#[test]
fn test_glob_entry_error_reports_stderr_and_lists_matches() {
    if Uid::effective().is_root() {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let readable_file = temp_dir.path().join("visible.txt");
    let unreadable_dir = temp_dir.path().join("private");
    fs::write(&readable_file, "visible").unwrap();
    fs::create_dir(&unreadable_dir).unwrap();
    fs::set_permissions(&unreadable_dir, fs::Permissions::from_mode(0o000))
        .unwrap();
    let _guard = PermissionGuard {
        path: unreadable_dir.clone(),
    };

    let pattern = format!("{}/**/*.txt", temp_dir.path().display());
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg(pattern);
    let (stdout, stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("visible.txt"));
    assert!(stderr.contains("lsplus:"));
    assert!(stderr.contains(temp_dir.path().to_string_lossy().as_ref()));
    assert!(stderr.contains("private"));
}

#[test]
fn test_long_format_permission_display_modes() {
    let home_dir = tempdir().unwrap();
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("script.sh");
    fs::write(&file_path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755))
        .unwrap();

    let mut symbolic = command_with_home(home_dir.path());
    symbolic.arg("-l").arg("--no-icons").arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut symbolic);
    assert!(stdout.contains("-rwxr-xr-x"));
    assert!(!stdout.contains("0755"));

    let mut octal = command_with_home(home_dir.path());
    octal
        .arg("-l")
        .arg("--no-icons")
        .arg("--permissions")
        .arg("octal")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut octal);
    assert!(stdout.contains("- 0755"));
    assert!(!stdout.contains("-rwxr-xr-x"));

    let mut both = command_with_home(home_dir.path());
    both.arg("-l")
        .arg("--no-icons")
        .arg("--permissions")
        .arg("both")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut both);
    assert!(stdout.contains("-rwxr-xr-x 0755"));

    let mut none = command_with_home(home_dir.path());
    none.arg("-l")
        .arg("--no-icons")
        .arg("--permissions")
        .arg("none")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut none);
    assert!(stdout.contains("script.sh"));
    assert!(!stdout.contains("- 0755"));
    assert!(!stdout.contains("-rwxr-xr-x"));
}

#[test]
fn test_recursive_continues_after_unreadable_directory_operand() {
    if Uid::effective().is_root() {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let ok_dir = temp_dir.path().join("ok");
    let blocked_dir = temp_dir.path().join("blocked");
    let later_dir = temp_dir.path().join("later");
    fs::create_dir(&ok_dir).unwrap();
    fs::create_dir(&blocked_dir).unwrap();
    fs::create_dir(&later_dir).unwrap();
    fs::write(ok_dir.join("ok.txt"), "ok").unwrap();
    fs::write(later_dir.join("later.txt"), "later").unwrap();
    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o000))
        .unwrap();
    let _guard = PermissionGuard {
        path: blocked_dir.clone(),
    };

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    let output = cmd
        .arg("-R")
        .arg("--no-icons")
        .arg(&ok_dir)
        .arg(&blocked_dir)
        .arg(&later_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = strip_str(String::from_utf8_lossy(&output.stdout));
    let stderr = strip_str(String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains(&format!("{}:", ok_dir.display())));
    assert!(stdout.contains("ok.txt"));
    assert!(stdout.contains(&format!("{}:", later_dir.display())));
    assert!(stdout.contains("later.txt"));
    assert!(stderr.contains("lsplus:"));
    assert!(stderr.contains(blocked_dir.to_string_lossy().as_ref()));
}

#[test]
fn test_recursive_filter_reports_unreadable_root() {
    if Uid::effective().is_root() {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let blocked_dir = temp_dir.path().join("blocked");
    fs::create_dir(&blocked_dir).unwrap();
    fs::set_permissions(&blocked_dir, fs::Permissions::from_mode(0o000))
        .unwrap();
    let _guard = PermissionGuard {
        path: blocked_dir.clone(),
    };
    let pattern = format!("{}/*.rs", blocked_dir.display());

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    let output = cmd
        .arg("-R")
        .arg("--no-icons")
        .arg(pattern)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = strip_str(String::from_utf8_lossy(&output.stderr));
    assert!(stderr.contains("lsplus:"));
    assert!(stderr.contains(blocked_dir.to_string_lossy().as_ref()));
}

#[test]
fn test_long_format_renders_symlink_icon() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link.txt");
    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let mut cmd = command_with_home(temp_dir.path());
    cmd.arg("-l").arg(&link);
    let (stdout_raw, _stderr) = run_and_capture_raw(&mut cmd);
    let stdout = strip_str(&stdout_raw).to_string();

    assert!(stdout.contains(&Icon::Symlink.to_string()));
    assert!(stdout.contains("link.txt"));
    assert!(stdout.contains("->"));
    assert!(!has_ansi(&stdout_raw));
}

#[test]
fn test_broken_symlink_argument_long_format() {
    let temp_dir = tempdir().unwrap();
    let broken_symlink = temp_dir.path().join("broken_link");

    std::os::unix::fs::symlink("missing-target", &broken_symlink).unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l")
        .arg(&broken_symlink)
        .assert()
        .success()
        .stdout(predicates::str::contains("[Broken Link]"))
        .stdout(predicates::str::contains("broken_link"));
}

#[test]
fn test_native_file_type_and_classify_indicator_output() {
    let temp_dir = create_indicator_fixture();

    let mut file_type = command_with_home(temp_dir.path());
    file_type
        .arg("--file-type")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (file_type_stdout, _stderr) = run_and_capture(&mut file_type);

    assert!(file_type_stdout.contains("child/"));
    assert!(file_type_stdout.contains("link@"));
    assert!(file_type_stdout.contains("run.sh"));
    assert!(!file_type_stdout.contains("run.sh*"));

    let mut classify = command_with_home(temp_dir.path());
    classify.arg("-F").arg("--no-icons").arg(temp_dir.path());
    let (classify_stdout, _stderr) = run_and_capture(&mut classify);

    assert!(classify_stdout.contains("child/"));
    assert!(classify_stdout.contains("link@"));
    assert!(classify_stdout.contains("run.sh*"));
}

#[test]
fn test_native_long_mode_omits_symlink_at_indicator() {
    let temp_dir = create_indicator_fixture();

    let mut cmd = command_with_home(temp_dir.path());
    cmd.arg("-l")
        .arg("--file-type")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("link -> "));
    assert!(!stdout.contains("link@ -> "));
}

#[test]
fn test_native_no_indicators_overrides_config_indicator_style() {
    let temp_dir = create_indicator_fixture();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "indicator_style = \"classify\"\n",
    )
    .unwrap();

    let mut cmd = command_with_home(temp_dir.path());
    cmd.arg("--no-indicators")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("child"));
    assert!(stdout.contains("link"));
    assert!(stdout.contains("run.sh"));
    assert!(!stdout.contains("child/"));
    assert!(!stdout.contains("link@"));
    assert!(!stdout.contains("run.sh*"));
}

#[test]
fn test_gnu_compat_mode_accepts_file_type_and_classify_output() {
    let temp_dir = create_indicator_fixture();

    let mut file_type = Command::cargo_bin("lsp").unwrap();
    file_type
        .env("LSP_COMPAT_MODE", "gnu")
        .arg("--file-type")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (file_type_stdout, _stderr) = run_and_capture(&mut file_type);

    assert!(file_type_stdout.contains("child/"));
    assert!(file_type_stdout.contains("link@"));
    assert!(!file_type_stdout.contains("run.sh*"));

    let mut classify = Command::cargo_bin("lsp").unwrap();
    classify
        .env("LSP_COMPAT_MODE", "gnu")
        .arg("-F")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (classify_stdout, _stderr) = run_and_capture(&mut classify);

    assert!(classify_stdout.contains("child/"));
    assert!(classify_stdout.contains("link@"));
    assert!(classify_stdout.contains("run.sh*"));
}

#[test]
fn test_gnu_compat_mode_omits_symlink_at_indicator_in_long_mode() {
    let temp_dir = create_indicator_fixture();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("-l")
        .arg("--file-type")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("link -> "));
    assert!(!stdout.contains("link@ -> "));
    assert!(stdout.contains("target.txt"));
}

#[test]
fn test_gnu_compat_mode_accepts_indicator_style_none() {
    let temp_dir = create_indicator_fixture();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--indicator-style=none")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("child"));
    assert!(stdout.contains("link"));
    assert!(stdout.contains("run.sh"));
    assert!(!stdout.contains("child/"));
    assert!(!stdout.contains("link@"));
    assert!(!stdout.contains("run.sh*"));
}
