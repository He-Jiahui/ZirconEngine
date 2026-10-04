/// 一次模块目标遍历的临时判重状态；不是持久化目标掩码或项目支持目标集合。
pub(super) type RuntimePluginModuleTargetModeRowState = u8;

/// 为每个模块重新建立未访问状态，防止相邻模块共享目标时被误判重复。
pub(super) const fn new_runtime_plugin_module_target_mode_row_state(
) -> RuntimePluginModuleTargetModeRowState {
    0
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
