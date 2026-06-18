use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_slot_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "slot.mount.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.mount",
                    value,
                );
                return;
            }
            "slot.padding.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.padding",
                    value,
                );
                return;
            }
            "slot.layout.width.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.width_preferred",
                    value,
                );
                return;
            }
            "slot.layout.height.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.height_preferred",
                    value,
                );
                return;
            }
            "slot.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.semantic.value",
                    value,
                );
                return;
            }
            "slot.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_slot_semantic(&instance_id)
                .map(|_| ()),
            other => {
                if let Some(path) = slot_semantic_action_path(other) {
                    self.dispatch_ui_asset_component_adapter_commit(
                        instance_id.0.as_str(),
                        action_id,
                        &format!("slot.semantic.field.{path}"),
                        value,
                    );
                    return;
                } else {
                    self.set_status_line(format!("Unknown UI asset slot action {other}"));
                    return;
                }
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_layout_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "layout.width.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.width_preferred",
                    value,
                );
                return;
            }
            "layout.height.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.height_preferred",
                    value,
                );
                return;
            }
            "layout.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.semantic.value",
                    value,
                );
                return;
            }
            "layout.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_layout_semantic(&instance_id)
                .map(|_| ()),
            other => {
                if let Some(path) = layout_semantic_action_path(other) {
                    self.dispatch_ui_asset_component_adapter_commit(
                        instance_id.0.as_str(),
                        action_id,
                        &format!("layout.semantic.field.{path}"),
                        value,
                    );
                    return;
                } else {
                    self.set_status_line(format!("Unknown UI asset layout action {other}"));
                    return;
                }
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn slot_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "slot.linear.width_weight.set" => Some("layout.width.weight"),
        "slot.linear.width_stretch.set" => Some("layout.width.stretch"),
        "slot.linear.height_weight.set" => Some("layout.height.weight"),
        "slot.linear.height_stretch.set" => Some("layout.height.stretch"),
        "slot.overlay.anchor_x.set" => Some("layout.anchor.x"),
        "slot.overlay.anchor_y.set" => Some("layout.anchor.y"),
        "slot.overlay.pivot_x.set" => Some("layout.pivot.x"),
        "slot.overlay.pivot_y.set" => Some("layout.pivot.y"),
        "slot.overlay.position_x.set" => Some("layout.position.x"),
        "slot.overlay.position_y.set" => Some("layout.position.y"),
        "slot.overlay.z_index.set" => Some("layout.z_index"),
        "slot.grid.row.set" => Some("row"),
        "slot.grid.column.set" => Some("column"),
        "slot.grid.row_span.set" => Some("row_span"),
        "slot.grid.column_span.set" => Some("column_span"),
        "slot.flow.break_before.set" => Some("break_before"),
        "slot.flow.alignment.set" => Some("alignment"),
        _ => None,
    }
}

fn layout_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "layout.box.gap.set" => Some("container.gap"),
        "layout.scroll.axis.set" => Some("container.axis"),
        "layout.scroll.gap.set" => Some("container.gap"),
        "layout.scroll.scrollbar_visibility.set" => Some("container.scrollbar_visibility"),
        "layout.scroll.virtualization.item_extent.set" => {
            Some("container.virtualization.item_extent")
        }
        "layout.scroll.virtualization.overscan.set" => Some("container.virtualization.overscan"),
        "layout.scroll.clip.set" => Some("clip"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{layout_semantic_action_path, slot_semantic_action_path};

    #[test]
    fn layout_semantic_action_path_maps_linear_box_gap_action() {
        assert_eq!(
            layout_semantic_action_path("layout.box.gap.set"),
            Some("container.gap")
        );
    }

    #[test]
    fn slot_semantic_action_path_maps_linear_slot_actions() {
        assert_eq!(
            slot_semantic_action_path("slot.linear.width_weight.set"),
            Some("layout.width.weight")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.width_stretch.set"),
            Some("layout.width.stretch")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.height_weight.set"),
            Some("layout.height.weight")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.height_stretch.set"),
            Some("layout.height.stretch")
        );
    }
}
