mod organizer;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // TODO: Update once sorting by console is supported
    /// Organize your ROM files based on file type
    Organize {
        /// Directory the unorganized files are in
        path: PathBuf,

        /// Directory where the organized files should be placed.
        /// Defaults to source directory
        #[arg(short, long)]
        target: Option<PathBuf>,

        /// Copy files instead of move them
        #[arg(short, long, default_value_t = false)]
        copy: bool,

        /// Method used to organize ROMs
        #[arg(short, long, value_enum, default_value_t=OrganizationType::Console)]
        sort_method: OrganizationType,
    },
    // TODO: File conversion??? ISO -> RVZ, BIN+CUE -> PBP/CHD
    // Allow for it to be done on a specific file or a whole directory
}

#[derive(ValueEnum, Copy, Clone)]
enum OrganizationType {
    /// Sort by console (WIP).
    /// If the console ID of a ROM cannot be determined it will be ignored
    Console,

    /// Sort by file extension
    FileExtension,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    match args.command {
        Command::Organize {
            path,
            target,
            copy,
            sort_method: sort,
        } => organizer::organize(&path, target.as_ref(), copy, sort)?,
    }

    Ok(())
}
