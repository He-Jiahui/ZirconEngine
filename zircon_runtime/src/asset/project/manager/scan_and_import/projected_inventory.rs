use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::asset::project::{AssetMetaDocument, ProjectGenerationObservation};
use crate::asset::registry::{AssetRegistryDiagnostic, AssetRegistryIndex};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetImportError, AssetKind};
use crate::core::resource::ResourceScheme;

use super::sources::AssetImportSource;
use super::ProjectManager;

pub(super) struct ProjectedMetaInventory {
    originals: BTreeMap<PathBuf, Option<AssetMetaDocument>>,
    documents: BTreeMap<PathBuf, AssetMetaDocument>,
    identity_changes: Vec<AssetChange>,
}

impl ProjectedMetaInventory {
    pub(super) fn load(
        manager: &ProjectManager,
        sources: &[AssetImportSource],
        observation: &mut ProjectGenerationObservation,
    ) -> Result<Self, AssetImportError> {
        let mut originals = BTreeMap::new();
        let mut documents = BTreeMap::new();
        let mut identity_changes = Vec::new();

        for source in sources {
            let fallback_kind = manager
                .importer
                .descriptor_for_source(&source.path)
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let original = if source.meta_path.exists() {
                Some(observation.load_metadata_document(&source.meta_path)?)
            } else {
                None
            };
            let mut projected = original.clone().unwrap_or_else(|| {
                AssetMetaDocument::new(
                    crate::asset::AssetUuid::new(),
                    source.uri.clone(),
                    fallback_kind,
                )
            });
            if let Some(previous) = original
                .as_ref()
                .filter(|previous| previous.url != source.uri)
            {
                identity_changes.push(AssetChange::new(
                    AssetChangeKind::Renamed,
                    source.uri.clone(),
                    Some(previous.url.clone()),
                ));
            }
            projected.url = source.uri.clone();
            projected.asset_kind = fallback_kind;
            originals.insert(source.meta_path.clone(), original);
            documents.insert(source.meta_path.clone(), projected);
        }

        observation.record_metadata_inventory(
            documents.len(),
            originals
                .values()
                .filter(|document| document.is_some())
                .count(),
        );
        Ok(Self {
            originals,
            documents,
            identity_changes,
        })
    }

    pub(super) fn normalize_duplicate_guids(
        &mut self,
        registry: &AssetRegistryIndex,
        watch_changes: Option<&[AssetChange]>,
    ) -> Vec<AssetRegistryDiagnostic> {
        let mut identity_changes = self.identity_changes.clone();
        identity_changes.extend_from_slice(watch_changes.unwrap_or_default());
        let changes = (!identity_changes.is_empty()).then_some(identity_changes.as_slice());
        registry.prepare_duplicate_guids_from_loaded(&mut self.documents, changes)
    }

    pub(super) fn document(&self, meta_path: &Path) -> &AssetMetaDocument {
        self.documents
            .get(meta_path)
            .expect("every collected source owns one projected metadata document")
    }

    pub(super) fn document_mut(&mut self, meta_path: &Path) -> &mut AssetMetaDocument {
        self.documents
            .get_mut(meta_path)
            .expect("every collected source owns one projected metadata document")
    }

    pub(super) fn preconditions(
        &self,
    ) -> impl Iterator<Item = (&PathBuf, Option<&AssetMetaDocument>)> {
        self.originals
            .iter()
            .map(|(path, original)| (path, original.as_ref()))
    }

    pub(super) fn project_documents(&self) -> impl Iterator<Item = &AssetMetaDocument> {
        self.documents
            .values()
            .filter(|document| document.url.scheme() == ResourceScheme::Res)
    }

    pub(super) fn documents(&self) -> impl Iterator<Item = &AssetMetaDocument> {
        self.documents.values()
    }

    pub(super) fn changed_documents(&self) -> impl Iterator<Item = (&PathBuf, &AssetMetaDocument)> {
        self.documents.iter().filter(|(path, document)| {
            self.originals.get(*path).and_then(Option::as_ref) != Some(*document)
        })
    }
}
