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
        /// Defaults to current ROM directory
        #[arg(short, long)]
        target: Option<PathBuf>,

        /// Enable this if you want to copy the files instead of move them.
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
    /// Sort by file extension
    FileExtension,

    /// Sort by console (not fully supported).
    /// If a ROM file isn't supported yet then it will be left untouched
    Console,
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
