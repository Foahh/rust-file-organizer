use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),

    #[error("Multiple errors occurred:\n{}", .0.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n"))]
    Multiple(Vec<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;
