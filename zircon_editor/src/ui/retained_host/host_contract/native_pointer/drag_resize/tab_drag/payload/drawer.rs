mod bottom;
mod left;
mod right;

use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

use self::bottom::bottom_drawer_tab_drag_payload;
use self::left::left_drawer_tab_drag_payload;
use self::right::right_drawer_tab_drag_payload;

pub(super) fn drawer_tab_drag_payload<'a>(
    presentation: &'a HostWindowPresentationData,
    surface_key: &SharedString,
    index: usize,
) -> Option<(&'a TabData, &'a SharedString)> {
    match surface_key.as_str() {
        "left" => left_drawer_tab_drag_payload(presentation, index),
        "right" => right_drawer_tab_drag_payload(presentation, index),
        "bottom" => bottom_drawer_tab_drag_payload(presentation, index),
        _ => None,
    }
}
