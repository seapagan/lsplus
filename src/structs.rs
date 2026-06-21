use super::utils::icons::Icon;
use config::Config;
use serde::Deserialize;
use std::convert::From;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::cli;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum IndicatorStyle {
    #[default]
    None,
    Slash,
    FileType,
    Classify,
}

#[derive(Debug, PartialEq)]
pub struct Params {
    /// Show entries whose names start with `.`.
    pub show_all: bool,
    /// Select which file indicator style to render.
    pub indicator_style: IndicatorStyle,
    /// Group directories before files.
    pub dirs_first: bool,
    /// Hide `.` and `..` while still showing other dotfiles.
    pub almost_all: bool,
    /// Render long-format output.
    pub long_format: bool,
    /// Render human-readable file sizes in long format.
    pub human_readable: bool,
    /// Disable file and directory icons.
    pub no_icons: bool,
    /// Disable colored or styled output.
    pub no_color: bool,
    /// Color file type and permission bits in long-format output.
    pub permission_colors: bool,
    /// Color timestamps by freshness in long-format output.
    pub time_colors: bool,
    /// Color large sizes in long-format output.
    pub size_colors: bool,
    /// Dim paths matched by `.gitignore` rules.
    pub gitignore: bool,
    /// Render humanized relative timestamps.
    pub fuzzy_time: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            show_all: false,
            indicator_style: IndicatorStyle::None,
            dirs_first: false,
            almost_all: false,
            long_format: false,
            human_readable: false,
            no_icons: false,
            no_color: false,
            permission_colors: true,
            time_colors: true,
            size_colors: true,
            gitignore: false,
            fuzzy_time: false,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(default)]
pub(crate) struct RawParams {
    show_all: bool,
    dirs_first: bool,
    almost_all: bool,
    long_format: bool,
    human_readable: bool,
    no_icons: bool,
    no_color: bool,
    permission_colors: Option<bool>,
    time_colors: Option<bool>,
    size_colors: Option<bool>,
    gitignore: bool,
    fuzzy_time: bool,
    indicator_style: Option<IndicatorStyle>,
    append_slash: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NameStyle {
    #[default]
    Plain,
    Directory,
    Symlink,
    Executable,
}

impl From<Config> for Params {
    fn from(settings: Config) -> Self {
        settings
            .try_deserialize::<RawParams>()
            .map(Into::into)
            .unwrap_or_default()
    }
}

impl From<RawParams> for Params {
    fn from(raw: RawParams) -> Self {
        Self {
            show_all: raw.show_all,
            indicator_style: raw.indicator_style.unwrap_or_else(|| {
                if raw.append_slash.unwrap_or(false) {
                    IndicatorStyle::Slash
                } else {
                    IndicatorStyle::None
                }
            }),
            dirs_first: raw.dirs_first,
            almost_all: raw.almost_all,
            long_format: raw.long_format,
            human_readable: raw.human_readable,
            no_icons: raw.no_icons,
            no_color: raw.no_color,
            permission_colors: raw.permission_colors.unwrap_or(true),
            time_colors: raw.time_colors.unwrap_or(true),
            size_colors: raw.size_colors.unwrap_or(true),
            gitignore: raw.gitignore,
            fuzzy_time: raw.fuzzy_time,
        }
    }
}

impl Params {
    /// Merge parsed CLI flags with config-file defaults.
    ///
    /// Boolean options are treated as opt-in toggles, so a value is enabled if
    /// either the command line or the config file enables it. Indicator style
    /// is selected by explicit CLI override when present, otherwise the config
    /// value is used.
    pub fn merge(flags: &cli::Flags, config: &Self) -> Self {
        Self {
            show_all: flags.show_all || config.show_all,
            indicator_style: flags
                .indicator_style
                .unwrap_or(config.indicator_style),
            dirs_first: flags.dirs_first || config.dirs_first,
            almost_all: flags.almost_all || config.almost_all,
            long_format: flags.long || config.long_format,
            human_readable: flags.human_readable || config.human_readable,
            no_icons: flags.no_icons || config.no_icons,
            no_color: flags.no_color || config.no_color,
            permission_colors: config.permission_colors
                && !flags.no_permission_colors,
            time_colors: config.time_colors && !flags.no_time_colors,
            size_colors: config.size_colors && !flags.no_size_colors,
            gitignore: flags.gitignore || config.gitignore,
            fuzzy_time: flags.fuzzy_time || config.fuzzy_time,
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub file_type: String,
    pub mode: String,
    pub nlink: u64,
    pub user: String,
    pub group: String,
    pub size: u64,
    pub mtime: SystemTime,
    pub item_icon: Option<Icon>,
    pub short_name: String,
    pub display_name: String,
    pub name_style: NameStyle,
    pub dimmed: bool,
    pub full_path: PathBuf,
}
