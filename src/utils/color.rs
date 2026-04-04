use colored_text::{ColorMode, ColorizeConfig};

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
