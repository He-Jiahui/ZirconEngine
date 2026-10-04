pub(super) type RuntimePluginPackageSupportedTargetState = u8;

pub(super) const fn new_runtime_plugin_package_supported_target_state(
) -> RuntimePluginPackageSupportedTargetState {
    0
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
