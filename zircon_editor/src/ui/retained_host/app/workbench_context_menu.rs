use super::*;
use crate::ui::template_runtime::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn dispatch_workbench_context_menu_requested(
        &mut self,
        request: WorkbenchContextMenuRequestData,
    ) {
        self.focus_callback_source_window();
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return;
        }

        match self.workbench_window_bridge.open_context_menu(&request) {
            Ok(true) => {
                let mut effects = UiHostEventEffects::default();
                effects.request_presentation();
                self.apply_dispatch_effects(effects);
                self.set_status_line(format!(
                    "Context menu opened for {}",
                    request.target_value_text
                ));
            }
            Ok(false) => self.set_status_line("Workbench context menu is not available"),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
