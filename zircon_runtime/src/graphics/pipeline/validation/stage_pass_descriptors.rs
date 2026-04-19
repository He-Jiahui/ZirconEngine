use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
use crate::graphics::pipeline::declarations::RenderPassStage;

pub(in crate::graphics::pipeline) fn stage_pass_descriptors(
    stage: RenderPassStage,
    descriptors: &[RenderFeatureDescriptor],
) -> Vec<RenderFeaturePassDescriptor> {
    descriptors
        .iter()
        .flat_map(|descriptor| descriptor.stage_passes.iter())
        .filter(|descriptor| descriptor.stage == stage)
        .cloned()
        .collect()
}
