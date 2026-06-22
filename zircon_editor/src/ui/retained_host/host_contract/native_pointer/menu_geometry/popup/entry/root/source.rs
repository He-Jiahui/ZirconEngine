use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::data::{FrameRect, HostMenuChromeItemData};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) struct RootMenuPopupSource {
    pub(super) menu_frame: FrameRect,
    pub(super) popup_width_px: f32,
    pub(super) popup_height_px: f32,
    pub(super) items: ModelRc<HostMenuChromeItemData>,
}

pub(super) fn opened_root_menu_popup_source(
    presentation: &HostWindowPresentationData,
) -> Option<RootMenuPopupSource> {
    let state = &presentation.menu_state;
    if state.open_menu_index < 0 {
        return None;
    }
    let menu_index = state.open_menu_index as usize;
    let menu_frame = presentation
        .host_scene_data
        .menu_chrome
        .menu_frames
        .row_data(menu_index)?;
    let menu = presentation
        .host_scene_data
        .menu_chrome
        .menus
        .row_data(menu_index)?;
    Some(RootMenuPopupSource {
        menu_frame: menu_frame.frame.clone(),
        popup_width_px: menu.popup_width_px,
        popup_height_px: menu.popup_height_px,
        items: menu.items.clone(),
    })
}
