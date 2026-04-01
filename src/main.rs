use clap::Parser;
use std::process::exit;

use lsplus::cli;

fn main() {
    let args = cli::Flags::parse();
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    if let Err(e) = lsplus::run_with_flags(args) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
