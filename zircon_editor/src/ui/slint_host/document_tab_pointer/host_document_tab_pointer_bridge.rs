use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, event_ui::UiNodeId, layout::UiFrame, surface::UiSurface,
};

use super::{
    host_document_tab_pointer_layout::HostDocumentTabPointerLayout,
    host_document_tab_pointer_target::HostDocumentTabPointerTarget,
};

#[derive(Default)]
pub(crate) struct HostDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) layout: HostDocumentTabPointerLayout,
    pub(in crate::ui::slint_host::document_tab_pointer) measured_frames:
        BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(in crate::ui::slint_host::document_tab_pointer) surface: UiSurface,
    pub(in crate::ui::slint_host::document_tab_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::slint_host::document_tab_pointer) targets:
        BTreeMap<UiNodeId, HostDocumentTabPointerTarget>,
}
