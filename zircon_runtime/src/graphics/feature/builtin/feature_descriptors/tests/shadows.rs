use super::super::super::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use super::*;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphExternalResourceBinding};

#[test]
fn shadow_atlas_pass_declares_required_external_texture_write() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "shadow-atlas")
        .expect("shadow atlas pass");

    let atlas = pass
        .resources
        .iter()
        .find(|resource| resource.name == PostProcessGraphResourceNames::SHADOW_ATLAS)
        .expect("shadow atlas resource");

    assert_eq!(atlas.kind, RenderFeatureResourceKind::External);
    assert_eq!(atlas.access, RenderFeatureResourceAccess::Write);
    assert_eq!(
        atlas.attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert_eq!(
        atlas.external_binding,
        RenderGraphExternalResourceBinding::required_texture()
    );
    assert!(!pass.flags.has_side_effects);
}
