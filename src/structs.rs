use super::utils::icons::Icon;
use clap::ValueEnum;
use config::Config;
use serde::Deserialize;
use std::convert::From;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::cli;
use crate::utils::format::SizeScale;

const NOISY_DIR_PRESET: [&str; 5] =
    [".git", ".hg", ".svn", "node_modules", "__pycache__"];

/// Entry-name indicator styles supported by `lsplus`.
///
/// These map to GNU-style indicator modes and the native `--slash-dirs`,
/// `--file-type`, `--classify`, and `--no-indicators` options.
#[derive(
    Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum IndicatorStyle {
    /// Do not append an indicator suffix.
    #[default]
    None,
    /// Append `/` to directories.
    Slash,
    /// Append file-type indicators, excluding executable `*` suffixes.
    FileType,
    /// Append file-type indicators, including executable `*` suffixes.
    Classify,
}

/// Long-format permission column display modes.
#[derive(
    Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum PermissionDisplay {
    /// Show the existing file-type character plus symbolic permissions.
    #[default]
    Symbolic,
    /// Show the file-type character plus octal permission bits.
    Octal,
    /// Show symbolic permissions and a separate octal permission cell.
    Both,
    /// Hide permission cells entirely.
    None,
}

/// Windows file-attribute display modes.
#[derive(
    Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum AttributeDisplay {
    /// Show readable Windows attribute names.
    #[default]
    Long,
    /// Show a fixed-position compact Windows attribute field.
    Short,
    /// Show the classic ReadOnly, Hidden, System, and Archive attributes.
    Minimal,
}

/// Short-format layouts that can be forced for terminal or redirected output.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum ShortFormat {
    /// Fill entries down variable-width columns.
    Vertical,
}

/// Controls when file and directory icons are displayed.
#[derive(
    Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum IconDisplay {
    /// Display icons only when stdout is a terminal.
    #[default]
    Auto,
    /// Display icons even when stdout is redirected.
    Always,
    /// Never display icons.
    Never,
}

impl IconDisplay {
    fn is_enabled(self, is_terminal: bool) -> bool {
        match self {
            Self::Auto => is_terminal,
            Self::Always => true,
            Self::Never => false,
        }
    }
}

/// Runtime options after CLI flags and config defaults have been merged.
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
    /// Force a short-format layout instead of selecting one from stdout.
    pub short_format: Option<ShortFormat>,
    /// Show a title row in long-format output.
    pub header: bool,
    /// Render human-readable file sizes in long format.
    pub human_readable: bool,
    /// Use decimal powers for human-readable file sizes.
    pub si: bool,
    /// Recurse into child directories.
    pub recursive: bool,
    /// Render long-format tree output.
    pub tree: bool,
    /// Maximum visible entry depth for recursive/tree output.
    pub tree_level: usize,
    /// Optional maximum depth for recursive output.
    pub recursive_level: Option<usize>,
    /// Directory basenames to skip while traversing recursive/tree output.
    pub prune_dirs: Vec<String>,
    /// Select when file and directory icons are displayed.
    pub icons: IconDisplay,
    /// Disable file and directory icons.
    pub no_icons: bool,
    /// Disable colored or styled output.
    pub no_color: bool,
    /// Color file type and permission bits in long-format output.
    pub permission_colors: bool,
    /// Select which permission fields to show in long-format output.
    pub permissions: PermissionDisplay,
    /// Select how Windows file attributes appear in long-format output.
    pub attributes: AttributeDisplay,
    /// Color timestamps by age in long-format output.
    pub time_gradient: bool,
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
            short_format: None,
            header: false,
            human_readable: false,
            si: false,
            recursive: false,
            tree: false,
            tree_level: 2,
            recursive_level: None,
            prune_dirs: Vec::new(),
            icons: IconDisplay::Auto,
            no_icons: false,
            no_color: false,
            permission_colors: true,
            permissions: PermissionDisplay::Symbolic,
            attributes: AttributeDisplay::Long,
            time_gradient: true,
            size_colors: true,
            gitignore: false,
            fuzzy_time: false,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(default)]
/// Raw config-file parameters before compatibility aliases are normalized.
pub(crate) struct RawParams {
    show_all: bool,
    dirs_first: bool,
    almost_all: bool,
    long_format: bool,
    short_format: Option<ShortFormat>,
    header: bool,
    human_readable: bool,
    si: bool,
    recursive: bool,
    tree: bool,
    tree_level: Option<usize>,
    prune_noisy_dirs: bool,
    prune_dirs: Vec<String>,
    icons: Option<IconDisplay>,
    no_icons: bool,
    no_color: bool,
    permission_colors: Option<bool>,
    permissions: PermissionDisplay,
    attributes: AttributeDisplay,
    time_gradient: Option<bool>,
    size_colors: Option<bool>,
    gitignore: bool,
    fuzzy_time: bool,
    indicator_style: Option<IndicatorStyle>,
    append_slash: Option<bool>,
}

/// Visual category used to style names in short-format output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NameStyle {
    /// A regular entry with no special name styling.
    #[default]
    Plain,
    /// A directory entry.
    Directory,
    /// A symbolic link entry.
    Symlink,
    /// A Windows directory junction.
    Junction,
    /// A regular executable file.
    Executable,
    /// A Unix socket.
    Socket,
    /// A Unix FIFO/pipe.
    Fifo,
    /// A Unix character device.
    CharDevice,
    /// A Unix block device.
    BlockDevice,
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
            short_format: raw.short_format,
            header: raw.header,
            human_readable: raw.human_readable,
            si: raw.si,
            recursive: raw.recursive,
            tree: raw.tree,
            tree_level: normalized_tree_level(raw.tree_level),
            recursive_level: raw.tree_level.filter(|level| *level > 0),
            prune_dirs: configured_prune_dirs(
                raw.prune_noisy_dirs,
                raw.prune_dirs,
            ),
            icons: raw.icons.unwrap_or_default(),
            no_icons: raw.no_icons,
            no_color: raw.no_color,
            permission_colors: raw.permission_colors.unwrap_or(true),
            permissions: raw.permissions,
            attributes: raw.attributes,
            time_gradient: raw.time_gradient.unwrap_or(true),
            size_colors: raw.size_colors.unwrap_or(true),
            gitignore: raw.gitignore,
            fuzzy_time: raw.fuzzy_time,
        }
    }
}

impl Params {
    /// Merge parsed CLI flags with config-file defaults.
    ///
    /// CLI flags and config both enable opt-in booleans.
    ///
    /// CLI `--no-*` flags disable long-format accent defaults from config.
    /// Explicit CLI indicator flags override the config indicator style.
    pub fn merge(flags: &cli::Flags, config: &Self) -> Self {
        let icons = flags.icons.unwrap_or(config.icons);
        let no_icons = if flags.icons.is_some() {
            false
        } else {
            flags.no_icons || config.no_icons
        };

        Self {
            show_all: flags.show_all || config.show_all,
            indicator_style: flags
                .indicator_style
                .unwrap_or(config.indicator_style),
            dirs_first: flags.dirs_first || config.dirs_first,
            almost_all: flags.almost_all || config.almost_all,
            long_format: flags.long
                || flags.tree
                || config.long_format
                || config.tree,
            short_format: flags.short_format.or(config.short_format),
            header: flags.header || config.header,
            human_readable: flags.si
                || flags.human_readable
                || config.si
                || config.human_readable,
            si: flags.si || config.si,
            recursive: flags.recursive || config.recursive,
            tree: flags.tree || config.tree,
            tree_level: flags.tree_level.unwrap_or(config.tree_level),
            recursive_level: flags.tree_level.or(config.recursive_level),
            prune_dirs: merged_prune_dirs(flags, config),
            icons,
            no_icons,
            no_color: flags.no_color || config.no_color,
            permission_colors: config.permission_colors
                && !flags.no_permission_colors,
            permissions: flags.permissions.unwrap_or(config.permissions),
            attributes: flags.attributes.unwrap_or(config.attributes),
            time_gradient: config.time_gradient && !flags.no_time_gradient,
            size_colors: config.size_colors && !flags.no_size_colors,
            gitignore: flags.gitignore || config.gitignore,
            fuzzy_time: flags.fuzzy_time || config.fuzzy_time,
        }
    }

    /// Resolve automatic icon display for the active stdout destination.
    pub(crate) fn resolve_icon_output(&mut self, is_terminal: bool) {
        self.no_icons = self.no_icons || !self.icons.is_enabled(is_terminal);
    }

    /// Return the size scaling mode for long-format output.
    pub fn size_scale(&self) -> Option<SizeScale> {
        if self.si {
            Some(SizeScale::Decimal)
        } else if self.human_readable {
            Some(SizeScale::Binary)
        } else {
            None
        }
    }
}

fn configured_prune_dirs(
    prune_noisy_dirs: bool,
    mut prune_dirs: Vec<String>,
) -> Vec<String> {
    if prune_noisy_dirs {
        append_prune_names(&mut prune_dirs, NOISY_DIR_PRESET);
    }
    prune_dirs
}

fn normalized_tree_level(tree_level: Option<usize>) -> usize {
    tree_level.filter(|level| *level > 0).unwrap_or(2)
}

fn merged_prune_dirs(flags: &cli::Flags, config: &Params) -> Vec<String> {
    let mut prune_dirs = Vec::new();

    if flags.prune_noisy_dirs {
        append_prune_names(&mut prune_dirs, NOISY_DIR_PRESET);
    }

    append_prune_names(
        &mut prune_dirs,
        config.prune_dirs.iter().map(String::as_str),
    );
    append_prune_names(
        &mut prune_dirs,
        flags.prune_dirs.iter().map(String::as_str),
    );
    prune_dirs
}

fn append_prune_names<'a>(
    prune_dirs: &mut Vec<String>,
    names: impl IntoIterator<Item = &'a str>,
) {
    for name in names {
        if !prune_dirs.iter().any(|existing| existing == name) {
            prune_dirs.push(name.to_string());
        }
    }
}

/// Metadata and pre-rendered name data for one listed filesystem entry.
#[derive(Debug)]
pub struct FileInfo {
    /// Long-format type marker, such as Unix `d` or `l`, or Windows `j`, `L`,
    /// or `r`.
    pub file_type: String,
    /// Long-format access text: Unix symbolic permissions or Windows file
    /// attributes.
    pub mode: String,
    /// Unix permission bits for octal rendering; zero is a Windows-only
    /// internal placeholder.
    pub mode_bits: u32,
    /// Unix link count; zero is a Windows-only internal placeholder.
    pub nlink: u64,
    /// Unix owner name or fallback numeric ID; empty on Windows and not
    /// rendered there.
    pub user: String,
    /// Unix group name or fallback numeric ID; empty on Windows and not
    /// rendered there.
    pub group: String,
    /// Size in bytes.
    pub size: u64,
    /// Last modification time.
    pub mtime: SystemTime,
    /// Optional icon selected from the entry type or name.
    pub item_icon: Option<Icon>,
    /// Sanitized entry name used by short-format rendering.
    pub short_name: String,
    /// Styled entry name used by long-format rendering.
    pub display_name: String,
    /// Styling category for short-format rendering.
    pub name_style: NameStyle,
    /// Whether the entry should be dimmed as gitignored.
    pub dimmed: bool,
    /// Full path used for metadata lookups and special display cases.
    pub full_path: PathBuf,
}
