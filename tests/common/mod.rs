use assert_cmd::Command;
use lsplus::settings::CONFIG_FILE_ENV_VAR;
use std::path::Path;
use std::process::{Command as StdCommand, Output};
use strip_ansi_escapes::strip_str;

fn run_and_assert(cmd: &mut Command) -> Output {
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

pub(crate) fn run_and_capture(cmd: &mut Command) -> (String, String) {
    let output = run_and_assert(cmd);
    let stdout = strip_str(String::from_utf8_lossy(&output.stdout)).to_owned();
    let stderr = strip_str(String::from_utf8_lossy(&output.stderr)).to_owned();
    (stdout, stderr)
}

pub(crate) fn run_and_capture_raw(cmd: &mut Command) -> (String, String) {
    let output = run_and_assert(cmd);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

pub(crate) fn command_with_home(home: &Path) -> Command {
    Command::from_std(std_command_with_home(home))
}

pub(crate) fn std_command_with_home(home: &Path) -> StdCommand {
    let mut cmd = StdCommand::new(assert_cmd::cargo::cargo_bin("lsp"));
    cmd.env("HOME", home).env(
        CONFIG_FILE_ENV_VAR,
        home.join(".config").join("lsplus").join("config.toml"),
    );
    cmd
}

pub(crate) fn has_ansi(text: &str) -> bool {
    text.contains("\u{1b}[")
}
