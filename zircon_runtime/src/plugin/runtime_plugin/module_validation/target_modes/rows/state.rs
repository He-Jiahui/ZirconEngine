use crate::builtin::RuntimeTargetMode;

pub(super) type RuntimePluginModuleTargetModeRowState = Vec<RuntimeTargetMode>;

pub(super) fn new_runtime_plugin_module_target_mode_row_state(
) -> RuntimePluginModuleTargetModeRowState {
    Vec::new()
}
