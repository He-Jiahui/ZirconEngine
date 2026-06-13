use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{
    CLUSTERED_LIGHTING_PIPELINE_LABEL, CLUSTERED_LIGHTING_WORKGROUP_SIZE,
};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "clustered_lighting",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "light-grid-build",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("lighting.light-grid")
        .with_compute_workload(RenderGraphComputeWorkload::cluster_grid(
            CLUSTERED_LIGHTING_PIPELINE_LABEL,
            CLUSTERED_LIGHTING_WORKGROUP_SIZE,
        ))
        .write_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
        .write_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
        .write_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
        .write_buffer(PostProcessGraphResourceNames::LIGHT_LIST)],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind,
    };
    use super::*;

    #[test]
    fn clustered_lighting_declares_light_grid_build_outputs() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "light-grid-build")
            .expect("light grid build pass");

        assert_eq!(pass.executor_id.as_str(), "lighting.light-grid");
        assert_eq!(pass.stage, RenderPassStage::Lighting);
        assert_eq!(pass.queue, QueueLane::AsyncCompute);
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::LIGHT_GRID_PARAMS
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Write
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::LIGHT_ZBINS
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Write
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::LIGHT_TILE_MASKS
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Write
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::LIGHT_LIST
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Write
        }));
    }
}
