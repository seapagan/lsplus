#![cfg(windows)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;

fn create_junction(link: &std::path::Path, target: &std::path::Path) {
    let command = format!(
        "mklink /J \"{}\" \"{}\"",
        command_path(link),
        command_path(target)
    );
    let output = ProcessCommand::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(command_path(link))
        .arg(command_path(target))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "mklink command {command:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn command_path(path: &std::path::Path) -> String {
    let path = path.to_string_lossy();
    path.strip_prefix(r"\\?\").unwrap_or(&path).to_string()
}

#[test]
fn test_windows_all_does_not_synthesize_dot_entries() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join(".visible-dotfile"), "visible").unwrap();

    let mut command = Command::cargo_bin("lsp").unwrap();
    command.arg("--all").arg("--no-icons").arg(temp_dir.path());
    let output = command.output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8_lossy(&output.stdout);
    let names = output.split_whitespace().collect::<Vec<_>>();
    assert_eq!(names, vec![".visible-dotfile"]);
}

#[test]
fn test_windows_hidden_attribute_requires_all() {
    let temp_dir = tempdir().unwrap();
    let hidden = temp_dir.path().join("hidden.txt");
    fs::write(&hidden, "hidden").unwrap();
    assert!(
        ProcessCommand::new("attrib")
            .arg("+h")
            .arg(&hidden)
            .status()
            .unwrap()
            .success()
    );

    let mut default_listing = Command::cargo_bin("lsp").unwrap();
    default_listing
        .arg("--no-icons")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden.txt").not());

    let mut all_listing = Command::cargo_bin("lsp").unwrap();
    all_listing
        .arg("--all")
        .arg("--no-icons")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden.txt"));
}

#[test]
fn test_windows_classify_uses_pathext_not_script_extension() {
    let temp_dir = tempdir().unwrap();
    let executable = temp_dir.path().join("tool.EXE");
    let script = temp_dir.path().join("script.ps1");
    fs::write(&executable, "not a real program").unwrap();
    fs::write(&script, "Write-Output test").unwrap();

    let mut command = Command::cargo_bin("lsp").unwrap();
    command
        .arg("--classify")
        .arg("--no-icons")
        .arg(&executable)
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("tool.EXE*"))
        .stdout(predicate::str::contains("script.ps1*").not());
}

#[test]
fn test_windows_long_permissions_octal_is_rejected_only_in_long_mode() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("sample.txt");
    fs::write(&file, "sample").unwrap();

    let mut short = Command::cargo_bin("lsp").unwrap();
    short
        .arg("--permissions")
        .arg("octal")
        .arg("--no-icons")
        .arg(&file)
        .assert()
        .success();

    let mut long = Command::cargo_bin("lsp").unwrap();
    long.arg("--long")
        .arg("--permissions")
        .arg("octal")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Windows does not support octal permission display",
        ));
}

#[test]
fn test_windows_long_short_attributes_uses_compact_field() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("sample.txt");
    fs::write(&file, "sample").unwrap();

    let mut command = Command::cargo_bin("lsp").unwrap();
    let output = command
        .arg("--long")
        .arg("--attributes")
        .arg("short")
        .arg("--no-icons")
        .arg("--no-color")
        .arg(&file)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let compact = stdout
        .split_whitespace()
        .find(|field| {
            field.chars().count() == 17
                && field
                    .chars()
                    .all(|character| "RHSATPCONEIVBXQGF-".contains(character))
        })
        .unwrap();

    assert_eq!(compact.chars().count(), 17);
    assert!(stdout.contains("sample.txt"));
}

#[test]
fn test_windows_short_listing_ignores_attribute_display() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("sample.txt");
    fs::write(&file, "sample").unwrap();

    let mut default = Command::cargo_bin("lsp").unwrap();
    default.arg("--no-icons").arg("--no-color").arg(&file);
    let default_output = default.output().unwrap();

    let mut compact = Command::cargo_bin("lsp").unwrap();
    compact
        .arg("--attributes")
        .arg("short")
        .arg("--no-icons")
        .arg("--no-color")
        .arg(&file);
    let compact_output = compact.output().unwrap();

    assert_eq!(compact_output.stdout, default_output.stdout);
}

#[test]
fn test_windows_permissions_none_omits_compact_attributes() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("sample.txt");
    fs::write(&file, "sample").unwrap();

    let mut command = Command::cargo_bin("lsp").unwrap();
    command
        .arg("--long")
        .arg("--header")
        .arg("--permissions")
        .arg("none")
        .arg("--attributes")
        .arg("short")
        .arg("--no-icons")
        .arg("--no-color")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Attributes").not());
}

#[test]
fn test_windows_junction_listing_and_recursion_behavior() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target");
    let parent = temp_dir.path().join("parent");
    let junction = parent.join("linked-target");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&parent).unwrap();
    fs::write(target.join("inside.txt"), "target content").unwrap();
    create_junction(&junction, &target);
    let junction_target = format!("linked-target -> {}", target.display());

    let mut direct = Command::cargo_bin("lsp").unwrap();
    direct
        .arg("--no-icons")
        .arg(&junction)
        .assert()
        .success()
        .stdout(predicate::str::contains("inside.txt"));

    let mut listed = Command::cargo_bin("lsp").unwrap();
    listed
        .arg("--long")
        .arg("--no-icons")
        .arg(&parent)
        .assert()
        .success()
        .stdout(predicate::str::contains("linked-target"))
        .stdout(predicate::str::contains("j"))
        .stdout(predicate::str::contains(junction_target))
        .stdout(predicate::str::contains("[Target Unavailable]").not());

    let mut slash = Command::cargo_bin("lsp").unwrap();
    slash
        .arg("--slash-dirs")
        .arg("--no-icons")
        .arg(&parent)
        .assert()
        .success()
        .stdout(predicate::str::contains("linked-target"))
        .stdout(predicate::str::contains("linked-target/").not());

    let mut file_type = Command::cargo_bin("lsp").unwrap();
    file_type
        .arg("--file-type")
        .arg("--no-icons")
        .arg(&parent)
        .assert()
        .success()
        .stdout(predicate::str::contains("linked-target@"));

    let mut recursive = Command::cargo_bin("lsp").unwrap();
    recursive
        .arg("--recursive")
        .arg("--no-icons")
        .arg(&parent)
        .assert()
        .success()
        .stdout(predicate::str::contains("linked-target"))
        .stdout(predicate::str::contains("inside.txt").not());
}
