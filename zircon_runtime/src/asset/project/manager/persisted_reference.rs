use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::safe_project_path::is_safe_regular_file;
use crate::asset::{AssetReference, ReferenceResolutionError};

use super::ProjectManager;

impl ProjectManager {
    pub fn persist_runtime_reference(
        &self,
        reference: &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError> {
        match reference.locator.scheme() {
            ResourceScheme::Builtin => {
                return Ok(PersistedAssetReference::builtin(reference.locator.clone()))
            }
            ResourceScheme::Res => {}
            scheme => {
                return Err(ReferenceResolutionError::UnsupportedScheme {
                    locator: reference.locator.clone(),
                })
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
        let mut candidates = Vec::new();
        for candidate @ (_, root) in self
            .manifest
            .asset_roots
            .iter()
            .zip(self.package_assets.project_roots())
        {
            let path = root.join(reference.locator.path());
            if is_safe_regular_file(root, &path).map_err(|source| {
                ReferenceResolutionError::PathIo {
                    path: path.clone(),
                    source,
                }
            })? {
                candidates.push(candidate);
            }
        }
        let (root_rel, _) = match candidates.as_slice() {
            [(root_rel, root)] => (*root_rel, *root),
            [] => {
                return Err(ReferenceResolutionError::MissingPath {
                    path: reference.locator.to_string(),
                })
            }
            _ => {
                return Err(ReferenceResolutionError::AmbiguousPath {
                    path: reference.locator.to_string(),
                })
            }
        };
        let path_hint = RelPath::parse(format!(
            "{}/{}",
            root_rel.as_str(),
            reference.locator.path()
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    #[test]
    fn writer_rejects_source_replaced_by_link_after_registry_load() {
        let root = std::env::temp_dir().join(format!(
            "zircon_persist_reference_link_{}",
            std::process::id()
        ));
        let outside = root.with_extension("outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "Writer link guard",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let source = root.join("assets/models/hero.glb");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"inside").unwrap();
        let guid: AssetUuid = "d1111111-2222-4333-8444-555555555555".parse().unwrap();
        let mut meta = AssetMetaDocument::new(
            guid,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
        );
        meta.source_digest = "digest".to_owned();
        meta.save(source.with_file_name("hero.glb.zmeta")).unwrap();
        let manager = ProjectManager::open(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let external = outside.join("hero.glb");
        fs::write(&external, b"outside").unwrap();
        fs::remove_file(&source).unwrap();
        if let Err(error) = create_file_link(&external, &source) {
            if error.kind() == std::io::ErrorKind::PermissionDenied {
                let _ = fs::remove_dir_all(&root);
                let _ = fs::remove_dir_all(&outside);
                return;
            }
            panic!("failed to create test link: {error}");
        }

        let error = manager
            .persist_runtime_reference(&AssetReference::new(
                guid,
                AssetUri::parse("res://models/hero.glb").unwrap(),
            ))
            .unwrap_err();
        assert!(matches!(
            error,
            ReferenceResolutionError::MissingPath { .. }
        ));
        assert_eq!(fs::read(&external).unwrap(), b"outside");
        let _ = fs::remove_file(&source);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }

    #[cfg(unix)]
    fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }
}
