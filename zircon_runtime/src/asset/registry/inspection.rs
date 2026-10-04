use std::path::PathBuf;

use super::rebuild::{
    build_index, build_index_from_documents, refresh_dependency_edges,
    refresh_dependency_edges_from_documents, scan_meta_paths, scan_project_metas,
};
use super::{AssetRegistryDiagnostic, AssetRegistryError, AssetRegistryIndex};
use crate::asset::project::AssetMetaDocument;

impl AssetRegistryIndex {
    /// Builds a strict read-only snapshot without reminting sidecars or persisting registry state.
    pub fn inspect_project(asset_roots: &[PathBuf]) -> Result<Self, AssetRegistryError> {
        let metas = scan_project_metas(asset_roots)?;
        let mut index = build_index(&metas, Vec::new())?;
        refresh_dependency_edges(&mut index, &metas);
        Ok(index)
    }

    /// Builds a read-only snapshot from a caller-owned, deterministically ordered metadata inventory.
    pub fn inspect_meta_paths(meta_paths: &[PathBuf]) -> Result<Self, AssetRegistryError> {
        let metas = scan_meta_paths(meta_paths)?;
        let mut index = build_index(&metas, Vec::new())?;
        refresh_dependency_edges(&mut index, &metas);
        Ok(index)
    }

    /// Builds a read-only snapshot from caller-owned, already parsed metadata.
    ///
    /// Asset scans that own a bounded metadata inventory use this path to avoid
    /// reopening every `.zmeta` file for a second registry pass.
    pub fn inspect_loaded_meta_documents(
        documents_by_path: &std::collections::BTreeMap<PathBuf, AssetMetaDocument>,
    ) -> Result<Self, AssetRegistryError> {
        Self::inspect_loaded_meta_document_refs(documents_by_path.values())
    }

    pub(crate) fn inspect_loaded_meta_document_refs<'a>(
        documents: impl IntoIterator<Item = &'a AssetMetaDocument>,
    ) -> Result<Self, AssetRegistryError> {
        let mut document_iter = documents.into_iter();
        let (lower_bound, upper_bound) = document_iter.size_hint();
        let mut documents = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));
        documents.extend(document_iter);
        let mut index = build_index_from_documents(documents.iter().copied(), Vec::new())?;
        refresh_dependency_edges_from_documents(&mut index, documents.iter().copied());
        Ok(index)
    }

    pub(crate) fn rebuild_after_import_from_loaded<'a>(
        &self,
        documents: impl IntoIterator<Item = &'a AssetMetaDocument>,
        duplicate_diagnostics: Vec<AssetRegistryDiagnostic>,
    ) -> Result<Self, AssetRegistryError> {
        let mut document_iter = documents.into_iter();
        let (lower_bound, upper_bound) = document_iter.size_hint();
        let mut documents = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));
        documents.extend(document_iter);
        let existing_diagnostics = self.diagnostics();
        let mut diagnostics = Vec::with_capacity(
            existing_diagnostics
                .len()
                .saturating_add(duplicate_diagnostics.len()),
        );
        for diagnostic in existing_diagnostics {
            if matches!(
                diagnostic,
                AssetRegistryDiagnostic::CorruptPersistenceRebuilt { .. }
            ) {
                diagnostics.push(diagnostic.clone());
            }
        }
        diagnostics.extend(duplicate_diagnostics);
        let mut index = build_index_from_documents(documents.iter().copied(), diagnostics)?;
        refresh_dependency_edges_from_documents(&mut index, documents.iter().copied());
        Ok(index)
    }
}

#[cfg(test)]
#[path = "tests/inspection.rs"]
mod tests;
