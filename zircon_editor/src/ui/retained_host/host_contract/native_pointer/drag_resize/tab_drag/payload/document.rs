mod floating;
mod root;

use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

use self::floating::floating_document_tab_drag_payload;
use self::root::root_document_tab_drag_payload;

pub(super) fn document_tab_drag_payload(
    presentation: &HostWindowPresentationData,
    surface_key: &SharedString,
    index: usize,
) -> Option<(TabData, SharedString)> {
    if surface_key.as_str() == "document" {
        return root_document_tab_drag_payload(presentation, index);
    }
    floating_document_tab_drag_payload(presentation, surface_key, index)
}
