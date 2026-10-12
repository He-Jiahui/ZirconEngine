use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_slot_commit_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
        value: &str,
    ) -> bool {
        let target_path = match action_id {
            "slot.mount.set" => "slot.mount",
            "slot.padding.set" => "slot.padding",
            "slot.layout.width.preferred.set" => "slot.width_preferred",
            "slot.layout.height.preferred.set" => "slot.height_preferred",
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
