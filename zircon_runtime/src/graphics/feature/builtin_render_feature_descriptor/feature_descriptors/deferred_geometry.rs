use crate::render_graph::QueueLane;

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
                "depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.depth-prepass")
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::GBuffer,
                "gbuffer-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.gbuffer")
            .read_texture("scene-depth")
            .write_texture("gbuffer-albedo")
            .write_texture("gbuffer-normal")
            .write_texture("gbuffer-material"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "transparent-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.transparent")
            .read_texture("scene-depth")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
}
