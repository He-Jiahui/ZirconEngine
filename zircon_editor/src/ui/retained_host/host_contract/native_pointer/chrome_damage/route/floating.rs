use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::floating::floating_window_header_damage_frame;

pub(super) fn route_floating_window_header_damage_frame(
    presentation: &HostWindowPresentationData,
    window_id: &str,
) -> Option<FrameRect> {
    floating_window_header_damage_frame(presentation, window_id)
}
