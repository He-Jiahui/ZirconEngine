pub(super) type RuntimePluginFeatureModuleRowState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_feature_module_row_state<'a>(
) -> RuntimePluginFeatureModuleRowState<'a> {
    Vec::new()
}
