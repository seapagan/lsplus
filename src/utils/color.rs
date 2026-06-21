use colored_text::{ColorMode, ColorizeConfig};
use std::env;
use std::io::{self, IsTerminal};

use crate::Params;

pub(crate) fn color_mode_for(params: &Params) -> ColorMode {
    if params.no_color {
        ColorMode::Never
    } else {
        ColorMode::Auto
    }
}

pub(crate) fn configure_color_output(params: &Params) {
    ColorizeConfig::set_color_mode(color_mode_for(params));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LongFormatColorLevel {
    None,
    Named,
    Ansi256,
    Truecolor,
}

impl LongFormatColorLevel {
    pub(crate) fn is_enabled(self) -> bool {
        self != Self::None
    }
}

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
