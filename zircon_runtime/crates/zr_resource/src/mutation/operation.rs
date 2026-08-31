use std::sync::Arc;

use crate::{ResourceData, ResourceDiagnostic, ResourceId, ResourceLocator, ResourceRecord};

#[derive(Debug)]
pub(crate) enum ResourceMutationOperation {
    UpsertLazy(ResourceRecord),
    UpsertReady {
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
        recover_from_error: bool,
    },
    StorePayload {
        id: ResourceId,
        expected_revision: u64,
        payload: Arc<dyn ResourceData>,
    },
    StartReload {
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    },
    FailReload {
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    },
    Rename {
        from: ResourceLocator,
        to: ResourceLocator,
    },
    Remove {
        locator: ResourceLocator,
        expected_kind: Option<crate::ResourceKind>,
    },
}
