use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::utils::{rapidhash_file_checksum, generate_unique_filename, is_dir_empty};
use glob::Pattern;
use log::error;
use std::collections::HashSet;
use std::{fs, io};
use walkdir::WalkDir;

pub struct Organizer {
    config: Config,
    files: Vec<FileEntry>,
}

impl Organizer {
    /// Scan all files in the working directory and return a `FileOrganizer` object.
    pub fn new(config: Config) -> io::Result<Organizer> {
        let working_dir = &config.target;

        // Collect known folder names from the mapping (e.g. "Documents", "Pictures")
        // plus built-in folders. Only these top-level subdirectories are scanned.
        let known_folders: HashSet<String> = config
            .mapping
            .values()
            .cloned()
            .chain(["Others".to_string(), "Duplicates".to_string()])
            .collect();

        let mut files = Vec::new();
        for entry in WalkDir::new(working_dir)
            .into_iter()
            .filter_entry(|e| {
                if e.path() == working_dir.as_path() {
                    return true;
                }
                // Skip any unrecognized directory at any depth
                if e.file_type().is_dir() {
                    let dir_name = e.file_name().to_string_lossy().to_string();
                    return known_folders.contains(&dir_name);
                }
                true
            })
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            match FileEntry::new(entry.path().to_path_buf()) {
                Ok(fe) => files.push(fe),
                Err(e) => error!(
                    "Error processing file \"{}\": {}",
                    entry.path().display(),
                    e
                ),
            }
        }

        // sort files by modification time, descending
        files.sort_by_key(|f| {
            f.path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        files.reverse();

        Ok(Organizer { config, files })
    }

    /// Sort all the files.
    pub fn sort_all_files(&mut self) -> io::Result<()> {
        for file in &mut self.files {
            if file.is_sorted(&self.config.ignored, &self.config.mapping) {
                continue;
            }

            let file_name = file.name().to_string();
            let mut new_path = self.config.target.join("Others").join(&file_name);
            for (ext, target) in &self.config.mapping {
                if Pattern::new(&format!("*{}", ext))
                    .unwrap()
                    .matches(&file_name)
                {
                    new_path = self.config.target.join(target).join(&file_name);
                    break;
                }
            }
            file.move_to(&new_path)?;
        }
        Ok(())
    }

    /// Remove duplicates by moving them to a "Duplicates" folder.
    pub fn move_duplicates(&mut self) -> io::Result<()> {
        let mut hash_set = HashSet::new();
        let duplicate_dir = &self.config.target.join("Duplicates");

        if !duplicate_dir.exists() {
            fs::create_dir_all(&duplicate_dir)?;
        }

        for file in &mut self.files {
            if file.match_globs(&self.config.ignored) {
                continue;
            }

            let hash = rapidhash_file_checksum(&file.path)?;

            if hash_set.contains(&hash) {
                let new_filename = generate_unique_filename(&file, &duplicate_dir)?;
                file.move_to(&duplicate_dir.join(new_filename))?;
            } else {
                hash_set.insert(hash);
            }
        }

        Ok(())
    }

    /// Remove empty folders from the working directory.
    pub fn remove_empty_folders(&self) -> io::Result<()> {
        let known_folders: HashSet<String> = self
            .config
            .mapping
            .values()
            .cloned()
            .chain(["Others".to_string(), "Duplicates".to_string()])
            .collect();

        let target = &self.config.target;
        let mut directories: Vec<_> = WalkDir::new(target)
            .into_iter()
            .filter_entry(|e| {
                if e.path() == target.as_path() {
                    return true;
                }
                if e.file_type().is_dir() {
                    let dir_name = e.file_name().to_string_lossy().to_string();
                    return known_folders.contains(&dir_name);
                }
                true
            })
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.into_path())
            .collect();

        directories.sort_by_key(|path| std::cmp::Reverse(path.clone()));

        for dir in directories {
            if is_dir_empty(&dir) {
                println!("Removing empty directory \"{}\"", dir.display());
                fs::remove_dir(&dir)?;
            }
        }

        Ok(())
    }
}
