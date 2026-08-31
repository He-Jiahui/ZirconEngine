use crate::core::framework::render::{
    AntiAliasSettings, AoSourceSettings, CameraRenderDescriptor, PostProcessEffectKind,
    PostProcessExtract, PostProcessGraphResourceNames, PostProcessStackDescriptor,
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderHybridGiPayloadSource, RenderPostProcessEffectStackSettings, RenderTonemapOperator,
    RenderTonemapSettings, RenderUpscalerKind, RenderViewExtract, RenderViewportRect,
    RenderVirtualGeometryPayloadSource, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::{BuiltinRenderFeature, RenderPipelineCompileOptions};

use super::super::super::super::budget::BudgetDegradeSettings;
use super::budget_degrade::compile_options_for_budget_degrade;
use super::effective_view_state::{
    build_renderer_owned_post_process_snapshot, resolve_view_family_pipeline_for_submission,
};
use super::{hybrid_gi_payload_source_for_frame, virtual_geometry_payload_source_for_extract};

#[test]
fn frame_submission_context_build_root_remains_a_declarative_orchestration_owner() {
    let root = include_str!("../build.rs");
    let budget = include_str!("budget_degrade.rs");
    let view_state = include_str!("effective_view_state.rs");
    let ui_stats = include_str!("ui_submission_stats.rs");

    for module in [
        "mod budget_degrade;",
        "mod effective_view_state;",
        "mod tests;",
        "mod ui_submission_stats;",
    ] {
        assert!(root.contains(module), "missing build owner: {module}");
    }
    assert!(root.lines().count() < 600);
    assert!(!root.contains("fn compile_options_for_budget_degrade("));
    assert!(!root.contains("fn frame_history_invalidation_reason("));
    assert!(!root.contains("fn compute_ui_submission_stats("));
    assert!(budget.contains("fn compile_options_for_budget_degrade("));
    assert!(view_state.contains("fn frame_history_invalidation_reason("));
    assert!(ui_stats.contains("fn compute_ui_submission_stats("));
}

#[test]
fn virtual_geometry_payload_source_prefers_authored_extract() {
    let source = virtual_geometry_payload_source_for_extract(true, true, true);

    assert_eq!(source, RenderVirtualGeometryPayloadSource::Authored);
}

#[test]
fn virtual_geometry_payload_source_reports_automatic_fallback() {
    let source = virtual_geometry_payload_source_for_extract(true, false, true);

    assert_eq!(
        source,
        RenderVirtualGeometryPayloadSource::AutomaticFallback
    );
}

#[test]
fn virtual_geometry_payload_source_clears_when_feature_disabled_or_missing() {
    assert_eq!(
        virtual_geometry_payload_source_for_extract(false, true, true),
        RenderVirtualGeometryPayloadSource::None
    );
    assert_eq!(
        virtual_geometry_payload_source_for_extract(true, false, false),
        RenderVirtualGeometryPayloadSource::None
    );
}

#[test]
fn hybrid_gi_payload_source_reports_scene_representation_only_when_enabled() {
    assert_eq!(
        hybrid_gi_payload_source_for_frame(true, true),
        RenderHybridGiPayloadSource::SceneRepresentation
    );
    assert_eq!(
        hybrid_gi_payload_source_for_frame(false, true),
        RenderHybridGiPayloadSource::None
    );
    assert_eq!(
        hybrid_gi_payload_source_for_frame(true, false),
        RenderHybridGiPayloadSource::None
    );
}

#[test]
fn bloom_budget_degrade_synchronizes_the_feature_gate_and_post_process_stack() {
    let rebuilt_stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
            &RenderBloomSettings {
                intensity: 0.6,
                ..Default::default()
            },
            &RenderColorGradingSettings::default(),
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
    let settings = BudgetDegradeSettings {
        disable_bloom_high: true,
        ..Default::default()
    };
    let initial_options =
        compile_options_for_budget_degrade(RenderPipelineCompileOptions::default(), settings);
    let options = compile_options_for_budget_degrade(
        initial_options.with_post_process_stack(rebuilt_stack),
        settings,
    );

    assert!(options
        .disabled_features
        .contains(&BuiltinRenderFeature::Bloom));
    let stack = options
        .post_process_stack
        .expect("budget degradation should preserve the remaining post-process stack");
    assert!(stack
        .effects
        .iter()
        .any(|effect| { effect.kind == PostProcessEffectKind::Bloom && !effect.enabled }));
    assert!(stack.effects.iter().all(|effect| {
        !effect
            .required_inputs
            .iter()
            .any(|resource| resource == PostProcessGraphResourceNames::BLOOM)
            && !effect.after.contains(&PostProcessEffectKind::Bloom)
    }));
}

#[test]
fn context_build_moves_owned_viewport_and_virtual_geometry_payloads() {
    let source = format!(
        "{}\n{}",
        include_str!("../build.rs"),
        include_str!("effective_view_state.rs")
    );

    assert!(source.contains("take_previous_particle_sprites()"));
    assert!(source.contains("VirtualGeometryRuntimeExtractOutput::into_parts"));
    assert!(!source.contains(concat!("previous_particle_sprites()", ".to_vec()")));
    assert!(!source.contains(concat!("virtual_geometry_runtime_provider", ".clone()")));
}

#[test]
fn post_process_resolution_builds_renderer_owned_snapshot_without_extract_mutation() {
    let source = include_str!("../build.rs");
    let view_state = include_str!("effective_view_state.rs");

    assert!(source.contains("build_renderer_owned_post_process_snapshot("));
    assert!(source.contains(".with_ambient_occlusion_source(effective_ambient_occlusion)"));
    assert!(!source.contains("effective_extract.post_process."));
    assert!(!source.contains("extract.post_process.volumes.clear()"));
    assert!(view_state.contains("volumes: Vec::new(),"));
}

#[test]
fn renderer_owned_post_process_snapshot_preserves_resolved_exposure_and_source() {
    let source = PostProcessExtract::default();
    let source_before = source.clone();
    let bloom = RenderBloomSettings {
        intensity: 0.42,
        ..RenderBloomSettings::default()
    };
    let exposure = RenderExposureSettings::manual_ev100(7.5);
    let stack = source.stack.clone();
    let graph = source.graph.clone();

    let snapshot = build_renderer_owned_post_process_snapshot(
        &source,
        AoSourceSettings::default(),
        bloom,
        exposure,
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
        stack.clone(),
        graph.clone(),
    );

    assert_eq!(source, source_before);
    assert_eq!(snapshot.bloom, bloom);
    assert_eq!(snapshot.exposure, exposure);
    assert!(snapshot.volumes.is_empty());
    assert_eq!(snapshot.stack, stack);
    assert_eq!(snapshot.graph, graph);
}

#[test]
fn resolved_view_family_is_installed_before_final_graph_compilation() {
    let source = include_str!("../build.rs");
    let install = source
        .find(".apply_view_family_pipeline(view_family_pipeline);")
        .expect("submission must install the resolved view family on the extract");
    let final_compile = source[install..]
        .find("let compiled_pipeline = compile_submission_pipeline_with_options(")
        .map(|offset| install + offset)
        .expect("submission must compile the final graph after installing ViewFamily");
    let context = source[final_compile..]
        .find("Ok(FrameSubmissionContext::new(")
        .map(|offset| final_compile + offset)
        .expect("submission must construct a frame context after compilation");

    assert!(install < final_compile);
    assert!(final_compile < context);
    assert!(source[context..].contains("view_family_pipeline,"));
}

#[test]
fn submission_view_family_preserves_the_selected_camera_viewport_rect() {
    let camera = ViewportCameraSnapshot::default();
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, camera.clone());
    descriptor.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(960, 0),
        UVec2::new(960, 1080),
    ));
    let view = RenderViewExtract::from_camera(camera).with_selected_camera_descriptor(descriptor);

    let pipeline = resolve_view_family_pipeline_for_submission(
        &view,
        UVec2::new(1920, 1080),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        pipeline.resolution().display_viewport(),
        RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080))
    );
    assert_eq!(
        pipeline.resolution().primary_viewport(),
        RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080))
    );
}

#[test]
fn submission_context_advanced_plan_consumption_is_fallible() {
    let owner = include_str!("../viewport_record_state.rs");
    let build = include_str!("../build.rs");

    assert!(owner.contains("Result<AdvancedProfileRuntimePlan, RenderFrameworkError>"));
    assert!(owner.contains("RenderFrameworkError::InvalidSubmissionState"));
    assert!(!owner.contains("advanced runtime plan is moved into one submission context"));
    assert!(build.contains("viewport_state.take_advanced_runtime_plan()?"));
}
