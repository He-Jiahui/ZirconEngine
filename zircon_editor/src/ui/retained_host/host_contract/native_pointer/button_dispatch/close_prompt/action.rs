use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::NativePointerButtonState;
use super::super::super::close_prompt_damage::close_prompt_action_damage_frame;

pub(super) fn dispatch_close_prompt_action_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    action_id: SharedString,
    state: NativePointerButtonState,
    button: UiPointerButton,
) -> NativePointerDispatchResult {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return NativePointerDispatchResult::idle();
    }
    ui.global::<UiHostContext>()
        .invoke_close_prompt_action_clicked(action_id);
    match close_prompt_action_damage_frame(presentation) {
        Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
        None => NativePointerDispatchResult::full_frame(),
    }
}
