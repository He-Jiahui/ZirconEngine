mod root;

use self::root::opened_root_menu_popup_bottom;
use super::super::super::super::data::{HostMenuStateData, HostWindowPresentationData};

pub(super) fn open_menu_popup_bottom(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    base_height: f32,
) -> f32 {
    if menu_state.open_menu_index < 0 {
        return base_height;
    }

    opened_root_menu_popup_bottom(presentation, menu_state).unwrap_or(base_height)
}
