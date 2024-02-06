use std::{fs, path::Path};

use anyhow::{anyhow, Result};

pub struct FileManager {}

impl FileManager {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).map_err(|e| anyhow!("unable to read {:?}: {}", path, e))
    }
}
