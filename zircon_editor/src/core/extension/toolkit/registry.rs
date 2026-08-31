use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_message::DocumentId;

use super::save::{DocumentSourceWriteAuthority, DocumentSourceWriteLease};
use super::{
    DocumentAutosavePayload, DocumentCloseLease, DocumentSaveReport, DocumentToolkit,
    DocumentToolkitDescriptor, DocumentToolkitSnapshot, SaveCtx, SaveError, SaveReason,
    ToolkitInstanceId, ToolkitRegistryError,
};

pub struct DocumentToolkitRegistry<Host> {
    state: Mutex<RegistryState<Host>>,
    source_writes: DocumentSourceWriteAuthority,
}

struct RegistryState<Host> {
    by_document: BTreeMap<DocumentId, RegistryEntry<Host>>,
    by_instance: BTreeMap<ToolkitInstanceId, DocumentId>,
    last_document_id: u64,
    snapshot: DocumentToolkitSnapshot,
}

struct RegistryEntry<Host> {
    toolkit: Arc<dyn DocumentToolkit<Host>>,
    descriptor: DocumentToolkitDescriptor,
    active_saves: usize,
    closing: bool,
}

fn validate_menu_items(descriptor: &DocumentToolkitDescriptor) -> Result<(), ToolkitRegistryError> {
    let instance = descriptor.instance_id();
    let mut paths = BTreeSet::new();
    for menu_item in descriptor.menu_items() {
        let mut segment_count = 0;
        let valid = menu_item.path().split('/').all(|segment| {
            segment_count += 1;
            !segment.is_empty() && segment.trim() == segment
        });
        if !valid || segment_count < 2 {
            return Err(ToolkitRegistryError::InvalidMenuPath {
                instance: instance.clone(),
                path: menu_item.path().to_string(),
            });
        }
        if !paths.insert(menu_item.path()) {
            return Err(ToolkitRegistryError::DuplicateMenuPath {
                instance: instance.clone(),
                path: menu_item.path().to_string(),
            });
        }
    }
    Ok(())
}

impl<Host> Default for RegistryState<Host> {
    fn default() -> Self {
        Self {
            by_document: BTreeMap::new(),
            by_instance: BTreeMap::new(),
            last_document_id: 0,
            snapshot: DocumentToolkitSnapshot::new(0, Vec::new()),
        }
    }
}

impl<Host> RegistryState<Host> {
    fn next_generation(&self) -> Result<u64, ToolkitRegistryError> {
        self.snapshot
            .generation()
            .checked_add(1)
            .ok_or(ToolkitRegistryError::GenerationExhausted)
    }

    fn publish_snapshot(&mut self, generation: u64) {
        self.snapshot = DocumentToolkitSnapshot::new(
            generation,
            self.by_document
                .values()
                .map(|entry| entry.descriptor.clone())
                .collect(),
        );
    }
}

impl<Host> Default for DocumentToolkitRegistry<Host> {
    fn default() -> Self {
        Self {
            state: Mutex::new(RegistryState::default()),
            source_writes: DocumentSourceWriteAuthority::default(),
        }
    }
}

impl<Host> DocumentToolkitRegistry<Host> {
    pub(crate) fn with_source_write<T>(
        &self,
        project_root: &Path,
        source_path: &Path,
        operation: impl FnOnce(&DocumentSourceWriteLease<'_>) -> T,
    ) -> io::Result<T> {
        let lease = self.source_writes.acquire(project_root, source_path)?;
        Ok(operation(&lease))
    }

    pub fn allocate_document_id(&self) -> Result<DocumentId, ToolkitRegistryError> {
        let mut state = self.lock_state();
        let next = state
            .last_document_id
            .checked_add(1)
            .ok_or(ToolkitRegistryError::DocumentIdExhausted)?;
        state.last_document_id = next;
        Ok(DocumentId::new(next))
    }

    pub fn register(
        &self,
        toolkit: Arc<dyn DocumentToolkit<Host>>,
    ) -> Result<(), ToolkitRegistryError> {
        let descriptor = toolkit.descriptor().clone();
        let document = descriptor.document_id();
        let instance = descriptor.instance_id().clone();
        validate_menu_items(&descriptor)?;
        let mut state = self.lock_state();
        if state.by_document.contains_key(&document) {
            return Err(ToolkitRegistryError::DocumentAlreadyRegistered { document });
        }
        if state.by_instance.contains_key(&instance) {
            return Err(ToolkitRegistryError::InstanceAlreadyRegistered { instance });
        }
        let generation = state.next_generation()?;

        state.by_instance.insert(instance, document);
        state.last_document_id = state.last_document_id.max(document.value());
        state.by_document.insert(
            document,
            RegistryEntry {
                toolkit,
                descriptor,
                active_saves: 0,
                closing: false,
            },
        );
        state.publish_snapshot(generation);
        Ok(())
    }

    pub fn unregister(
        &self,
        instance: &ToolkitInstanceId,
    ) -> Result<Option<DocumentToolkitDescriptor>, ToolkitRegistryError> {
        let Some(close) = self.begin_close(instance)? else {
            return Ok(None);
        };
        close.commit().map(Some)
    }

    pub fn begin_close(
        &self,
        instance: &ToolkitInstanceId,
    ) -> Result<Option<DocumentCloseLease<'_, Host>>, ToolkitRegistryError> {
        let mut state = self.lock_state();
        let Some(document) = state.by_instance.get(instance).copied() else {
            return Ok(None);
        };
        let entry = state
            .by_document
            .get_mut(&document)
            .ok_or(ToolkitRegistryError::CloseLeaseInvalid { document })?;
        if entry.active_saves > 0 {
            return Err(ToolkitRegistryError::DocumentBusy {
                document,
                active_saves: entry.active_saves,
            });
        }
        if entry.closing {
            return Err(ToolkitRegistryError::CloseAlreadyInProgress { document });
        }
        entry.closing = true;
        Ok(Some(DocumentCloseLease::new(
            self,
            document,
            instance.clone(),
        )))
    }

    pub fn clear(&self) -> Result<Vec<DocumentToolkitDescriptor>, ToolkitRegistryError> {
        let mut state = self.lock_state();
        if state.by_document.is_empty() {
            return Ok(Vec::new());
        }
        let busy_documents = state
            .by_document
            .iter()
            .filter_map(|(document, entry)| (entry.active_saves > 0).then_some(*document))
            .collect::<Vec<_>>();
        if !busy_documents.is_empty() {
            return Err(ToolkitRegistryError::DocumentsBusy {
                documents: busy_documents,
            });
        }
        let closing_documents = state
            .by_document
            .iter()
            .filter_map(|(document, entry)| entry.closing.then_some(*document))
            .collect::<Vec<_>>();
        if !closing_documents.is_empty() {
            return Err(ToolkitRegistryError::DocumentsClosing {
                documents: closing_documents,
            });
        }
        let generation = state.next_generation()?;
        let closed = state
            .by_document
            .values()
            .map(|entry| entry.descriptor.clone())
            .collect();
        let retired = std::mem::take(&mut state.by_document);
        state.by_instance.clear();
        state.publish_snapshot(generation);
        drop(state);
        drop(retired);
        Ok(closed)
    }

    pub fn document_for_instance(&self, instance: &ToolkitInstanceId) -> Option<DocumentId> {
        self.lock_state().by_instance.get(instance).copied()
    }

    pub fn snapshot(&self) -> DocumentToolkitSnapshot {
        self.lock_state().snapshot.clone()
    }

    pub fn save(
        &self,
        document: DocumentId,
        host: &Host,
        reason: SaveReason,
    ) -> Result<DocumentSaveReport, SaveError> {
        let (toolkit, instance, _lease) = self.begin_save(document)?;
        toolkit
            .validate_references(host)
            .map_err(|source| SaveError::ReferenceValidationFailed { document, source })?;
        let mut context = SaveCtx::new(reason);
        toolkit
            .save(host, &mut context)
            .map_err(|source| SaveError::HookFailed { document, source })?;
        Ok(DocumentSaveReport::new(
            document,
            instance,
            reason,
            context.written_bytes(),
            context.source_write_guarantee(),
        ))
    }

    pub fn validate_references(&self, document: DocumentId, host: &Host) -> Result<(), SaveError> {
        let (toolkit, _instance, _lease) = self.begin_save(document)?;
        toolkit
            .validate_references(host)
            .map_err(|source| SaveError::ReferenceValidationFailed { document, source })
    }

    /// Uses the foreground-save lease for snapshot capture. Autosave therefore
    /// shares the document authority's real exclusion in addition to the job
    /// queue mutex group used for asynchronous ordering.
    pub fn capture_autosave(
        &self,
        document: DocumentId,
        host: &Host,
    ) -> Result<DocumentAutosavePayload, SaveError> {
        let (toolkit, _instance, _lease) = self.begin_save(document)?;
        toolkit
            .capture_autosave(host)
            .map_err(|source| SaveError::AutosaveHookFailed { document, source })
    }

    pub fn autosave_source_path(
        &self,
        document: DocumentId,
        host: &Host,
    ) -> Result<std::path::PathBuf, SaveError> {
        let toolkit = {
            let state = self.lock_state();
            let entry = state
                .by_document
                .get(&document)
                .ok_or(SaveError::DocumentNotRegistered { document })?;
            if entry.closing {
                return Err(SaveError::DocumentClosing { document });
            }
            Arc::clone(&entry.toolkit)
        };
        toolkit
            .autosave_source_path(host)
            .map_err(|source| SaveError::AutosaveHookFailed { document, source })
    }

    fn lock_state(&self) -> MutexGuard<'_, RegistryState<Host>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn begin_save(
        &self,
        document: DocumentId,
    ) -> Result<
        (
            Arc<dyn DocumentToolkit<Host>>,
            ToolkitInstanceId,
            SaveLease<'_, Host>,
        ),
        SaveError,
    > {
        let mut state = self.lock_state();
        let entry = state
            .by_document
            .get_mut(&document)
            .ok_or(SaveError::DocumentNotRegistered { document })?;
        if entry.closing {
            return Err(SaveError::DocumentClosing { document });
        }
        if entry.active_saves > 0 {
            return Err(SaveError::SaveAlreadyInProgress { document });
        }
        entry.active_saves = 1;
        Ok((
            Arc::clone(&entry.toolkit),
            entry.descriptor.instance_id().clone(),
            SaveLease {
                registry: self,
                document,
            },
        ))
    }

    fn finish_save(&self, document: DocumentId) {
        let mut state = self.lock_state();
        let Some(entry) = state.by_document.get_mut(&document) else {
            return;
        };
        entry.active_saves = entry.active_saves.saturating_sub(1);
    }

    pub(super) fn commit_close(
        &self,
        document: DocumentId,
        instance: &ToolkitInstanceId,
    ) -> Result<DocumentToolkitDescriptor, ToolkitRegistryError> {
        let mut state = self.lock_state();
        if state.by_instance.get(instance).copied() != Some(document) {
            return Err(ToolkitRegistryError::CloseLeaseInvalid { document });
        }
        let entry = state
            .by_document
            .get(&document)
            .ok_or(ToolkitRegistryError::CloseLeaseInvalid { document })?;
        if !entry.closing || entry.active_saves > 0 {
            return Err(ToolkitRegistryError::CloseLeaseInvalid { document });
        }
        let generation = state.next_generation()?;
        let entry = state
            .by_document
            .remove(&document)
            .ok_or(ToolkitRegistryError::CloseLeaseInvalid { document })?;
        state.by_instance.remove(instance);
        state.publish_snapshot(generation);
        let descriptor = entry.descriptor.clone();
        drop(state);
        drop(entry);
        Ok(descriptor)
    }

    pub(super) fn rollback_close(&self, document: DocumentId, instance: &ToolkitInstanceId) {
        let mut state = self.lock_state();
        if state.by_instance.get(instance).copied() != Some(document) {
            return;
        }
        if let Some(entry) = state.by_document.get_mut(&document) {
            entry.closing = false;
        }
    }
}

struct SaveLease<'a, Host> {
    registry: &'a DocumentToolkitRegistry<Host>,
    document: DocumentId,
}

impl<Host> Drop for SaveLease<'_, Host> {
    fn drop(&mut self) {
        self.registry.finish_save(self.document);
    }
}
