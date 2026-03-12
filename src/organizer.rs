use crate::config::Config;
use crate::error::{Error, Result};
use crate::file_entry::FileEntry;
use crate::utils::{generate_unique_filename, is_dir_empty, rapidhash_file_checksum};
use glob::Pattern;
use log::{error, info};
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

pub struct Organizer {
    config: Config,
    files: Vec<FileEntry>,
}

impl Organizer {
    /// Scan all files in the working directory.
    pub fn new(config: Config) -> Result<Organizer> {
        let working_dir = &config.target;

        let mut files = Vec::new();
        for entry in Self::walk_known_dirs(working_dir, &config.known_folders)
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            files.push(FileEntry::new(entry.path().to_path_buf()));
        }

        // Sort files by modification time, descending
        files.sort_by_key(|f| {
            f.path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        files.reverse();

        Ok(Organizer { config, files })
    }

    /// Sort all files into their target folders based on extension mapping.
    pub fn sort_all_files(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        for file in &mut self.files {
            match file.is_sorted(&self.config.ignored, &self.config.mapping) {
                Ok(true) => continue,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
                Ok(false) => {}
            }

            let file_name = file.name().to_string();
            let ext = file.extension();

            // Try exact HashMap lookup first
            let target_folder = self.config.mapping.get(&*ext).cloned().or_else(|| {
                // Fall back to glob matching for wildcard patterns (e.g. r0*, r1*)
                self.config.mapping.iter().find_map(|(pattern, folder)| {
                    if pattern.contains('*') || pattern.contains('?') {
                        Pattern::new(pattern)
                            .ok()
                            .filter(|p| p.matches(&ext))
                            .map(|_| folder.clone())
                    } else {
                        None
                    }
                })
            });

            let new_path = match target_folder {
                Some(folder) => self.config.target.join(&folder).join(&file_name),
                None => self.config.target.join("Others").join(&file_name),
            };

            if let Err(e) = file.move_to(&new_path) {
                error!("Failed to sort \"{}\": {}", file.path(), e);
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Multiple(errors))
        }
    }

    /// Move duplicate files to a "Duplicates" folder.
    pub fn move_duplicates(&mut self) -> Result<()> {
        let mut hash_set = HashSet::new();
        let duplicate_dir = self.config.target.join("Duplicates");
        let mut dir_created = false;
        let mut errors = Vec::new();

        for file in &mut self.files {
            match file.match_globs(&self.config.ignored) {
                Ok(true) => continue,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
                Ok(false) => {}
            }

            let hash = match rapidhash_file_checksum(&file.path) {
                Ok(h) => h,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };

            if hash_set.contains(&hash) {
                // Lazily create Duplicates dir on first duplicate
                if !dir_created {
                    if let Err(e) = fs::create_dir_all(&duplicate_dir) {
                        errors.push(Error::Io(e));
                        continue;
                    }
                    dir_created = true;
                }

                let new_filename = match generate_unique_filename(file, &duplicate_dir) {
                    Ok(f) => f,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                if let Err(e) = file.move_to(&duplicate_dir.join(new_filename)) {
                    errors.push(e);
                }
            } else {
                hash_set.insert(hash);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Multiple(errors))
        }
    }

    /// Remove empty folders from the working directory.
    pub fn remove_empty_folders(&self) -> Result<()> {
        let target = &self.config.target;
        let mut directories: Vec<_> = Self::walk_known_dirs(target, &self.config.known_folders)
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.into_path())
            .collect();

        // Sort deepest-first so children are removed before parents
        directories.sort_by_key(|path| std::cmp::Reverse(path.clone()));

        let mut errors = Vec::new();
        for dir in directories {
            if is_dir_empty(&dir) {
                info!("Removing empty directory \"{}\"", dir.display());
                if let Err(e) = fs::remove_dir(&dir) {
                    errors.push(Error::Io(e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Multiple(errors))
        }
    }

    /// Walk directory tree, only entering known subdirectories.
    fn walk_known_dirs<'a>(
        root: &'a std::path::Path,
        known_folders: &'a HashSet<String>,
    ) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> + 'a {
        WalkDir::new(root).into_iter().filter_entry(move |e| {
            if e.path() == root {
                return true;
            }
            if e.file_type().is_dir() {
                let dir_name = e.file_name().to_string_lossy().to_string();
                return known_folders.contains(&dir_name);
            }
            true
        })
    }
}
