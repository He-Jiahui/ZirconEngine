use crate::ui::retained_host::host_contract::data::{
    HostDragStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::tab_drag_damage::tab_drag_release_damage_frame;

pub(in crate::ui::retained_host::host_contract) fn tab_drag_release_redraw(
    presentation: &HostWindowPresentationData,
    drag_state: &HostDragStateData,
) -> NativePointerDispatchResult {
    match tab_drag_release_damage_frame(presentation, drag_state) {
        Some(frame) => NativePointerDispatchResult::region_with_frame_update(frame),
        None => NativePointerDispatchResult::full_frame(),
    }
}
