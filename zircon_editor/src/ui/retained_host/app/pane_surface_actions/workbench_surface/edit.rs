use super::super::super::*;
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_componentized_workbench_surface_control_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return None;
        }
        let binding_id = self
            .workbench_window_bridge
            .binding_id_for_action_id(binding_id)
            .unwrap_or_else(|| binding_id.to_string());
        if let Some(result) = self.dispatch_workbench_scene_picker_window_requested(
            control_id,
            binding_id.as_str(),
            value,
        ) {
            return Some(result);
        }
        if let Some(result) = self.dispatch_workbench_command_palette_window_requested(
            control_id,
            binding_id.as_str(),
            value,
        ) {
            return Some(result);
        }
        if let Some(result) = self.dispatch_workbench_scene_picker_query_edited(
            control_id,
            binding_id.as_str(),
            value,
        ) {
            return Some(result);
        }
        if let Some(result) = self.dispatch_workbench_command_palette_query_edited(
            control_id,
            binding_id.as_str(),
            value,
        ) {
            return Some(result);
        }
        if let Some(result) =
            self.dispatch_workbench_scene_picker_committed(control_id, binding_id.as_str(), value)
        {
            return Some(result);
        }
        if let Some(result) =
            callback_dispatch::dispatch_componentized_workbench_command_palette_committed(
                &self.runtime,
                &self.workbench_window_bridge,
                control_id,
                binding_id.as_str(),
                value,
            )
        {
            return Some(result);
        }
        if let Some(result) =
            callback_dispatch::dispatch_componentized_workbench_transform_axis_commit(
                &self.runtime,
                &self.workbench_window_bridge,
                control_id,
                binding_id.as_str(),
                value,
            )
        {
            return Some(result);
        }
        callback_dispatch::dispatch_componentized_workbench_surface_control_edited(
            &mut self.workbench_window_bridge,
            control_id,
            binding_id.as_str(),
            value,
        )
    }
}
