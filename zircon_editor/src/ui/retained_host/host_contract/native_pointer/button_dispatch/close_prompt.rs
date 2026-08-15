mod action;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use self::action::dispatch_close_prompt_action_press;
use super::super::routing::contains;
use super::super::NativePointerButtonState;
use super::close_prompt_hit::close_prompt_action_at;

pub(super) fn dispatch_close_prompt_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    if let Some(action_id) = close_prompt_action_at(presentation, x, y) {
        return Some(dispatch_close_prompt_action_press(
            ui,
            presentation,
            action_id,
            state,
            button,
        ));
    }
    if presentation.close_prompt.visible && contains(&presentation.close_prompt.overlay_frame, x, y)
    {
        return Some(NativePointerDispatchResult::idle());
    }
    None
}
