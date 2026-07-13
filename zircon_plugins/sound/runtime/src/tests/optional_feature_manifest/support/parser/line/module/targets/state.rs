pub(super) fn set_module_target_modes(
    target_modes: &mut Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
    modes: Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
) {
    *target_modes = modes;
}
