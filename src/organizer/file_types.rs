use std::fs::File;
use std::io::{BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use log::{error, warn};
use log4rs::append::file;

const ISO_MIN_SIZE: u64 = 0xF000;
const ISO_BUFFER_LEN: usize = 0xF000;

const ISO_GAMECUBE_OFFSET: usize = 0x1C;
const ISO_GAMECUBE_FINGERPRINT: [u8; 4] = [0xC2, 0x33, 0x9F, 0x3D];
const ISO_WII_OFFSET: usize = 0x18;
const ISO_WII_FINGERPRINT: [u8; 4] = [0x5D, 0x1C, 0x9E, 0xA3];

const ISO_PS2_USA_JPN_OFFSET: usize = 0x42F;
const ISO_PS2_EUR_OFFSET: usize = 0xA97;
const ISO_PS2_USA_FINGERPRINT: [u8; 27] = [
    0x06, 0x01, 0x00, 0x00, 0x03, 0x03, 0x02, 0x02, 0x02, 0x0D, 0x0D, 0x0C, 0x0C, 0x0E, 0x0E, 0x0E,
    0x09, 0x08, 0x08, 0x08, 0x08, 0x09, 0x0E, 0x0D, 0x00, 0x06, 0x05,
];
const ISO_PS2_JPN_FINGERPRINT: [u8; 27] = [
    0x0E, 0x09, 0x08, 0x08, 0x0B, 0x0B, 0x0A, 0x0A, 0x0A, 0x05, 0x05, 0x04, 0x04, 0x06, 0x06, 0x06,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x05, 0x08, 0x0E, 0x0D,
];
const ISO_PS2_EUR_FINGERPRINT: [u8; 67] = [
    0x0E, 0x09, 0x09, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
    0x0F, 0x0F, 0x0F, 0x0E, 0x09, 0x05, 0x02, 0x00, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x06,
    0x0F, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x04, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x04, 0x05, 0x0B,
    0x08, 0x0E, 0x0D,
];
const ISO_PSP_OFFSET: usize = 0x8000;
const ISO_PSP_FINGERPRINT: [u8; 16] = [
    0x01, 0x43, 0x44, 0x30, 0x30, 0x31, 0x01, 0x00, 0x50, 0x53, 0x50, 0x20, 0x47, 0x41, 0x4D, 0x45,
];

const DIR_PS3_STRUCTURE: [&str; 4] = [
    "PS3_DISC.SFB",
    "PS3_GAME/ICON0.PNG",
    "PS3_GAME/PARAM.SFO",
    "PS3_GAME/PS3LOGO.DAT",
];

// TODO: Add regions?

#[derive(Copy, Clone, Debug)]
pub enum Console {
    NES,
    SNES,
    VirtualBoy,
    N64,
    GameCube,
    Wii,
    Switch,
    Gameboy,
    GameboyColor,
    GameboyAdvance,
    NDS,
    DS3,
    Playstation2,
    Playstation3,
    PSP,
    PlaystationVita,
    Dreamcast,
    Genesis,
    GameGear,
    NeoGeo,
    WonderSwan,
    WonderSwanColor,
    Xbox,
}

// NOTE: Maybe add support for sorting saves, too?
fn get_console_id_by_ext(ext: &str) -> Option<Console> {
    match ext {
        "gb" => Some(Console::Gameboy),
        "gbc" => Some(Console::GameboyColor),
        "gba" => Some(Console::GameboyAdvance),
        "cdi" | "gdi" => Some(Console::Dreamcast),
        "nes" | "nez" | "unf" | "unif" => Some(Console::NES),
        "sfc" | "smc" => Some(Console::SNES),
        "gen" | "md" | "smd" => Some(Console::Genesis),
        "gg" => Some(Console::GameGear),
        "n64" | "v64" | "z64" => Some(Console::N64),
        "gcm" | "gcz" => Some(Console::GameCube),
        "xiso" => Some(Console::Xbox),
        "nds" | "dsi" => Some(Console::NDS),
        "wad" | "wbfs" => Some(Console::Wii),
        "3ds" | "cia" => Some(Console::DS3),
        "nsp" | "xci" => Some(Console::Switch),
        "ngp" | "ngc" => Some(Console::NeoGeo),
        "vpk" => Some(Console::PlaystationVita),
        "vb" => Some(Console::VirtualBoy),
        "ws" => Some(Console::WonderSwan),
        "wsc" => Some(Console::WonderSwanColor),
        _ => None,
    }
}

pub fn get_console_id(file_path: &Path) -> Option<Console> {

    if file_path.is_dir() {
        return check_dir_level_rom(file_path);
    }

    let extension = file_path.extension()?.to_str()?;

    if let Some(id) = get_console_id_by_ext(extension) {
        return Some(id);
    }

    // TODO: CHD, PBP, RVZ?, BIN+QUE?
    match extension {
        "iso" => try_fingerprint_iso(file_path),
        // "chd" => try_fingerprint_chd(file_path),
        // "pbp" => try_fingerprint_pbp(file_path),
        // "rvz" => try_fingerprint_rvz(file_path),
        _ => None,
    }
}

pub fn check_dir_level_rom(file_path: &Path) -> Option<Console> {
    if !file_path.is_dir() {
        return None;
    }

    for file in DIR_PS3_STRUCTURE {
        if !file_path.join(file).exists() {
            return None;
        }
    }
    Some(Console::Playstation3)
}

fn try_fingerprint_iso(file_path: &Path) -> Option<Console> {
    let target_file = File::open(file_path);
    if target_file.is_err() {
        return None;
    }

    let file = target_file.unwrap();
    let file_size = &file.metadata().unwrap().size();

    if *file_size < ISO_MIN_SIZE {
        warn!(
            "{:?} not large enough to fingerprint, skipping...",
            file_path
        );
        return None;
    }

    let mut buffer = BufReader::new(file);
    let mut file_contents = [0_u8; ISO_BUFFER_LEN];

    if buffer.read_exact(&mut file_contents).is_err() {
        error!("Error reading file {:?} to buffer, skipping...", file_path);
    }

    if is_fingerprint_match(&file_contents, ISO_WII_OFFSET, &ISO_WII_FINGERPRINT) {
        return Some(Console::Wii);
    }

    if is_fingerprint_match(
        &file_contents,
        ISO_GAMECUBE_OFFSET,
        &ISO_GAMECUBE_FINGERPRINT,
    ) {
        return Some(Console::GameCube);
    }

    // TODO: PSX ISOs

    if is_ps2_game(&file_contents) {
        return Some(Console::Playstation2);
    }

    if is_fingerprint_match(&file_contents, ISO_PSP_OFFSET, &ISO_PSP_FINGERPRINT) {
        return Some(Console::PSP);
    }

    None
}

fn is_fingerprint_match(buff: &[u8], offset: usize, fingerprint: &[u8]) -> bool {
    buff[offset..offset + fingerprint.len()] == *fingerprint
}

fn is_ps2_game(buf: &[u8]) -> bool {
    // NOTE: We need to mask off the upper 4 bits to match the fingerprint
    let masked_buf: Vec<u8> = buf.iter().map(|b| b & 0b0000_1111).collect();

    is_fingerprint_match(
        &masked_buf,
        ISO_PS2_USA_JPN_OFFSET,
        &ISO_PS2_USA_FINGERPRINT,
    ) || is_fingerprint_match(
        &masked_buf,
        ISO_PS2_USA_JPN_OFFSET,
        &ISO_PS2_JPN_FINGERPRINT,
    ) || is_fingerprint_match(&masked_buf, ISO_PS2_EUR_OFFSET, &ISO_PS2_EUR_FINGERPRINT)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_return_none_if_no_file_extension() {
        let file_path = Path::new("my_file");
        assert!(get_console_id(file_path).is_none())
    }

    #[test]
    fn should_return_none_if_unknown_extension() {
        let file_path = Path::new("my_file.asdf");
        assert!(get_console_id(file_path).is_none())
    }
}
