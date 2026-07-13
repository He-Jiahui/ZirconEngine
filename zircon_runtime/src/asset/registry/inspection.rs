use std::path::PathBuf;

use super::rebuild::{build_index, refresh_dependency_edges, scan_project_metas};
use super::{AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// Builds a strict read-only snapshot without reminting sidecars or persisting registry state.
    pub fn inspect_project(asset_roots: &[PathBuf]) -> Result<Self, AssetRegistryError> {
        let metas = scan_project_metas(asset_roots)?;
        let mut index = build_index(&metas, Vec::new())?;
        refresh_dependency_edges(&mut index, &metas);
        Ok(index)
    }
}
