//! URI 到物理路径的解析是项目 I/O 的共同入口；现有源选择唯一匹配，尚未创建的目标明确落在首个项目根。

use std::path::{Component, Path, PathBuf};

use crate::core::resource::ResourceScheme;

use crate::asset::project::{ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::asset::{AssetImportError, AssetUri};

use super::ProjectManager;

impl ProjectManager {
    // BUG: [CR-ASSET-TYPESPROJECT-0001] 已存在的 res:// 源仅在各根下检查路径存在，随后物理解析却未复核是否仍位于该根；链接可使场景保存写到项目外。证据：source_operation_path_for_project_uri 与 scene/world/project_io/scene_asset.rs::save_scene_to_project。
    /// Resolves a logical asset URI through the project/package root registry once.
    ///
    /// The returned operation path is the sole filesystem input. Its display path is retained
    /// for diagnostics and external platform APIs, so consumers do not strip Windows verbatim
    /// prefixes or re-canonicalize aliases independently.
    pub fn resolve_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<ResolvedProjectPath, AssetImportError> {
        Ok(ProjectPaths::resolve_path(
            self.source_operation_path_for_uri(uri)?,
        )?)
    }

    /// Re-checks a source destination's physical identity immediately before publication.
    ///
    /// URI selection is lexical, while a directory junction/symlink can redirect an existing
    /// tail outside every registered project asset root.  Keep the physical admission in this
    /// lower resolver so World, editor, and importer writes share one boundary.
    pub(crate) fn validate_project_source_path_for_write(
        &self,
        path: &ResolvedProjectPath,
    ) -> Result<(), AssetImportError> {
        let physical = ProjectPaths::resolve_path(path.operation_path())?;
        let physical_identity = ResolvedProjectPathIdentity::from(physical.clone());
        let contained = self.package_assets.project_roots().iter().any(|root| {
            ProjectPaths::resolve_existing(root)
                .map(ResolvedProjectPathIdentity::from)
                .map(|root| physical_identity.is_within(&root))
                .unwrap_or(false)
        });
        if !contained {
            return Err(AssetImportError::UnsupportedFormat(format!(
                "project source destination resolves outside registered asset roots: {}",
                physical.display_path().display()
            )));
        }
        Ok(())
    }

    pub fn source_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        self.resolve_source_path_for_uri(uri)
            .map(ResolvedProjectPath::into_operation_path)
    }

    fn source_operation_path_for_uri(&self, uri: &AssetUri) -> Result<PathBuf, AssetImportError> {
        match uri.scheme() {
            ResourceScheme::Res => self.source_operation_path_for_project_uri(uri),
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
    pub fn resolve_primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<ResolvedProjectPath, AssetImportError> {
        Ok(ProjectPaths::resolve_path(
            self.primary_project_source_operation_path_for_uri(uri)?,
        )?)
    }

    /// Resolves a not-yet-existing `res://` destination into the first manifest root.
    pub fn primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        self.resolve_primary_project_source_path_for_uri(uri)
            .map(ResolvedProjectPath::into_operation_path)
    }

    fn primary_project_source_operation_path_for_uri(
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
    pub fn resolve_existing_or_primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<ResolvedProjectPath, AssetImportError> {
        match self.resolve_source_path_for_uri(uri) {
            Ok(path) => {
                self.validate_project_source_path_for_write(&path)?;
                Ok(path)
            }
            Err(AssetImportError::MissingProjectAssetUri { .. }) => {
                let path = self.resolve_primary_project_source_path_for_uri(uri)?;
                self.validate_project_source_path_for_write(&path)?;
                Ok(path)
            }
            Err(error) => Err(error),
        }
    }

    /// Resolves an existing unique source, or explicitly chooses the primary root for a new one.
    pub fn existing_or_primary_project_source_path_for_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        self.resolve_existing_or_primary_project_source_path_for_uri(uri)
            .map(ResolvedProjectPath::into_operation_path)
    }

    fn source_operation_path_for_project_uri(
        &self,
        uri: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        validate_relative_package_path(uri.path())?;
        let mut existing = self
            .package_assets
            .project_roots()
            .iter()
            .map(|root| root.join(uri.path()))
            .filter(|candidate| candidate.exists());
        let Some(first) = existing.next() else {
            return Err(AssetImportError::MissingProjectAssetUri { uri: uri.clone() });
        };
        let Some(second) = existing.next() else {
            return Ok(first);
        };

        let mut ambiguous = Vec::with_capacity(2 + existing.size_hint().0);
        ambiguous.push(first);
        ambiguous.push(second);
        ambiguous.extend(existing);
        Err(AssetImportError::ambiguous_project_asset_uri(
            uri.clone(),
            ambiguous,
        ))
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

#[cfg(all(test, windows))]
#[path = "tests/source_path_for_uri.rs"]
mod tests;
