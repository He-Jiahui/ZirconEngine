use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

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
        true
    }

    pub(super) fn try_admission(&self) -> Option<ImportAdmission> {
        self.admission
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
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

    pub(super) fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetImportResult, JobError>> {
        let mut result = self.lock_result();
        loop {
            if let Some(result) = result.as_ref() {
                return Some(result.clone());
            }
            let remaining = deadline.checked_duration_since(Instant::now())?;
            let (next, _) = self
                .completed
                .wait_timeout(result, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            result = next;
        }
    }

    fn lock_result(&self) -> MutexGuard<'_, Option<Result<EditorAssetImportResult, JobError>>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug, Default)]
pub(super) struct SharedImportReasons(AtomicU8);

impl SharedImportReasons {
    pub(super) fn add(&self, reason: EditorAssetImportReason) {
        self.0.fetch_or(reason_bit(reason), Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> Vec<EditorAssetImportReason> {
        let bits = self.0.load(Ordering::Relaxed);
        let mut reasons = Vec::with_capacity(bits.count_ones() as usize);
        for reason in IMPORT_REASON_ORDER {
            if bits & reason_bit(reason) != 0 {
                reasons.push(reason);
            }
        }
        reasons
    }

    pub(super) fn len(&self) -> usize {
        self.0.load(Ordering::Relaxed).count_ones() as usize
    }
}

const IMPORT_REASON_ORDER: [EditorAssetImportReason; 3] = [
    EditorAssetImportReason::Watch,
    EditorAssetImportReason::DigestMismatch,
    EditorAssetImportReason::Manual,
];

const fn reason_bit(reason: EditorAssetImportReason) -> u8 {
    match reason {
        EditorAssetImportReason::Watch => 1 << 0,
        EditorAssetImportReason::DigestMismatch => 1 << 1,
        EditorAssetImportReason::Manual => 1 << 2,
    }
}

#[cfg(test)]
#[path = "tests/flight.rs"]
mod tests;
