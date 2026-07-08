//! Library entry points for `lsplus`.
//!
//! The crate exposes the runtime building blocks behind the `lsp` binary,
//! including CLI parsing, startup configuration, and file-listing helpers.
//! `lsplus` supports both its native CLI surface and a GNU compatibility mode
//! for users who want `ls`-style option parsing.

pub mod app;
pub mod cli;
mod platform;
pub mod settings;
pub mod structs;
pub mod utils;

pub use structs::{FileInfo, IndicatorStyle, NameStyle, Params};

#[cfg(test)]
#[path = "../tests/crate/app.rs"]
mod app_tests;
#[cfg(test)]
#[path = "../tests/crate/cli.rs"]
mod cli_tests;
#[cfg(test)]
#[path = "../tests/crate/common.rs"]
mod common_tests;
#[cfg(test)]
#[path = "../tests/crate/file.rs"]
mod file_tests;
#[cfg(all(test, unix))]
#[path = "../tests/crate/file_unix.rs"]
mod file_unix_tests;
#[cfg(test)]
#[path = "../tests/crate/gitignore.rs"]
mod gitignore_tests;
#[cfg(test)]
#[path = "../tests/crate/icons.rs"]
mod icons_tests;
#[cfg(test)]
#[path = "../tests/crate/render.rs"]
mod render_tests;
#[cfg(test)]
#[path = "../tests/crate/settings.rs"]
mod settings_tests;
