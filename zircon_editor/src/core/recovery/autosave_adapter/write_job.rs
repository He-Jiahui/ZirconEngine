use std::path::PathBuf;
use std::sync::Arc;

use crate::core::jobs::{EditorJob, JobContext, JobError};

use super::super::{AutosaveDocumentId, AutosaveSourcePath, AutosaveStore};
use super::model::AutosaveWriteFailure;
use super::{
    AutosaveDocumentOutcome, AutosaveFailureStage, AutosaveSnapshotSource, AutosaveWriteResult,
};

pub(super) struct AutosaveWriteJob {
    pub(super) document: AutosaveDocumentId,
    pub(super) source_path: AutosaveSourcePath,
    pub(super) source: Arc<dyn AutosaveSnapshotSource>,
    pub(super) store: AutosaveStore,
}

impl EditorJob for AutosaveWriteJob {
    type Output = AutosaveWriteResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        match self.execute(&context) {
            Ok(result) => {
                let outcome = AutosaveDocumentOutcome::saved(
                    result.document.clone(),
                    self.source_path.clone(),
                    result.snapshot_path.clone(),
                    true,
                );
                self.store.persist_diagnostic(&outcome).map_err(|error| {
                    JobError::failed(
                        AutosaveWriteFailure::from_error(
                            AutosaveFailureStage::DiagnosticPersistence,
                            &error,
                        )
                        .with_usable_snapshot(result.snapshot_path.clone()),
                    )
                })?;
                Ok(result)
            }
            Err(failure) => {
                let outcome = AutosaveDocumentOutcome::failed(
                    self.document.clone(),
                    self.source_path.clone(),
                    &failure,
                );
                let failure = match self.store.persist_diagnostic(&outcome) {
                    Ok(_) => failure,
                    Err(error) => failure.with_diagnostic_persistence_failure(&error),
                };
                Err(JobError::failed(failure))
            }
        }
    }
}

impl AutosaveWriteJob {
    fn execute(&self, context: &JobContext) -> Result<AutosaveWriteResult, AutosaveWriteFailure> {
        context.check_cancelled().map_err(|error| {
            AutosaveWriteFailure::from_job_error(AutosaveFailureStage::Cancelled, error)
        })?;
        let snapshot = self.source.capture(&self.document).map_err(|error| {
            AutosaveWriteFailure::from_job_error(AutosaveFailureStage::Capture, error)
        })?;
        if snapshot.source_path != self.source_path {
            return Err(AutosaveWriteFailure::source_identity_changed());
        }
        context.check_cancelled().map_err(|error| {
            AutosaveWriteFailure::from_job_error(AutosaveFailureStage::Cancelled, error)
        })?;
        let sequence = self
            .store
            .next_sequence(&self.document, snapshot.sequence)
            .map_err(|error| {
                AutosaveWriteFailure::from_autosave_error(AutosaveFailureStage::Sequence, error)
            })?;
        let snapshot_path = self
            .store
            .write_snapshot(
                &self.document,
                &snapshot.source_path,
                sequence,
                &snapshot.extension,
                &snapshot.provenance,
                &snapshot.bytes,
            )
            .map_err(|error| {
                AutosaveWriteFailure::from_autosave_error(
                    AutosaveFailureStage::SnapshotCommit,
                    error,
                )
            })?;
        Ok(AutosaveWriteResult {
            document: self.document.clone(),
            snapshot_path,
            diagnostic_persisted: true,
        })
    }
}

#[cfg(test)]
mod outcome_tests {
    use std::path::PathBuf;

    use super::{AutosaveDocumentOutcome, AutosaveFailureStage, AutosaveWriteFailure};
    use crate::core::recovery::{AutosaveDocumentId, AutosaveError, AutosaveSourcePath};

    #[test]
    fn retention_failure_keeps_the_persisted_snapshot_available_to_the_outcome() {
        let snapshot_path = PathBuf::from(".zircon/autosave/scene_main/1.zscene");
        let failure = AutosaveWriteFailure::from_autosave_error(
            AutosaveFailureStage::SnapshotCommit,
            AutosaveError::RotationAfterWrite {
                snapshot: snapshot_path.clone(),
                source: Box::new(AutosaveError::InvalidSequence { sequence: 0 }),
            },
        );
        let outcome = AutosaveDocumentOutcome::failed(
            AutosaveDocumentId::parse("scene_main").unwrap(),
            AutosaveSourcePath::parse("scenes/main.zscene").unwrap(),
            &failure,
        );

        assert_eq!(
            outcome.failure_stage(),
            Some(AutosaveFailureStage::Retention)
        );
        assert_eq!(outcome.usable_snapshot(), Some(snapshot_path.as_path()));
        assert!(outcome.diagnostic_persisted());
    }
}
