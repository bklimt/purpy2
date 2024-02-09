use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use log::{error, info, warn};

use crate::utils::normalize_path;

pub enum DirEntryType {
    Directory,
    File,
}

pub struct DirEntry {
    pub full_path: PathBuf,
    pub name: String,
    pub file_type: DirEntryType,
}

trait FileManagerImpl {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn read_dir(&self, dir_path: &Path) -> Result<Vec<DirEntry>>;
}

struct DefaultFileManagerImpl {}

impl FileManagerImpl for DefaultFileManagerImpl {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let path = normalize_path(path)?;
        fs::read(&path).map_err(|e| anyhow!("unable to read {:?}: {}", &path, e))
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| anyhow!("unable to read {:?}: {}", path, e))
    }

    fn read_dir(&self, dir_path: &Path) -> Result<Vec<DirEntry>> {
        let dir_path = normalize_path(dir_path)?;

        let dir = fs::read_dir(&dir_path)
            .map_err(|e| anyhow!("unable to read directory {:?}: {}", &dir_path, e))?;
        let mut entries = Vec::new();
        for entry in dir {
            let entry = entry.map_err(|e| {
                anyhow!("unable to unwrap directory entry in {:?}: {}", &dir_path, e)
            })?;
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

struct ArchiveFileManager {
    files: BTreeMap<PathBuf, Vec<u8>>,
}

impl ArchiveFileManager {
    pub fn from_file(path: &Path) -> Result<ArchiveFileManager> {
        info!("Reading archive {:?}", path);
        let file = fs::File::open(path)
            .map_err(|e| anyhow!("unable to open archive at {:?}: {}", path, e))?;
        Self::from_reader(file)
            .map_err(|e| anyhow!("error reading archive from file {:?}: {}", path, e))
    }

    pub fn from_reader<R>(reader: R) -> Result<ArchiveFileManager>
    where
        R: Read,
    {
        let gz_file = GzDecoder::new(reader);

        let mut tar_file = tar::Archive::new(gz_file);
        let entries = tar_file
            .entries()
            .map_err(|e| anyhow!("unable to read entries of archive: {}", e))?;

        let mut files = BTreeMap::new();

        for entry in entries {
            let mut entry = entry.map_err(|e| anyhow!("error with entry: {}", e))?;
            let file_path = entry
                .path()
                .map_err(|e| anyhow!("error decoding path: {}", e))?
                .to_path_buf();
            info!("  {:?}", file_path);

            let mut data = Vec::new();
            let _ = entry
                .read_to_end(&mut data)
                .map_err(|e| anyhow!("unable to read bytes for {:?}: {}", file_path, e))?;
            files.insert(file_path, data);
        }

        Ok(ArchiveFileManager { files })
    }
}

impl FileManagerImpl for ArchiveFileManager {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let path = normalize_path(path)?;
        let Some(data) = self.files.get(&path) else {
            return Err(anyhow!("file not found: {:?}", &path));
        };
        Ok(data.clone())
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        let data = self.read(path)?;
        let s = String::from_utf8(data)
            .map_err(|e| anyhow!("unable to convert data to string for {:?}: {}", path, e))?;
        Ok(s)
    }

    fn read_dir(&self, dir_path: &Path) -> Result<Vec<DirEntry>> {
        let dir_path = normalize_path(dir_path)?;
        let mut children: Vec<DirEntry> = self
            .files
            .keys()
            .filter_map(|known_path| {
                if !known_path.starts_with(&dir_path) {
                    return None;
                }
                let rest = match known_path.strip_prefix(&dir_path) {
                    Ok(rest) => rest,
                    Err(e) => {
                        error!(
                            "unable to strip prefix {:?} from {:?}: {}",
                            &dir_path, known_path, e
                        );
                        return None;
                    }
                };

                let file_type = if rest.components().count() == 1 {
                    DirEntryType::File
                } else {
                    DirEntryType::Directory
                };

                let child = Path::new(rest.components().nth(0).unwrap().as_os_str());
                let full_path = dir_path.join(child);
                let name = child.to_string_lossy().to_string();

                Some(DirEntry {
                    full_path,
                    name,
                    file_type,
                })
            })
            .collect();

        children.dedup_by_key(|entry| entry.name.clone());

        Ok(children)
    }
}

pub struct FileManager {
    internal: Box<dyn FileManagerImpl>,
}

impl FileManager {
    pub fn from_fs() -> Result<Self> {
        Ok(Self {
            internal: Box::new(DefaultFileManagerImpl {}),
        })
    }

    pub fn from_archive_file(path: &Path) -> Result<Self> {
        Ok(Self {
            internal: Box::new(ArchiveFileManager::from_file(path)?),
        })
    }

    pub fn from_archive_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            internal: Box::new(ArchiveFileManager::from_reader(bytes)?),
        })
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.internal.read(path)
    }

    pub fn read_to_string(&self, path: &Path) -> Result<String> {
        self.internal.read_to_string(path)
    }

    pub fn read_dir(&self, dir_path: &Path) -> Result<Vec<DirEntry>> {
        self.internal.read_dir(dir_path)
    }
}
