use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_theme_source_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "theme.promote.asset_id.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_asset_id(&instance_id, value)
                .map(|_| ()),
            "theme.promote.document_id.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_document_id(&instance_id, value)
                .map(|_| ()),
            "theme.promote.display_name.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_display_name(&instance_id, value)
                .map(|_| ()),
            "theme.rule_helper.apply" => self
                .editor_manager
                .apply_ui_asset_editor_theme_rule_helper_item(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            "theme.refactor.apply" => self
                .editor_manager
                .apply_ui_asset_editor_theme_refactor_item(&instance_id, item_index.max(0) as usize)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset theme source action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
