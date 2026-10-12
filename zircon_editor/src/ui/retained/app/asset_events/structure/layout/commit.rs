use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_layout_commit_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
        value: &str,
    ) -> bool {
        let target_path = match action_id {
            "layout.width.preferred.set" => "layout.width_preferred",
            "layout.height.preferred.set" => "layout.height_preferred",
            _ => return false,
        };

        self.dispatch_ui_asset_component_adapter_commit(
            instance_id.0.as_str(),
            action_id,
            target_path,
            value,
        );
        true
    }
}
