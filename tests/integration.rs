use assert_cmd::Command;
use filetime::FileTime;
use lsplus::utils::icons::Icon;
use std::fs;
use std::time::{Duration, SystemTime};
use strip_ansi_escapes::strip_str;
use tempfile::tempdir;

fn run_and_capture(cmd: &mut Command) -> (String, String) {
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = strip_str(String::from_utf8_lossy(&output.stdout)).to_owned();
    let stderr = strip_str(String::from_utf8_lossy(&output.stderr)).to_owned();
    (stdout, stderr)
}

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
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("alpha.txt"), "alpha").unwrap();
    fs::write(temp_dir.path().join("beta.txt"), "beta").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.current_dir(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("alpha.txt"));
    assert!(stdout.contains("beta.txt"));
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

    // Set the home directory environment variable temporarily
    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.assert().success(); // Should use default params when config is invalid
    });
}

#[test]
fn test_long_format() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("size.txt");
    fs::write(&file_path, vec![b'x'; 2048]).unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l").arg("-h").arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("size.txt"));
    assert!(stdout.contains("2 KB"));
}

#[test]
fn test_multiple_paths() {
    let temp_dir = tempdir().unwrap();
    let left = temp_dir.path().join("left.txt");
    let right = temp_dir.path().join("right.txt");
    fs::write(&left, "left").unwrap();
    fs::write(&right, "right").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg(&left).arg(&right);
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("left.txt"));
    assert!(stdout.contains("right.txt"));
}

#[test]
fn test_no_icons_omits_file_icons() {
    let temp_dir = tempdir().unwrap();
    let rust_file = temp_dir.path().join("example.rs");
    fs::write(&rust_file, "fn main() {}").unwrap();

    let mut with_icons = Command::cargo_bin("lsp").unwrap();
    with_icons.arg(&rust_file);
    let (stdout_with_icons, _stderr) = run_and_capture(&mut with_icons);

    let mut without_icons = Command::cargo_bin("lsp").unwrap();
    without_icons.arg("--no-icons").arg(&rust_file);
    let (stdout_without_icons, _stderr) = run_and_capture(&mut without_icons);

    assert!(stdout_with_icons.contains(""));
    assert!(!stdout_without_icons.contains(""));
    assert!(stdout_without_icons.contains("example.rs"));
}

#[test]
fn test_short_output_handles_wide_filename_without_panicking() {
    let temp_dir = tempdir().unwrap();
    let wide_name = format!("{}.txt", "界".repeat(50));
    let wide_file = temp_dir.path().join(&wide_name);
    fs::write(&wide_file, "wide").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("--no-icons").arg(&wide_file);
        let (stdout, _stderr) = run_and_capture(&mut cmd);

        assert!(stdout.contains(&wide_name));
    });
}

#[test]
fn test_dirs_first_lists_directories_before_files() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("zeta_dir")).unwrap();
    fs::write(temp_dir.path().join("alpha.txt"), "alpha").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-D").arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    let dir_position = stdout.find("zeta_dir").unwrap();
    let file_position = stdout.find("alpha.txt").unwrap();

    assert!(dir_position < file_position);
}

#[test]
fn test_fuzzy_time_uses_human_readable_timestamp() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("aged.txt");
    fs::write(&file_path, "aged").unwrap();

    let old_time = FileTime::from_system_time(
        SystemTime::now()
            .checked_sub(Duration::from_secs(2 * 60 * 60))
            .unwrap(),
    );
    filetime::set_file_mtime(&file_path, old_time).unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l").arg("-Z").arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("aged.txt"));
    assert!(stdout.contains("2 hours ago"));
}

#[test]
fn test_long_format_handles_wide_filename_rows() {
    let temp_dir = tempdir().unwrap();
    let wide_name = "界界界-report.txt";
    let ascii_name = "plain.txt";
    fs::write(temp_dir.path().join(wide_name), "wide").unwrap();
    fs::write(temp_dir.path().join(ascii_name), "plain").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("-l").arg("--no-icons").arg(temp_dir.path());
        let (stdout, _stderr) = run_and_capture(&mut cmd);

        let rows: Vec<_> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|line| line.trim_start().starts_with('-')
            && line.contains(wide_name)));
        assert!(rows.iter().any(|line| line.trim_start().starts_with('-')
            && line.contains(ascii_name)));
    });
}

#[test]
fn test_long_format_renders_hidden_git_icons() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    let gitignore = temp_dir.path().join(".gitignore");
    fs::create_dir(&git_dir).unwrap();
    fs::write(&gitignore, "*.log\n").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("-l").arg("-a").arg(temp_dir.path());
        let (stdout, _stderr) = run_and_capture(&mut cmd);

        let rows: Vec<_> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let git_icon = Icon::GitFile.to_string();

        assert!(rows.iter().any(
            |line| line.contains(&git_icon) && line.contains(".gitignore")
        ));
        assert!(
            rows.iter()
                .any(|line| line.contains(&git_icon) && line.contains(".git"))
        );
    });
}

#[cfg(unix)]
#[test]
fn test_long_format_renders_symlink_icon() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link.txt");
    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("-l").arg(&link);
        let (stdout, _stderr) = run_and_capture(&mut cmd);

        assert!(stdout.contains(&Icon::Symlink.to_string()));
        assert!(stdout.contains("link.txt"));
        assert!(stdout.contains("->"));
    });
}

#[cfg(unix)]
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
