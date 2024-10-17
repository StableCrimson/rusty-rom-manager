use log::{debug, info, warn};

use sha1_smol::Sha1;

use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::organizer::file_types::{self, Console};
use crate::organizer::move_file;

#[derive(Debug)]
pub enum GameStatus {
    Verified,
    Unverified,
}
pub struct Rom {
    path: PathBuf,
    status: GameStatus,
    console: Console,
}

impl Rom {
    pub fn new(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("File does not exist".to_string());
        }

        let id = file_types::get_console_id(path);

        if id.is_none() {
            return Err("Unable to determine console ID".to_string());
        }

        Ok(Rom {
            path: path.to_path_buf(),
            status: GameStatus::Unverified,
            console: id.unwrap(),
        })
    }

    pub fn verify(&mut self, dat_file: &PathBuf) {
        let result = verify(&self.path, dat_file);

        if result.is_ok() {
            self.status = GameStatus::Verified;
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn console_id(&self) -> Console {
        self.console
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct DatFile {
    // header: Header,
    #[serde(rename = "game")]
    games: Vec<Game>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Header {}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Game {
    category: Category,
    // description: Description,
    rom: Entry,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Category {
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Description {
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Entry {
    name: String,
    size: String,
    // crc: String,
    // md5: String,
    sha1: String,
}

impl DatFile {
    pub fn from_file(file_path: &PathBuf) -> std::io::Result<DatFile> {
        let dat_contents = std::fs::read_to_string(file_path)?;
        let dat_file: DatFile = from_str(&dat_contents).unwrap();
        Ok(dat_file)
    }
}

// TODO: Verify directory-level ROMs like PS3 games
// TODO: Allow for passing a directory for large-scale verification
// TODO: Allow for automatically grabbing DATs based on console id?
pub fn verify(file_path: &PathBuf, dat_file: &PathBuf) -> Result<(), String> {
    let dat = DatFile::from_file(dat_file).unwrap();
    let file_size = get_file_size(file_path);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    let mut try_rom = None;
    for game in &dat.games {
        if game.rom.name == file_name {
            debug!("{:?}", game.rom);
            try_rom = Some(game.rom.clone());
            break;
        }
    }

    if try_rom.is_none() {
        let msg = "Unable to find ROM in dat file";
        println!("{msg}");
        warn!("{msg}");
        return Err(msg.to_string());
    }

    let rom = try_rom.unwrap();
    let rom_size: u64 = rom.size.parse().unwrap();

    if file_size != rom_size {
        let msg = format!("File sizes don't match {} {}", file_size, rom_size);
        warn!("{}", msg);
        return Err(msg);
    }

    let sha1 = calculate_sha1(file_path);
    debug!("{}", sha1);

    if sha1 != rom.sha1 {
        let msg = format!("File hashes don't match {} {}", sha1, rom.sha1);
        warn!("{}", msg);
        return Err(msg);
    }

    info!("ROM verified!");
    println!("ROM verified!");
    Ok(())
}

pub fn identify(file_path: &PathBuf, dat_file: &PathBuf, rename: bool) -> Result<String, String> {
    let sha1 = calculate_sha1(file_path);
    debug!("{}", sha1);
    let dat = DatFile::from_file(dat_file).unwrap();

    for game in &dat.games {
        if game.rom.sha1 == sha1 {
            debug!("{:?}", game.rom);
            println!("ROM identified as {}", game.rom.name);

            if rename {
                // TODO:: Allow for copying?
                let mut new_file_dest = file_path.to_owned();
                new_file_dest.pop();

                if !new_file_dest.exists() {
                    info!("{:?} does not exist, creating directory...", new_file_dest);
                    let _ = fs::create_dir(&new_file_dest);
                }

                new_file_dest.push(&game.rom.name);
                println!("{:?}", new_file_dest);
                debug!("{:?}", new_file_dest);

                let _ = move_file(file_path, new_file_dest, false);
            }

            return Ok(game.rom.name.clone());
        }
    }

    Err("Unable to identify ROM".to_string())
}

fn calculate_sha1(file_path: &PathBuf) -> String {
    let mut hasher = Sha1::new();

    let file = File::open(file_path).unwrap();
    let mut buffer = [0; 0xFFFF];
    let mut reader = BufReader::new(file);

    loop {
        let bytes_read = reader.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    hasher.digest().to_string()
}

fn get_file_size(file_path: &PathBuf) -> u64 {
    File::open(file_path).unwrap().metadata().unwrap().size()
}
