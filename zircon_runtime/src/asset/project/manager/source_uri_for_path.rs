//! 已存在的物理源经根身份解析后才得到 res:// URI；扫描器以此保持别名根下的源身份稳定，目标创建另走首根选择。

use std::path::{Path, PathBuf};

use crate::asset::{AssetImportError, AssetUri};

use super::{ProjectManager, ProjectPaths};

impl ProjectManager {
    /// Resolves an existing project source to its stable logical `res://` identity.
    ///
    /// Both the source and configured asset root are resolved through the filesystem before
    /// containment is checked, so casing and filesystem aliases cannot change the URI. New
    /// destinations intentionally use `primary_project_source_path_for_uri` instead.
    pub fn project_uri_for_source_path(&self, path: &Path) -> Result<AssetUri, AssetImportError> {
        let (_, resolved_root, resolved_path) = self.resolve_project_source_path(path)?;
        Self::source_uri_from_resolved_project_path(&resolved_root, &resolved_path)
    }

    pub(super) fn resolve_project_source_path(
        &self,
        source_path: &Path,
    ) -> Result<(&Path, PathBuf, PathBuf), AssetImportError> {
        let requested_path = source_path.to_path_buf();
        let resolved_path = ProjectPaths::resolve_existing_path(source_path)?;
        let mut roots = self
            .project_asset_roots()
            .iter()
            .map(|root| {
                ProjectPaths::resolve_existing_path(root)
                    .map(|resolved_root| (root.as_path(), resolved_root))
                    .map_err(|source| AssetImportError::CanonicalProjectAssetRoot {
                        path: root.clone(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|roots| {
                roots
                    .into_iter()
                    .filter(|(_, resolved_root)| resolved_path.starts_with(resolved_root))
                    .collect::<Vec<_>>()
            })?;

        match roots.len() {
            1 => {
                let (root, resolved_root) = roots.pop().unwrap();
                Ok((root, resolved_root, resolved_path))
            }
            0 => Err(AssetImportError::SourceOutsideProjectAssetRoots {
                path: requested_path,
            }),
            _ => Err(AssetImportError::ambiguous_project_source_path(
                requested_path,
                roots
                    .into_iter()
                    .map(|(root, _)| root.to_path_buf())
                    .collect(),
            )),
        }
    }

    fn source_uri_from_resolved_project_path(
        asset_root: &Path,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        let relative = path.strip_prefix(asset_root).map_err(|_| {
            AssetImportError::SourceOutsideProjectAssetRoots {
                path: path.to_path_buf(),
            }
        })?;
        let relative = relative_uri_path(relative);
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
        let relative = relative_uri_path(relative);
        Ok(AssetUri::parse(&format!(
            "package://{package_id}/{relative}"
        ))?)
    }
}

fn relative_uri_path(path: &Path) -> String {
    let mut relative = String::with_capacity(path.as_os_str().len());
    for (index, component) in path.components().enumerate() {
        if index != 0 {
            relative.push('/');
        }
        relative.push_str(&component.as_os_str().to_string_lossy());
    }
    relative
}

#[cfg(test)]
#[path = "source_uri_for_path/tests/direct_join_tests.rs"]
mod direct_join_tests;

#[cfg(test)]
#[path = "tests/source_uri_for_path.rs"]
mod tests;
