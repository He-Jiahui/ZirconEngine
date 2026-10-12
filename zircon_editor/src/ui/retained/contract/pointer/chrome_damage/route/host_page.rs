use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::host_page::host_page_tab_damage_frame;

pub(super) fn route_host_page_tab_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    host_page_tab_damage_frame(presentation)
}
