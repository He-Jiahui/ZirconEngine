use std::path::PathBuf;

use crate::core::resource::ResourceScheme;

use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            ResourceScheme::Res => Ok(self.paths.assets_root().join(uri.path())),
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
                Ok(root.join(package_path))
            }
            ResourceScheme::Builtin | ResourceScheme::Memory => {
                Err(AssetImportError::UnsupportedFormat(format!(
                    "source path requested for non-project uri {uri}"
                )))
            }
        }
    }
}
