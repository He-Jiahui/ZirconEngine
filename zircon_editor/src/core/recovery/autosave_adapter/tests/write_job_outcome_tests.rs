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
