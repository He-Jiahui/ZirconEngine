pub(super) type RuntimePluginPackageRootArrayState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_package_root_array_state<'a>(
) -> RuntimePluginPackageRootArrayState<'a> {
    Vec::new()
}
