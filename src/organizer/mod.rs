mod file_types;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn organize(
    src: &PathBuf,
    dest: Option<&PathBuf>,
    copy: bool,
    sort_method: super::OrganizationType,
) -> std::io::Result<()> {
    let target_dir = if let Some(dir) = dest {
        dir
    } else {
        println!("No output path specified, using current file directory");
        src
    };

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

    // TODO: This is a very rudimentary way of organizing. Do this by console in the future
    for entry in entries {
        if entry.is_dir() {
            // TODO: Here is where we will need to fingerprint before treating it like
            // a rom directory
            println!(
                "{} is a directory. Directory scanning is not supported, skipping...",
                entry.to_str().unwrap()
            );
            continue;
        }

        let extension = entry.extension();

        if extension.is_none() {
            println!("{} has no extension, skipping...", entry.to_str().unwrap());
            continue;
        }

        let map_key = match sort_method {
            crate::OrganizationType::FileExtension => extension.unwrap().to_str().unwrap(),
            crate::OrganizationType::Console => {
                let console = file_types::try_get_console_id(&entry);
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

    println!("{:?}", output_map);
    /*
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

            if !copy {
                std::fs::rename(file, new_file_dest)?;
            } else {
                std::fs::copy(file, new_file_dest)?;
            }
        }
    }
    */

    Ok(())
}
