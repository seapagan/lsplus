//! Terminal color-mode detection and global color configuration.
//!
//! Short-format names use the `colored_text` global color mode. Long-format
//! accent colors also need a capability level so table style specs and
//! timestamp gradients can choose named, ANSI 256, or truecolor output.

use colored_text::{ColorMode, ColorizeConfig};
use std::env;
use std::io::{self, IsTerminal};

use crate::Params;

/// Return the global color mode implied by runtime parameters.
pub(crate) fn color_mode_for(params: &Params) -> ColorMode {
    if params.no_color {
        ColorMode::Never
    } else {
        ColorMode::Auto
    }
}

/// Apply the runtime color setting to the process-wide coloring backend.
pub(crate) fn configure_color_output(params: &Params) {
    ColorizeConfig::set_color_mode(color_mode_for(params));
}

/// Terminal color capability for long-format accent rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LongFormatColorLevel {
    /// Do not emit long-format accent colors.
    None,
    /// Use named ANSI colors only.
    Named,
    /// Use ANSI 256-color escape sequences.
    Ansi256,
    /// Use RGB truecolor escape sequences.
    Truecolor,
}

impl LongFormatColorLevel {
    /// Return true when long-format accents may emit color.
    pub(crate) fn is_enabled(self) -> bool {
        self != Self::None
    }
}

/// Detect the color capability for long-format accents.
///
/// This respects `--no-color`, `NO_COLOR`, the configured `colored_text` mode,
/// stdout terminal detection, and common `COLORTERM`/`TERM` capability names.
pub(crate) fn long_format_color_level(
    params: &Params,
) -> LongFormatColorLevel {
    if !color_output_enabled(params) {
        return LongFormatColorLevel::None;
    }

    if env_contains_truecolor("COLORTERM") || env_contains_truecolor("TERM") {
        LongFormatColorLevel::Truecolor
    } else if env_contains_256color("COLORTERM")
        || env_contains_256color("TERM")
    {
        LongFormatColorLevel::Ansi256
    } else {
        LongFormatColorLevel::Named
    }
}

fn color_output_enabled(params: &Params) -> bool {
    if params.no_color {
        return false;
    }

    match ColorizeConfig::color_mode() {
        ColorMode::Never => false,
        ColorMode::Always => env::var_os("NO_COLOR").is_none(),
        ColorMode::Auto => {
            env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
        }
    }
}

fn env_contains_truecolor(name: &str) -> bool {
    env::var(name)
        .map(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("truecolor") || value.contains("24bit")
        })
        .unwrap_or(false)
}

fn env_contains_256color(name: &str) -> bool {
    env::var(name)
        .map(|value| value.to_ascii_lowercase().contains("256color"))
        .unwrap_or(false)
}
