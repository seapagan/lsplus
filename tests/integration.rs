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

fn run_and_capture_raw(cmd: &mut Command) -> (String, String) {
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn has_ansi(text: &str) -> bool {
    text.contains("\u{1b}[")
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
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-D")
        .arg(temp_dir.path());
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
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-l")
        .arg("-Z")
        .arg(&file_path);
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
fn test_long_format_does_not_pad_short_rows_to_longest_filename() {
    let temp_dir = tempdir().unwrap();
    let short_name = "plain.txt";
    let long_name =
        "this-is-a-very-long-filename-that-should-not-pad-other-rows.txt";
    fs::write(temp_dir.path().join(short_name), "plain").unwrap();
    fs::write(temp_dir.path().join(long_name), "long").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l").arg("--no-icons").arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    let short_row = stdout
        .lines()
        .find(|line| line.contains(short_name))
        .unwrap();

    assert!(short_row.ends_with(short_name));
}

#[test]
fn test_captured_long_format_output_is_plain_by_default() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("plain.txt");
    fs::write(&file_path, "plain").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("-l").arg("--no-icons").arg(&file_path);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    assert!(!has_ansi(&stdout));
}

#[test]
fn test_no_color_flag_keeps_short_output_plain() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("plain.txt");
    fs::write(&file_path, "plain").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-N")
        .arg("--no-icons")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    assert!(!has_ansi(&stdout));
    assert!(stdout.contains("plain.txt"));
}

#[test]
fn test_no_color_config_keeps_long_output_plain() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    let file_path = temp_dir.path().join("plain.txt");

    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "no_color = true\n").unwrap();
    fs::write(&file_path, "plain").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("-l").arg("--no-icons").arg(&file_path);
        let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

        assert!(!has_ansi(&stdout));
        assert!(stdout.contains("plain.txt"));
    });
}

#[test]
fn test_short_format_does_not_pad_short_rows_to_longest_filename() {
    let temp_dir = tempdir().unwrap();
    let short_name = "plain.txt";
    let long_name = "this-is-a-very-long-filename-that-forces-one-column.txt";
    fs::write(temp_dir.path().join(short_name), "plain").unwrap();
    fs::write(temp_dir.path().join(long_name), "long").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.arg("--no-icons").arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    let short_row = stdout
        .lines()
        .find(|line| line.contains(short_name))
        .unwrap();

    assert_eq!(short_row, " plain.txt  ");
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

#[test]
fn test_gitignore_flag_keeps_captured_short_output_plain() {
    let temp_dir = tempdir().unwrap();
    let ignored_name =
        "ignored-entry-name-that-forces-single-column-output.log";
    let visible_name =
        "visible-entry-name-that-forces-single-column-output.txt";
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(temp_dir.path().join(ignored_name), "ignored").unwrap();
    fs::write(temp_dir.path().join(visible_name), "visible").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(ignored_name))
        .unwrap();
    let visible_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(visible_name))
        .unwrap();

    assert!(!has_ansi(ignored_line));
    assert!(!has_ansi(visible_line));
}

#[test]
fn test_gitignore_flag_keeps_captured_long_output_plain() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(temp_dir.path().join("ignored.log"), "ignored").unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "visible").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-l")
        .arg("-I")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| strip_str(line).contains("ignored.log"))
        .unwrap();
    let visible_line = stdout
        .lines()
        .find(|line| strip_str(line).contains("visible.txt"))
        .unwrap();

    assert!(!has_ansi(ignored_line));
    assert!(!has_ansi(visible_line));
}

#[test]
fn test_gitignore_flag_honors_nested_unignore_rules() {
    let temp_dir = tempdir().unwrap();
    let nested_dir = temp_dir.path().join("nested");
    let ignored_name =
        "ignored-entry-name-that-forces-single-column-output.log";
    let kept_name = "keep-entry-name-that-forces-single-column-output.log";
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::create_dir(&nested_dir).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(nested_dir.join(".gitignore"), format!("!{kept_name}\n"))
        .unwrap();
    fs::write(nested_dir.join(ignored_name), "ignored").unwrap();
    fs::write(nested_dir.join(kept_name), "kept").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(&nested_dir);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(ignored_name))
        .unwrap();
    let kept_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(kept_name))
        .unwrap();

    assert!(!has_ansi(ignored_line));
    assert!(!has_ansi(kept_line));
}

#[test]
fn test_gitignore_flag_dims_explicit_file_arguments() {
    let temp_dir = tempdir().unwrap();
    let ignored_file = temp_dir.path().join("ignored.log");
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(&ignored_file, "ignored").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(&ignored_file);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| strip_str(line).contains("ignored.log"))
        .unwrap();

    assert!(!has_ansi(ignored_line));
}

#[test]
fn test_gitignore_flag_does_not_dim_outside_git_worktree() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("plain.log"), "plain").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let plain_line = stdout
        .lines()
        .find(|line| strip_str(line).contains("plain.log"))
        .unwrap();

    assert!(!has_ansi(plain_line));
}

#[test]
fn test_gitignore_flag_honors_git_info_exclude() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    let ignored_dir = temp_dir
        .path()
        .join("build-directory-that-forces-single-column-output");
    let ignored_file = ignored_dir
        .join("ignored-entry-name-that-forces-single-column-output.txt");
    let visible_name =
        "visible-entry-name-that-forces-single-column-output.txt";
    fs::create_dir_all(git_dir.join("info")).unwrap();
    fs::create_dir_all(&ignored_dir).unwrap();
    fs::write(
        git_dir.join("info").join("exclude"),
        "build-directory-that-forces-single-column-output/\n",
    )
    .unwrap();
    fs::write(&ignored_file, "ignored").unwrap();
    fs::write(temp_dir.path().join(visible_name), "visible").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(&ignored_file)
        .arg(temp_dir.path().join(visible_name));
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| {
            strip_str(line)
                .contains(ignored_file.file_name().unwrap().to_str().unwrap())
        })
        .unwrap();
    let visible_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(visible_name))
        .unwrap();

    assert!(!has_ansi(ignored_line));
    assert!(!has_ansi(visible_line));
}

#[test]
fn test_gitignore_flag_honors_global_excludes() {
    let temp_dir = tempdir().unwrap();
    let home_dir = temp_dir.path().join("home");
    let repo_dir = temp_dir.path().join("repo");
    let excludes_file = home_dir.join(".global_ignore");
    let ignored_dir =
        repo_dir.join("build-directory-that-forces-single-column-output");
    let ignored_file = ignored_dir
        .join("ignored-entry-name-that-forces-single-column-output.txt");
    let visible_name =
        "visible-entry-name-that-forces-single-column-output.txt";

    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(repo_dir.join(".git")).unwrap();
    fs::create_dir_all(&ignored_dir).unwrap();
    fs::write(
        home_dir.join(".gitconfig"),
        format!("[core]\n\texcludesFile = {}\n", excludes_file.display()),
    )
    .unwrap();
    fs::write(
        &excludes_file,
        "build-directory-that-forces-single-column-output/\n",
    )
    .unwrap();
    fs::write(&ignored_file, "ignored").unwrap();
    fs::write(repo_dir.join(visible_name), "visible").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.current_dir(&repo_dir)
        .env("HOME", &home_dir)
        .env("LSP_COMPAT_MODE", "native")
        .arg("-I")
        .arg("--no-icons")
        .arg(&ignored_file)
        .arg(repo_dir.join(visible_name));
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    let ignored_line = stdout
        .lines()
        .find(|line| {
            strip_str(line)
                .contains(ignored_file.file_name().unwrap().to_str().unwrap())
        })
        .unwrap();
    let visible_line = stdout
        .lines()
        .find(|line| strip_str(line).contains(visible_name))
        .unwrap();

    assert!(!has_ansi(ignored_line));
    assert!(!has_ansi(visible_line));
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
        let (stdout_raw, _stderr) = run_and_capture_raw(&mut cmd);
        let stdout = strip_str(&stdout_raw).to_string();

        assert!(stdout.contains(&Icon::Symlink.to_string()));
        assert!(stdout.contains("link.txt"));
        assert!(stdout.contains("->"));
        assert!(!has_ansi(&stdout_raw));
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

#[test]
fn test_gnu_compat_mode_from_env_rejects_conflicting_short_flag() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("-D")
        .assert()
        .failure()
        .stderr(predicates::str::contains("unexpected argument '-D'"));
}

#[test]
fn test_gnu_compat_mode_from_config_rejects_conflicting_short_flag() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");

    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "compat_mode = \"gnu\"\n")
        .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.arg("-N")
            .assert()
            .failure()
            .stderr(predicates::str::contains("unexpected argument '-N'"));
    });
}

#[test]
fn test_invalid_env_compat_mode_fails_startup() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "bogus")
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid LSP_COMPAT_MODE value"))
        .stderr(predicates::str::contains("bogus"));
}

#[test]
fn test_invalid_config_compat_mode_fails_startup() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");

    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "compat_mode = \"bogus\"\n")
        .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("invalid compat_mode setting"))
            .stderr(predicates::str::contains("bogus"));
    });
}

#[test]
fn test_env_compat_mode_overrides_config_mode() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");

    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "compat_mode = \"gnu\"\n")
        .unwrap();
    fs::create_dir(temp_dir.path().join("zeta_dir")).unwrap();
    fs::write(temp_dir.path().join("alpha.txt"), "alpha").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let mut cmd = Command::cargo_bin("lsp").unwrap();
        cmd.env("LSP_COMPAT_MODE", "native")
            .arg("-D")
            .arg(temp_dir.path());
        let (stdout, _stderr) = run_and_capture(&mut cmd);

        let dir_position = stdout.find("zeta_dir").unwrap();
        let file_position = stdout.find("alpha.txt").unwrap();

        assert!(dir_position < file_position);
    });
}

#[test]
fn test_gnu_compat_mode_help_omits_conflicting_short_flags() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu").arg("--help");
    let output = cmd.output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-p"));
    assert!(stdout.contains("Append / indicator to directories"));
    assert!(stdout.contains("--indicator-style=<WORD>"));
    assert!(
        stdout.contains("Append indicator with style WORD to entry names")
    );
    assert!(stdout.contains("--group-directories-first"));
    assert!(!stdout.contains("--slash-dirs"));
    assert!(!stdout.contains("-D,"));
    assert!(!stdout.contains("-I,"));
    assert!(!stdout.contains("-N,"));
    assert!(!stdout.contains("-Z,"));
}

#[test]
fn test_gnu_compat_mode_accepts_group_directories_first_long_option() {
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("zeta_dir")).unwrap();
    fs::write(temp_dir.path().join("alpha.txt"), "alpha").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--group-directories-first")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    let dir_position = stdout.find("zeta_dir").unwrap();
    let file_position = stdout.find("alpha.txt").unwrap();

    assert!(dir_position < file_position);
}

#[test]
fn test_gnu_compat_mode_rejects_native_sort_dirs_long_option() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--sort-dirs")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "unexpected argument '--sort-dirs'",
        ));
}

#[test]
fn test_gnu_compat_mode_accepts_no_color_long_option() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("plain.txt");
    fs::write(&file_path, "plain").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--no-color")
        .arg("--no-icons")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    assert!(!has_ansi(&stdout));
    assert!(stdout.contains("plain.txt"));
}

#[test]
fn test_gnu_compat_mode_accepts_gitignore_long_option() {
    let temp_dir = tempdir().unwrap();
    let ignored_file = temp_dir.path().join("ignored.log");
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(&ignored_file, "ignored").unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--gitignore")
        .arg("--no-icons")
        .arg(&ignored_file);
    let (stdout, _stderr) = run_and_capture_raw(&mut cmd);

    assert!(stdout.contains("ignored.log"));
}

#[test]
fn test_gnu_compat_mode_accepts_fuzzy_time_long_option() {
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
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("-l")
        .arg("--fuzzy-time")
        .arg(&file_path);
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("aged.txt"));
    assert!(stdout.contains("2 hours ago"));
}

#[test]
fn test_gnu_compat_mode_accepts_indicator_style_slash() {
    let temp_dir = tempdir().unwrap();
    let child_dir = temp_dir.path().join("child");
    fs::create_dir(&child_dir).unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--indicator-style=slash")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("child/"));
}

#[test]
fn test_gnu_compat_mode_accepts_short_p() {
    let temp_dir = tempdir().unwrap();
    let child_dir = temp_dir.path().join("child");
    fs::create_dir(&child_dir).unwrap();

    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("-p")
        .arg("--no-icons")
        .arg(temp_dir.path());
    let (stdout, _stderr) = run_and_capture(&mut cmd);

    assert!(stdout.contains("child/"));
}

#[test]
fn test_gnu_compat_mode_rejects_native_slash_dirs_long_option() {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("LSP_COMPAT_MODE", "gnu")
        .arg("--slash-dirs")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "unexpected argument '--slash-dirs'",
        ));
}
