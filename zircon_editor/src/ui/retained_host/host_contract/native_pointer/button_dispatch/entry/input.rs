use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;

use super::super::viewport_button::viewport_button_id;

pub(super) struct ButtonDispatchInput {
    pub(super) presentation: HostWindowPresentationData,
    pub(super) button: UiPointerButton,
    pub(super) button_id: i32,
    pub(super) modifiers: UiInputModifiers,
}

pub(super) fn button_dispatch_input(
    ui: &UiHostWindow,
    button: Option<UiPointerButton>,
    modifiers: UiInputModifiers,
) -> Option<ButtonDispatchInput> {
    let button = button.unwrap_or(UiPointerButton::Primary);
    let presentation = ui.get_host_presentation();
    let button_id = viewport_button_id(button)?;
    Some(ButtonDispatchInput {
        presentation,
        button,
        button_id,
        modifiers,
    })
}
