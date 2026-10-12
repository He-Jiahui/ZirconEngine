use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::routing::contains;
use super::super::NativePointerButtonState;

pub(in crate::ui::retained_host::host_contract) fn asset_deletion_blocker_action_at(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    let blocker = &presentation.asset_deletion_blocker;
    blocker.visible && contains(&blocker.close_button_frame, x, y)
}

pub(super) fn dispatch_asset_deletion_blocker_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let blocker = &presentation.asset_deletion_blocker;
    if !blocker.visible {
        return None;
    }
    if state == NativePointerButtonState::Pressed
        && button == UiPointerButton::Primary
        && asset_deletion_blocker_action_at(presentation, x, y)
    {
        ui.global::<UiHostContext>()
            .invoke_asset_deletion_blocker_closed();
        return Some(NativePointerDispatchResult::region_with_frame_update(
            blocker.overlay_frame.clone(),
        ));
    }
    if contains(&blocker.overlay_frame, x, y) {
        return Some(NativePointerDispatchResult::idle());
    }
    None
}
