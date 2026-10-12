use super::super::super::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use super::*;
use crate::render_graph::RenderGraphExternalResourceBinding;

#[test]
fn preview_sky_runs_before_transparent_mesh_and_only_reads_scene_depth() {
    let descriptor = descriptor();
    let sky_index = descriptor
        .stage_passes
        .iter()
        .position(|pass| pass.pass_name == "preview-sky")
        .expect("preview sky pass");
    let transparent_index = descriptor
        .stage_passes
        .iter()
        .position(|pass| pass.pass_name == "transparent-mesh")
        .expect("transparent mesh pass");
    let sky = &descriptor.stage_passes[sky_index];

    assert_eq!(sky.stage, RenderPassStage::Transparent3d);
    assert!(sky_index < transparent_index);
    assert!(sky.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
            && resource.access == RenderFeatureResourceAccess::Read
    }));
    assert!(!sky.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
            && resource.access == RenderFeatureResourceAccess::Write
    }));
}

#[test]
fn mesh_shadow_receivers_require_shadow_atlas_external_texture() {
    let descriptor = descriptor();

    for pass_name in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == pass_name)
            .unwrap_or_else(|| panic!("{pass_name} pass"));
        let atlas = pass
            .resources
            .iter()
            .find(|resource| resource.name == PostProcessGraphResourceNames::SHADOW_ATLAS)
            .unwrap_or_else(|| panic!("{pass_name} shadow atlas resource"));

        assert_eq!(atlas.kind, RenderFeatureResourceKind::External);
        assert_eq!(atlas.access, RenderFeatureResourceAccess::Read);
        assert_eq!(
            atlas.external_binding,
            RenderGraphExternalResourceBinding::required_texture()
        );
    }
}
