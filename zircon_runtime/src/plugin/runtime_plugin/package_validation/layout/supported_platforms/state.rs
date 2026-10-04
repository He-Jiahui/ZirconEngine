pub(super) type RuntimePluginPackageSupportedPlatformState = u8;

pub(super) const fn new_runtime_plugin_package_supported_platform_state(
) -> RuntimePluginPackageSupportedPlatformState {
    0
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
