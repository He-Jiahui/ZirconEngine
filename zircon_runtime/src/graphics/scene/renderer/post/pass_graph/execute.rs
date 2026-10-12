use crate::core::framework::render::{PostProcessEffectKind, PostProcessPassGraph};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use std::collections::HashSet;

/// 将已执行的 GPU executor 汇总为后处理节点记录；没有执行记录时按已绑定资源推导可用节点。
/// 此入口只更新执行报告，节点按图的既有顺序登记，GPU 命令由各效果执行入口录制。
pub(crate) fn execute_post_process_pass_graph(
    graph: &PostProcessPassGraph,
    resources: &RenderGraphExecutionResources,
    record: &mut RenderGraphExecutionRecord,
) {
    if !record.executed_executor_ids().is_empty() {
        let executed_effect_mask =
            executed_post_process_effect_mask(record.executed_executor_ids());
        for node in &graph.nodes {
            if executed_effect_mask & post_process_effect_bit(node.kind) != 0 {
                record.push_executed_post_process_node(node.name.clone());
            }
        }
        return;
    }

    let produced_resource_count = graph
        .nodes
        .iter()
        .map(|node| node.produced_outputs.len())
        .sum();
    let resource_reference_count = graph.nodes.iter().fold(0, |count, node| {
        count + node.required_inputs.len() + node.produced_outputs.len()
    });
    let mut produced_resources: HashSet<&str> = HashSet::with_capacity(produced_resource_count);
    for node in &graph.nodes {
        produced_resources.extend(node.produced_outputs.iter().map(String::as_str));
    }
    // 图内产出的输入须等待前置节点登记，只有已绑定的图外输入可以作为起始可用资源。
    let mut available_resources: HashSet<&str> = HashSet::with_capacity(resource_reference_count);
    for node in &graph.nodes {
        available_resources.extend(
            node.required_inputs
                .iter()
                .map(String::as_str)
                .filter(|resource| !produced_resources.contains(resource))
                .filter(|resource| resources.has_bound_resource(resource)),
        );
    }

    for node in &graph.nodes {
        if !node
            .required_inputs
            .iter()
            .all(|resource| available_resources.contains(resource.as_str()))
        {
            continue;
        }
        record.push_executed_post_process_node(node.name.clone());
        available_resources.extend(
            node.produced_outputs
                .iter()
                .map(String::as_str)
                .filter(|resource| resources.has_bound_resource(resource)),
        );
    }
}

fn executed_post_process_effect_mask(executor_ids: &[String]) -> u32 {
    executor_ids.iter().fold(0_u32, |mask, executor_id| {
        mask | post_process_effect_for_executor_id(executor_id)
            .map(post_process_effect_bit)
            .unwrap_or(0)
    })
}

fn post_process_effect_for_executor_id(executor_id: &str) -> Option<PostProcessEffectKind> {
    match executor_id {
        "post.blur" => Some(PostProcessEffectKind::Blur),
        "post.bloom" | "post.bloom-extract" => Some(PostProcessEffectKind::Bloom),
        "post.color-lut-bake" => Some(PostProcessEffectKind::ColorLutBake),
        "post.depth-of-field" => Some(PostProcessEffectKind::DepthOfField),
        "post.exposure.histogram" => Some(PostProcessEffectKind::ExposureHistogram),
        "post.exposure.resolve" => Some(PostProcessEffectKind::ExposureResolve),
        "post.motion-blur" => Some(PostProcessEffectKind::MotionBlur),
        "post.scene-composite" => Some(PostProcessEffectKind::SceneComposite),
        "temporal.taa-resolve" => Some(PostProcessEffectKind::TaaResolve),
        "post.uber" => Some(PostProcessEffectKind::Uber),
        "post.screen-space-reflection-reflection-pyramid" => {
            Some(PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid)
        }
        "post.screen-space-reflection-reflection-pyramid-coarse" => {
            Some(PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse)
        }
        "post.screen-space-reflection-specular-occlusion" => {
            Some(PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion)
        }
        "post.screen-space-reflection-resolve" => {
            Some(PostProcessEffectKind::ScreenSpaceReflectionResolve)
        }
        "post.primary-upscale" => Some(PostProcessEffectKind::PrimaryUpscale),
        "post.secondary-upscale" => Some(PostProcessEffectKind::SecondaryUpscale),
        "post.output-transfer" => Some(PostProcessEffectKind::OutputTransfer),
        "post.fxaa" => Some(PostProcessEffectKind::Fxaa),
        "post.smaa" => Some(PostProcessEffectKind::Smaa),
        _ => None,
    }
}

const fn post_process_effect_bit(kind: PostProcessEffectKind) -> u32 {
    match kind {
        PostProcessEffectKind::Blur => 1 << 0,
        PostProcessEffectKind::Bloom => 1 << 1,
        PostProcessEffectKind::ColorLutBake => 1 << 2,
        PostProcessEffectKind::DepthOfField => 1 << 3,
        PostProcessEffectKind::ExposureHistogram => 1 << 4,
        PostProcessEffectKind::ExposureResolve => 1 << 5,
        PostProcessEffectKind::MotionBlur => 1 << 6,
        PostProcessEffectKind::SceneComposite => 1 << 7,
        PostProcessEffectKind::TaaResolve => 1 << 8,
        PostProcessEffectKind::Uber => 1 << 9,
        PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid => 1 << 10,
        PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse => 1 << 11,
        PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion => 1 << 12,
        PostProcessEffectKind::ScreenSpaceReflectionResolve => 1 << 13,
        PostProcessEffectKind::PrimaryUpscale => 1 << 14,
        PostProcessEffectKind::SecondaryUpscale => 1 << 15,
        PostProcessEffectKind::OutputTransfer => 1 << 16,
        PostProcessEffectKind::Fxaa => 1 << 17,
        PostProcessEffectKind::Smaa => 1 << 18,
    }
}

#[cfg(test)]
#[path = "tests/execute.rs"]
mod tests;
