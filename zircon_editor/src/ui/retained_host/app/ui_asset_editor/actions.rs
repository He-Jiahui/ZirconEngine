use super::super::*;
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

mod canvas;
mod mode_preview;
mod palette;
mod theme;
mod workspace;

pub(super) type UiAssetActionResult = Result<(), EditorError>;

pub(super) enum UiAssetActionDispatch {
    Handled(UiAssetActionResult),
    Consumed,
    Unhandled,
}

impl UiAssetActionDispatch {
    pub(super) fn handled<T>(result: Result<T, EditorError>) -> Self {
        Self::Handled(result.map(|_| ()))
    }
}

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_action_impl(&mut self, instance_id: &str, action_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let mut dispatch = self.dispatch_ui_asset_workspace_action(&instance_id, action_id);
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_theme_action(&instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_palette_action(&instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_canvas_action(&instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_mode_preview_action(&instance_id, action_id);
        }

        let UiAssetActionDispatch::Handled(result) = dispatch else {
            if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
                self.set_status_line(format!("Unknown UI asset editor action {action_id}"));
            }
            return;
        };

        match result {
            Ok(()) => {
                if action_id == "save" || action_id == "workspace.keep_local_and_save" {
                    self.sync_asset_workspace();
                }
                self.mark_presentation_dirty();
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
