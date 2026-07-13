use std::path::{Component, Path, PathBuf};

use crate::core::resource::ResourceScheme;

use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            ResourceScheme::Res => self.source_path_for_project_uri(uri),
            ResourceScheme::Library => Err(AssetImportError::UnsupportedFormat(format!(
                "source path requested for library uri {uri}"
            ))),
            ResourceScheme::Package => {
                let package_id = uri.package_id().ok_or_else(|| {
                    AssetImportError::UnsupportedFormat(format!(
                        "source path requested for malformed package uri {uri}"
                    ))
                })?;
                let package_path = uri.package_path().ok_or_else(|| {
                    AssetImportError::UnsupportedFormat(format!(
                        "source path requested for package uri {uri} without a package path"
                    ))
                })?;
                let root = self
                    .package_assets
                    .root_for_package(package_id)
                    .ok_or_else(|| {
                        AssetImportError::UnsupportedFormat(format!(
                            "source path requested for unknown package {package_id}"
                        ))
                    })?;
                validate_relative_package_path(package_path)?;
                Ok(root.join(package_path))
            }
            ResourceScheme::Builtin | ResourceScheme::Memory => {
                Err(AssetImportError::UnsupportedFormat(format!(
                    "source path requested for non-project uri {uri}"
                )))
            }
        }
    }

    /// Resolves a not-yet-existing `res://` destination into the first manifest root.
    pub fn primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        if uri.scheme() != ResourceScheme::Res {
            return Err(AssetImportError::UnsupportedFormat(format!(
                "primary project destination requested for non-res uri {uri}"
            )));
        }
        validate_relative_package_path(uri.path())?;
        Ok(self.primary_project_asset_root()?.join(uri.path()))
    }

    /// Resolves an existing unique source, or explicitly chooses the primary root for a new one.
    pub fn existing_or_primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        match self.source_path_for_uri(uri) {
            Ok(path) => Ok(path),
            Err(AssetImportError::MissingProjectAssetUri { .. }) => {
                self.primary_project_source_path_for_uri(uri)
            }
            Err(error) => Err(error),
        }
    }

    fn source_path_for_project_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        validate_relative_package_path(uri.path())?;
        let existing = self
            .package_assets
            .project_roots()
            .iter()
            .map(|root| root.join(uri.path()))
            .filter(|candidate| candidate.exists())
            .collect::<Vec<_>>();
        match existing.as_slice() {
            [path] => Ok(path.clone()),
            [] => Err(AssetImportError::MissingProjectAssetUri { uri: uri.clone() }),
            _ => Err(AssetImportError::AmbiguousProjectAssetUri {
                uri: uri.clone(),
                paths: existing,
            }),
        }
    }
}

fn validate_relative_package_path(package_path: &str) -> Result<(), AssetImportError> {
    if Path::new(package_path).components().any(|component| {
        matches!(
            component,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "source path requested for package path {package_path} that escapes the package root"
        )));
    }
    Ok(())
}
