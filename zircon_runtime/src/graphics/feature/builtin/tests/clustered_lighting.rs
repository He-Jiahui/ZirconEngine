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
