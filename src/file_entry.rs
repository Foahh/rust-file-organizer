use glob::Pattern;
use log::{error, info};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct FileEntry {
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> io::Result<FileEntry> {
        Ok(FileEntry { path })
    }

    /// Move the file to a new path.
    pub fn move_to(&mut self, new_path: &Path) -> io::Result<()> {
        info!("Moving \"{}\" to \"{}\"", self.path(), new_path.display());

        fs::create_dir_all(new_path.parent().unwrap())?;
        fs::rename(&self.path, &new_path)
            .map(|_| {
                self.path = new_path.to_path_buf();
            })
            .map_err(|e| {
                error!(
                    "Failed to move \"{}\" to \"{}\": {}",
                    self.path(),
                    new_path.display(),
                    e
                );
                e
            })
    }

    /// Check if the file is sorted.
    pub fn is_sorted(&self, globs: &[String], mapping: &HashMap<String, String>) -> bool {
        if self.match_globs(&globs) {
            info!("File \"{}\" is skipped", &self.path());
            return true;
        }
        let dir_name = self.parent_path();

        if let Some(target) = mapping.get(&self.extension()) {
            let pattern = format!("*/{}", target);
            if Pattern::new(&pattern).unwrap().matches(&dir_name) {
                return true;
            }
        } else if dir_name.ends_with("Others") {
            return true;
        }
        info!("File \"{}\" is not sorted", &self.path());
        false
    }

    /// Check if the file matches any of the given glob patterns.
    pub fn match_globs(&self, globs: &[String]) -> bool {
        globs.iter().any(|pattern| {
            let pattern = Pattern::new(pattern);

            pattern.map_or(false, |glob_pattern| {
                glob_pattern.matches(&self.path()) || glob_pattern.matches(&self.name())
            })
        })
    }

    /// Get the parent path of the file.
    pub fn parent_path(&self) -> Cow<str> {
        self.path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
    }

    /// Get the path of the file.
    pub fn path(&self) -> Cow<str> {
        self.path.to_string_lossy()
    }

    /// Get the name of the file (with the extension).
    pub fn name(&self) -> Cow<str> {
        self.path.file_name().unwrap().to_string_lossy()
    }

    /// Get the stem of the file name.
    pub fn stem(&self) -> Cow<str> {
        match self.path.file_stem() {
            Some(stem) => stem.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }

    /// Get the extension of the file name.
    pub fn extension(&self) -> String {
        match self.path.extension() {
            Some(ext) => format!(".{}", ext.to_string_lossy()),
            None => String::new(),
        }
    }
}
