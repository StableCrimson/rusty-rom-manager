mod organizer;

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
        sort_method: OrganizationType,
    },
    // TODO: File conversion??? ISO -> RVZ, BIN+CUE -> PBP/CHD
    // Allow for it to be done on a specific file or a whole directory
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
        .encoder(Box::new(PatternEncoder::new("{l}-{m}\n")))
        .build("log/output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Debug));

    if config.is_err() {
        return Err(Error::new(ErrorKind::Other, "Issue building config"));
    }

    if log4rs::init_config(config.unwrap()).is_err() {
        return Err(Error::new(ErrorKind::Other, "Issue initializing log4rs"));
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
            sort_method,
        } => organizer::organize(&path, target.as_ref(), copy, sort_method)?,
    }

    Ok(())
}
