use colored_text::{ColorMode, ColorizeConfig};

use crate::Params;

pub(crate) struct ColorModeGuard(ColorMode);

impl ColorModeGuard {
    pub(crate) fn set(mode: ColorMode) -> Self {
        let previous = ColorizeConfig::color_mode();
        ColorizeConfig::set_color_mode(mode);
        Self(previous)
    }
}

impl Drop for ColorModeGuard {
    fn drop(&mut self) {
        ColorizeConfig::set_color_mode(self.0);
    }
}

pub(crate) fn has_ansi(text: &str) -> bool {
    text.contains("\u{1b}[")
}

pub(crate) fn with_color_output_enabled<T>(test: impl FnOnce() -> T) -> T {
    temp_env::with_var("NO_COLOR", None::<&str>, || {
        let _guard = ColorModeGuard::set(ColorMode::Always);
        test()
    })
}

pub(crate) fn fixed_time_params() -> Params {
    Params {
        time_gradient: false,
        ..Params::default()
    }
}

pub(crate) fn plain_permission_params() -> Params {
    Params {
        permission_colors: false,
        time_gradient: false,
        ..Params::default()
    }
}

pub(crate) fn accentless_params() -> Params {
    Params {
        permission_colors: false,
        time_gradient: false,
        size_colors: false,
        ..Params::default()
    }
}

pub(crate) fn time_only_params() -> Params {
    Params {
        permission_colors: false,
        size_colors: false,
        ..Params::default()
    }
}
