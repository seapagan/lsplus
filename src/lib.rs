pub mod app;
pub mod cli;
pub mod settings;
pub mod structs;
pub mod utils;

pub use structs::{FileInfo, Params};

#[cfg(test)]
#[path = "../tests/crate/app.rs"]
mod app_tests;
#[cfg(test)]
#[path = "../tests/crate/cli.rs"]
mod cli_tests;
#[cfg(test)]
#[path = "../tests/crate/file.rs"]
mod file_tests;
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
