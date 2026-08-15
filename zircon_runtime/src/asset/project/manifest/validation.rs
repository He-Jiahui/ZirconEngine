use std::collections::BTreeSet;

use zircon_runtime_interface::project::{validate_engine_version_req, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use super::{ProjectManifest, ProjectManifestError};
use crate::asset::project::ProjectPaths;
use std::path::PathBuf;

pub(super) fn default_asset_roots() -> Vec<RelPath> {
    vec![RelPath::project_assets()]
}

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), ProjectManifestError> {
        validate_engine_version_req(self.engine_version_req.as_deref())?;
        if self.asset_roots.is_empty() {
            return Err(ProjectManifestError::EmptyAssetRoots);
        }
        let mut roots = BTreeSet::new();
        for root in &self.asset_roots {
            if !roots.insert(root.as_str()) {
                return Err(ProjectManifestError::DuplicateAssetRoot {
                    root: root.to_string(),
                });
            }
        }
        for (index, left) in self.asset_roots.iter().enumerate() {
            for right in self.asset_roots.iter().skip(index + 1) {
                if is_descendant(left, right) {
                    return Err(ProjectManifestError::OverlappingAssetRoots {
                        ancestor: left.to_string(),
                        descendant: right.to_string(),
                    });
                }
                if is_descendant(right, left) {
                    return Err(ProjectManifestError::OverlappingAssetRoots {
                        ancestor: right.to_string(),
                        descendant: left.to_string(),
                    });
                }
            }
        }
        let mut ui_roots = BTreeSet::new();
        for root in &self.ui_roots {
            if root.scheme() != ResourceScheme::Res {
                return Err(ProjectManifestError::InvalidUiRootScheme {
                    root: root.to_string(),
                });
            }
            if root.path().trim().is_empty() {
                return Err(ProjectManifestError::EmptyUiRoot);
            }
            if root.label().is_some() {
                return Err(ProjectManifestError::LabelledUiRoot {
                    root: root.to_string(),
                });
            }
            if !ui_roots.insert(root.to_string()) {
                return Err(ProjectManifestError::DuplicateUiRoot {
                    root: root.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn primary_asset_root(&self) -> Result<&RelPath, ProjectManifestError> {
        self.asset_roots
            .first()
            .ok_or(ProjectManifestError::EmptyAssetRoots)
    }

    pub fn primary_asset_root_path(
        &self,
        paths: &ProjectPaths,
    ) -> Result<PathBuf, ProjectManifestError> {
        self.primary_asset_root().map(|root| paths.asset_root(root))
    }

    pub fn asset_root_paths(&self, paths: &ProjectPaths) -> Vec<PathBuf> {
        self.asset_roots
            .iter()
            .map(|root| paths.asset_root(root))
            .collect()
    }
}

fn is_descendant(ancestor: &RelPath, candidate: &RelPath) -> bool {
    candidate
        .as_str()
        .strip_prefix(ancestor.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}
