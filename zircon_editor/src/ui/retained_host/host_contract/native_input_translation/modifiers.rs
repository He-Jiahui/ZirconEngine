use winit::keyboard::ModifiersState;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;

pub(in crate::ui::retained_host::host_contract) fn native_modifiers_to_shared(
    modifiers: ModifiersState,
) -> UiInputModifiers {
    UiInputModifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        super_key: modifiers.meta_key(),
        caps_lock: false,
        num_lock: false,
    }
}
