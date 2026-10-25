use super::super::super::*;
use crate::ui::settings::{
    SETTINGS_COMMIT_CHORD_ACTION_ID, SETTINGS_COMMIT_NUMERIC_ACTION_ID,
    SETTINGS_COMMIT_STRING_ACTION_ID,
};
use crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_componentized_workbench_surface_control_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        match self.active_activity_window_template_document_is(UNREAL_EDITOR_WINDOW_DOCUMENT_ID) {
            Ok(true) => {}
            Ok(false) => return None,
            Err(error) => return Some(Err(error.to_string())),
        }
        if binding_id == SETTINGS_COMMIT_STRING_ACTION_ID {
            let result = (|| {
                let context = self
                    .workbench_window_bridge
                    .settings_mutation_context(control_id)?;
                let receipt = self
                    .runtime
                    .set_string_setting(&context, control_id, value)
                    .map_err(|error| error.to_string())?;
                if receipt.changed() {
                    self.refresh_after_settings_receipt(false)
                } else {
                    Ok(UiHostEventEffects::default())
                }
            })();
            return Some(result);
        }
        if binding_id == SETTINGS_COMMIT_NUMERIC_ACTION_ID {
            let result = (|| {
                let context = self
                    .workbench_window_bridge
                    .settings_mutation_context(control_id)?;
                let receipt = self
                    .runtime
                    .set_numeric_setting(&context, control_id, value)
                    .map_err(|error| error.to_string())?;
                if receipt.changed() {
                    self.refresh_after_settings_receipt(false)
                } else {
                    Ok(UiHostEventEffects::default())
                }
            })();
            return Some(result);
        }
        if binding_id == SETTINGS_COMMIT_CHORD_ACTION_ID {
            let result = (|| {
                let context = self
                    .workbench_window_bridge
                    .settings_mutation_context(control_id)?;
                let receipt = self
                    .runtime
                    .set_chord_setting(&context, control_id, value)
                    .map_err(|error| error.to_string())?;
                if receipt.changed() {
                    self.refresh_after_settings_receipt(false)
                } else {
                    Ok(UiHostEventEffects::default())
                }
            })();
            return Some(result);
        }
        let binding_id = self
            .workbench_window_bridge
            .binding_id_for_action_id(binding_id)
            .unwrap_or_else(|| binding_id.to_string());
        match self.workbench_window_bridge.content_drawer_search_binding(
            control_id,
            binding_id.as_str(),
            value,
        ) {
            Ok(Some(binding)) => {
                return Some(callback_dispatch::dispatch_template_binding_with_arguments(
                    &self.runtime,
                    binding,
                    vec![UiBindingValue::string(value)],
                ))
            }
            Ok(None) => {}
            Err(error) => return Some(Err(error.to_string())),
        }
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
        if let Some(result) =
            callback_dispatch::dispatch_componentized_workbench_render_layer_mask_commit(
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
