// Set up the CLI arguments
use clap::{Arg, ArgAction, CommandFactory, Parser, ValueEnum};
#[cfg(test)]
use std::ffi::OsString;
use std::{env, process::exit};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum IndicatorStyle {
    Slash,
}

const GNU_INDICATOR_STYLE_HELP_FROM: &str = "  -p, --indicator-style[=<WORD>]  Append indicator with style WORD to \
     entry names [possible values: slash]\n";
const GNU_INDICATOR_STYLE_HELP_TO: &str =
    "  -p, --indicator-style=slash     Append / indicator to directories\n";

#[derive(Parser)]
#[command(
    name = "lsplus",
    author = env!("CARGO_PKG_AUTHORS"),
    about =env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_flag = true,
    arg(
        Arg::new("help")
            .long("help")
            .action(ArgAction::Help)
            .help("Print help information")
    )
)]
#[derive(Debug)]
pub struct Flags {
    #[arg(
        short = 'a',
        long = "all",
        help = "Do not ignore entries starting with ."
    )]
    pub show_all: bool,

    #[arg(
        short = 'A',
        long = "almost-all",
        help = "Do not list implied . and .."
    )]
    pub almost_all: bool,

    #[arg(short = 'l', long = "long", help = "Display detailed information")]
    pub long: bool,

    #[arg(
        short = 'h',
        long = "human-readable",
        help = "with -l, print sizes like 1K 234M 2G etc."
    )]
    pub human_readable: bool,

    #[arg(default_value = ".", help = "The path to list")]
    pub paths: Vec<String>,

    #[arg(
        short = 'p',
        long = "slash-dirs",
        help = "Append a slash to directories"
    )]
    pub slash: bool,

    #[arg(short = 'D', long = "sort-dirs", help = "Sort directories first")]
    pub dirs_first: bool,

    #[arg(long = "no-icons", help = "Do not display file or folder icons")]
    pub no_icons: bool,

    #[arg(
        short = 'N',
        long = "no-color",
        help = "Do not display colored or styled output"
    )]
    pub no_color: bool,

    #[arg(
        short = 'I',
        long = "gitignore",
        help = "Dim entries matched by active .gitignore rules"
    )]
    pub gitignore: bool,

    #[arg(
        long = "version",
        short = 'V',
        action = ArgAction::SetTrue,
        help = "Print version information and exit",
        global = true
    )]
    pub version: bool,

    #[arg(long = "fuzzy-time", short = 'Z', help = "Use fuzzy time format")]
    pub fuzzy_time: bool,
}

#[derive(Parser, Debug)]
#[command(
    name = "lsplus",
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_flag = true,
    arg(
        Arg::new("help")
            .long("help")
            .action(ArgAction::Help)
            .help("Print help information")
    )
)]
struct GnuFlags {
    #[arg(
        short = 'a',
        long = "all",
        help = "Do not ignore entries starting with ."
    )]
    show_all: bool,

    #[arg(
        short = 'A',
        long = "almost-all",
        help = "Do not list implied . and .."
    )]
    almost_all: bool,

    #[arg(short = 'l', long = "long", help = "Display detailed information")]
    long: bool,

    #[arg(
        short = 'h',
        long = "human-readable",
        help = "with -l, print sizes like 1K 234M 2G etc."
    )]
    human_readable: bool,

    #[arg(default_value = ".", help = "The path to list")]
    paths: Vec<String>,

    #[arg(
        short = 'p',
        long = "indicator-style",
        value_enum,
        value_name = "WORD",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "slash",
        help = "Append indicator with style WORD to entry names"
    )]
    indicator_style: Option<IndicatorStyle>,

    #[arg(
        long = "group-directories-first",
        help = "Group directories before files"
    )]
    dirs_first: bool,

    #[arg(long = "no-icons", help = "Do not display file or folder icons")]
    no_icons: bool,

    #[arg(
        long = "no-color",
        help = "Do not display colored or styled output"
    )]
    no_color: bool,

    #[arg(
        long = "gitignore",
        help = "Dim entries matched by active .gitignore rules"
    )]
    gitignore: bool,

    #[arg(
        long = "version",
        short = 'V',
        action = ArgAction::SetTrue,
        help = "Print version information and exit",
        global = true
    )]
    version: bool,

    #[arg(long = "fuzzy-time", help = "Use fuzzy time format")]
    fuzzy_time: bool,
}

impl From<GnuFlags> for Flags {
    fn from(value: GnuFlags) -> Self {
        Self {
            show_all: value.show_all,
            almost_all: value.almost_all,
            long: value.long,
            human_readable: value.human_readable,
            paths: value.paths,
            slash: matches!(
                value.indicator_style,
                Some(IndicatorStyle::Slash)
            ),
            dirs_first: value.dirs_first,
            no_icons: value.no_icons,
            no_color: value.no_color,
            gitignore: value.gitignore,
            version: value.version,
            fuzzy_time: value.fuzzy_time,
        }
    }
}

pub fn parse_from_mode(mode: CompatMode) -> Flags {
    match mode {
        CompatMode::Native => Flags::parse(),
        CompatMode::Gnu => {
            print_gnu_help_and_exit_if_requested();
            GnuFlags::parse().into()
        }
    }
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
    match mode {
        CompatMode::Native => Flags::try_parse_from(args),
        CompatMode::Gnu => GnuFlags::try_parse_from(args).map(Into::into),
    }
}

fn print_gnu_help_and_exit_if_requested() {
    if !env::args_os().skip(1).any(|arg| arg == "--help") {
        return;
    }

    print!("{}", render_gnu_help());
    exit(0);
}

pub(crate) fn render_gnu_help() -> String {
    let mut command = GnuFlags::command();
    command = command.bin_name("lsp");

    command.render_help().to_string().replacen(
        GNU_INDICATOR_STYLE_HELP_FROM,
        GNU_INDICATOR_STYLE_HELP_TO,
        1,
    )
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
