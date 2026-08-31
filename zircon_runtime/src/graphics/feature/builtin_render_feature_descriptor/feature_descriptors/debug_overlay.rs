use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::final_output_resource_schema;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "debug_overlay",
        vec!["view".to_string(), "debug".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Debug,
                "overlay-gizmo",
                QueueLane::Graphics,
            )
            .with_executor_id("overlay.gizmo")
            .read_texture("scene-color")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_present_external_texture_with_schema(
                PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
                final_output_resource_schema(),
            ),
        ],
    )
}
