mod frame;
mod source;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::primitives::ModelRc;

use self::frame::root_menu_popup_frame_with_state;
use self::source::opened_root_menu_popup_source_with_state;

pub(super) struct RootMenuPopup {
    pub(super) frame: FrameRect,
    pub(super) items: ModelRc<HostMenuChromeItemData>,
}

pub(super) fn opened_root_menu_popup_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> Option<RootMenuPopup> {
    let source = opened_root_menu_popup_source_with_state(presentation, menu_state)?;
    let frame = root_menu_popup_frame_with_state(presentation, menu_state, &source);
    Some(RootMenuPopup {
        frame,
        items: source.items,
    })
}
