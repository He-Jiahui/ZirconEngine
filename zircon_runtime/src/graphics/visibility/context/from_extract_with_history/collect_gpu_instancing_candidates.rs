use crate::core::framework::scene::Mobility;

use super::super::super::declarations::VisibilityBatch;

// TODO: [CR-GRAPHICS-SHADER-VIS-0004] 确认 GPU instancing 候选的实际消费入口；当前只见写入 VisibilityContext 和测试读取。
// 从主视图可见批次标识多实例动态组，保留与普通绘制批次相同的身份和顺序。
pub(super) fn collect_gpu_instancing_candidates(
    visible_batches: &[VisibilityBatch],
) -> Vec<VisibilityBatch> {
    let mut candidates = Vec::with_capacity(visible_batches.len());
    candidates.extend(
        visible_batches
            .iter()
            .filter(|batch| {
                batch.key.mobility == Mobility::Dynamic && batch.stable_instance_keys.len() > 1
            })
            .cloned(),
    );
    candidates
}

#[cfg(test)]
#[path = "tests/collect_gpu_instancing_candidates_optimization_tests.rs"]
mod optimization_tests;
