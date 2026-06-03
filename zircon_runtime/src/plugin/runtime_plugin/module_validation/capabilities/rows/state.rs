pub(super) type RuntimePluginModuleCapabilityRowState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_module_capability_row_state<'a>(
) -> RuntimePluginModuleCapabilityRowState<'a> {
    Vec::new()
}
