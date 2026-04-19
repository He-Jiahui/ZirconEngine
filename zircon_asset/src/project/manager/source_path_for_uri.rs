use std::path::PathBuf;

use zircon_resource::ResourceScheme;

use crate::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            ResourceScheme::Res => Ok(self.paths.assets_root().join(uri.path())),
            ResourceScheme::Library => Err(AssetImportError::UnsupportedFormat(format!(
                "source path requested for library uri {uri}"
            ))),
            ResourceScheme::Builtin | ResourceScheme::Memory => {
                Err(AssetImportError::UnsupportedFormat(format!(
                    "source path requested for non-project uri {uri}"
                )))
            }
        }
    }
}
