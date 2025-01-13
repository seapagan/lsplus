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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flags() {
        let args = Flags::parse_from(["lsplus"]);
        assert_eq!(args.paths, vec![String::from(".")]);
        assert!(!args.show_all);
        assert!(!args.almost_all);
        assert!(!args.long);
        assert!(!args.human_readable);
    }

    #[test]
    fn test_multiple_paths() {
        let args = Flags::parse_from(["lsplus", "path1", "path2"]);
        assert_eq!(args.paths, vec![String::from("path1"), String::from("path2")]);
    }

    #[test]
    fn test_all_flags() {
        let args = Flags::parse_from([
            "lsplus",
            "-a",
            "-A",
            "-l",
            "-h",
            "-p",
            "--sort-dirs",
            "--no-icons",
            "--fuzzy-time",
        ]);
        assert!(args.show_all);
        assert!(args.almost_all);
        assert!(args.long);
        assert!(args.human_readable);
        assert!(args.slash);
        assert!(args.dirs_first);
        assert!(args.no_icons);
        assert!(args.fuzzy_time);
    }

    #[test]
    fn test_version_flag() {
        let args = Flags::parse_from(["lsplus", "--version"]);
        assert!(args.version);
    }
}
