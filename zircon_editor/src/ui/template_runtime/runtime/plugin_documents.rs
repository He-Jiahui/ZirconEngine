use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use thiserror::Error;

use crate::core::editor_extension::EditorUiTemplateDescriptor;

use super::runtime_host::{
    v2_template_file_cache, EditorUiHostRuntime, EditorUiHostRuntimeError, EditorUiHostV2Document,
};

const PLUGIN_DOCUMENT_URI_SCHEME: &str = "plugins://";

pub(super) fn plugin_v2_document_sources(
    owner_id: &str,
    descriptors: &[EditorUiTemplateDescriptor],
) -> Result<Vec<EditorPluginV2DocumentSource>, EditorUiHostRuntimeError> {
    descriptors
        .iter()
        .filter(|descriptor| {
            descriptor
                .ui_document()
                .starts_with(PLUGIN_DOCUMENT_URI_SCHEME)
        })
        .map(|descriptor| plugin_v2_document_source(owner_id, descriptor))
        .collect()
}

fn plugin_v2_document_source(
    owner_id: &str,
    descriptor: &EditorUiTemplateDescriptor,
) -> Result<EditorPluginV2DocumentSource, EditorUiHostRuntimeError> {
    let source_uri = descriptor.ui_document();
    let prefix = format!("{PLUGIN_DOCUMENT_URI_SCHEME}{owner_id}/");
    let relative_path = source_uri.strip_prefix(&prefix).ok_or_else(|| {
        EditorUiHostRuntimeError::PluginDocumentUri {
            owner_id: owner_id.to_string(),
            source_uri: source_uri.to_string(),
        }
    })?;
    if relative_path.is_empty()
        || relative_path.contains('\\')
        || relative_path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(EditorUiHostRuntimeError::PluginDocumentTemplatePath {
            source_uri: source_uri.to_string(),
        });
    }
    let plugin_root = descriptor.plugin_root().ok_or_else(|| {
        EditorUiHostRuntimeError::PluginDocumentTemplateRoot {
            owner_id: owner_id.to_string(),
            template_id: descriptor.id().to_string(),
        }
    })?;
    EditorPluginV2DocumentSource::new(
        descriptor.id(),
        source_uri,
        std::iter::once(plugin_root.join(PathBuf::from(relative_path))),
    )
    .map_err(EditorUiHostRuntimeError::from)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct EditorPluginV2DocumentOwner {
    owner_id: String,
    generation: u64,
}

impl EditorPluginV2DocumentOwner {
    pub(super) fn new(
        owner_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, EditorPluginV2DocumentSourceError> {
        let owner_id = owner_id.into();
        if owner_id.trim().is_empty() || owner_id.trim() != owner_id {
            return Err(EditorPluginV2DocumentSourceError::InvalidOwnerId { owner_id });
        }
        if generation == 0 {
            return Err(EditorPluginV2DocumentSourceError::InvalidGeneration);
        }
        Ok(Self {
            owner_id,
            generation,
        })
    }

    pub(super) fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn uri_prefix(&self) -> String {
        format!("{PLUGIN_DOCUMENT_URI_SCHEME}{}/", self.owner_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditorPluginV2DocumentSource {
    document_id: String,
    source_uri: String,
    source_files: Vec<PathBuf>,
}

impl EditorPluginV2DocumentSource {
    pub(super) fn new<P, I>(
        document_id: impl Into<String>,
        source_uri: impl Into<String>,
        source_files: I,
    ) -> Result<Self, EditorPluginV2DocumentSourceError>
    where
        P: Into<PathBuf>,
        I: IntoIterator<Item = P>,
    {
        let document_id = document_id.into();
        if document_id.trim().is_empty() || document_id.trim() != document_id {
            return Err(EditorPluginV2DocumentSourceError::InvalidDocumentId { document_id });
        }

        let source_uri = source_uri.into();
        if !source_uri.starts_with(PLUGIN_DOCUMENT_URI_SCHEME)
            || source_uri.len() == PLUGIN_DOCUMENT_URI_SCHEME.len()
        {
            return Err(EditorPluginV2DocumentSourceError::InvalidPluginDocumentUri { source_uri });
        }

        let source_files = source_files.into_iter().map(Into::into).collect::<Vec<_>>();
        if source_files.is_empty() {
            return Err(EditorPluginV2DocumentSourceError::MissingSourceFiles { document_id });
        }

        Ok(Self {
            document_id,
            source_uri,
            source_files,
        })
    }

    pub(super) fn document_id(&self) -> &str {
        &self.document_id
    }

    pub(super) fn source_uri(&self) -> &str {
        &self.source_uri
    }

    pub(super) fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditorPluginV2DocumentUpdate {
    owner: EditorPluginV2DocumentOwner,
    document_ids: Vec<String>,
    retired_document_ids: Vec<String>,
}

#[cfg(test)]
impl EditorPluginV2DocumentUpdate {
    pub(super) fn new(
        owner: EditorPluginV2DocumentOwner,
        document_ids: Vec<String>,
        retired_document_ids: Vec<String>,
    ) -> Self {
        Self {
            owner,
            document_ids,
            retired_document_ids,
        }
    }

    pub(super) fn owner(&self) -> &EditorPluginV2DocumentOwner {
        &self.owner
    }

    pub(super) fn document_ids(&self) -> &[String] {
        &self.document_ids
    }

    pub(super) fn retired_document_ids(&self) -> &[String] {
        &self.retired_document_ids
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EditorPluginV2DocumentSourceError {
    #[error("plugin V2 document owner id must be non-empty and trimmed: {owner_id}")]
    InvalidOwnerId { owner_id: String },
    #[error("plugin V2 document generation must be non-zero")]
    InvalidGeneration,
    #[error("plugin V2 document id must be non-empty and trimmed: {document_id}")]
    InvalidDocumentId { document_id: String },
    #[error("plugin V2 document URI must use plugins://: {source_uri}")]
    InvalidPluginDocumentUri { source_uri: String },
    #[error("plugin V2 document {document_id} has no resolved source files")]
    MissingSourceFiles { document_id: String },
}

#[derive(Clone, Debug)]
pub(super) struct EditorUiHostPluginV2Document {
    owner: EditorPluginV2DocumentOwner,
    document: EditorUiHostV2Document,
}

impl EditorUiHostRuntime {
    pub(crate) fn sync_plugin_v2_template_descriptor_sets(
        &self,
        descriptors_by_owner: &BTreeMap<String, Vec<EditorUiTemplateDescriptor>>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let mut candidates_by_owner = BTreeMap::new();
        for (owner_id, descriptors) in descriptors_by_owner {
            let validation_owner = EditorPluginV2DocumentOwner::new(owner_id.clone(), 1)?;
            let sources = plugin_v2_document_sources(owner_id, descriptors)?;
            let candidates = self.compile_plugin_v2_document_sources(&validation_owner, sources)?;
            candidates_by_owner.insert(owner_id.clone(), candidates);
        }
        self.replace_compiled_plugin_v2_document_batch(candidates_by_owner)
    }

    #[cfg(test)]
    pub(super) fn replace_plugin_v2_documents<I>(
        &self,
        owner: EditorPluginV2DocumentOwner,
        documents: I,
    ) -> Result<EditorPluginV2DocumentUpdate, EditorUiHostRuntimeError>
    where
        I: IntoIterator<Item = EditorPluginV2DocumentSource>,
    {
        let candidate_documents =
            self.compile_plugin_v2_document_sources(&owner, documents.into_iter().collect())?;
        let mut generations = self
            .plugin_v2_generations
            .lock()
            .expect("plugin V2 document generation mutex should not be poisoned");
        self.replace_compiled_plugin_v2_documents(owner, candidate_documents, &mut generations)
    }

    fn compile_plugin_v2_document_sources(
        &self,
        owner: &EditorPluginV2DocumentOwner,
        sources: Vec<EditorPluginV2DocumentSource>,
    ) -> Result<BTreeMap<String, EditorUiHostV2Document>, EditorUiHostRuntimeError> {
        let mut source_ids = BTreeSet::new();
        let expected_uri_prefix = owner.uri_prefix();
        for source in &sources {
            if !source.source_uri().starts_with(&expected_uri_prefix) {
                return Err(EditorUiHostRuntimeError::PluginDocumentUri {
                    owner_id: owner.owner_id().to_string(),
                    source_uri: source.source_uri().to_string(),
                });
            }
            if !source_ids.insert(source.document_id().to_string()) {
                return Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
                    document_id: source.document_id().to_string(),
                    owner_id: owner.owner_id().to_string(),
                });
            }
            if self.v2_documents.contains_key(source.document_id()) {
                return Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
                    document_id: source.document_id().to_string(),
                    owner_id: "builtin-or-local".to_string(),
                });
            }
        }

        // No document or route-visible catalog state is changed until every V2 source compiles.
        let mut candidate_documents = BTreeMap::new();
        let mut cache = v2_template_file_cache()
            .lock()
            .expect("v2 template file cache mutex should not be poisoned");
        for source in sources {
            let outcome = cache.load_store(source.source_files().iter())?;
            candidate_documents.insert(
                source.document_id().to_string(),
                EditorUiHostV2Document {
                    document: outcome.root_document,
                    compiled: outcome.compiled,
                },
            );
        }
        drop(cache);

        Ok(candidate_documents)
    }

    #[cfg(test)]
    fn replace_compiled_plugin_v2_documents(
        &self,
        owner: EditorPluginV2DocumentOwner,
        candidate_documents: BTreeMap<String, EditorUiHostV2Document>,
        generations: &mut BTreeMap<String, u64>,
    ) -> Result<EditorPluginV2DocumentUpdate, EditorUiHostRuntimeError> {
        let tracked_generation = generations
            .get(owner.owner_id())
            .copied()
            .unwrap_or_default();

        let mut plugin_v2_documents = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned");
        let catalog_generation = plugin_v2_documents
            .values()
            .filter(|document| document.owner.owner_id() == owner.owner_id())
            .map(|document| document.owner.generation())
            .max()
            .unwrap_or_default();
        let current_generation = tracked_generation.max(catalog_generation);
        if current_generation >= owner.generation() {
            return Err(EditorUiHostRuntimeError::PluginDocumentGenerationStale {
                owner_id: owner.owner_id().to_string(),
                requested_generation: owner.generation(),
                current_generation,
            });
        }
        for document_id in candidate_documents.keys() {
            if let Some(existing) = plugin_v2_documents.get(document_id) {
                if existing.owner.owner_id() != owner.owner_id() {
                    return Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
                        document_id: document_id.clone(),
                        owner_id: existing.owner.owner_id().to_string(),
                    });
                }
            }
        }

        let document_ids = candidate_documents.keys().cloned().collect();
        let previous_document_ids = plugin_v2_documents
            .iter()
            .filter(|(_, document)| document.owner.owner_id() == owner.owner_id())
            .map(|(document_id, _)| document_id)
            .cloned()
            .collect::<Vec<_>>();
        let retired_document_ids = previous_document_ids
            .iter()
            .filter(|document_id| !candidate_documents.contains_key(*document_id))
            .cloned()
            .collect();
        plugin_v2_documents.retain(|_, document| document.owner.owner_id() != owner.owner_id());
        plugin_v2_documents.extend(candidate_documents.into_iter().map(
            |(document_id, document)| {
                (
                    document_id,
                    EditorUiHostPluginV2Document {
                        owner: owner.clone(),
                        document,
                    },
                )
            },
        ));
        generations.insert(owner.owner_id().to_string(), owner.generation());
        let update = EditorPluginV2DocumentUpdate::new(owner, document_ids, retired_document_ids);
        drop(plugin_v2_documents);
        // A same-id replacement has no retired document id, but its old generation's action
        // slots and control state must not survive until the next pane rebuild.
        self.remove_template_actions_for_documents(&previous_document_ids);
        Ok(update)
    }

    fn replace_compiled_plugin_v2_document_batch(
        &self,
        candidates_by_owner: BTreeMap<String, BTreeMap<String, EditorUiHostV2Document>>,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let mut candidate_owner_by_document = BTreeMap::new();
        for (owner_id, candidate_documents) in &candidates_by_owner {
            for document_id in candidate_documents.keys() {
                if let Some(existing_owner) =
                    candidate_owner_by_document.insert(document_id.clone(), owner_id.clone())
                {
                    return Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
                        document_id: document_id.clone(),
                        owner_id: existing_owner,
                    });
                }
            }
        }

        let mut generations = self
            .plugin_v2_generations
            .lock()
            .expect("plugin V2 document generation mutex should not be poisoned");
        let mut plugin_v2_documents = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned");
        let mut owner_ids = candidates_by_owner.keys().cloned().collect::<BTreeSet<_>>();
        owner_ids.extend(
            plugin_v2_documents
                .values()
                .map(|document| document.owner.owner_id().to_string()),
        );
        if owner_ids.is_empty() {
            return Ok(());
        }

        let mut next_owners = BTreeMap::new();
        for owner_id in &owner_ids {
            let tracked_generation = generations.get(owner_id).copied().unwrap_or_default();
            let catalog_generation = plugin_v2_documents
                .values()
                .filter(|document| document.owner.owner_id() == owner_id)
                .map(|document| document.owner.generation())
                .max()
                .unwrap_or_default();
            let owner = EditorPluginV2DocumentOwner::new(
                owner_id.clone(),
                tracked_generation.max(catalog_generation).saturating_add(1),
            )?;
            next_owners.insert(owner_id.clone(), owner);
        }

        let previous_document_ids = plugin_v2_documents
            .iter()
            .filter(|(_, document)| owner_ids.contains(document.owner.owner_id()))
            .map(|(document_id, _)| document_id.clone())
            .collect::<Vec<_>>();
        plugin_v2_documents.retain(|_, document| !owner_ids.contains(document.owner.owner_id()));
        for (owner_id, candidate_documents) in candidates_by_owner {
            let owner = next_owners
                .get(&owner_id)
                .expect("candidate owner should have a prepared next generation");
            plugin_v2_documents.extend(candidate_documents.into_iter().map(
                |(document_id, document)| {
                    (
                        document_id,
                        EditorUiHostPluginV2Document {
                            owner: owner.clone(),
                            document,
                        },
                    )
                },
            ));
        }
        for (owner_id, owner) in next_owners {
            generations.insert(owner_id, owner.generation());
        }
        drop(plugin_v2_documents);
        drop(generations);

        // Same-id replacements also retire their action state: token ownership includes generation.
        self.remove_template_actions_for_documents(&previous_document_ids);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn replace_plugin_v2_documents_for_owner<I>(
        &self,
        owner_id: impl Into<String>,
        documents: I,
    ) -> Result<EditorPluginV2DocumentUpdate, EditorUiHostRuntimeError>
    where
        I: IntoIterator<Item = EditorPluginV2DocumentSource>,
    {
        let owner_id = owner_id.into();
        let validation_owner = EditorPluginV2DocumentOwner::new(owner_id.clone(), 1)?;
        let candidate_documents = self.compile_plugin_v2_document_sources(
            &validation_owner,
            documents.into_iter().collect(),
        )?;
        let mut generations = self
            .plugin_v2_generations
            .lock()
            .expect("plugin V2 document generation mutex should not be poisoned");
        let generation = generations
            .get(&owner_id)
            .copied()
            .unwrap_or_default()
            .saturating_add(1);
        let owner = EditorPluginV2DocumentOwner::new(owner_id.clone(), generation)?;
        self.replace_compiled_plugin_v2_documents(owner, candidate_documents, &mut generations)
    }

    #[cfg(test)]
    pub(super) fn unregister_plugin_v2_documents(
        &self,
        owner: &EditorPluginV2DocumentOwner,
    ) -> Vec<String> {
        let mut plugin_v2_documents = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned");
        let retired_document_ids = plugin_v2_documents
            .iter()
            .filter(|(_, document)| document.owner == *owner)
            .map(|(document_id, _)| document_id.clone())
            .collect::<Vec<_>>();
        plugin_v2_documents.retain(|_, document| document.owner != *owner);
        drop(plugin_v2_documents);
        self.remove_template_actions_for_documents(&retired_document_ids);
        retired_document_ids
    }

    #[cfg(test)]
    pub(super) fn unregister_plugin_v2_documents_for_owner(&self, owner_id: &str) -> Vec<String> {
        let owner = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned")
            .values()
            .find(|document| document.owner.owner_id() == owner_id)
            .map(|document| document.owner.clone());
        owner
            .as_ref()
            .map(|owner| self.unregister_plugin_v2_documents(owner))
            .unwrap_or_default()
    }

    pub(super) fn plugin_v2_document_owner(
        &self,
        document_id: &str,
    ) -> Option<EditorPluginV2DocumentOwner> {
        self.plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned")
            .get(document_id)
            .map(|document| document.owner.clone())
    }

    pub(super) fn v2_document(&self, document_id: &str) -> Option<EditorUiHostV2Document> {
        self.plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned")
            .get(document_id)
            .map(|document| document.document.clone())
            .or_else(|| self.v2_documents.get(document_id).cloned())
    }

    pub(super) fn ensure_document_id_is_not_plugin_owned(
        &self,
        document_id: &str,
    ) -> Result<(), EditorUiHostRuntimeError> {
        if let Some(existing) = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned")
            .get(document_id)
        {
            return Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
                document_id: document_id.to_string(),
                owner_id: existing.owner.owner_id().to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "plugin_documents/tests.rs"]
mod tests;
