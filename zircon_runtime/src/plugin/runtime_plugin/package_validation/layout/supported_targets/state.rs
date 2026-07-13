use crate::core::framework::platform::RuntimeTargetMode;

pub(super) type RuntimePluginPackageSupportedTargetState = Vec<RuntimeTargetMode>;

pub(super) fn new_runtime_plugin_package_supported_target_state(
) -> RuntimePluginPackageSupportedTargetState {
    Vec::new()
}
