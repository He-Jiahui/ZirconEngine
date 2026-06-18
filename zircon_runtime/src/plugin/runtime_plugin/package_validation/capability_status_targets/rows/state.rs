use crate::builtin::RuntimeTargetMode;

pub(super) type RuntimePluginPackageCapabilityStatusTargetRowState = Vec<RuntimeTargetMode>;

pub(super) fn new_runtime_plugin_package_capability_status_target_row_state(
) -> RuntimePluginPackageCapabilityStatusTargetRowState {
    Vec::new()
}
