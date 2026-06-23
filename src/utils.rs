//! Utility modules used by the `lsplus` runtime.
//!
//! These modules handle filesystem inspection, terminal rendering, color
//! selection, icon lookup, and small formatting helpers shared by the CLI app.

pub mod color;
pub mod file;
pub mod format;
pub mod fuzzy_time;
pub mod gitignore;
pub mod icons;
pub mod render;
pub mod table;
pub(crate) mod time;

pub use fuzzy_time::fuzzy_time;
