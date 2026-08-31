use std::sync::Arc;

use crate::{ResourceData, ResourceDiagnostic, ResourceId, ResourceLocator, ResourceRecord};

use super::ResourceMutationOperation;

#[derive(Debug, Default)]
pub struct ResourceMutationBatch {
    operations: Vec<ResourceMutationOperation>,
}

impl ResourceMutationBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn upsert_lazy(mut self, record: ResourceRecord) -> Self {
        self.operations
            .push(ResourceMutationOperation::UpsertLazy(record));
        self
    }

    pub fn upsert_ready<TData>(self, record: ResourceRecord, payload: TData) -> Self
    where
        TData: ResourceData,
    {
        self.upsert_ready_erased(record, Arc::new(payload))
    }

    pub(crate) fn upsert_ready_erased(
        self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.push_ready(record, payload, false)
    }

    /// Adds an imported payload that has already crossed a type-erased importer boundary.
    pub fn upsert_imported_erased(
        self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.push_ready(record, payload, true)
    }

    fn push_ready(
        mut self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
        recover_from_error: bool,
    ) -> Self {
        self.operations
            .push(ResourceMutationOperation::UpsertReady {
                record,
                payload,
                recover_from_error,
            });
        self
    }

    pub fn store_payload<TData>(
        self,
        id: ResourceId,
        expected_revision: u64,
        payload: TData,
    ) -> Self
    where
        TData: ResourceData,
    {
        self.store_payload_erased(id, expected_revision, Arc::new(payload))
    }

    /// Stores a payload that has already crossed a type-erased resource-service boundary.
    pub fn store_payload_erased(
        mut self,
        id: ResourceId,
        expected_revision: u64,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.operations
            .push(ResourceMutationOperation::StorePayload {
                id,
                expected_revision,
                payload,
            });
        self
    }

    pub fn start_reload(mut self, id: ResourceId, diagnostics: Vec<ResourceDiagnostic>) -> Self {
        self.operations
            .push(ResourceMutationOperation::StartReload { id, diagnostics });
        self
    }

    pub fn fail_reload(mut self, id: ResourceId, diagnostics: Vec<ResourceDiagnostic>) -> Self {
        self.operations
            .push(ResourceMutationOperation::FailReload { id, diagnostics });
        self
    }

    pub fn rename(mut self, from: ResourceLocator, to: ResourceLocator) -> Self {
        self.operations
            .push(ResourceMutationOperation::Rename { from, to });
        self
    }

    pub fn remove(mut self, locator: ResourceLocator) -> Self {
        self.operations.push(ResourceMutationOperation::Remove {
            locator,
            expected_kind: None,
        });
        self
    }

    pub fn remove_kind(
        mut self,
        locator: ResourceLocator,
        expected_kind: crate::ResourceKind,
    ) -> Self {
        self.operations.push(ResourceMutationOperation::Remove {
            locator,
            expected_kind: Some(expected_kind),
        });
        self
    }

    pub(crate) fn operations(self) -> Vec<ResourceMutationOperation> {
        self.operations
    }
}
