use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, event_ui::UiNodeId, layout::UiFrame, surface::UiSurface,
};

use super::{
    workbench_document_tab_pointer_layout::WorkbenchDocumentTabPointerLayout,
    workbench_document_tab_pointer_target::WorkbenchDocumentTabPointerTarget,
};

#[derive(Default)]
pub(crate) struct WorkbenchDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) layout: WorkbenchDocumentTabPointerLayout,
    pub(in crate::ui::slint_host::document_tab_pointer) measured_frames:
        BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(in crate::ui::slint_host::document_tab_pointer) surface: UiSurface,
    pub(in crate::ui::slint_host::document_tab_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::slint_host::document_tab_pointer) targets:
        BTreeMap<UiNodeId, WorkbenchDocumentTabPointerTarget>,
}
