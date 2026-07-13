use std::path::Path;

use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    pub fn project_uri_for_source_path(&self, path: &Path) -> Result<AssetUri, AssetImportError> {
        let roots = self
            .project_asset_roots()
            .iter()
            .filter(|root| path.starts_with(root))
            .collect::<Vec<_>>();
        match roots.as_slice() {
            [root] => self.source_uri_for_path(root, path),
            [] => Err(AssetImportError::SourceOutsideProjectAssetRoots {
                path: path.to_path_buf(),
            }),
            _ => Err(AssetImportError::AmbiguousProjectSourcePath {
                path: path.to_path_buf(),
                roots: roots.into_iter().cloned().collect(),
            }),
        }
    }

    pub(super) fn source_uri_for_path(
        &self,
        asset_root: &Path,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        let relative = path.strip_prefix(asset_root).map_err(|_| {
            AssetImportError::SourceOutsideProjectAssetRoots {
                path: path.to_path_buf(),
            }
        })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!("res://{relative}"))?)
    }

    pub(super) fn source_uri_for_package_path(
        &self,
        package_id: &str,
        package_assets_root: &Path,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        let relative = path.strip_prefix(package_assets_root).map_err(|error| {
            AssetImportError::Parse(format!(
                "package asset path {} is outside package assets root {}: {error}",
                path.display(),
                package_assets_root.display()
            ))
        })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!(
            "package://{package_id}/{relative}"
        ))?)
    }
}
