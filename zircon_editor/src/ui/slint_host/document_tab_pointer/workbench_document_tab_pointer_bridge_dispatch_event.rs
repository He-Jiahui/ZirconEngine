use zircon_ui::UiPointerEvent;

use super::{
    workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge,
    workbench_document_tab_pointer_target::WorkbenchDocumentTabPointerTarget,
};

impl WorkbenchDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchDocumentTabPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
