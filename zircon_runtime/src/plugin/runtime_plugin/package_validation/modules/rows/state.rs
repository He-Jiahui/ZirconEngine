pub(super) type RuntimePluginPackageModuleRowState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_package_module_row_state<'a>(
) -> RuntimePluginPackageModuleRowState<'a> {
    Vec::new()
}
