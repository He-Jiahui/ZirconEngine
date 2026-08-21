use super::*;
use crate::core::framework::render::{
    PostProcessEffectKind, RenderBloomSettings, RenderTonemapOperator, RenderTonemapSettings,
};

#[test]
fn forward_plus_pipeline_compilation_is_deterministic() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let extract = test_extract();

    let first = pipeline.compile(&extract).unwrap();
    let second = pipeline.compile(&extract).unwrap();

    assert_eq!(first, second);
}

#[test]
fn builtin_pipeline_lookup_exposes_deferred_pipeline_handle() {
    let builtin =
        RenderPipelineAsset::builtin(crate::core::framework::render::RenderPipelineHandle::new(2))
            .expect("handle 2 should map to the built-in deferred pipeline");

    assert_eq!(builtin, RenderPipelineAsset::default_deferred());
}

#[test]
fn history_binding_accessors_construct_expected_bindings() {
    assert_eq!(
        FrameHistoryBinding::read(FrameHistorySlot::AmbientOcclusion),
        FrameHistoryBinding {
            slot: FrameHistorySlot::AmbientOcclusion,
            access: FrameHistoryAccess::Read,
        }
    );
    assert_eq!(
        FrameHistoryBinding::write(FrameHistorySlot::TaaSceneColor),
        FrameHistoryBinding {
            slot: FrameHistorySlot::TaaSceneColor,
            access: FrameHistoryAccess::Write,
        }
    );
}

#[test]
fn compile_options_can_disable_clustered_history_and_rendering_plugin_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::Temporal)
        .with_plugin_feature_disabled("screen_space_ambient_occlusion");

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let pass_names = compiled
        .graph()
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"light-grid-build"));
    assert!(!pass_names.contains(&"taa-resolve"));
    assert!(!compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion
        )));
}

#[test]
fn compile_options_can_disable_bloom_before_a_post_process_stack_is_available() {
    let options =
        RenderPipelineCompileOptions::default().with_feature_disabled(BuiltinRenderFeature::Bloom);

    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(&test_extract(), &options)
        .expect("the bootstrap graph must not retain inputs from disabled features");
    assert!(!compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("the bootstrap graph should retain the uber pass");
    assert!(!uber.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::BLOOM
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
}

#[test]
fn compile_options_can_disable_bloom_without_leaving_an_unproduced_uber_input() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings {
            intensity: 0.6,
            ..Default::default()
        },
        &extract.post_process.color_grading,
        &RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Aces,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let options = RenderPipelineCompileOptions::default()
        .with_post_process_stack(stack)
        .with_feature_disabled(BuiltinRenderFeature::Bloom)
        .with_post_process_effect_disabled(PostProcessEffectKind::Bloom);

    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(&extract, &options)
        .expect("disabling Bloom must also remove its post-process resource dependencies");
    assert!(!compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
    let uber = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "uber")
        .expect("tonemapping should retain the uber pass");
    assert!(!uber.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::BLOOM
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
}

#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled
            .graph()
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-occlusion-cull"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-build"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "light-grid-build"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    let light_zbins_output = pass_resource_access(
        &compiled,
        "light-grid-build",
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        RenderGraphResourceAccessKind::Write,
    );
    let light_tile_masks_output = pass_resource_access(
        &compiled,
        "light-grid-build",
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(
        light_zbins_output.kind,
        RenderGraphResourceKind::TransientBuffer
    );
    assert_eq!(
        light_tile_masks_output.kind,
        RenderGraphResourceKind::TransientBuffer
    );
    let ssao_output = pass_resource_access(
        &compiled,
        "ssao-evaluate",
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        RenderGraphResourceAccessKind::Write,
    );
    assert_eq!(ssao_output.kind, RenderGraphResourceKind::External);
    assert_eq!(
        ssao_output.attachment_ops, None,
        "compute storage writes must not inherit render attachment load/store ops"
    );
    assert_eq!(compiled.graph().stats().queue_fallback_pass_count, 4);
}

#[test]
fn compile_options_gate_hzb_occlusion_cull_without_removing_hzb_build() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default().with_hzb_occlusion_culling(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert!(!compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-occlusion-cull"));
    assert!(compiled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "hzb-build"));
    assert_eq!(compiled.pass_stage("hzb-occlusion-cull"), None);
    assert_eq!(
        compiled.pass_stage("hzb-build"),
        Some(RenderPassStage::AmbientOcclusion)
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::SCENE_DEPTH,
        RenderGraphResourceAccessKind::Read,
    );
    pass_resource_access(
        &compiled,
        "hzb-build",
        PostProcessGraphResourceNames::HZB_FURTHEST,
        RenderGraphResourceAccessKind::Write,
    );
}
