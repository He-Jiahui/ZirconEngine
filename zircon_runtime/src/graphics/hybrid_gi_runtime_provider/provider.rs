use std::fmt::Debug;

use super::HybridGiRuntimeState;

/// 混合 GI 扩展入口；注册对象可共享，渲染框架为每个视口相机历史创建独立状态。
/// 启用该质量特性前必须有可选中的 provider，否则帧准备返回能力错误。
pub trait HybridGiRuntimeProvider: Debug + Send + Sync {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState>;
}
