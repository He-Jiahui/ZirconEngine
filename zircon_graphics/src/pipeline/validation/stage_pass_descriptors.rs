use crate::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
use crate::pipeline::declarations::RenderPassStage;

pub(in crate::pipeline) fn stage_pass_descriptors(
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
