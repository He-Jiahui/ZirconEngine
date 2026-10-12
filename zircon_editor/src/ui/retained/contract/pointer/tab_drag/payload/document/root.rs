use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn root_document_tab_drag_payload<'a>(
    presentation: &'a HostWindowPresentationData,
    surface_key: &str,
    index: usize,
) -> Option<(&'a TabData, &'a SharedString)> {
    let dock = presentation
        .host_scene_data
        .document_surfaces()
        .iter()
        .find(|dock| dock.surface_key.as_str() == surface_key)?;
    dock.tabs.get(index).map(|tab| (tab, &dock.surface_key))
}
