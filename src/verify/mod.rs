use log::{debug, info, warn};

use checksums::{hash_file, Algorithm};

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::organizer::file_types::{self, Console};

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
    md5: String,
    // sha1: String,
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
        println!("Unable to find ROM in dat file");
        warn!("Unable to find ROM in dat file");
        return Err("Unable to find ROM in dat file".to_string());
    }

    let rom = try_rom.unwrap();
    let rom_size: u64 = rom.size.parse().unwrap();

    if file_size != rom_size {
        let msg = format!("File sizes don't match {} {}", file_size, rom_size);
        warn!("{}", msg);
        return Err(msg);
    }

    let md5 = hash_file(file_path, Algorithm::MD5).to_lowercase();
    debug!("{}", md5);

    if md5 != rom.md5 {
        let msg = format!("File hashes don't match {} {}", md5, rom.md5);
        warn!("{}", msg);
        return Err(msg);
    }

    info!("ROM verified!");
    println!("ROM verified!");
    Ok(())
}

fn get_file_size(file_path: &PathBuf) -> u64 {
    std::fs::File::open(file_path)
        .unwrap()
        .metadata()
        .unwrap()
        .size()
}
