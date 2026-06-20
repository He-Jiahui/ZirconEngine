use super::super::super::*;
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_componentized_workbench_option_selected(
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
}
