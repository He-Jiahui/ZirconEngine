use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::AssetUri;

const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub format_version: u32,
    pub default_scene: AssetUri,
    pub library_version: u32,
}

impl ProjectManifest {
    pub fn new(name: impl Into<String>, default_scene: AssetUri, library_version: u32) -> Self {
        Self {
            name: name.into(),
            format_version: PROJECT_FORMAT_VERSION,
            default_scene,
            library_version,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let document = fs::read_to_string(path)?;
        toml::from_str(&document).map_err(invalid_data)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = toml::to_string_pretty(self).map_err(invalid_data)?;
        fs::write(path, document)
    }
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
