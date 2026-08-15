use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_document_tab_pointer_layout::HostDocumentTabPointerLayout;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

#[derive(Default)]
pub(crate) struct HostDocumentTabPointerBridge {
    pub(in crate::ui::retained_host::document_tab_pointer) layout: HostDocumentTabPointerLayout,
    pub(in crate::ui::retained_host::document_tab_pointer) measured_frames:
        std::collections::BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(in crate::ui::retained_host::document_tab_pointer) surface: UiSurface,
    pub(in crate::ui::retained_host::document_tab_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::retained_host::document_tab_pointer) route_intents: EditorRouteIntentMap,
    #[cfg(test)]
    pub(in crate::ui::retained_host::document_tab_pointer) surface_authority_generation: u64,
}

impl HostDocumentTabPointerBridge {
    #[cfg(test)]
    pub(crate) const fn debug_surface_authority_generation(&self) -> u64 {
        self.surface_authority_generation
    }
}
