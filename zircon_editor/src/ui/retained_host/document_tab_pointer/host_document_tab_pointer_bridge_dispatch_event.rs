use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::{
    host_document_tab_pointer_bridge::HostDocumentTabPointerBridge,
    host_document_tab_pointer_route::HostDocumentTabPointerRoute,
};

impl HostDocumentTabPointerBridge {
    pub(in crate::ui::retained_host::document_tab_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostDocumentTabPointerRoute>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .document_tab_route_for_pointer_dispatch(&dispatch))
    }
}
