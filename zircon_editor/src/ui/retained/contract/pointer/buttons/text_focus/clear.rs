use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::NativePointerButtonState;

pub(in super::super) fn clear_focused_text_input_on_primary_press(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    button: UiPointerButton,
) -> Option<FrameRect> {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return None;
    }
    let host = ui.global::<UiHostContext>();
    let focus = host.get_text_input_focus();
    if !focus.is_active() {
        return None;
    }
    let frame = focus.edit_frame.clone();
    host.clear_text_input_focus();
    Some(frame)
}
