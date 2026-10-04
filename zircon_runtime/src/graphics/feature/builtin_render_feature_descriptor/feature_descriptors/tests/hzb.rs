use super::super::super::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use super::*;
use crate::render_graph::{
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};

#[test]
fn hzb_occlusion_cull_declares_execution_owned_external_buffers() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hzb-occlusion-cull")
        .expect("hzb occlusion cull pass");

    assert!(pass.resources.iter().any(|resource| {
        resource.name == HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_buffer()
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_buffer()
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding
                == RenderGraphExternalResourceBinding::report_only_texture()
            && resource.usage.persistent
            && resource.schema.is_none()
            && resource.access_metadata
                == Some(RenderGraphResourceAccessMetadata::new(
                    RenderGraphResourceAccessRange::Texture(
                        RenderGraphTextureSubresourceRange::full(),
                    ),
                    RenderGraphResourceAccessIntent::sampled_texture(
                        RenderGraphShaderStages::COMPUTE,
                    ),
                ))
    }));
    for name in [
        HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
        HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
        HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
        HZB_OCCLUSION_STATS_RESOURCE,
    ] {
        assert!(pass.resources.iter().any(|resource| {
            resource.name == name
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Write
                && resource.write_mode == RenderFeatureResourceWriteMode::Storage
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::required_buffer()
        }));
    }
}

#[test]
fn hzb_build_retains_the_actual_cross_frame_history_copy_source() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hzb-build")
        .expect("hzb build pass");

    assert!(!pass.flags.has_side_effects);
    assert!(pass.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::HZB_FURTHEST
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.write_mode == RenderFeatureResourceWriteMode::Storage
            && resource.usage.persistent
    }));
}
