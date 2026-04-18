use crate::view::ViewInstanceId;

use super::document_tab_model::DocumentTabModel;
use super::floating_window_model::FloatingWindowModel;

impl FloatingWindowModel {
    pub(crate) fn focus_target_tab(&self) -> Option<&DocumentTabModel> {
        self.focused_view
            .as_ref()
            .and_then(|focused_view| {
                self.tabs
                    .iter()
                    .find(|tab| &tab.instance_id == focused_view)
            })
            .or_else(|| self.tabs.iter().find(|tab| tab.active))
            .or_else(|| self.tabs.first())
    }

    pub(crate) fn focus_target_instance(&self) -> Option<&ViewInstanceId> {
        self.focus_target_tab().map(|tab| &tab.instance_id)
    }
}
