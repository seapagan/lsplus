// Set up the CLI arguments
use clap::{Arg, ArgAction, Parser};

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
pub struct Flags {
    #[arg(short ='a', long = "all", action = ArgAction::SetTrue, help = "Do not ignore entries starting with .")]
    pub show_all: bool,

    #[arg(short ='A', long = "almost-all", action = ArgAction::SetTrue, help = "Do not list implied . and ..")]
    pub almost_all: bool,

    #[arg(short='l', long="long", action = ArgAction::SetTrue, help = "Display detailed information")]
    pub long: bool,

    #[arg(short='h', long="human-readable", action = ArgAction::SetTrue, help = "with -l, print sizes like 1K 234M 2G etc.")]
    pub human_readable: bool,

    #[arg(default_value = ".", help = "The path to list")]
    pub path: String,

    #[arg(short = 'p', long = "slash-dirs", action = ArgAction::SetTrue, help = "Append a slash to directories")]
    pub slash: bool,

    #[arg(short = 'D', long = "sort-dirs", action = ArgAction::SetTrue, help = "Sort directories first")]
    pub dirs_first: bool,

    #[arg(
        long = "version",
        short = 'V',
        action = ArgAction::SetTrue,
        help = "Print version information and exit",
        global = true
    )]
    pub version: bool,
}

pub fn version_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    // Provide default values if fields are empty
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
