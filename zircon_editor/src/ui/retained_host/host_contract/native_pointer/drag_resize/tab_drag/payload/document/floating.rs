use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn floating_document_tab_drag_payload(
    presentation: &HostWindowPresentationData,
    surface_key: &SharedString,
    index: usize,
) -> Option<(TabData, SharedString)> {
    for row in 0..presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .row_count()
    {
        let window = presentation
            .host_scene_data
            .floating_layer
            .floating_windows
            .row_data(row)?;
        if window.window_id.as_str() == surface_key.as_str() {
            return window
                .tabs
                .row_data(index)
                .map(|tab| (tab, window.target_group.clone()));
        }
    }
    None
}
