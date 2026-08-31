use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime_interface::project::ProjectActivationOperationId;

use super::ProjectCloseCoordinatorPhase;

#[derive(Debug, Error)]
pub(crate) enum ProjectCloseTransitionError {
    #[error("project close has no active operation in phase `{phase:?}`")]
    MissingOperation { phase: ProjectCloseCoordinatorPhase },
    #[error(
        "project close operation mismatch: active `{active_root:?}`/{active_operation_id:?}, requested `{requested_root:?}`/{requested_operation_id:?}"
    )]
    OperationMismatch {
        active_root: PathBuf,
        active_operation_id: ProjectActivationOperationId,
        requested_root: PathBuf,
        requested_operation_id: ProjectActivationOperationId,
    },
    #[error("project close cannot transition from `{current:?}` to `{requested:?}`")]
    InvalidTransition {
        current: ProjectCloseCoordinatorPhase,
        requested: ProjectCloseCoordinatorPhase,
    },
}
