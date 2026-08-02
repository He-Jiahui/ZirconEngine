use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::NativePointerButtonState;
use super::super::super::pane_button_damage::pane_pointer_press_damage_frame;
use super::super::super::routing::PanePointerRoute;

pub(super) fn dispatch_passive_pane_button(
    state: NativePointerButtonState,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if state != NativePointerButtonState::Pressed {
        return None;
    }
    if let Some(frame) = cleared_text_input_frame {
        return Some(NativePointerDispatchResult::region(frame));
    }
    Some(NativePointerDispatchResult::idle())
}

pub(super) fn pane_button_fallback_damage(
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if state == NativePointerButtonState::Released {
        return NativePointerDispatchResult::region(pointer.frame.clone());
    }
    match pane_pointer_press_damage_frame(presentation, &pointer.frame, cleared_text_input_frame) {
        Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
        None => NativePointerDispatchResult::full_frame(),
    }
}
