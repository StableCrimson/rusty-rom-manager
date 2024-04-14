mod organizer;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // TODO: Update once sorting by console is supported
    /// Organize your ROM files based on file type
    Organize {
        /// Directory the unorganized files are in
        path: PathBuf,

        /// Directory where the organized files should be placed.
        /// Defaults to current ROM directory
        #[arg(short, long)]
        target: Option<PathBuf>,

        /// Enable this if you want to copy the files instead of move them.
        #[arg(short, long, default_value_t = false)]
        copy: bool,
    },
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    match args.command {
        Some(Commands::Organize { path, target, copy }) => {
            organizer::organize(&path, target.as_ref(), copy)?
        }
        None => {}
    }

    Ok(())
}
