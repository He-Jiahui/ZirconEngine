use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_locale_preview_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "locale.preview.authoring_fallback" => {
                self.set_ui_asset_editor_locale_preview(instance_id, "authoring-fallback")
            }
            "locale.preview.en_us" => self.set_ui_asset_editor_locale_preview(instance_id, "en-US"),
            "locale.preview.zh_cn" => self.set_ui_asset_editor_locale_preview(instance_id, "zh-CN"),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }

    fn set_ui_asset_editor_locale_preview(
        &self,
        instance_id: &ViewInstanceId,
        locale: &str,
    ) -> UiAssetActionDispatch {
        UiAssetActionDispatch::handled(
            self.editor_manager
                .set_ui_asset_editor_locale_preview(instance_id, locale),
        )
    }
}
