use clap::Parser;
use file_organizer::{Config, Organizer};
use log::info;
use log::LevelFilter::Info;
use simple_logger::SimpleLogger;
use std::process;

#[derive(Parser)]
struct Args {
    /// The target directory to operate on.
    #[arg(short)]
    directory: String,

    /// Path to the configuration file.
    #[arg(short, default_value = "rules.toml")]
    config_path: String,

    /// Sort files.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    sort_files: bool,

    /// Move duplicates to a separate folder.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    move_duplicates: bool,

    /// Remove empty folders.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    remove_empty_folders: bool,
}

fn main() {
    let args = Args::parse();

    SimpleLogger::new().with_level(Info).init().unwrap();

    if let Err(e) = run(args) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(args: Args) -> file_organizer::error::Result<()> {
    let config = Config::new(&args.directory, &args.config_path)?;
    let mut organizer = Organizer::new(config)?;

    if args.sort_files {
        info!("Sorting files...");
        organizer.sort_all_files()?;
    }

    if args.move_duplicates {
        info!("Moving duplicates...");
        organizer.move_duplicates()?;
    }

    if args.remove_empty_folders {
        info!("Removing empty folders...");
        organizer.remove_empty_folders()?;
    }

    Ok(())
}
