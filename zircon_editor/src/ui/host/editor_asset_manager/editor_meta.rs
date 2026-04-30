use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EditorAssetMetaDocument {
    #[serde(default)]
    pub editor_adapter: Option<String>,
}

impl EditorAssetMetaDocument {
    pub(crate) fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let document = fs::read_to_string(path)?;
        toml::from_str(&document).map_err(invalid_data)
    }

    pub(crate) fn load_or_default(editor_meta_path: &Path) -> Result<Self, std::io::Error> {
        if editor_meta_path.exists() {
            return Self::load(editor_meta_path);
        }
        Ok(Self::default())
    }
}

pub(crate) fn editor_meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    path.with_file_name(format!("{file_name}.editor.meta.toml"))
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
