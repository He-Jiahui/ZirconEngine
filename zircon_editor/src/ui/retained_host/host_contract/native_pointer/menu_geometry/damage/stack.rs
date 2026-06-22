mod level;

use super::super::super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::primitives::ModelRc;

use self::level::next_menu_popup_stack_level;

pub(super) fn menu_popup_stack_bottom(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
) -> f32 {
    let mut bottom = parent_popup.y + parent_popup.height;
    for (level, selected_index) in menu_state.open_submenu_path.iter().copied().enumerate() {
        let Some(next_level) = next_menu_popup_stack_level(
            presentation,
            menu_state,
            &items,
            &parent_popup,
            selected_index,
            level,
        ) else {
            break;
        };
        bottom = bottom.max(next_level.popup.y + next_level.popup.height);
        items = next_level.items;
        parent_popup = next_level.popup;
    }
    bottom
}
