pub(super) type RuntimePluginDefaultPackagingStrategyState = u8;

pub(super) const fn new_runtime_plugin_default_packaging_strategy_state(
) -> RuntimePluginDefaultPackagingStrategyState {
    0
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
