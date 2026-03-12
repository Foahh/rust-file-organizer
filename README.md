# File Organizer

A simple Rust-based file organizer that manages and organizes files in a specified directory. It supports custom sorting
rules, useful for managing folders like Downloads.

## Features

- **Sort Files**: Automatically sort files based on their extensions.
- **Move Duplicates**: Detect (CRC64-based) and move duplicate files to a "Duplicates" folder.
- **Remove Empty Folders**: Clean up empty folders in the target directory.

## Build

```sh
git clone https://github.com/NFHr/file-organizer-rs.git
cd file-organizer-rs
cargo build --release
```

## Usage

```sh
file-organizer --d <TARGET_DIRECTORY> [OPTIONS]
```

### Options

- `-d, <TARGET_DIRECTORY>`
- `-c, <CONFIG_PATH>`
- `--sort-files <BOOLEAN>`
- `--move-duplicates <BOOLEAN>`
- `--remove-empty-folders <BOOLEAN>`
- use `--help` for more information

## Organization Rules

The rule file (`rules.toml`) defines the sorting rules and ignored patterns.
