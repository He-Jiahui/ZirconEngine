use std::path::{Path, PathBuf};

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;
use crate::core::resource::io::AtomicWriteFault;

use super::rebuild::{
    identity_owners_for_changes, normalize_duplicate_guids, refresh_dependency_edges,
    replace_entries_for_paths, scan_project_metas,
};
use super::{AssetRegistryDiagnostic, AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// Applies watcher deltas to entries, then refreshes metadata-only edges for consistency.
    pub fn apply_watch_changes(
        &mut self,
        asset_roots: &[PathBuf],
        registry_root: impl AsRef<Path>,
        changes: &[AssetChange],
    ) -> Result<(), AssetRegistryError> {
        self.apply_watch_changes_with_atomic_fault(
            asset_roots,
            registry_root,
            changes,
            AtomicWriteFault::None,
        )
    }

    pub(crate) fn apply_watch_changes_with_atomic_fault(
        &mut self,
        asset_roots: &[PathBuf],
        registry_root: impl AsRef<Path>,
        changes: &[AssetChange],
        fault: AtomicWriteFault,
    ) -> Result<(), AssetRegistryError> {
        if changes.is_empty() {
            return Ok(());
        }
        let owners = identity_owners_for_changes(self, changes);
        let mut candidate = self.clone();
        for change in changes {
            if let Some(previous) = &change.previous_uri {
                candidate.remove_source_path(previous);
            }
            candidate.remove_source_path(&change.uri);
        }
        let mut metas = scan_project_metas(asset_roots)?;
        let mut duplicate_diagnostics = Vec::new();
        let reminted_paths =
            normalize_duplicate_guids(&mut metas, &mut duplicate_diagnostics, &owners)?;
        let mut changed_paths = Vec::new();
        for change in changes {
            if change.kind != AssetChangeKind::Removed {
                changed_paths.push(change.uri.clone());
            }
        }
        for path in reminted_paths {
            if !changed_paths
                .iter()
                .any(|changed| same_source_path(changed, &path))
            {
                changed_paths.push(path);
            }
        }
        replace_entries_for_paths(&mut candidate, &metas, &changed_paths)?;
        candidate.diagnostics.retain(|diagnostic| {
            !matches!(
                diagnostic,
                AssetRegistryDiagnostic::UnresolvedDependency { .. }
            )
        });
        candidate.diagnostics.extend(duplicate_diagnostics);
        refresh_dependency_edges(&mut candidate, &metas);
        candidate.persist_with_atomic_fault(registry_root.as_ref(), fault)?;
        *self = candidate;
        Ok(())
    }
}

fn same_source_path(left: &AssetUri, right: &AssetUri) -> bool {
    left.scheme() == right.scheme() && left.path() == right.path()
}
