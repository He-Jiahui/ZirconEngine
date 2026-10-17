use crate::core::document::{OverwriteLease, SceneSaveOutcome};
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, HistorySaveMarkOutcome, HistorySaveToken,
};
use crate::core::editor_event::MenuAction;
use crate::ui::host::EditorHostEventController;

use super::{ExecutionOutcome, MenuActionExecutionError};

/// Explicit decision route into the same history-token and publication executor as menu Save.
pub(crate) fn execute_scene_save(
    controller: &EditorHostEventController,
    overwrite: Option<OverwriteLease>,
) -> Result<ExecutionOutcome, MenuActionExecutionError> {
    super::menu_action::execute_menu_action_with_save_authorization(
        controller,
        &MenuAction::SaveProject,
        overwrite,
    )
}

/// Both ordinary and authorized Save reach this gate before advancing the clean checkpoint.
pub(crate) fn complete_scene_save_checkpoint(
    outcome: &SceneSaveOutcome,
    transactions: &EditorTransactionEngine,
    history: HistoryContextId,
    token: HistorySaveToken,
) -> Result<HistorySaveMarkOutcome, MenuActionExecutionError> {
    match outcome {
        SceneSaveOutcome::Persisted { receipt } => {
            receipt
                .ensure_durable()
                .map_err(crate::ui::host::EditorError::from)?;
        }
        SceneSaveOutcome::PersistedProjectionFailed { receipt, phase } => {
            return Err(MenuActionExecutionError::PersistedProjectionFailed {
                receipt: Box::new(receipt.clone()),
                phase: *phase,
            });
        }
        SceneSaveOutcome::RecoveryDeferred { receipt } => {
            return Err(crate::ui::host::EditorError::from(
                zircon_runtime::scene::world::SceneProjectError::PublicationRecoveryDeferred {
                    receipt: Box::new(receipt.clone()),
                },
            )
            .into());
        }
    }
    transactions
        .mark_saved_if_unchanged(history, token)
        .map_err(
            |source| MenuActionExecutionError::PersistedCheckpointFailed {
                receipt: Box::new(outcome.receipt().clone()),
                source,
            },
        )
}
