use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::clustered_lighting_dispatch_plan;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    let light_grid_dispatch = clustered_lighting_dispatch_plan();

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
        .with_compute_dispatch_plan(light_grid_dispatch)],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind,
    };
    use super::*;
    use crate::core::framework::render::PostProcessGraphResourceNames;

    #[test]
    fn clustered_lighting_reuses_the_static_dispatch_contract() {
        assert!(std::ptr::eq(
            clustered_lighting_dispatch_plan(),
            clustered_lighting_dispatch_plan()
        ));
    }

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
        assert_eq!(
            pass.compute_workload.as_ref().unwrap().pipeline_label,
            "zircon-cluster-pipeline"
        );
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
