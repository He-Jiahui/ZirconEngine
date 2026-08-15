use crate::ui::asset_editor::UiAssetEditorReflectionModel;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::ui_asset_editor_session::UiAssetEditorSession;

impl UiAssetEditorSession {
    pub(super) fn reflection_pane_presentation(&self) -> UiAssetEditorReflectionModel {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "reflection",);
        let reflection = self.reflection_model();
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneReflectionBuildCount, 1.0);
        reflection
    }

    pub fn reflection_model(&self) -> UiAssetEditorReflectionModel {
        let mut model = UiAssetEditorReflectionModel::new(
            self.route.clone(),
            self.last_valid_document.asset.display_name.clone(),
        )
        .with_source_dirty(self.source_buffer.is_dirty())
        .with_undo_state(self.can_undo(), self.can_redo())
        .with_preview_available(self.preview_host.is_some())
        .with_shell_state(
            self.shell_state(),
            self.emergency_summary(),
            !self.diagnostics.is_empty(),
            self.can_revert_to_last_valid_source(),
            !self.diagnostics.is_empty(),
        )
        .with_designer_tool_state(
            self.designer_tool_mode(),
            self.can_resize_selected_slot(),
            self.can_preview_interact(),
        )
        .with_selection(self.selection.clone())
        .with_style_inspector(self.style_inspector.clone());
        if let Some(error) = self.diagnostics.first() {
            model = model.with_last_error(error.clone());
        }
        model
    }
}
