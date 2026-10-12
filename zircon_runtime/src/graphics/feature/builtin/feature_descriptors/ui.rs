use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::final_output_resource_schema;

// UI 读取 final color、写入带物理 schema 的 viewport output；surface 或纹理终点由 terminal authoring 另行选择。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "ui",
        vec!["view".to_string(), "ui".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Ui,
            "runtime-ui",
            QueueLane::Graphics,
        )
        .with_executor_id("ui.screen-space")
        .read_external_texture(PostProcessGraphResourceNames::FINAL_COLOR)
        .write_present_external_texture_with_schema(
            PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
            final_output_resource_schema(),
        )],
    )
}
