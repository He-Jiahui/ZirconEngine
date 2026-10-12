use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::resize_damage::resize_damage_frame;

pub(in crate::ui::retained_host::host_contract) fn resize_pointer_redraw(
    presentation: &HostWindowPresentationData,
    extra_damage: Option<FrameRect>,
) -> NativePointerDispatchResult {
    match union_optional_frames(resize_damage_frame(presentation), extra_damage) {
        Some(frame) => NativePointerDispatchResult::region_with_frame_update(frame),
        None => NativePointerDispatchResult::full_frame(),
    }
}
