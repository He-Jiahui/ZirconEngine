mod frame;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};

use self::frame::next_level_popup_frame;

pub(super) struct MenuPopupStackLevel {
    pub(super) popup: FrameRect,
    pub(super) items: ModelRc<HostMenuChromeItemData>,
}

pub(super) fn next_menu_popup_stack_level(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    items: &ModelRc<HostMenuChromeItemData>,
    parent_popup: &FrameRect,
    selected_index: usize,
    level: usize,
) -> Option<MenuPopupStackLevel> {
    let branch = items.row_data(selected_index)?;
    if branch.children.row_count() == 0 {
        return None;
    }
    let popup = next_level_popup_frame(
        presentation,
        menu_state,
        parent_popup,
        selected_index,
        branch.children.row_count(),
        level,
    );
    Some(MenuPopupStackLevel {
        popup,
        items: branch.children.clone(),
    })
}
