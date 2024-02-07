use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use log::warn;

pub struct FileManager {}

pub enum DirEntryType {
    Directory,
    File,
}

pub struct DirEntry {
    pub full_path: PathBuf,
    pub name: String,
    pub file_type: DirEntryType,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).map_err(|e| anyhow!("unable to read {:?}: {}", path, e))
    }

    pub fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| anyhow!("unable to read {:?}: {}", path, e))
    }

    pub fn read_dir(&self, dir_path: &Path) -> Result<Vec<DirEntry>> {
        let dir = fs::read_dir(dir_path)
            .map_err(|e| anyhow!("unable to read directory {:?}: {}", dir_path, e))?;
        let mut entries = Vec::new();
        for entry in dir {
            let entry =
                entry.map_err(|e| anyhow!("unable to unwrap directory entry in {:?}", dir_path))?;
            let full_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let file_type = entry
                .file_type()
                .map_err(|e| anyhow!("unable to get file type for {:?}: {}", full_path, e))?;

            let file_type = if file_type.is_dir() {
                DirEntryType::Directory
            } else if file_type.is_file() {
                DirEntryType::File
            } else {
                warn!("skipping dir entry: {:?}", full_path);
                continue;
            };

            entries.push(DirEntry {
                full_path,
                name,
                file_type,
            });
        }
        Ok(entries)
    }
}
