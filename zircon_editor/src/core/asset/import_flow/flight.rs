use std::collections::BTreeSet;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use zircon_runtime::asset::AssetUri;

use crate::core::jobs::{JobError, JobId};

use super::{EditorAssetImportReason, EditorAssetImportResult, EditorAssetImportSubmitError};

#[derive(Clone, Debug)]
pub(super) enum ImportAdmission {
    Admitted(JobId),
    Revalidate,
    Rejected(EditorAssetImportSubmitError),
}

#[derive(Debug)]
pub(super) struct ImportFlight {
    uri: Arc<AssetUri>,
    reasons: Arc<SharedImportReasons>,
    admission: Mutex<Option<ImportAdmission>>,
    admitted: Condvar,
    result: Mutex<Option<Result<EditorAssetImportResult, JobError>>>,
    completed: Condvar,
}

impl ImportFlight {
    pub(super) fn new(uri: Arc<AssetUri>, reason: EditorAssetImportReason) -> Self {
        let reasons = Arc::new(SharedImportReasons::default());
        reasons.add(reason);
        Self {
            uri,
            reasons,
            admission: Mutex::new(None),
            admitted: Condvar::new(),
            result: Mutex::new(None),
            completed: Condvar::new(),
        }
    }

    pub(super) fn uri(&self) -> &Arc<AssetUri> {
        &self.uri
    }

    pub(super) fn reasons(&self) -> &Arc<SharedImportReasons> {
        &self.reasons
    }

    pub(super) fn add_reason(&self, reason: EditorAssetImportReason) {
        self.reasons.add(reason);
    }

    pub(super) fn publish_admission(&self, admission: ImportAdmission) -> bool {
        let mut slot = self
            .admission
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if slot.is_some() {
            return false;
        }
        *slot = Some(admission);
        self.admitted.notify_all();
        true
    }

    pub(super) fn wait_admission(&self) -> ImportAdmission {
        let mut admission = self
            .admission
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            if let Some(admission) = admission.as_ref() {
                return admission.clone();
            }
            admission = self
                .admitted
                .wait(admission)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub(super) fn complete(&self, result: Result<EditorAssetImportResult, JobError>) -> bool {
        let mut slot = self.lock_result();
        if slot.is_some() {
            return false;
        }
        *slot = Some(result);
        self.completed.notify_all();
        true
    }

    pub(super) fn try_result(&self) -> Option<Result<EditorAssetImportResult, JobError>> {
        self.lock_result().clone()
    }

    pub(super) fn wait(&self) -> Result<EditorAssetImportResult, JobError> {
        let mut result = self.lock_result();
        loop {
            if let Some(result) = result.as_ref() {
                return result.clone();
            }
            result = self
                .completed
                .wait(result)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn lock_result(&self) -> MutexGuard<'_, Option<Result<EditorAssetImportResult, JobError>>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug, Default)]
pub(super) struct SharedImportReasons(Mutex<BTreeSet<EditorAssetImportReason>>);

impl SharedImportReasons {
    pub(super) fn add(&self, reason: EditorAssetImportReason) {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(reason);
    }

    pub(super) fn snapshot(&self) -> Vec<EditorAssetImportReason> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .copied()
            .collect()
    }

    pub(super) fn len(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}
