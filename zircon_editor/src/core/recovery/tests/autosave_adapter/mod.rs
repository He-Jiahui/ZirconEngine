use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::super::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDiagnosticStore, AutosaveDocumentId,
    AutosaveDocumentOutcomeKind, AutosaveDocumentRequest, AutosaveDocumentState,
    AutosaveFailureStage, AutosaveJobAdapter, AutosaveJobPolicy, AutosavePolicy,
    AutosaveRetryability, AutosaveScheduler, AutosaveSnapshot, AutosaveSnapshotProvenance,
    AutosaveSnapshotSource, AutosaveSourceDigest, AutosaveSourcePath, AutosaveStore,
    EditorAutosaveService,
};
use super::{document_id, extension, recovery_source_path, remove_temporary_root, temporary_root};
use crate::core::jobs::{
    test_job_system_with_limits, EditorJob, EditorJobAdmissionLimits, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, MutexGroup,
};

mod admission;
mod completion;
mod outcomes;
mod scheduling;
mod shutdown;
mod support;

use support::{
    wait_for_autosave_completion, wait_for_autosave_completion_state, wait_for_capture_count,
    CountingSnapshotSource, GateJob,
};
