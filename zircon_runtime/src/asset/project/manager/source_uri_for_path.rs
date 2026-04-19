use std::path::Path;

use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    pub(super) fn source_uri_for_path(&self, path: &Path) -> Result<AssetUri, AssetImportError> {
        let relative = path
            .strip_prefix(self.paths.assets_root())
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "asset path {} is outside assets root {}: {error}",
                    path.display(),
                    self.paths.assets_root().display()
                ))
            })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!("res://{relative}"))?)
    }
}
