//! Terminal color-mode detection and thread-local color configuration.
//!
//! Short-format names use the `colored_text` thread-local color mode.
//! Long-format accent colors use the same terminal capability detection.

use colored_text::{ColorLevel, ColorMode, ColorizeConfig, RenderTarget};

use crate::Params;

/// Return the color mode implied by runtime parameters.
pub(crate) fn color_mode_for(params: &Params) -> ColorMode {
    if params.no_color {
        ColorMode::Never
    } else {
        ColorMode::Auto
    }
}

/// Apply the runtime color setting to the current thread.
pub(crate) fn configure_color_output(params: &Params) {
    ColorizeConfig::set_color_mode(color_mode_for(params));
}

/// Detect the color capability for long-format accents.
///
/// `params.no_color` is kept as an explicit guard so direct calls remain safe
/// even before the thread-local color configuration has been applied on the
/// calling thread.
pub(crate) fn long_format_color_level(params: &Params) -> ColorLevel {
    if params.no_color {
        return ColorLevel::NoColor;
    }

    ColorizeConfig::color_level(RenderTarget::Stdout)
}
