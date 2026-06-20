use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_theme_source_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "theme.source.open" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .open_ui_asset_editor_selected_theme_source(instance_id),
            ),
            "theme.local.promote" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .promote_ui_asset_editor_local_theme_to_external_style_asset(instance_id),
            ),
            "theme.source.detach_local" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .detach_ui_asset_editor_selected_theme_source_to_local(instance_id),
            ),
            "theme.source.clone_local" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .clone_ui_asset_editor_selected_theme_source_to_local(instance_id),
            ),
            "theme.local.prune_duplicates" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .prune_ui_asset_editor_duplicate_local_theme_overrides(instance_id),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
