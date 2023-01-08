mod app;

use anyhow::Result;
use app::{attach, backup, reload};
use clap::{Parser, Subcommand};
use colorize::AnsiColor;
use std::path::PathBuf;
use ttsst::ExternalEditorApi;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Attach script to object
    Attach {
        /// Path to the file that should be attached
        #[arg(value_parser)]
        path: PathBuf,
        /// Optional: The guid of the object the script should be attached to.
        /// If not provided a list of all objects will be shown.
        #[arg(value_parser)]
        guid: Option<String>,
    },
    /// Update scripts and reload save
    Reload {
        /// Path to the directory with all scripts
        #[arg(value_parser)]
        path: PathBuf,
    },
    /// Backup current save
    Backup {
        /// Path to save location
        #[arg(value_parser)]
        path: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    if let Err(err) = run(args) {
        eprintln!("{} {}", "error:".red().bold(), err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let mut api = ExternalEditorApi::new();
    match args.command {
        Commands::Attach { path, guid } => attach(&mut api, &path, guid)?,
        Commands::Backup { path } => backup(&mut api, &path)?,
        Commands::Reload { path } => reload(&mut api, &path)?,
    }
    Ok(())
}
