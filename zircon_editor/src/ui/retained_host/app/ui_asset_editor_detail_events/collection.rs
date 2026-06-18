use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_ui_asset_collection_event(
        &mut self,
        instance_id: &str,
        collection_id: &str,
        event_kind: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let item_index = item_index.max(0) as usize;
        let result = match (collection_id, event_kind) {
            ("matched_style_rule", "selected") => self
                .editor_manager
                .select_ui_asset_editor_matched_style_rule(&instance_id, item_index)
                .map(|_| ()),
            ("palette", "selected") => self
                .editor_manager
                .select_ui_asset_editor_palette_index(&instance_id, item_index)
                .map(|_| ()),
            ("palette_target_candidate", "selected") => self
                .editor_manager
                .select_ui_asset_editor_palette_target_candidate(&instance_id, item_index)
                .map(|_| ()),
            ("hierarchy", "selected") => self
                .editor_manager
                .select_ui_asset_editor_hierarchy_index(&instance_id, item_index)
                .map(|_| ()),
            ("hierarchy", "activated") => self
                .editor_manager
                .activate_ui_asset_editor_hierarchy_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview", "activated") => self
                .editor_manager
                .activate_ui_asset_editor_preview_index(&instance_id, item_index)
                .map(|_| ()),
            ("source_outline", "selected") => self
                .editor_manager
                .select_ui_asset_editor_source_outline_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock_subject", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_subject(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_property(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock_nested", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, item_index)
                .map(|_| ()),
            ("binding", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding(&instance_id, item_index)
                .map(|_| ()),
            ("binding_event", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_event_option(&instance_id, item_index)
                .map(|_| ()),
            ("binding_action_kind", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_action_kind(&instance_id, item_index)
                .map(|_| ()),
            ("binding_payload", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_payload(&instance_id, item_index)
                .map(|_| ()),
            ("slot_semantic", "selected") => self
                .editor_manager
                .select_ui_asset_editor_slot_semantic(&instance_id, item_index)
                .map(|_| ()),
            ("layout_semantic", "selected") => self
                .editor_manager
                .select_ui_asset_editor_layout_semantic(&instance_id, item_index)
                .map(|_| ()),
            _ => {
                self.set_status_line(format!(
                    "Unknown UI asset collection event {collection_id}:{event_kind}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
