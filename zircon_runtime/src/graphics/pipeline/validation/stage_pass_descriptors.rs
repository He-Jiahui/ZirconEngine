use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
use crate::graphics::pipeline::declarations::RenderPassStage;

/// 按 renderer 的 feature 顺序收集指定阶段的 pass，保留原顺序供后续 authoring 排序。
pub(in crate::graphics::pipeline) fn stage_pass_descriptors(
    stage: RenderPassStage,
    descriptors: &[RenderFeatureDescriptor],
) -> Vec<RenderFeaturePassDescriptor> {
    let pass_capacity = descriptors
        .iter()
        .map(|descriptor| descriptor.stage_passes.len())
        .sum();
    let mut passes = Vec::with_capacity(pass_capacity);
    passes.extend(
        descriptors
            .iter()
            .flat_map(|descriptor| descriptor.stage_passes.iter())
            .filter(|descriptor| descriptor.stage == stage)
            .cloned(),
    );
    passes
}

#[cfg(test)]
#[path = "tests/stage_pass_descriptors_optimization_tests.rs"]
mod optimization_tests;
