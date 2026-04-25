use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::render_graph::{QueueLane, RenderGraphResourceKind};
use crate::scene::world::World;

use crate::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
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
fn flagship_feature_descriptors_declare_backend_capability_requirements() {
    assert_eq!(
        BuiltinRenderFeature::VirtualGeometry
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
    );
    assert_eq!(
        BuiltinRenderFeature::GlobalIllumination
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::HybridGlobalIllumination]
    );
    assert_eq!(
        BuiltinRenderFeature::RayTracing
            .descriptor()
            .capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::AccelerationStructures,
            RenderFeatureCapabilityRequirement::RayTracingPipeline,
        ]
    );
}

#[test]
fn compiled_pipeline_collects_enabled_feature_capability_requirements() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default()
        .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
        .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled.capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::VirtualGeometry,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ]
    );
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
fn feature_pass_descriptors_drive_executor_ids_and_resource_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    let depth_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "depth-prepass")
        .expect("default forward pipeline should include depth prepass");
    assert_eq!(
        depth_pass.executor_id.as_deref(),
        Some("mesh.depth-prepass")
    );

    let opaque_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque-mesh")
        .expect("default forward pipeline should include opaque mesh pass");
    assert_eq!(opaque_pass.executor_id.as_deref(), Some("mesh.opaque"));

    let lifetimes = compiled.graph.resource_lifetimes();
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-depth" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-color" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let disabled = pipeline.compile(&test_extract()).unwrap();
    let disabled_pass_names = disabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    assert!(!disabled_pass_names.contains(&"hybrid-gi-resolve"));
    assert!(!disabled_pass_names.contains(&"virtual-geometry-node-cluster-cull"));

    let options = RenderPipelineCompileOptions::default()
        .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
        .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry);
    let enabled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let enabled_pass_names = enabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    for pass_name in [
        "hybrid-gi-scene-prepare",
        "hybrid-gi-trace-schedule",
        "hybrid-gi-resolve",
        "hybrid-gi-history",
        "virtual-geometry-prepare",
        "virtual-geometry-node-cluster-cull",
        "virtual-geometry-page-feedback",
        "virtual-geometry-visbuffer",
        "virtual-geometry-debug-overlay",
    ] {
        assert!(
            enabled_pass_names.contains(&pass_name),
            "enabled graph should contain {pass_name}"
        );
    }
    assert!(enabled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination
        )));
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
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh));

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

#[test]
fn renderer_feature_asset_quality_gate_controls_compiled_passes() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom")
        .quality_gate = Some(BuiltinRenderFeature::VirtualGeometry);

    let without_gate = pipeline.compile(&test_extract()).unwrap();
    assert!(!without_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));

    let with_gate = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry),
        )
        .unwrap();
    assert!(with_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
}

#[test]
fn pipeline_compile_validates_quality_gated_descriptor_overrides_even_when_gate_is_closed() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::GBuffer,
                "bad-gated-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("bad-gated-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn renderer_feature_asset_local_config_and_capabilities_survive_compile() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let color_grading = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::ColorGrading)
        .expect("default pipeline should include color grading");
    *color_grading = color_grading
        .clone()
        .with_local_config("variant", "cinematic")
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline);

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let compiled_color_grading = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature == BuiltinRenderFeature::ColorGrading)
        .expect("color grading should remain enabled");

    assert_eq!(
        compiled_color_grading.local_config.get("variant"),
        Some(&"cinematic".to_string())
    );
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::RayTracingPipeline));
}

#[test]
fn renderer_feature_asset_descriptor_override_changes_compiled_graph() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "custom-bloom",
            vec!["custom_post".to_string()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "custom-bloom-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture("scene-color")
            .write_external("viewport-output")],
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"bloom-extract"));
    assert!(pass_names.contains(&"custom-bloom-pass"));
    assert!(compiled
        .required_extract_sections
        .contains(&"custom_post".to_string()));
    assert!(compiled.graph.resource_lifetimes().iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn pipeline_compile_rejects_descriptor_passes_for_undeclared_stages() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-stage-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::GBuffer,
                "custom-gbuffer-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("custom-gbuffer-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_descriptor_pass_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "post-process",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate render graph pass name `post-process`"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_conflicting_descriptor_resource_kinds() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_buffer("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("conflicting resource kind"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_explicit_external_resource_name_conflicts() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-external-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-external-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_external("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("explicit external resource"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("feature descriptor name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_names_after_descriptor_name_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("pass name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("executor id must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_resource_names_after_executor_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.feature == BuiltinRenderFeature::Bloom)
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource name must not be empty"),
        "unexpected error: {error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
