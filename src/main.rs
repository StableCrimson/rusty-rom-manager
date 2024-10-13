mod organizer;
mod verify;

use clap::{Parser, Subcommand, ValueEnum};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use log::info;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
        method: OrganizationType,

        /// Recursively search subdirectories.
        /// Will skip directory-level ROMs
        #[arg(short, long)]
        recursive: bool,
    },

    // TODO: File conversion??? ISO -> RVZ, BIN+CUE -> PBP/CHD
    // Allow for it to be done on a specific file or a whole directory
    /// Verify ROM based on DAT files
    Verify {
        /// Path to ROM file
        path: PathBuf,

        /// Path to the DAT file used to check against
        dat: PathBuf,
    },

    /// Try to find the information of a ROM based on the provided
    /// DAT and an MD5 hash.
    Identify {
        /// Path to ROM file
        path: PathBuf,

        /// Path to the DAT file used to check against
        dat: PathBuf,

        /// Rename the file according to the DAT
        #[arg(short, long, default_value_t = false)]
        rename: bool,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug)]
enum OrganizationType {
    /// Sort by console (WIP).
    /// If the console ID of a ROM cannot be determined it will be ignored
    Console,

    /// Sort by file extension
    FileExtension,
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();

    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        );

    if config.is_err() {
        return Err(Error::new(
            ErrorKind::Other,
            "Unable to build logger config",
        ));
    }

    if log4rs::init_config(config.unwrap()).is_err() {
        return Err(Error::new(ErrorKind::Other, "Unable to intialize log4rs"));
    }

    info!("{:?}", args);

    // TODO: Find a nice way to log the organizer result if it's an error
    // The thing that has me hesitant is that means that all other subcommands
    // will have to return io::Result, unless this is all refactored
    match args.command {
        Command::Organize {
            path,
            target,
            copy,
            method,
            recursive,
        } => organizer::organize(&path, target.as_ref(), copy, method, recursive)?,
        Command::Verify { path, dat } => {
            if verify::verify(&path, &dat).is_err() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Verification failed",
                ));
            }
        }
        Command::Identify { path, dat, rename } => {
            if verify::identify(&path, &dat, rename).is_err() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Idenfication failed",
                ));
            }
        }
    }

    Ok(())
}
