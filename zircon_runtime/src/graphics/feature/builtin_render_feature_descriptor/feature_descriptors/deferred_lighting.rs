use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "deferred_lighting",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "deferred-lighting",
                QueueLane::Graphics,
            )
            .with_executor_id("lighting.deferred")
            .read_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::clear_store(),
            ),
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

        assert!(
            !pass
                .resources
                .iter()
                .any(|resource| { resource.name == PostProcessGraphResourceNames::FINAL_COLOR })
        );
    }
}
