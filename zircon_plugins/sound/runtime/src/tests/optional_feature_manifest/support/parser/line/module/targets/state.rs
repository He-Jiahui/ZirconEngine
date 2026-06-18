pub(super) fn set_module_target_modes(
    target_modes: &mut Vec<zircon_runtime::builtin::RuntimeTargetMode>,
    modes: Vec<zircon_runtime::builtin::RuntimeTargetMode>,
) {
    *target_modes = modes;
}
