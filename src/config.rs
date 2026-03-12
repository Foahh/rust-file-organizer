use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Rules {
    mapping: HashMap<String, Vec<String>>,
    ignore: Vec<String>,
}

pub struct Config {
    pub target: PathBuf,
    pub mapping: HashMap<String, String>,
    pub ignored: Vec<String>,
    pub known_folders: HashSet<String>,
}

impl Config {
    pub fn new(target: impl Into<PathBuf>, config_path: impl Into<PathBuf>) -> Result<Self> {
        let target = target.into();
        let config_path = config_path.into();

        if !target.exists() {
            return Err(Error::DirectoryNotFound(target));
        }

        let content =
            fs::read_to_string(&config_path).map_err(|_| Error::ConfigNotFound(config_path))?;

        let rules: Rules = toml::from_str(&content)?;

        let mut mapping: HashMap<String, String> = HashMap::new();
        for (key, exts) in rules.mapping {
            for ext in exts {
                // Store without dot prefix — FileEntry::extension() returns bare extension
                let bare_ext = if ext.starts_with('.') {
                    ext[1..].to_string()
                } else {
                    ext
                };
                mapping.insert(bare_ext, key.clone());
            }
        }

        let known_folders: HashSet<String> = mapping
            .values()
            .cloned()
            .chain(["Others".to_string(), "Duplicates".to_string()])
            .collect();

        Ok(Self {
            target,
            mapping,
            ignored: rules.ignore,
            known_folders,
        })
    }
}
