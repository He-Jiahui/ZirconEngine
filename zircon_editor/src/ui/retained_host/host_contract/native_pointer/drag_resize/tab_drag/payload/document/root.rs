use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn root_document_tab_drag_payload(
    presentation: &HostWindowPresentationData,
    index: usize,
) -> Option<(&TabData, &SharedString)> {
    presentation
        .host_scene_data
        .document_dock
        .tabs
        .get(index)
        .map(|tab| (tab, &presentation.host_scene_data.document_dock.surface_key))
}
