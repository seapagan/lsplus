//! CLI parsing for `lsplus`.
//!
//! The command line can be parsed in either native `lsplus` mode or GNU
//! compatibility mode. Both modes map into the same internal [`Flags`] type so
//! the rest of the application can work with one normalized representation.

use clap::ArgGroup;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::env;
use std::ffi::OsString;

use crate::IndicatorStyle;

const ARG_SHOW_ALL: &str = "show_all";
const ARG_ALMOST_ALL: &str = "almost_all";
const ARG_LONG: &str = "long";
const ARG_HUMAN_READABLE: &str = "human_readable";
const ARG_PATHS: &str = "paths";
const ARG_SLASH: &str = "slash";
const ARG_INDICATOR_STYLE: &str = "indicator_style";
const ARG_FILE_TYPE: &str = "file_type";
const ARG_CLASSIFY: &str = "classify";
const ARG_NO_INDICATORS: &str = "no_indicators";
const ARG_DIRS_FIRST: &str = "dirs_first";
const ARG_NO_ICONS: &str = "no_icons";
const ARG_NO_COLOR: &str = "no_color";
const ARG_GITIGNORE: &str = "gitignore";
const ARG_VERSION: &str = "version";
const ARG_FUZZY_TIME: &str = "fuzzy_time";
const ARG_HELP: &str = "help";
const ARG_INDICATOR_GROUP: &str = "indicator_style_group";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompatMode {
    /// Parse arguments using the native `lsplus` option set.
    #[default]
    Native,
    /// Parse arguments using the GNU-compatible option set.
    Gnu,
}

impl CompatMode {
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

#[derive(Debug)]
pub struct Flags {
    /// Show entries whose names start with `.`.
    pub show_all: bool,
    /// Hide `.` and `..` while still showing other dotfiles.
    pub almost_all: bool,
    /// Render long-format output.
    pub long: bool,
    /// Render human-readable file sizes in long format.
    pub human_readable: bool,
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
        .arg(human_readable_arg())
        .arg(paths_arg())
        .arg(slash_arg(mode))
        .arg(file_type_arg(mode))
        .arg(classify_arg(mode))
        .arg(dirs_first_arg(mode))
        .arg(no_icons_arg())
        .arg(no_color_arg(mode))
        .arg(gitignore_arg(mode))
        .arg(version_arg())
        .arg(fuzzy_time_arg(mode))
        .arg(help_arg())
        .group(indicator_group(mode));

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

fn human_readable_arg() -> Arg {
    Arg::new(ARG_HUMAN_READABLE)
        .short('h')
        .long("human-readable")
        .action(ArgAction::SetTrue)
        .help("with -l, print sizes like 1K 234M 2G etc.")
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
        .value_parser(["none", "slash", "file-type", "classify"])
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
        human_readable: matches.get_flag(ARG_HUMAN_READABLE),
        paths: matches
            .get_many::<String>(ARG_PATHS)
            .map(|values| values.cloned().collect())
            .unwrap_or_else(|| vec![String::from(".")]),
        indicator_style: indicator_style_from_matches(mode, matches),
        dirs_first: matches.get_flag(ARG_DIRS_FIRST),
        no_icons: matches.get_flag(ARG_NO_ICONS),
        no_color: matches.get_flag(ARG_NO_COLOR),
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
            .get_one::<String>(ARG_INDICATOR_STYLE)
            .and_then(|value| match value.as_str() {
                "none" => Some(IndicatorStyle::None),
                "slash" => Some(IndicatorStyle::Slash),
                "file-type" => Some(IndicatorStyle::FileType),
                "classify" => Some(IndicatorStyle::Classify),
                _ => None,
            }),
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
