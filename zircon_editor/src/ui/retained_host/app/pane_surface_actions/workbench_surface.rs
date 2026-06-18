use super::super::*;
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn dispatch_componentized_workbench_surface_control(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return None;
        }
        let workbench_binding_id = (!action_id.is_empty())
            .then(|| {
                self.workbench_window_bridge
                    .binding_id_for_action_id(action_id)
            })
            .flatten();
        let has_workbench_binding = workbench_binding_id.is_some();
        let has_workbench_control = self.workbench_window_bridge.has_control(control_id);
        if !has_workbench_binding && !has_workbench_control {
            return None;
        }
        if let Some(result) = callback_dispatch::dispatch_componentized_workbench_popup_cancelled(
            &mut self.workbench_window_bridge,
            control_id,
            action_id,
        ) {
            return Some(result);
        }
        if !has_workbench_binding && !action_id.is_empty() {
            if let Some(result) =
                callback_dispatch::dispatch_componentized_workbench_menu_item_selected(
                    &self.runtime,
                    &mut self.workbench_window_bridge,
                    control_id,
                    action_id,
                )
            {
                return Some(result);
            }
        }
        let result = if has_workbench_binding {
            callback_dispatch::dispatch_componentized_workbench_binding(
                &self.runtime,
                &mut self.workbench_window_bridge,
                control_id,
                workbench_binding_id.as_deref().unwrap_or(action_id),
            )
        } else {
            callback_dispatch::dispatch_componentized_workbench_control(
                &self.runtime,
                &mut self.workbench_window_bridge,
                control_id,
                UiEventKind::Click,
            )
        };
        result.or_else(|| {
            Some(Err(format!(
                "Unknown componentized workbench control {control_id}"
            )))
        })
    }

    pub(super) fn dispatch_componentized_workbench_option_selected(
        &mut self,
        control_id: &str,
        _action_id: &str,
        option_id: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return None;
        }
        if !self.workbench_window_bridge.has_control(control_id) {
            return None;
        }
        Some(
            callback_dispatch::dispatch_componentized_workbench_option_selected(
                &self.runtime,
                &mut self.workbench_window_bridge,
                control_id,
                option_id,
            ),
        )
    }

    pub(super) fn dispatch_componentized_workbench_surface_control_edited(
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
        callback_dispatch::dispatch_componentized_workbench_surface_control_edited(
            &mut self.workbench_window_bridge,
            control_id,
            binding_id.as_str(),
            value,
        )
    }
}
