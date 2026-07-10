use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "deferred_geometry",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "preview-sky",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-final-color")
            .write_external_texture_with_ops(
                PostProcessGraphResourceNames::FINAL_COLOR,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Deferred,
                "gbuffer-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.gbuffer")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "transparent-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind,
    };
    use super::*;
    use crate::render_graph::RenderGraphExternalResourceBinding;

    #[test]
    fn deferred_transparent_mesh_requires_shadow_atlas_external_texture() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "transparent-mesh")
            .expect("transparent mesh pass");
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
    fn deferred_geometry_writes_hdr_emissive_gbuffer_resource() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "gbuffer-mesh")
            .expect("gbuffer mesh pass");

        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::GBUFFER_EMISSIVE
                && resource.access == RenderFeatureResourceAccess::Write
        }));
    }
}
