use std::fs::File;
use std::io::{BufReader, Read};
use std::{os::unix::fs::MetadataExt, path::Path};

use phf::phf_map;

const ISO_MIN_SIZE: u64 = 0xF000;
const ISO_BUFFER_LEN: usize = 0xF000;
const ISO_GAMECUBE_OFFSET: usize = 0x1C;
const ISO_GAMECUBE_FINGERPRINT: [u8; 4] = [0xC2, 0x33, 0x9F, 0x3D];
const ISO_WII_OFFSET: usize = 0x18;
const ISO_WII_FINGERPRINT: [u8; 4] = [0x5D, 0x1C, 0x9E, 0xA3];

/*
*   TODO: Fingerprinting
*   PS1
*   PS2
*   PS3
*   PSP
*/

static EXT_MAP: phf::Map<&str, &str> = phf_map! {
    "gb" => "Gameboy",
    "gbc" => "Gameboy Color",
    "gba" => "Gameboy Advance",
    "cdi" => "Dreamcast",
    "gdi" => "Dreamcast",
    "nes" => "NES",
    "nez" => "NES",
    "unf" => "NES",
    "unif" => "NES",
    "smc" => "SNES",
    "sfc" => "SNES",
    "md" => "Genesis",
    "smd" => "Genesis",
    "gen" => "Genesis",
    "gg" => "Game Gear",
    "z64" => "Nintendo 64",
    "n64" => "Nintendo 64",
    "v64" => "Nintendo 64",
    "gcm" => "GameCube",
    "gcz" => "GameCube",
    "xiso" => "Xbox",
    "nds" => "Nintendo DS",
    "dsi" => "Nintendo DSi",
    "wbfs" => "Wii",
    "wad" => "Wii",
    "cia" => "3DS",
    "3ds" => "3DS",
    "nsp" => "Nintendo Switch",
    "xci" => "Nintendo Switch",
    "ngp" => "Neo Geo",
    "ngc" => "Neo Geo",
    "pce" => "PC Engine",
    "vpk" => "PlayStation Vita",
    "vb" => "Virtual Boy",
    "ws" => "WonderSwan",
    "wsc" => "WonderSwan Color"
};

// NOTE: Maybe add support for sorting saves, too?
fn get_console_id_by_ext(ext: &str) -> Option<&str> {
    EXT_MAP.get(ext).copied()
}

pub fn try_get_console_id(file_path: &Path) -> Option<&str> {
    let extension = file_path.extension().unwrap().to_str();

    if let Some(ext) = extension {
        if let Some(id) = get_console_id_by_ext(ext) {
            return Some(id);
        }
    }

    try_fingerprint_iso(file_path)
}

fn try_fingerprint_iso(file_path: &Path) -> Option<&str> {
    let target_file = File::open(file_path);
    if target_file.is_err() {
        return None;
    }

    let file = target_file.unwrap();
    let file_size = &file.metadata().unwrap().size();

    if *file_size < ISO_MIN_SIZE {
        println!("{:?} not large enough to fingerprint, skipping...", file_path);
        return None;
    }

    let mut buffer = BufReader::new(file);
    let mut file_contents = [0_u8; ISO_BUFFER_LEN];

    if buffer.read_exact(&mut file_contents).is_err() {
        println!("Error reading file {:?} to buffer, skipping...", file_path);
    }

    let wii_fingerprint = &file_contents[ISO_WII_OFFSET..ISO_WII_OFFSET + 4];
    if wii_fingerprint == ISO_WII_FINGERPRINT {
        return Some("Wii");
    }

    let gc_fingerprint = &file_contents[ISO_GAMECUBE_OFFSET..ISO_GAMECUBE_OFFSET + 4];
    if gc_fingerprint == ISO_GAMECUBE_FINGERPRINT {
        return Some("GameCube");
    }

    None
}
