use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::super::super::viewport_toolbar_damage::viewport_toolbar_press_damage_frame;

pub(super) fn viewport_toolbar_click_damage_result(
    presentation: &HostWindowPresentationData,
    control_id: &str,
    pointer_frame: &FrameRect,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    match viewport_toolbar_press_damage_frame(
        presentation,
        control_id,
        pointer_frame,
        cleared_text_input_frame,
    ) {
        Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
        None => NativePointerDispatchResult::full_frame(),
    }
}
