/// 仅保存当前状态行已声明的目标；位映射由唯一性校验的穷尽匹配定义。
pub(super) type RuntimePluginPackageCapabilityStatusTargetRowState = u8;

pub(super) const fn new_runtime_plugin_package_capability_status_target_row_state(
) -> RuntimePluginPackageCapabilityStatusTargetRowState {
    0
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
