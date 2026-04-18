use crate::SceneRenderer;

use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn release_previous_history(
    renderer: &mut SceneRenderer,
    record_update: &SubmissionRecordUpdate,
) {
    if let Some(previous_handle) = record_update.previous_handle {
        if previous_handle != record_update.history_handle {
            renderer.release_history(previous_handle);
        }
    }
}
