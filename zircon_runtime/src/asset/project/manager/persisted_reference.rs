//! 运行时引用写入项目文档前同时核对 UUID、逻辑路径与唯一物理源；scene/world 保存由此得到可迁移 path hint，而非信任旧 locator。

use std::{fs, io};

use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::project::{AssetMetaDocument, AssetSourceUnit, ProjectPaths};
use crate::asset::reference_resolver::persisted_source_path_for_locator;
use crate::asset::safe_project_path::is_safe_regular_file;
use crate::asset::{AssetId, AssetKind, AssetReference, AssetUri, ReferenceResolutionError};

use super::meta_path_for_source::meta_path_for_source;
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

    /// Find the current source for the stable GUID of a typed Scene prefab reference.
    /// If the previous address is still occupied or the current source cannot be verified,
    /// keep the original reference so the normal writer reports its original error.
    pub(super) fn relocated_scene_prefab_locator(
        &self,
        reference: &AssetReference,
    ) -> Option<&AssetUri> {
        let previous = &reference.locator;
        if previous.scheme() != ResourceScheme::Res
            || previous.label().is_some()
            || self.asset_registry.entry_by_path(previous).is_some()
        {
            return None;
        }
        let entry = self.asset_registry.entry_by_uuid(reference.uuid)?;
        let current = entry.path();
        if entry.type_marker() != AssetKind::Prefab
            || current.scheme() != ResourceScheme::Res
            || current.label().is_some()
            || !self.asset_registry.source_entries(previous).is_empty()
        {
            return None;
        }

        let record = self
            .catalog_input_generation
            .record(AssetId::from_asset_uuid(reference.uuid))?;
        let indexed_meta = record.meta();
        if indexed_meta.uuid != reference.uuid
            || &indexed_meta.url != current
            || indexed_meta.asset_kind != AssetKind::Prefab
            || indexed_meta.unit != AssetSourceUnit::Single
            || !indexed_meta.included_files.is_empty()
            || indexed_meta.source_digest != entry.source_digest()
        {
            return None;
        }
        let source_path = self.source_path_for_no_follow(current).ok()?;
        let root = self.project_asset_root_for_source_path(&source_path).ok()?;
        let meta_path = meta_path_for_source(&source_path);
        if !ProjectPaths::same_lexical_path(&source_path, record.source_path()).ok()?
            || !ProjectPaths::same_lexical_path(&meta_path, record.meta_path()).ok()?
            || !is_safe_regular_file(root, &source_path).ok()?
            || !is_safe_regular_file(root, &meta_path).ok()?
        {
            return None;
        }

        // Check the old source and sidecar even when they have not reached the registry yet.
        // Only NotFound is evidence of vacancy. Links, directories and probe errors block repair.
        let previous_is_vacant = self.project_asset_roots().iter().all(|root| {
            let path = root.join(previous.path());
            [meta_path_for_source(&path), path].iter().all(|path| {
                fs::symlink_metadata(path)
                    .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
            })
        });
        if !previous_is_vacant {
            return None;
        }

        let live_meta = AssetMetaDocument::load(meta_path).ok()?;
        if live_meta.uuid != indexed_meta.uuid
            || live_meta.url != indexed_meta.url
            || live_meta.asset_kind != indexed_meta.asset_kind
            || live_meta.unit != indexed_meta.unit
            || live_meta.included_files != indexed_meta.included_files
            || live_meta.source_digest != indexed_meta.source_digest
        {
            return None;
        }
        Some(current)
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
#[path = "tests/unique_candidate.rs"]
mod unique_candidate_tests;
