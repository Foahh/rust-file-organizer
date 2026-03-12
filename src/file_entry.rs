use crate::error::{Error, Result};
use glob::Pattern;
use log::info;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
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
        info!("Moving \"{}\" to \"{}\"", self.path.display(), new_path.display());

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
            info!("File \"{}\" is skipped", self.path.display());
            return Ok(true);
        }
        let dir_name = self.parent_path_lossy();
        let ext = self.extension_lossy();

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
        info!("File \"{}\" is not sorted", self.path.display());
        Ok(false)
    }

    /// Check if the file matches any of the given glob patterns.
    pub fn match_globs(&self, globs: &[String]) -> Result<bool> {
        let path_str = self.path_lossy();
        let name_str = self.file_name().to_string_lossy();
        for pattern in globs {
            let glob_pattern = Pattern::new(pattern)?;
            if glob_pattern.matches(&path_str) || glob_pattern.matches(&name_str) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get the parent path as a lossy string (for display and glob matching).
    pub fn parent_path_lossy(&self) -> Cow<'_, str> {
        self.path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
    }

    /// Get the full path as a lossy string (for display and glob matching).
    pub fn path_lossy(&self) -> Cow<'_, str> {
        self.path.to_string_lossy()
    }

    /// Get the file name as an OsStr (preserves non-UTF-8 bytes).
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or_default()
    }

    /// Get the stem of the file name as an OsStr (preserves non-UTF-8 bytes).
    pub fn file_stem(&self) -> &OsStr {
        self.path.file_stem().unwrap_or_default()
    }

    /// Get the extension as an OsStr (preserves non-UTF-8 bytes).
    pub fn file_extension(&self) -> &OsStr {
        self.path.extension().unwrap_or_default()
    }

    /// Get the extension as a lossy string (for HashMap lookups against UTF-8 config keys).
    pub fn extension_lossy(&self) -> Cow<'_, str> {
        match self.path.extension() {
            Some(ext) => ext.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }
}
