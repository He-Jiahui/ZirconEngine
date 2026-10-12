use super::{
    execute_post_process_pass_graph, post_process_effect_bit, post_process_effect_for_executor_id,
};
use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessPassGraph, PostProcessPassNode,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use crate::graphics::RenderPassStage;
use crate::render_graph::QueueLane;

#[test]
fn post_process_pass_graph_uses_a_fixed_mask_without_cloning_executor_ids() {
    let source = include_str!("../execute.rs");
    // TODO: [CR-W13-PPRES-0003] 将热路径截取界限限定到生产部分；当前分段标记仅命中测试文本，未验证资源回退前的真实范围。
    let hot_path = source
        .split_once("let produced_resources")
        .expect("pass-graph executor should contain its resource fallback")
        .0;
    let mask_builder = ["fn executed_post_process_effect_", "mask("].concat();
    let bit_check = [
        "executed_effect_mask & post_process_",
        "effect_bit(node.kind)",
    ]
    .concat();

    assert!(!hot_path.contains(".cloned()"));
    assert!(!hot_path.contains(".collect::<BTreeSet"));
    assert!(source.contains(&mask_builder));
    assert!(hot_path.contains(&bit_check));
}

#[test]
fn post_process_executor_ids_map_to_unique_effect_bits() {
    let executor_effects = [
        ("post.blur", PostProcessEffectKind::Blur),
        ("post.bloom", PostProcessEffectKind::Bloom),
        ("post.bloom-extract", PostProcessEffectKind::Bloom),
        ("post.color-lut-bake", PostProcessEffectKind::ColorLutBake),
        ("post.depth-of-field", PostProcessEffectKind::DepthOfField),
        (
            "post.exposure.histogram",
            PostProcessEffectKind::ExposureHistogram,
        ),
        (
            "post.exposure.resolve",
            PostProcessEffectKind::ExposureResolve,
        ),
        ("post.motion-blur", PostProcessEffectKind::MotionBlur),
        (
            "post.scene-composite",
            PostProcessEffectKind::SceneComposite,
        ),
        ("temporal.taa-resolve", PostProcessEffectKind::TaaResolve),
        ("post.uber", PostProcessEffectKind::Uber),
        (
            "post.screen-space-reflection-reflection-pyramid",
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
        ),
        (
            "post.screen-space-reflection-reflection-pyramid-coarse",
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
        ),
        (
            "post.screen-space-reflection-specular-occlusion",
            PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
        ),
        (
            "post.screen-space-reflection-resolve",
            PostProcessEffectKind::ScreenSpaceReflectionResolve,
        ),
        (
            "post.primary-upscale",
            PostProcessEffectKind::PrimaryUpscale,
        ),
        (
            "post.secondary-upscale",
            PostProcessEffectKind::SecondaryUpscale,
        ),
        (
            "post.output-transfer",
            PostProcessEffectKind::OutputTransfer,
        ),
        ("post.fxaa", PostProcessEffectKind::Fxaa),
        ("post.smaa", PostProcessEffectKind::Smaa),
    ];
    let mut seen_effect_bits = 0_u32;

    for (executor_id, expected_effect) in executor_effects {
        let effect = post_process_effect_for_executor_id(executor_id)
            .expect("known post-process executor should map to an effect");
        assert_eq!(effect, expected_effect);
        let bit = post_process_effect_bit(effect);
        if effect != PostProcessEffectKind::Bloom || executor_id == "post.bloom" {
            assert_eq!(seen_effect_bits & bit, 0);
            seen_effect_bits |= bit;
        }
    }
    // BUG: [CR-W13-PPRES-0002] 此表去重后覆盖十九个效果位，但末尾断言期望十八位；该断言会报告数量不符。
    assert_eq!(seen_effect_bits.count_ones(), 18);
    assert_eq!(post_process_effect_for_executor_id("post.unknown"), None);
}

#[test]
fn post_process_pass_graph_executes_nodes_with_buffer_backed_inputs() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-postprocess-buffer-input"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("postprocess.compute-sideband", buffer);
    let graph = PostProcessPassGraph::from_ordered_nodes(
        vec![
            PostProcessPassNode::new("buffer-backed-effect", PostProcessEffectKind::Uber)
                .with_required_inputs(["postprocess.compute-sideband"])
                .with_produced_outputs(["postprocess.effect-stacked"]),
        ],
        Vec::new(),
        None,
    );
    let mut record = RenderGraphExecutionRecord::default();

    execute_post_process_pass_graph(&graph, &resources, &mut record);

    assert_eq!(
        record.executed_post_process_nodes(),
        &["buffer-backed-effect".to_string()]
    );
}

#[test]
fn post_process_pass_graph_records_nodes_from_executed_executor_ids() {
    let resources = RenderGraphExecutionResources::new();
    let graph = PostProcessPassGraph::from_ordered_nodes(
        vec![
            PostProcessPassNode::new(
                "screen-space-reflection-resolve",
                PostProcessEffectKind::ScreenSpaceReflectionResolve,
            )
            .with_required_inputs(["screen-space-reflection-reflection-pyramid-coarse"])
            .with_produced_outputs(["screen-space-reflection-history"]),
            PostProcessPassNode::new("scene-composite", PostProcessEffectKind::SceneComposite)
                .with_required_inputs(["screen-space-reflection-history"])
                .with_produced_outputs(["scene-composited"]),
            PostProcessPassNode::new("uber", PostProcessEffectKind::Uber)
                .with_required_inputs(["scene-composited"])
                .with_produced_outputs(["tonemapped"]),
        ],
        Vec::new(),
        None,
    );
    let mut record = RenderGraphExecutionRecord::default();
    for executor_id in [
        "post.screen-space-reflection-resolve",
        "post.scene-composite",
        "post.uber",
    ] {
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::PostProcess),
            executor_id,
            executor_id,
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );
    }

    execute_post_process_pass_graph(&graph, &resources, &mut record);

    assert_eq!(
        record.executed_post_process_nodes(),
        &[
            "screen-space-reflection-resolve".to_string(),
            "scene-composite".to_string(),
            "uber".to_string(),
        ]
    );
}

#[test]
fn post_process_pass_graph_records_bloom_extract_executor_as_bloom_node() {
    let resources = RenderGraphExecutionResources::new();
    let graph = PostProcessPassGraph::from_ordered_nodes(
        vec![
            PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom)
                .with_required_inputs(["scene-color"])
                .with_produced_outputs(["bloom-texture"]),
        ],
        Vec::new(),
        None,
    );
    let mut record = RenderGraphExecutionRecord::default();
    record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        Some(RenderPassStage::PostProcess),
        "bloom-extract",
        "post.bloom-extract",
        QueueLane::Graphics,
        QueueLane::Graphics,
        Vec::new(),
        Vec::new(),
    );

    execute_post_process_pass_graph(&graph, &resources, &mut record);

    assert_eq!(record.executed_post_process_nodes(), &["bloom".to_string()]);
}
