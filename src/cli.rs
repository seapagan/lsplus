// Set up the CLI arguments
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::env;
use std::ffi::OsString;

const ARG_SHOW_ALL: &str = "show_all";
const ARG_ALMOST_ALL: &str = "almost_all";
const ARG_LONG: &str = "long";
const ARG_HUMAN_READABLE: &str = "human_readable";
const ARG_PATHS: &str = "paths";
const ARG_SLASH: &str = "slash";
const ARG_INDICATOR_STYLE: &str = "indicator_style";
const ARG_DIRS_FIRST: &str = "dirs_first";
const ARG_NO_ICONS: &str = "no_icons";
const ARG_NO_COLOR: &str = "no_color";
const ARG_GITIGNORE: &str = "gitignore";
const ARG_VERSION: &str = "version";
const ARG_FUZZY_TIME: &str = "fuzzy_time";
const ARG_HELP: &str = "help";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatMode {
    Native,
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
    pub show_all: bool,
    pub almost_all: bool,
    pub long: bool,
    pub human_readable: bool,
    pub paths: Vec<String>,
    pub slash: bool,
    pub dirs_first: bool,
    pub no_icons: bool,
    pub no_color: bool,
    pub gitignore: bool,
    pub version: bool,
    pub fuzzy_time: bool,
}

impl Flags {
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
        .arg(dirs_first_arg(mode))
        .arg(no_icons_arg())
        .arg(no_color_arg(mode))
        .arg(gitignore_arg(mode))
        .arg(version_arg())
        .arg(fuzzy_time_arg(mode))
        .arg(help_arg());

    match mode {
        CompatMode::Native => command,
        CompatMode::Gnu => command.arg(indicator_style_arg()),
    }
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

fn indicator_style_arg() -> Arg {
    Arg::new(ARG_INDICATOR_STYLE)
        .long("indicator-style")
        .action(ArgAction::Set)
        .require_equals(true)
        .value_name("WORD")
        .value_parser(["slash"])
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
        slash: match mode {
            CompatMode::Native => matches.get_flag(ARG_SLASH),
            CompatMode::Gnu => {
                matches.get_flag(ARG_SLASH)
                    || matches
                        .get_one::<String>(ARG_INDICATOR_STYLE)
                        .is_some_and(|value| value == "slash")
            }
        },
        dirs_first: matches.get_flag(ARG_DIRS_FIRST),
        no_icons: matches.get_flag(ARG_NO_ICONS),
        no_color: matches.get_flag(ARG_NO_COLOR),
        gitignore: matches.get_flag(ARG_GITIGNORE),
        version: matches.get_flag(ARG_VERSION),
        fuzzy_time: matches.get_flag(ARG_FUZZY_TIME),
    }
}

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

pub fn version_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    format_version_info(version, authors, description)
}
