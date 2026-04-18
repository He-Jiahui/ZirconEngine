use zircon_render_graph::QueueLane;
use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

use crate::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
};

#[test]
fn default_forward_plus_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque,
            RenderPassStage::Transparent,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "ssao-evaluate",
            "clustered-light-culling",
            "opaque-mesh",
            "transparent-mesh",
            "particle-render",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "post-process",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "particles".to_string(),
            "post_process".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![
            FrameHistoryBinding::read_write(FrameHistorySlot::AmbientOcclusion),
            FrameHistoryBinding::read_write(FrameHistorySlot::SceneColor),
        ]
    );
}

#[test]
fn default_deferred_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_deferred();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::GBuffer,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Transparent,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "gbuffer-mesh",
            "ssao-evaluate",
            "clustered-light-culling",
            "deferred-lighting",
            "transparent-mesh",
            "particle-render",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "post-process",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "particles".to_string(),
            "post_process".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![
            FrameHistoryBinding::read_write(FrameHistorySlot::AmbientOcclusion),
            FrameHistoryBinding::read_write(FrameHistorySlot::SceneColor),
        ]
    );
}

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
        RenderPipelineAsset::builtin(zircon_framework::render::RenderPipelineHandle::new(2))
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
        FrameHistoryBinding::write(FrameHistorySlot::SceneColor),
        FrameHistoryBinding {
            slot: FrameHistorySlot::SceneColor,
            access: FrameHistoryAccess::Write,
        }
    );
}

#[test]
fn compile_options_can_disable_clustered_ssao_and_history_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::HistoryResolve);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"clustered-light-culling"));
    assert!(!pass_names.contains(&"history-resolve"));
    assert!(compiled.history_bindings.is_empty());
}

#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate" && pass.queue == QueueLane::Graphics));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "clustered-light-culling" && pass.queue == QueueLane::Graphics));
}

#[test]
fn pipeline_compile_rejects_duplicate_stage_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline.renderer.stages.push(RenderPassStage::Opaque);

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_feature_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline.renderer.features.push(RendererFeatureAsset {
        feature: BuiltinRenderFeature::Mesh,
        enabled: true,
    });

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature"),
        "unexpected error: {error}"
    );
}

#[test]
fn disabling_post_process_keeps_overlay_extract_requirements_for_debug_gizmos() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::PostProcess)
        .expect("default pipeline should include post-process")
        .enabled = false;

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"post-process"),
        "post-process pass should be removed when the feature is disabled"
    );
    assert!(
        pass_names.contains(&"overlay-gizmo"),
        "overlay stage should remain available for debug and gizmo rendering"
    );
    assert!(
        compiled
            .required_extract_sections
            .contains(&"debug".to_string()),
        "overlay feature should keep requiring debug extract data"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
