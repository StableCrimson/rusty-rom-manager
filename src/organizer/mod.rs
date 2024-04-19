mod file_types;

use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::OrganizationType;

pub fn organize(
    src: &PathBuf,
    dest: Option<&PathBuf>,
    copy: bool,
    sort_method: OrganizationType,
) -> std::io::Result<()> {
    let target_dir = if let Some(dir) = dest { dir } else { src };

    if !src.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("{:?} does not exist. Exitting", src.as_path()),
        ));
    }

    let dir_contents = src.read_dir()?;

    // TODO: Allow for recursively scanning directories
    let entries = dir_contents
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    if !target_dir.exists() {
        println!(
            "{:?} does not exist, creating directory...",
            target_dir.as_path()
        );
        std::fs::create_dir(target_dir)?;
        println!("{:?} created successfully", target_dir.as_path());
    }

    let mut output_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for entry in entries {
        if entry.is_dir() {
            // TODO: What to do if fingerprinting fails and recursive scanning
            // is enabled?
            // Ex: If it's a malformed PS3 game and the user is scanning
            // subdirectories then are we going to scan thousands of files?
            if let Some(id) = file_types::check_dir_level_rom(&entry) {
                output_map.entry(id.to_string()).or_default().push(entry);
            }
            continue;
        }

        let extension = entry.extension();

        if extension.is_none() {
            println!("{:?} has no extension, skipping...", entry);
            continue;
        }

        let map_key = match sort_method {
            crate::OrganizationType::FileExtension => extension.unwrap().to_str().unwrap(),
            crate::OrganizationType::Console => {
                let console = file_types::get_console_id(&entry);
                if console.is_none() {
                    println!("Unable to determine type of {:?}, skipping...", entry);
                    continue;
                }

                console.unwrap()
            }
        };

        output_map
            .entry(map_key.to_string())
            .or_default()
            .push(entry);
    }

    println!("{:#?}", output_map);

    // NOTE: What if the file already exists???
    // As it is now, the original file will be overwritten.
    // Do we want to rename it so that file becomes file-1.gb? Or should we just skip?
    for (ext, path) in &output_map {
        for file in path {
            let mut new_file_dest = target_dir.to_owned();
            new_file_dest.push(ext);

            if !new_file_dest.exists() {
                println!("{:?} does not exist, creating directory...", new_file_dest);
                std::fs::create_dir(&new_file_dest)?;
            }

            new_file_dest.push(file.file_name().unwrap());
            println!("{:?}", new_file_dest);

            if file.is_dir() {
                move_folder(file, new_file_dest, copy)?;
            } else {
                move_file(file, new_file_dest, copy)?;
            }
        }
    }

    Ok(())
}

fn move_folder(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    copy: bool,
) -> std::io::Result<()> {
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
