use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::asset_editor::UiAssetPreviewPreset;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_preview_preset_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "preview.preset.editor_docked" => self.set_ui_asset_editor_preview_preset(
                instance_id,
                UiAssetPreviewPreset::EditorDocked,
            ),
            "preview.preset.editor_floating" => self.set_ui_asset_editor_preview_preset(
                instance_id,
                UiAssetPreviewPreset::EditorFloating,
            ),
            "preview.preset.game_hud" => {
                self.set_ui_asset_editor_preview_preset(instance_id, UiAssetPreviewPreset::GameHud)
            }
            "preview.preset.dialog" => {
                self.set_ui_asset_editor_preview_preset(instance_id, UiAssetPreviewPreset::Dialog)
            }
            _ => UiAssetActionDispatch::Unhandled,
        }
    }

    fn set_ui_asset_editor_preview_preset(
        &self,
        instance_id: &ViewInstanceId,
        preset: UiAssetPreviewPreset,
    ) -> UiAssetActionDispatch {
        UiAssetActionDispatch::handled(
            self.editor_manager
                .set_ui_asset_editor_preview_preset(instance_id, preset),
        )
    }
}
