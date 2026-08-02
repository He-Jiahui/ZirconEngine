use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

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
                RenderPassStage::Transparent3d,
                "preview-sky",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-scene-color")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
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
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::clear_store(),
            ),
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
}
