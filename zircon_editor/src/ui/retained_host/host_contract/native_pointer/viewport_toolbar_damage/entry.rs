use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::status::center_band_status_damage_frame;
use super::target::viewport_toolbar_click_affects_viewport_or_status;
use super::union::{union_optional_frames, visible_frame};

pub(in crate::ui::retained_host::host_contract) fn viewport_toolbar_press_damage_frame(
    presentation: &HostWindowPresentationData,
    control_id: &str,
    toolbar_frame: &FrameRect,
    extra_damage: Option<FrameRect>,
) -> Option<FrameRect> {
    let base_damage = if viewport_toolbar_click_affects_viewport_or_status(control_id) {
        center_band_status_damage_frame(presentation)
    } else {
        visible_frame(toolbar_frame).then_some(toolbar_frame.clone())
    };
    union_optional_frames(base_damage, extra_damage)
}
