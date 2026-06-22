mod frame;
mod source;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostMenuChromeItemData, HostWindowPresentationData,
};
use crate::ui::retained_host::primitives::ModelRc;

use self::frame::root_menu_popup_frame;
use self::source::opened_root_menu_popup_source;

pub(super) struct RootMenuPopup {
    pub(super) frame: FrameRect,
    pub(super) items: ModelRc<HostMenuChromeItemData>,
}

pub(super) fn opened_root_menu_popup(
    presentation: &HostWindowPresentationData,
) -> Option<RootMenuPopup> {
    let source = opened_root_menu_popup_source(presentation)?;
    let frame = root_menu_popup_frame(presentation, &source);
    Some(RootMenuPopup {
        frame,
        items: source.items,
    })
}
