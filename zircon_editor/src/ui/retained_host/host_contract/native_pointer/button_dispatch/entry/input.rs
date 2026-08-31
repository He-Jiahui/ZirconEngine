use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

pub(super) struct ButtonDispatchInput {
    pub(super) presentation: HostPresentationGeneration,
    pub(super) button: UiPointerButton,
    pub(super) button_id: i32,
    pub(super) modifiers: UiInputModifiers,
}

pub(super) fn button_dispatch_input(
    ui: &UiHostWindow,
    button: UiPointerButton,
    button_id: i32,
    modifiers: UiInputModifiers,
) -> ButtonDispatchInput {
    let presentation = ui.get_host_presentation_generation();
    ButtonDispatchInput {
        presentation,
        button,
        button_id,
        modifiers,
    }
}
