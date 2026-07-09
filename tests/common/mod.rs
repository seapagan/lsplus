use assert_cmd::Command;
use std::path::Path;
use strip_ansi_escapes::strip_str;

pub(crate) fn run_and_capture(cmd: &mut Command) -> (String, String) {
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

pub(crate) fn run_and_capture_raw(cmd: &mut Command) -> (String, String) {
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

pub(crate) fn command_with_home(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("lsp").unwrap();
    cmd.env("HOME", home);
    cmd
}

pub(crate) fn has_ansi(text: &str) -> bool {
    text.contains("\u{1b}[")
}
