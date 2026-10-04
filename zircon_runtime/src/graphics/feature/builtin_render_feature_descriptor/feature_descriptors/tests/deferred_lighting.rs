use super::super::super::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use super::*;
use crate::render_graph::RenderGraphExternalResourceBinding;

#[test]
fn deferred_lighting_requires_shadow_atlas_external_texture() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "deferred-lighting")
        .expect("deferred lighting pass");
    let atlas = pass
        .resources
        .iter()
        .find(|resource| resource.name == PostProcessGraphResourceNames::SHADOW_ATLAS)
        .expect("shadow atlas resource");

    assert_eq!(atlas.kind, RenderFeatureResourceKind::External);
    assert_eq!(atlas.access, RenderFeatureResourceAccess::Read);
    assert_eq!(
        atlas.external_binding,
        RenderGraphExternalResourceBinding::required_texture()
    );
}

#[test]
fn deferred_lighting_reads_hdr_emissive_gbuffer_resource() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "deferred-lighting")
        .expect("deferred lighting pass");

    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::GBUFFER_EMISSIVE
            && resource.access == RenderFeatureResourceAccess::Read
    }));
}

#[test]
fn deferred_lighting_does_not_depend_on_pre_rendered_final_color_background() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "deferred-lighting")
        .expect("deferred lighting pass");

    assert!(!pass
        .resources
        .iter()
        .any(|resource| { resource.name == PostProcessGraphResourceNames::FINAL_COLOR }));
}
