use colored_text::{ColorMode, ColorizeConfig};
use std::sync::{Mutex, MutexGuard};

use crate::Params;

static COLOR_MODE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct ColorModeGuard {
    previous: ColorMode,
    _lock: MutexGuard<'static, ()>,
}

impl ColorModeGuard {
    pub(crate) fn set(mode: ColorMode) -> Self {
        let lock = COLOR_MODE_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = ColorizeConfig::color_mode();
        ColorizeConfig::set_color_mode(mode);
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for ColorModeGuard {
    fn drop(&mut self) {
        ColorizeConfig::set_color_mode(self.previous);
    }
}

pub(crate) fn has_ansi(text: &str) -> bool {
    text.contains("\u{1b}[")
}

pub(crate) fn with_color_output_enabled<T>(test: impl FnOnce() -> T) -> T {
    with_color_environment(None, None, ColorMode::Always, test)
}

pub(crate) fn with_color_environment<T>(
    term: Option<&str>,
    colorterm: Option<&str>,
    mode: ColorMode,
    test: impl FnOnce() -> T,
) -> T {
    temp_env::with_vars(
        [
            ("NO_COLOR", None),
            ("FORCE_COLOR", None),
            ("CLICOLOR", None),
            ("CLICOLOR_FORCE", None),
            ("TERM", term),
            ("COLORTERM", colorterm),
            ("WT_SESSION", None),
            ("ConEmuANSI", None),
            ("ANSICON", None),
            ("CI", None),
        ],
        || {
            let _guard = ColorModeGuard::set(mode);
            test()
        },
    )
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
