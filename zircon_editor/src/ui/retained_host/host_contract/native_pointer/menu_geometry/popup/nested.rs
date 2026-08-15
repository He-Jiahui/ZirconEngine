mod hit;
mod level;

use crate::ui::retained_host::primitives::ModelRc;

use self::level::nested_menu_level_hit;
use super::super::super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};

pub(super) fn nested_menu_popup_handles_point(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
    x: f32,
    y: f32,
) -> bool {
    for (level, selected_index) in menu_state.open_submenu_path.iter().copied().enumerate() {
        let Some(level_hit) = nested_menu_level_hit(
            presentation,
            menu_state,
            &items,
            &parent_popup,
            selected_index,
            level,
            x,
            y,
        ) else {
            return false;
        };
        if level_hit.contains_point {
            return true;
        }
        items = level_hit.items;
        parent_popup = level_hit.popup;
    }
    false
}
