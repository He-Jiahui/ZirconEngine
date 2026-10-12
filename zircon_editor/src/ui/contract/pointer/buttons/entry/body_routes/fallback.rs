use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

pub(super) fn cleared_text_input_fallback_result(
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if let Some(frame) = cleared_text_input_frame {
        return NativePointerDispatchResult::region(frame);
    }
    NativePointerDispatchResult::idle()
}
