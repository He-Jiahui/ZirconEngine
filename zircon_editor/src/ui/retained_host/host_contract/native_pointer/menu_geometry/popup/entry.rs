mod blocking;
mod root;

use self::blocking::popup_blocking_region_handles_point;
use self::root::opened_root_menu_popup_with_state;
use super::super::super::super::data::{HostMenuStateData, HostWindowPresentationData};
use super::super::super::routing::contains;
use super::nested::nested_menu_popup_handles_point;

pub(in crate::ui::retained_host::host_contract) fn menu_popup_handles_point_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    x: f32,
    y: f32,
) -> bool {
    let Some(root_popup) = opened_root_menu_popup_with_state(presentation, menu_state) else {
        return false;
    };
    contains(&root_popup.frame, x, y)
        || nested_menu_popup_handles_point(
            presentation,
            menu_state,
            root_popup.items,
            root_popup.frame,
            x,
            y,
        )
        || popup_blocking_region_handles_point(presentation, x, y)
}
