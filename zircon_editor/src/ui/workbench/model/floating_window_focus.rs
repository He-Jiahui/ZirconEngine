use crate::ui::workbench::view::ViewInstanceId;

use super::document_tab_model::DocumentTabModel;
use super::floating_window_model::FloatingWindowModel;

impl FloatingWindowModel {
    pub(crate) fn focus_target_tab(&self) -> Option<&DocumentTabModel> {
        let focused_view = self.focused_view.as_ref();
        let mut active_tab = None;
        for tab in &self.tabs {
            if focused_view == Some(&tab.instance_id) {
                return Some(tab);
            }
            if active_tab.is_none() && tab.active {
                active_tab = Some(tab);
            }
        }
        active_tab.or_else(|| self.tabs.first())
    }

    pub(crate) fn focus_target_instance(&self) -> Option<&ViewInstanceId> {
        self.focus_target_tab().map(|tab| &tab.instance_id)
    }
}

#[cfg(test)]
#[path = "tests/floating_window_focus_optimization_batch_20260830cl_editor_tests.rs"]
mod optimization_batch_20260830cl_editor_tests;
