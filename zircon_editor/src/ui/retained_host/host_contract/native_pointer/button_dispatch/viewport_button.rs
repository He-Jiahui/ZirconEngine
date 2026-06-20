use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::{
    VIEWPORT_POINTER_BUTTON_MIDDLE, VIEWPORT_POINTER_BUTTON_PRIMARY,
    VIEWPORT_POINTER_BUTTON_SECONDARY,
};

pub(in crate::ui::retained_host::host_contract) fn viewport_button_id(
    button: UiPointerButton,
) -> Option<i32> {
    match button {
        UiPointerButton::Primary => Some(VIEWPORT_POINTER_BUTTON_PRIMARY),
        UiPointerButton::Secondary => Some(VIEWPORT_POINTER_BUTTON_SECONDARY),
        UiPointerButton::Middle => Some(VIEWPORT_POINTER_BUTTON_MIDDLE),
    }
}
