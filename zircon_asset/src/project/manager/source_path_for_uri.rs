use std::path::PathBuf;

use crate::{AssetImportError, AssetUri, AssetUriScheme};

use super::ProjectManager;

impl ProjectManager {
    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            AssetUriScheme::Res => Ok(self.paths.assets_root().join(uri.path())),
            AssetUriScheme::Library => Err(AssetImportError::UnsupportedFormat(format!(
                "source path requested for library uri {uri}"
            ))),
            AssetUriScheme::Builtin | AssetUriScheme::Memory => {
                Err(AssetImportError::UnsupportedFormat(format!(
                    "source path requested for non-project uri {uri}"
                )))
            }
        }
    }
}
