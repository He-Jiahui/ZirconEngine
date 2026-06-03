use crate::plugin::ExportTargetPlatform;

pub(super) type RuntimePluginPackageSupportedPlatformState = Vec<ExportTargetPlatform>;

pub(super) fn new_runtime_plugin_package_supported_platform_state(
) -> RuntimePluginPackageSupportedPlatformState {
    Vec::new()
}
