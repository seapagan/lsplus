use std::process::exit;

use lsplus::cli;

fn main() {
    let startup = match lsplus::settings::load_startup_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let args = cli::parse_from_mode(startup.compat_mode);
    if args.version {
        println!("{}", cli::version_info());
        exit(0);
    }

    if let Err(e) =
        lsplus::app::run_with_flags_and_config(args, &startup.params)
    {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
