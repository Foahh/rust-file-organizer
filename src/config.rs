use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Deserialize)]
struct Rules {
    mapping: HashMap<String, Vec<String>>,
    ignore: Vec<String>,
}

pub struct Config {
    pub target: PathBuf,
    pub mapping: HashMap<String, String>,
    pub ignored: Vec<String>,
}

impl Config {
    pub fn new(target: &str) -> Result<Self, io::Error> {
        let path = PathBuf::from(target);

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory {} does not exist", path.display()),
            ));
        }

        let content = fs::read_to_string("rules.toml").map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "The 'rules.toml' file was not found. Please ensure the file exists in the current directory and is properly formatted.",
                )
            } else {
                err
            }
        })?;

        let rules: Rules = toml::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse 'rules.toml': {}", e),
            )
        })?;

        let mut mapping: HashMap<String, String> = HashMap::new();
        for (key, exts) in rules.mapping {
            for ext in exts {
                let formatted_ext = if ext.starts_with('.') {
                    ext
                } else {
                    format!(".{}", ext)
                };
                mapping.insert(formatted_ext, key.clone());
            }
        }

        Ok(Self {
            target: path,
            mapping,
            ignored: rules.ignore,
        })
    }
}
