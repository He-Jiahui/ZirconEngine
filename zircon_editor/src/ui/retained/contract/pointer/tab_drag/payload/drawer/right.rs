use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn right_drawer_tab_drag_payload(
    presentation: &HostWindowPresentationData,
    index: usize,
) -> Option<(&TabData, &SharedString)> {
    presentation
        .host_scene_data
        .right_dock
        .tabs
        .get(index)
        .map(|tab| (tab, &presentation.host_scene_data.right_dock.surface_key))
}
