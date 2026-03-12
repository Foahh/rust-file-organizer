use crate::error::Result;
use crate::file_entry::FileEntry;
use rapidhash::fast::RapidHasher;
use std::ffi::OsString;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufReader, Read};
use std::path::Path;
use std::fs;

/// Check if a directory is empty.
pub fn is_dir_empty(dir: &Path) -> bool {
    fs::read_dir(dir)
        .ok()
        .is_some_and(|mut entries| entries.next().is_none())
}

/// Generate a unique filename if the file already exists in the destination.
pub fn generate_unique_filename(file: &FileEntry, target_dir: &Path) -> Result<OsString> {
    let stem = file.file_stem();
    let ext = file.file_extension();

    for counter in 1.. {
        let mut filename = OsString::from(stem);
        filename.push(format!("_{counter}"));
        if !ext.is_empty() {
            filename.push(".");
            filename.push(ext);
        }
        if !target_dir.join(&filename).exists() {
            return Ok(filename);
        }
    }

    unreachable!()
}

/// Calculate the rapidhash checksum of a file.
pub fn rapidhash_file_checksum(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = RapidHasher::default();
    let mut buffer = [0; 8192];

    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buffer[..bytes_read]);
    }

    Ok(format!("{:016x}", hasher.finish()))
}
