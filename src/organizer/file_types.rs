use std::path::Path;

use phf::phf_map;

static ISO_GAMECUBE_OFFSET: usize = 0x1C;
static ISO_GAMECUBE_FINGERPRINT: [u8; 4] = [0xC2, 0x33, 0x9F, 0x3D];
static ISO_WII_OFFSET: usize = 0x18;
static ISO_WII_FINGERPRINT: [u8; 4] = [0x5D, 0x1C, 0x9E, 0xA3];

/*
*   TODO: Fingerprinting
*   GameCube
*   Wii
*   PS1
*   PS2
*   PS3
*   PSP
*   NES (URI)
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
pub fn get_console_id_by_ext(ext: &str) -> Option<&str> {
    EXT_MAP.get(ext).copied()
}

pub fn try_get_console_id(file_path: &Path) -> Option<&str> {

    let extension = file_path.extension().unwrap().to_str();

    if let Some(ext) = extension {
        let id = get_console_id_by_ext(ext);
        if id.is_some() {
            return id;
        } 
        println!("Could not find console by extension, try fingerprinting...");
    }

    try_fingerprint_iso(file_path)

}

pub fn try_fingerprint_iso(file_path: &Path) -> Option<&str> {

    // WARN: Will read the WHOLE file into RAM. Maybe we can read starting at the 
    // fingerprint offset?
    let try_read = std::fs::read(file_path);
    if try_read.is_err() {
        println!("Unable to read {:?}, skipping...", file_path.to_str());
        return None;
    }

    let file_contents = try_read.unwrap(); 
    if file_contents.len() <= ISO_GAMECUBE_OFFSET+4 {
        println!("File not large enough to fingerprint, skipping...");
        return None;
    }

    let gc_fingerprint = &file_contents[ISO_GAMECUBE_OFFSET..ISO_GAMECUBE_OFFSET+4];

    if gc_fingerprint == ISO_GAMECUBE_FINGERPRINT {
        return Some("GameCube");
    }

    None
}
