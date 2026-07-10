//! Platform-specific filesystem metadata interpretation.

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub(crate) use unix::*;
