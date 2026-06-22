use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

pub(super) fn text_input_focus_redraw(
    focus: &HostTextInputFocusData,
) -> NativePointerDispatchResult {
    let result = NativePointerDispatchResult::region(focus.edit_frame.clone());
    if result.request_redraw() {
        result
    } else {
        NativePointerDispatchResult::full_frame()
    }
}
