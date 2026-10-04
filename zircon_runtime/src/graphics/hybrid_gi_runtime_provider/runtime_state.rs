use super::HybridGiRuntimeUpdate;
use super::{HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput};

/// 单个相机历史的混合 GI 状态；提交前消费场景与可见性计划，成功渲染后才吸收反馈并公布统计。
/// 实现需跨帧保留该相机的驻留与历史信息，不应把不同相机的反馈混用。
pub trait HybridGiRuntimeState: Send + Sync {
    fn prepare_frame(
        &mut self,
        input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput;

    fn update_after_render(&mut self, feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate;
}
