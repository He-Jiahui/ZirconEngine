use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn floating_document_tab_drag_payload<'a>(
    presentation: &'a HostWindowPresentationData,
    surface_key: &SharedString,
    index: usize,
) -> Option<(&'a TabData, &'a SharedString)> {
    for window in presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
    {
        if window.window_id.as_str() == surface_key.as_str() {
            return window
                .tabs
                .get(index)
                .map(|tab| (tab, &window.target_group));
        }
    }
    None
}
