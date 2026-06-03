pub(super) type RuntimePluginFeatureCapabilityRowState<'a> = Vec<&'a str>;

pub(super) fn new_runtime_plugin_feature_capability_row_state<'a>(
) -> RuntimePluginFeatureCapabilityRowState<'a> {
    Vec::new()
}
