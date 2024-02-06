use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use log::debug;

use crate::imagemanager::ImageLoader;
use crate::tilemap::TileMap;

struct LevelDirectory {
    directory: PathBuf,
    directories: BTreeMap<PathBuf, Rc<LevelDirectory>>,
    files: BTreeMap<PathBuf, Rc<TileMap>>,
}

impl LevelDirectory {
    pub fn from_path(directory: &Path, images: &mut dyn ImageLoader) -> Result<Self> {
        let mut directories = BTreeMap::new();
        let mut files = BTreeMap::new();

        debug!("Scanning directory {:?}", directory);
        let file_list = fs::read_dir(&directory)
            .map_err(|e| anyhow!("unable to read {:?}: {}", directory, e))?;
        for entry in file_list {
            let entry = entry.map_err(|e| {
                anyhow!("error iterating through contents of {:?}: {}", directory, e)
            })?;
            let path = entry.path();
            let name = entry
                .file_name()
                .to_str()
                .context(format!("unable to encode name {:?}", path))?
                .to_owned();
            debug!("Found directory entry {:?} named {}", path, &name);

            let file_type = entry
                .file_type()
                .map_err(|e| anyhow!("unable to to determine type of {:?}: {}", path, e))?;
            if file_type.is_dir() {
                let subdirectory = LevelDirectory::from_path(&path, images)?;
                let subdirectory = Rc::new(subdirectory);
                directories.insert(path, subdirectory);
            } else if file_type.is_file() {
                let file = TileMap::from_file(&path, images)?;
                let file = Rc::new(file);
                files.insert(path, file);
            }
        }

        let directory = directory.to_owned();

        Ok(Self {
            directory,
            directories,
            files,
        })
    }
}

pub struct LevelIndex {
    root: Rc<LevelDirectory>,
}

pub enum LevelIndexEntry {
    Level(PathBuf),
    Directory(PathBuf),
}

impl LevelIndex {
    pub fn from_path(directory: &Path, images: &mut dyn ImageLoader) -> Result<Self> {
        let root = LevelDirectory::from_path(directory, images)?;
        let root = Rc::new(root);
        Ok(Self { root })
    }

    pub fn list(path: &Path) -> Result<Vec<LevelIndexEntry>> {}
}
