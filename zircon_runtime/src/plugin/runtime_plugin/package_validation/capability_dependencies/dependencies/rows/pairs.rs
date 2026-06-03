pub(super) type RuntimePluginPackageDependencyPairState<'a> = Vec<(&'a str, &'a str)>;

pub(super) fn new_runtime_plugin_package_dependency_pair_state<'a>(
) -> RuntimePluginPackageDependencyPairState<'a> {
    Vec::new()
}
