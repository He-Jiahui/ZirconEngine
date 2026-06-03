use crate::plugin::ExportPackagingStrategy;

pub(super) type RuntimePluginDefaultPackagingStrategyState = Vec<ExportPackagingStrategy>;

pub(super) fn new_runtime_plugin_default_packaging_strategy_state(
) -> RuntimePluginDefaultPackagingStrategyState {
    Vec::new()
}
