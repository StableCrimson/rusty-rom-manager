pub mod file_types;

use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, io};

use log::{debug, error, info, warn};

use crate::verify::Rom;

use super::OrganizationType;

pub fn organize(
    src: &PathBuf,
    dest: Option<&PathBuf>,
    copy: bool,
    sort_method: OrganizationType,
    recursive: bool,
) -> std::io::Result<()> {
    let mut roms = Vec::new();

    find_roms(src, &mut roms, recursive)?;

    let target_dir = if let Some(dir) = dest { dir } else { src };

    if !target_dir.exists() {
        info!(
            "{:?} does not exist, creating directory...",
            target_dir.as_path()
        );
        fs::create_dir_all(target_dir)?;
    }

    for rom in roms {
        let folder = match sort_method {
            OrganizationType::FileExtension => {
                if rom.path().is_dir() {
                    warn!(
                        "{:?} is a directory-level ROM and has no extension, skipping...",
                        rom.path()
                    );
                    continue;
                }
                String::from(rom.path().extension().unwrap().to_str().unwrap())
            }
            OrganizationType::Console => {
                format!("{:?}", rom.console_id())
            }
        };

        let mut new_file_dest = target_dir.to_owned();
        new_file_dest.push(folder);

        if !new_file_dest.exists() {
            info!("{:?} does not exist, creating directory...", new_file_dest);
            // fs::create_dir(&new_file_dest)?;
        }

        new_file_dest.push(rom.path().file_name().unwrap());
        println!("{:?}", new_file_dest);
        debug!("{:?}", new_file_dest);

        if new_file_dest.exists() {
            warn!("{:?} already exists, skipping...", new_file_dest);
            continue;
        }

        if rom.path().is_dir() {
            move_folder(rom.path(), new_file_dest, copy)?;
        } else {
            move_file(rom.path(), new_file_dest, copy)?;
        }
    }

    Ok(())
}

fn find_roms(root_folder: &Path, rom_list: &mut Vec<Rom>, recursive: bool) -> io::Result<()> {
    if !root_folder.exists() {
        let msg = format!("{:?} does not exist. Exitting", root_folder);
        error!("{}", msg);
        return Err(Error::new(ErrorKind::NotFound, msg));
    }

    let dir_contents = root_folder.read_dir()?;

    for entry in dir_contents {
        let entry = entry?.path();

        if entry.is_dir() {
            if file_types::check_dir_level_rom(&entry).is_none() {
                if !recursive {
                    continue;
                }

                find_roms(&entry, rom_list, recursive)?;
            }
        }

        let Ok(rom) = Rom::new(&entry) else {
            warn!("Unable to determine type of {:?}, skipping...", entry);
            continue;
        };
        rom_list.push(rom);
    }

    Ok(())
}

fn move_folder(src: impl AsRef<Path>, dest: impl AsRef<Path>, copy: bool) -> std::io::Result<()> {
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            move_folder(entry.path(), dest.as_ref().join(entry.file_name()), copy)?;
        } else {
            move_file(entry.path(), dest.as_ref().join(entry.file_name()), copy)?;
        }
    }
    Ok(())
}

fn move_file(src: impl AsRef<Path>, dest: impl AsRef<Path>, copy: bool) -> std::io::Result<()> {
    if copy {
        fs::copy(src, dest)?;
    } else {
        fs::rename(src, dest)?;
    }
    Ok(())
}
