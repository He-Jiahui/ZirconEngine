use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::extension::{SaveCtx, SaveReason, ToolkitSaveFailure};
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn save_animation_document(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
    context: &mut SaveCtx,
) -> Result<(), ToolkitSaveFailure> {
    let written_bytes = host.save_animation_editor_canonical(instance_id)?;
    context.record_written_bytes(written_bytes)?;
    Ok(())
}

impl EditorUiHost {
    pub fn save_animation_editor(&self, instance_id: &ViewInstanceId) -> Result<(), EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        self.save_document_toolkit(instance_id, SaveReason::Explicit)?;
        Ok(())
    }

    fn save_animation_editor_canonical(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<u64, EditorError> {
        let asset_locator = {
            let mut sessions = self.lock_animation_editor_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing animation editor session {}",
                    instance_id.0
                ))
            })?;
            entry
                .session
                .save()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            entry.route.asset_locator().to_string()
        };
        let source_path = self.resolve_asset_locator_path(&asset_locator)?;
        let written_bytes = fs::metadata(source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?
            .len();
        let _ = self.asset_manager()?.import_asset(&asset_locator);
        self.sync_animation_editor_instance(instance_id)?;
        Ok(written_bytes)
    }
}
use std::fs;
