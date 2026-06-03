pub(super) type RuntimePluginPackageCapabilityRowState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_package_capability_row_state<'a>(
) -> RuntimePluginPackageCapabilityRowState<'a> {
    Vec::new()
}
