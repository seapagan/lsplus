//! CLI parsing for `lsplus`.
//!
//! The command line can be parsed in either native `lsplus` mode or GNU
//! compatibility mode. Both modes map into the same internal [`Flags`] type so
//! the rest of the application can work with one normalized representation.

use clap::ArgGroup;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::env;
use std::ffi::OsString;

use crate::{
    IndicatorStyle,
    structs::{AttributeDisplay, PermissionDisplay, ShortFormat},
};

const ARG_SHOW_ALL: &str = "show_all";
const ARG_ALMOST_ALL: &str = "almost_all";
const ARG_LONG: &str = "long";
const ARG_VERTICAL: &str = "vertical";
const ARG_FORMAT: &str = "format";
const ARG_HEADER: &str = "header";
const ARG_HUMAN_READABLE: &str = "human_readable";
const ARG_SI: &str = "si";
const ARG_RECURSIVE: &str = "recursive";
const ARG_TREE: &str = "tree";
const ARG_TREE_LEVEL: &str = "tree_level";
const ARG_PRUNE_NOISY_DIRS: &str = "prune_noisy_dirs";
const ARG_PRUNE_DIR: &str = "prune_dir";
const ARG_PATHS: &str = "paths";
const ARG_SLASH: &str = "slash";
const ARG_INDICATOR_STYLE: &str = "indicator_style";
const ARG_FILE_TYPE: &str = "file_type";
const ARG_CLASSIFY: &str = "classify";
const ARG_NO_INDICATORS: &str = "no_indicators";
const ARG_DIRS_FIRST: &str = "dirs_first";
const ARG_NO_ICONS: &str = "no_icons";
const ARG_NO_COLOR: &str = "no_color";
const ARG_NO_PERMISSION_COLORS: &str = "no_permission_colors";
const ARG_PERMISSIONS: &str = "permissions";
const ARG_ATTRIBUTES: &str = "attributes";
const ARG_NO_TIME_GRADIENT: &str = "no_time_gradient";
const ARG_NO_SIZE_COLORS: &str = "no_size_colors";
const ARG_GITIGNORE: &str = "gitignore";
const ARG_VERSION: &str = "version";
const ARG_FUZZY_TIME: &str = "fuzzy_time";
const ARG_HELP: &str = "help";
const ARG_INDICATOR_GROUP: &str = "indicator_style_group";
const ARG_TREE_MODE_GROUP: &str = "tree_mode_group";

/// CLI compatibility mode used when building the clap command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompatMode {
    /// Parse arguments using the native `lsplus` option set.
    #[default]
    Native,
    /// Parse arguments using the GNU-compatible option set.
    Gnu,
}

impl CompatMode {
    /// Parse a config or environment value into a compatibility mode.
    pub(crate) fn parse_value(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "native" => Ok(Self::Native),
            "gnu" => Ok(Self::Gnu),
            _ => Err(format!(
                "unsupported compatibility mode `{}`; expected `native` or \
                 `gnu`",
                value
            )),
        }
    }
}

/// Parsed command-line flags before they are merged with config defaults.
#[derive(Debug)]
pub struct Flags {
    /// Show entries whose names start with `.`.
    pub show_all: bool,
    /// Hide `.` and `..` while still showing other dotfiles.
    pub almost_all: bool,
    /// Render long-format output.
    pub long: bool,
    /// Force a short-format layout.
    pub short_format: Option<ShortFormat>,
    /// Show a title row in long-format output.
    pub header: bool,
    /// Render human-readable file sizes in long format.
    pub human_readable: bool,
    /// Render human-readable file sizes using powers of 1000.
    pub si: bool,
    /// Recurse into child directories.
    pub recursive: bool,
    /// Render long-format tree output.
    pub tree: bool,
    /// Maximum visible entry depth for recursive/tree output.
    pub tree_level: Option<usize>,
    /// Enable the built-in noisy-directory traversal prune preset.
    pub prune_noisy_dirs: bool,
    /// Directory basenames to skip while traversing recursive/tree output.
    pub prune_dirs: Vec<String>,
    /// Raw path arguments collected from the CLI.
    pub paths: Vec<String>,
    /// Override the configured indicator style for this invocation.
    pub indicator_style: Option<IndicatorStyle>,
    /// Group directories before files.
    pub dirs_first: bool,
    /// Disable file and directory icons.
    pub no_icons: bool,
    /// Disable colored or styled output.
    pub no_color: bool,
    /// Disable permission and file-type colors in long-format output.
    pub no_permission_colors: bool,
    /// Override long-format permission display mode.
    pub permissions: Option<PermissionDisplay>,
    /// Override the Windows file-attribute display mode.
    pub attributes: Option<AttributeDisplay>,
    /// Use the fixed timestamp color instead of age-based colors.
    pub no_time_gradient: bool,
    /// Disable large-size colors in long-format output.
    pub no_size_colors: bool,
    /// Dim paths matched by `.gitignore` rules.
    pub gitignore: bool,
    /// Print version information and exit.
    pub version: bool,
    /// Render humanized relative timestamps.
    pub fuzzy_time: bool,
}

impl Flags {
    /// Parse native-mode CLI arguments, exiting on parse failure.
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::try_parse_from(args).unwrap_or_else(|err| err.exit())
    }

    /// Parse native-mode CLI arguments without exiting on parse failure.
    pub fn try_parse_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        parse_matches(CompatMode::Native, args)
    }
}

/// Parse CLI arguments using the requested compatibility mode.
///
/// This is the main entry point used after startup mode selection has already
/// resolved whether `lsplus` should behave in native or GNU-compatible mode.
pub fn parse_from_mode(mode: CompatMode) -> Flags {
    let matches = build_command(mode).get_matches();
    flags_from_matches(mode, &matches)
}

#[cfg(test)]
pub(crate) fn try_parse_from_mode<I, T>(
    mode: CompatMode,
    args: I,
) -> Result<Flags, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    parse_matches(mode, args)
}

fn parse_matches<I, T>(mode: CompatMode, args: I) -> Result<Flags, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = build_command(mode).try_get_matches_from(args)?;
    Ok(flags_from_matches(mode, &matches))
}

fn build_command(mode: CompatMode) -> Command {
    let command = Command::new("lsplus")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about(None)
        .disable_help_flag(true)
        .arg(show_all_arg())
        .arg(almost_all_arg())
        .arg(long_arg())
        .arg(vertical_arg())
        .arg(format_arg())
        .arg(header_arg())
        .arg(human_readable_arg())
        .arg(si_arg())
        .arg(recursive_arg())
        .arg(tree_arg())
        .arg(tree_level_arg())
        .arg(prune_noisy_dirs_arg())
        .arg(prune_dir_arg())
        .arg(paths_arg())
        .arg(slash_arg(mode))
        .arg(file_type_arg(mode))
        .arg(classify_arg(mode))
        .arg(dirs_first_arg(mode))
        .arg(no_icons_arg())
        .arg(no_color_arg(mode))
        .arg(no_permission_colors_arg())
        .arg(permissions_arg())
        .arg(attributes_arg())
        .arg(no_time_gradient_arg())
        .arg(no_size_colors_arg())
        .arg(gitignore_arg(mode))
        .arg(version_arg())
        .arg(fuzzy_time_arg(mode))
        .arg(help_arg())
        .group(indicator_group(mode))
        .group(tree_mode_group());

    match mode {
        CompatMode::Native => command.arg(no_indicators_arg()),
        CompatMode::Gnu => command.arg(indicator_style_arg()),
    }
}

fn indicator_group(mode: CompatMode) -> ArgGroup {
    let args = match mode {
        CompatMode::Native => {
            vec![ARG_SLASH, ARG_FILE_TYPE, ARG_CLASSIFY, ARG_NO_INDICATORS]
        }
        CompatMode::Gnu => {
            vec![ARG_SLASH, ARG_INDICATOR_STYLE, ARG_FILE_TYPE, ARG_CLASSIFY]
        }
    };

    ArgGroup::new(ARG_INDICATOR_GROUP)
        .multiple(false)
        .args(args)
}

fn tree_mode_group() -> ArgGroup {
    ArgGroup::new(ARG_TREE_MODE_GROUP)
        .multiple(false)
        .args([ARG_RECURSIVE, ARG_TREE])
}

fn help_arg() -> Arg {
    Arg::new(ARG_HELP)
        .long("help")
        .action(ArgAction::Help)
        .help("Print help information")
}

fn show_all_arg() -> Arg {
    Arg::new(ARG_SHOW_ALL)
        .short('a')
        .long("all")
        .action(ArgAction::SetTrue)
        .help("Do not ignore entries starting with .")
}

fn almost_all_arg() -> Arg {
    Arg::new(ARG_ALMOST_ALL)
        .short('A')
        .long("almost-all")
        .action(ArgAction::SetTrue)
        .help("Do not list implied . and ..")
}

fn long_arg() -> Arg {
    Arg::new(ARG_LONG)
        .short('l')
        .long("long")
        .action(ArgAction::SetTrue)
        .help("Display detailed information")
}

fn vertical_arg() -> Arg {
    Arg::new(ARG_VERTICAL)
        .short('C')
        .action(ArgAction::SetTrue)
        .help("List entries in columns sorted vertically")
}

fn format_arg() -> Arg {
    Arg::new(ARG_FORMAT)
        .long("format")
        .value_name("FORMAT")
        .value_parser(clap::value_parser!(ShortFormat))
        .help("Select short output format: vertical")
}

fn header_arg() -> Arg {
    Arg::new(ARG_HEADER)
        .long("header")
        .action(ArgAction::SetTrue)
        .help("Show a title row in long-format output")
}

fn human_readable_arg() -> Arg {
    Arg::new(ARG_HUMAN_READABLE)
        .short('h')
        .long("human-readable")
        .action(ArgAction::SetTrue)
        .help("with -l, print sizes using 1024-byte units, like 1K 234M 2G")
}

fn si_arg() -> Arg {
    Arg::new(ARG_SI)
        .long("si")
        .action(ArgAction::SetTrue)
        .help("with -l, print sizes using 1000-byte units, like 1k 234M 2G")
}

fn recursive_arg() -> Arg {
    Arg::new(ARG_RECURSIVE)
        .short('R')
        .long("recursive")
        .action(ArgAction::SetTrue)
        .help("List subdirectories recursively")
}

fn tree_arg() -> Arg {
    Arg::new(ARG_TREE)
        .long("tree")
        .action(ArgAction::SetTrue)
        .help("Display a long-format directory tree")
}

fn tree_level_arg() -> Arg {
    Arg::new(ARG_TREE_LEVEL)
        .long("level")
        .value_name("N")
        .value_parser(parse_tree_level)
        .requires(ARG_TREE_MODE_GROUP)
        .help("Limit recursive or tree output to N visible entry levels")
}

fn prune_noisy_dirs_arg() -> Arg {
    Arg::new(ARG_PRUNE_NOISY_DIRS)
        .long("prune-noisy-dirs")
        .action(ArgAction::SetTrue)
        .help("Skip descending into built-in noisy directories")
}

fn prune_dir_arg() -> Arg {
    Arg::new(ARG_PRUNE_DIR)
        .long("prune-dir")
        .value_name("NAME")
        .action(ArgAction::Append)
        .help("Skip descending into directory basename NAME")
}

fn parse_tree_level(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(0) => Err(String::from("value must be at least 1")),
        Ok(level) => Ok(level),
        Err(err) => Err(err.to_string()),
    }
}

fn paths_arg() -> Arg {
    Arg::new(ARG_PATHS)
        .help("The path to list")
        .value_name("PATHS")
        .default_value(".")
        .action(ArgAction::Append)
        .num_args(0..)
}

fn slash_arg(mode: CompatMode) -> Arg {
    match mode {
        CompatMode::Native => Arg::new(ARG_SLASH)
            .short('p')
            .long("slash-dirs")
            .action(ArgAction::SetTrue)
            .help("Append a slash to directories"),
        CompatMode::Gnu => Arg::new(ARG_SLASH)
            .short('p')
            .action(ArgAction::SetTrue)
            .help("Append / indicator to directories"),
    }
}

fn file_type_arg(_mode: CompatMode) -> Arg {
    Arg::new(ARG_FILE_TYPE)
        .long("file-type")
        .action(ArgAction::SetTrue)
        .help("Append type indicators except '*' for executables")
}

fn classify_arg(mode: CompatMode) -> Arg {
    let arg = Arg::new(ARG_CLASSIFY)
        .short('F')
        .action(ArgAction::SetTrue)
        .help("Append type indicators, including '*' for executables");

    match mode {
        CompatMode::Native => arg.long("classify"),
        CompatMode::Gnu => arg,
    }
}

fn no_indicators_arg() -> Arg {
    Arg::new(ARG_NO_INDICATORS)
        .long("no-indicators")
        .action(ArgAction::SetTrue)
        .help("Do not append file type indicators")
}

fn indicator_style_arg() -> Arg {
    Arg::new(ARG_INDICATOR_STYLE)
        .long("indicator-style")
        .action(ArgAction::Set)
        .require_equals(true)
        .value_name("WORD")
        .value_parser(clap::value_parser!(IndicatorStyle))
        .help("Append indicator with style WORD to entry names")
}

fn dirs_first_arg(mode: CompatMode) -> Arg {
    match mode {
        CompatMode::Native => Arg::new(ARG_DIRS_FIRST)
            .short('D')
            .long("sort-dirs")
            .action(ArgAction::SetTrue)
            .help("Sort directories first"),
        CompatMode::Gnu => Arg::new(ARG_DIRS_FIRST)
            .long("group-directories-first")
            .action(ArgAction::SetTrue)
            .help("Group directories before files"),
    }
}

fn no_icons_arg() -> Arg {
    Arg::new(ARG_NO_ICONS)
        .long("no-icons")
        .action(ArgAction::SetTrue)
        .help("Do not display file or folder icons")
}

fn no_color_arg(mode: CompatMode) -> Arg {
    match mode {
        CompatMode::Native => Arg::new(ARG_NO_COLOR)
            .short('N')
            .long("no-color")
            .action(ArgAction::SetTrue)
            .help("Do not display colored or styled output"),
        CompatMode::Gnu => Arg::new(ARG_NO_COLOR)
            .long("no-color")
            .action(ArgAction::SetTrue)
            .help("Do not display colored or styled output"),
    }
}

fn no_permission_colors_arg() -> Arg {
    Arg::new(ARG_NO_PERMISSION_COLORS)
        .long("no-permission-colors")
        .action(ArgAction::SetTrue)
        .help(
            "Do not color file type character or permission bits in long-format output",
        )
}

fn permissions_arg() -> Arg {
    Arg::new(ARG_PERMISSIONS)
        .long("permissions")
        .action(ArgAction::Set)
        .value_name("MODE")
        .value_parser(clap::value_parser!(PermissionDisplay))
        .help("Select long-format permission display: symbolic, octal, both, or none")
}

fn attributes_arg() -> Arg {
    Arg::new(ARG_ATTRIBUTES)
        .long("attributes")
        .action(ArgAction::Set)
        .value_name("MODE")
        .value_parser(clap::value_parser!(AttributeDisplay))
        .help("Select Windows attribute display: long, short, or minimal")
}

fn no_time_gradient_arg() -> Arg {
    Arg::new(ARG_NO_TIME_GRADIENT)
        .long("no-time-gradient")
        .action(ArgAction::SetTrue)
        .help("Use the fixed timestamp color instead of age-based colors")
}

fn no_size_colors_arg() -> Arg {
    Arg::new(ARG_NO_SIZE_COLORS)
        .long("no-size-colors")
        .action(ArgAction::SetTrue)
        .help("Do not color large sizes in long-format output")
}

fn gitignore_arg(mode: CompatMode) -> Arg {
    match mode {
        CompatMode::Native => Arg::new(ARG_GITIGNORE)
            .short('I')
            .long("gitignore")
            .action(ArgAction::SetTrue)
            .help("Dim entries matched by active .gitignore rules"),
        CompatMode::Gnu => Arg::new(ARG_GITIGNORE)
            .long("gitignore")
            .action(ArgAction::SetTrue)
            .help("Dim entries matched by active .gitignore rules"),
    }
}

fn version_arg() -> Arg {
    Arg::new(ARG_VERSION)
        .short('V')
        .long("version")
        .action(ArgAction::SetTrue)
        .global(true)
        .help("Print version information and exit")
}

fn fuzzy_time_arg(mode: CompatMode) -> Arg {
    match mode {
        CompatMode::Native => Arg::new(ARG_FUZZY_TIME)
            .short('Z')
            .long("fuzzy-time")
            .action(ArgAction::SetTrue)
            .help("Use fuzzy time format"),
        CompatMode::Gnu => Arg::new(ARG_FUZZY_TIME)
            .long("fuzzy-time")
            .action(ArgAction::SetTrue)
            .help("Use fuzzy time format"),
    }
}

fn flags_from_matches(mode: CompatMode, matches: &ArgMatches) -> Flags {
    Flags {
        show_all: matches.get_flag(ARG_SHOW_ALL),
        almost_all: matches.get_flag(ARG_ALMOST_ALL),
        long: matches.get_flag(ARG_LONG),
        short_format: matches
            .get_one::<ShortFormat>(ARG_FORMAT)
            .copied()
            .or_else(|| {
                matches
                    .get_flag(ARG_VERTICAL)
                    .then_some(ShortFormat::Vertical)
            }),
        header: matches.get_flag(ARG_HEADER),
        human_readable: matches.get_flag(ARG_HUMAN_READABLE),
        si: matches.get_flag(ARG_SI),
        recursive: matches.get_flag(ARG_RECURSIVE),
        tree: matches.get_flag(ARG_TREE),
        tree_level: matches.get_one::<usize>(ARG_TREE_LEVEL).copied(),
        prune_noisy_dirs: matches.get_flag(ARG_PRUNE_NOISY_DIRS),
        prune_dirs: matches
            .get_many::<String>(ARG_PRUNE_DIR)
            .map(|values| values.cloned().collect())
            .unwrap_or_default(),
        paths: matches
            .get_many::<String>(ARG_PATHS)
            .map(|values| values.cloned().collect())
            .unwrap_or_else(|| vec![String::from(".")]),
        indicator_style: indicator_style_from_matches(mode, matches),
        dirs_first: matches.get_flag(ARG_DIRS_FIRST),
        no_icons: matches.get_flag(ARG_NO_ICONS),
        no_color: matches.get_flag(ARG_NO_COLOR),
        no_permission_colors: matches.get_flag(ARG_NO_PERMISSION_COLORS),
        permissions: matches
            .get_one::<PermissionDisplay>(ARG_PERMISSIONS)
            .copied(),
        attributes: matches
            .get_one::<AttributeDisplay>(ARG_ATTRIBUTES)
            .copied(),
        no_time_gradient: matches.get_flag(ARG_NO_TIME_GRADIENT),
        no_size_colors: matches.get_flag(ARG_NO_SIZE_COLORS),
        gitignore: matches.get_flag(ARG_GITIGNORE),
        version: matches.get_flag(ARG_VERSION),
        fuzzy_time: matches.get_flag(ARG_FUZZY_TIME),
    }
}

fn indicator_style_from_matches(
    mode: CompatMode,
    matches: &ArgMatches,
) -> Option<IndicatorStyle> {
    if matches.get_flag(ARG_SLASH) {
        return Some(IndicatorStyle::Slash);
    }

    if matches.get_flag(ARG_FILE_TYPE) {
        return Some(IndicatorStyle::FileType);
    }

    if matches.get_flag(ARG_CLASSIFY) {
        return Some(IndicatorStyle::Classify);
    }

    match mode {
        CompatMode::Native => {
            if matches.get_flag(ARG_NO_INDICATORS) {
                Some(IndicatorStyle::None)
            } else {
                None
            }
        }
        CompatMode::Gnu => matches
            .get_one::<IndicatorStyle>(ARG_INDICATOR_STYLE)
            .copied(),
    }
}

/// Format the user-facing version output from package metadata values.
pub(crate) fn format_version_info(
    version: &str,
    authors: &str,
    description: &str,
) -> String {
    let authors = if authors.is_empty() {
        "Unknown"
    } else {
        authors
    };
    let description = if description.is_empty() {
        "No description provided"
    } else {
        description
    };

    format!(
        "lsplus v{}\n\
        \n{}\n\
        \nReleased under the MIT license by {}\n",
        version, description, authors
    )
}

/// Return the formatted version banner for the current build.
pub fn version_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    format_version_info(version, authors, description)
}
