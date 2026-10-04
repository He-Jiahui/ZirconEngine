//! 运行时引用写入项目文档前同时核对 UUID、逻辑路径与唯一物理源；scene/world 保存由此得到可迁移 path hint，而非信任旧 locator。

use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::reference_resolver::persisted_source_path_for_locator;
use crate::asset::{AssetReference, ReferenceResolutionError};

use super::ProjectManager;

impl ProjectManager {
    /// 写回 scene/model/material 文档前将实时引用转换为带 UUID 与项目相对提示的持久引用；歧义根或身份不匹配必须报错。
    pub fn persist_runtime_reference(
        &self,
        reference: &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError> {
        match reference.locator.scheme() {
            ResourceScheme::Builtin => {
                return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
            }
            ResourceScheme::Res => {}
            scheme => {
                return Err(ReferenceResolutionError::UnsupportedScheme {
                    locator: reference.locator.clone(),
                });
            }
        }
        let by_guid = self
            .asset_registry
            .entry_by_uuid(reference.uuid)
            .ok_or_else(|| ReferenceResolutionError::MissingGuid {
                guid: reference.uuid,
            })?;
        let by_path = self
            .asset_registry
            .entry_by_path(&reference.locator)
            .ok_or_else(|| ReferenceResolutionError::MissingPath {
                path: reference.locator.to_string(),
            })?;
        if by_guid.uuid() != by_path.uuid() {
            return Err(ReferenceResolutionError::Registry {
                message: format!(
                    "asset guid {} and path {} resolve to different registry entries",
                    reference.uuid, reference.locator
                ),
            });
        }
        let mut unique_candidate = None;
        let mut ambiguous = false;
        for candidate @ (_, root) in self
            .manifest
            .asset_roots
            .iter()
            .zip(self.package_assets.project_roots())
        {
            let path =
                persisted_source_path_for_locator(root, &reference.locator).map_err(|source| {
                    ReferenceResolutionError::PathIo {
                        path: root.join(reference.locator.path()),
                        source,
                    }
                })?;
            if let Some(path) = path {
                ambiguous |= store_unique_candidate(&mut unique_candidate, (candidate, path));
            }
        }
        if ambiguous {
            return Err(ReferenceResolutionError::AmbiguousPath {
                path: reference.locator.to_string(),
            });
        }
        let Some(((root_rel, root), source_path)) = unique_candidate else {
            return Err(ReferenceResolutionError::MissingPath {
                path: reference.locator.to_string(),
            });
        };
        let relative =
            source_path
                .strip_prefix(root)
                .map_err(|error| ReferenceResolutionError::Registry {
                    message: format!(
                        "persisted source {} escaped root {}: {error}",
                        source_path.display(),
                        root.display()
                    ),
                })?;
        let path_hint = RelPath::parse(format!(
            "{}/{}",
            root_rel.as_str(),
            relative.to_string_lossy()
        ))
        .map_err(|source| ReferenceResolutionError::Path {
            path: reference.locator.to_string(),
            source,
        })?;
        let asset_ref = AssetRef::try_new(
            reference.uuid,
            path_hint,
            reference.locator.label().map(str::to_string),
        )
        .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
        Ok(PersistedAssetReference::project(asset_ref))
    }
}

fn store_unique_candidate<T>(unique: &mut Option<T>, candidate: T) -> bool {
    if unique.is_some() {
        true
    } else {
        *unique = Some(candidate);
        false
    }
}

#[cfg(test)]
#[path = "tests/persisted_reference.rs"]
mod tests;

#[cfg(test)]
#[path = "persisted_reference/tests/unique_candidate_tests.rs"]
mod unique_candidate_tests;
