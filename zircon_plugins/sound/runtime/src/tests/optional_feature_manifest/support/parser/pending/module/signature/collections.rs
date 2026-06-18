pub(super) fn take_target_modes(
    target_modes: &mut Vec<zircon_runtime::builtin::RuntimeTargetMode>,
) -> Vec<zircon_runtime::builtin::RuntimeTargetMode> {
    std::mem::take(target_modes)
}

pub(super) fn take_capabilities(capabilities: &mut Vec<String>) -> Vec<String> {
    std::mem::take(capabilities)
}
