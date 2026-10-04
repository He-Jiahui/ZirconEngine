use std::collections::{HashMap, HashSet};

use zircon_runtime_interface::project::{
    validate_engine_version_req, ProjectManifestSummaryError, RelPath, MAX_PROJECT_ASSET_ROOTS,
};
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
        if let Some(receipt) = &self.template_receipt {
            if receipt.project_guid() != self.project_guid {
                return Err(ProjectManifestError::TemplateReceiptProjectGuidMismatch {
                    manifest_guid: self.project_guid,
                    receipt_guid: receipt.project_guid(),
                });
            }
            if receipt.descriptor().engine_version_req() != self.engine_version_req.as_deref() {
                return Err(ProjectManifestError::TemplateReceiptEngineRequirementMismatch);
            }
        }
        if self.asset_roots.is_empty() {
            return Err(ProjectManifestError::EmptyAssetRoots);
        }
        if self.asset_roots.len() > MAX_PROJECT_ASSET_ROOTS {
            return Err(ProjectManifestError::Summary(
                ProjectManifestSummaryError::TooManyAssetRoots {
                    max: MAX_PROJECT_ASSET_ROOTS,
                    found: self.asset_roots.len(),
                },
            ));
        }
        if self.asset_roots.len() > 1 {
            let mut roots = HashMap::with_capacity(self.asset_roots.len());
            for (index, root) in self.asset_roots.iter().enumerate() {
                if roots.insert(root.as_str(), index).is_some() {
                    return Err(ProjectManifestError::DuplicateAssetRoot {
                        root: root.to_string(),
                    });
                }
            }
            if let Some((ancestor, descendant)) =
                first_overlapping_asset_roots(&self.asset_roots, &roots)
            {
                return Err(ProjectManifestError::OverlappingAssetRoots {
                    ancestor: self.asset_roots[ancestor].to_string(),
                    descendant: self.asset_roots[descendant].to_string(),
                });
            }
        }
        let mut ui_roots = HashSet::with_capacity(self.ui_roots.len());
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
            if !ui_roots.insert(root.path()) {
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

fn first_overlapping_asset_roots(
    roots: &[RelPath],
    indices: &HashMap<&str, usize>,
) -> Option<(usize, usize)> {
    let mut first: Option<(usize, usize)> = None;
    for (descendant, root) in roots.iter().enumerate() {
        for (boundary, _) in root.as_str().match_indices('/') {
            let Some(&ancestor) = indices.get(&root.as_str()[..boundary]) else {
                continue;
            };
            // Keep the original nested-loop error precedence, independent of hash order.
            let pair = (ancestor.min(descendant), ancestor.max(descendant));
            if first.is_none_or(|(a, d)| pair < (a.min(d), a.max(d))) {
                first = Some((ancestor, descendant));
            }
        }
    }
    first
}

#[cfg(test)]
#[path = "validation/tests/astra_root_tests.rs"]
mod astra_root_tests;

#[cfg(test)]
#[path = "tests/validation_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "validation/tests/optimization_batch_ir_runtime629_tests.rs"]
mod optimization_batch_ir_runtime629_tests;
