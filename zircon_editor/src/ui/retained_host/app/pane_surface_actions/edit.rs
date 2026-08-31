use super::super::*;
use super::ui_asset_detail::UiAssetDetailSurfaceBinding;
use crate::ui::retained_host::app::hierarchy_rename::HIERARCHY_INLINE_RENAME_CONTROL_ID;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_pane_surface_control_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        if matches!(
            binding_id,
            "Workbench/SceneSearchEdit" | "Workbench/SceneSearchCommit"
        ) {
            self.set_hierarchy_filter_query(value);
            return;
        }
        if control_id == HIERARCHY_INLINE_RENAME_CONTROL_ID {
            self.dispatch_hierarchy_rename_edit(binding_id, value);
            return;
        }
        if let Some(result) = self
            .dispatch_componentized_workbench_surface_control_edited(control_id, binding_id, value)
        {
            self.apply_dispatch_result(result);
            return;
        }
        if let Some(binding) = UiAssetDetailSurfaceBinding::parse(binding_id) {
            self.dispatch_ui_asset_detail_event(
                &binding.instance_id,
                &binding.detail_id,
                &binding.action_id,
                binding.item_index,
                value,
                "",
            );
            return;
        }

        let resolved_binding_id = self
            .pane_surface_bridge
            .binding_id_for_action_id(binding_id)
            .unwrap_or_else(|| binding_id.to_string());
        let Some(binding) = self
            .pane_surface_bridge
            .binding_by_id(resolved_binding_id.as_str())
            .cloned()
        else {
            let result = callback_dispatch::dispatch_builtin_template_binding_with_arguments(
                &self.runtime,
                binding_id,
                vec![UiBindingValue::string(value)],
            )
            .unwrap_or_else(|| Err(format!("Unknown pane surface edit binding {binding_id}")));
            self.apply_dispatch_result(result);
            return;
        };
        let result = callback_dispatch::dispatch_template_binding_with_arguments(
            &self.runtime,
            binding,
            vec![UiBindingValue::string(value)],
        );
        self.apply_dispatch_result(result.map_err(|error| {
            format!("Pane surface edit {control_id} via {binding_id} failed: {error}")
        }));
    }
}
