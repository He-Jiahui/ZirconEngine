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
        let documents = documents.into_iter().collect::<Vec<_>>();
        let mut index = build_index_from_documents(documents.iter().copied(), Vec::new())?;
        refresh_dependency_edges_from_documents(&mut index, documents.iter().copied());
        Ok(index)
    }

    pub(crate) fn rebuild_after_import_from_loaded<'a>(
        &self,
        documents: impl IntoIterator<Item = &'a AssetMetaDocument>,
        duplicate_diagnostics: Vec<AssetRegistryDiagnostic>,
    ) -> Result<Self, AssetRegistryError> {
        let documents = documents.into_iter().collect::<Vec<_>>();
        let mut diagnostics = self
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic,
                    AssetRegistryDiagnostic::CorruptPersistenceRebuilt { .. }
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        diagnostics.extend(duplicate_diagnostics);
        let mut index = build_index_from_documents(documents.iter().copied(), diagnostics)?;
        refresh_dependency_edges_from_documents(&mut index, documents.iter().copied());
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::asset::project::AssetMetaDocument;
    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    use super::super::rebuild::{
        build_index_from_documents, refresh_dependency_edges_from_documents,
    };

    #[test]
    fn borrowed_metadata_inventory_builds_the_same_registry_index() {
        let uuid = AssetUuid::new();
        let uri = AssetUri::parse("res://shaders/inventory.wgsl")
            .expect("fixture asset URI should parse");
        let document = AssetMetaDocument::new(uuid, uri, AssetKind::Shader);
        let documents = BTreeMap::from([(PathBuf::from("inventory.wgsl.zmeta"), document)]);

        let mut index = build_index_from_documents(documents.values(), Vec::new())
            .expect("borrowed inventory should build an index");
        refresh_dependency_edges_from_documents(&mut index, documents.values());

        assert_eq!(index.len(), 1);
        assert_eq!(
            index
                .entry_by_uuid(uuid)
                .expect("borrowed inventory entry should be indexed")
                .path()
                .to_string(),
            "res://shaders/inventory.wgsl"
        );
    }
}
