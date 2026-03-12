use crate::error::{Error, Result};
use glob::Pattern;
use log::info;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> FileEntry {
        FileEntry { path }
    }

    /// Move the file to a new path.
    pub fn move_to(&mut self, new_path: &Path) -> Result<()> {
        info!("Moving \"{}\" to \"{}\"", self.path(), new_path.display());

        let parent = new_path.parent().ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path has no parent: {}", new_path.display()),
            ))
        })?;
        fs::create_dir_all(parent)?;
        fs::rename(&self.path, new_path)?;
        self.path = new_path.to_path_buf();
        Ok(())
    }

    /// Check if the file is already sorted into the correct folder.
    pub fn is_sorted(&self, globs: &[String], mapping: &HashMap<String, String>) -> Result<bool> {
        if self.match_globs(globs)? {
            info!("File \"{}\" is skipped", &self.path());
            return Ok(true);
        }
        let dir_name = self.parent_path();
        let ext = self.extension();

        // Try exact lookup first, then glob fallback for wildcard patterns (e.g. r0*, r1*)
        let target = mapping.get(&*ext).cloned().or_else(|| {
            mapping.iter().find_map(|(pattern, folder)| {
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

        if let Some(target) = target {
            let pattern = format!("*/{}", target);
            if Pattern::new(&pattern)?.matches(&dir_name) {
                return Ok(true);
            }
        } else if dir_name.ends_with("Others") {
            return Ok(true);
        }
        info!("File \"{}\" is not sorted", &self.path());
        Ok(false)
    }

    /// Check if the file matches any of the given glob patterns.
    pub fn match_globs(&self, globs: &[String]) -> Result<bool> {
        for pattern in globs {
            let glob_pattern = Pattern::new(pattern)?;
            if glob_pattern.matches(&self.path()) || glob_pattern.matches(&self.name()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get the parent path of the file.
    pub fn parent_path(&self) -> Cow<'_, str> {
        self.path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
    }

    /// Get the full path as a string.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.to_string_lossy()
    }

    /// Get the file name (with extension).
    pub fn name(&self) -> Cow<'_, str> {
        match self.path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }

    /// Get the stem of the file name (without extension).
    pub fn stem(&self) -> Cow<'_, str> {
        match self.path.file_stem() {
            Some(stem) => stem.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }

    /// Get the extension without the dot prefix (e.g. "pdf", not ".pdf").
    pub fn extension(&self) -> Cow<'_, str> {
        match self.path.extension() {
            Some(ext) => ext.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }
}
