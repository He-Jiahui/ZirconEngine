use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "mesh",
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
            .with_executor_id("sky.preview-scene-color")
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque3d,
                "opaque-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.opaque")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AlphaMask3d,
                "alpha-mask-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.alpha-mask")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "transparent-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
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
}
