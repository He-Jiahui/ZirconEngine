use crate::core::framework::render::{
    AdvancedProviderAvailability, AdvancedRenderFeature, AntiAliasFallbackReason,
    RenderCapabilitySummary, RenderFrameExtract, RenderPipelinePhase, RenderProfileBundle,
    RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
    RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::{CompiledRenderPipeline, RenderPassStage};
use crate::render_graph::RenderGraphBuilder;
use crate::scene::world::World;

use super::*;

#[test]
fn disabled_temporal_history_does_not_clear_hybrid_gi_cache_every_frame() {
    assert!(!hgi_history_invalidation_active(
        false,
        Some(FrameHistoryInvalidationReason::NoPreviousFrame),
    ));
    assert!(hgi_history_invalidation_active(
        true,
        Some(FrameHistoryInvalidationReason::RenderSizeChanged),
    ));
    assert!(!hgi_history_invalidation_active(true, None));
}

#[test]
fn advanced_runtime_plan_gates_provider_missing_feature_payloads() {
    let context = context_with_advanced_plan(AdvancedProfileRuntimePlan::from_profile_bundle(
        &RenderProfileBundle::advanced_render(),
        &advanced_capabilities(),
        &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
    ));

    assert!(context.hybrid_gi_enabled());
    assert!(!context.virtual_geometry_enabled());

    let virtual_geometry = context
        .advanced_provider_reports()
        .iter()
        .find(|report| report.feature == AdvancedRenderFeature::VirtualGeometry)
        .expect("virtual geometry report");
    assert_eq!(
        virtual_geometry.degradation_reason_labels(),
        vec!["provider-missing"]
    );
}

#[test]
fn advanced_runtime_plan_keeps_provider_backed_features_enabled() {
    let context = context_with_advanced_plan(AdvancedProfileRuntimePlan::from_profile_bundle(
        &RenderProfileBundle::advanced_render(),
        &advanced_capabilities(),
        &AdvancedProviderAvailability::new()
            .with_virtual_geometry_provider("vg")
            .with_hybrid_gi_provider("hgi"),
    ));

    assert!(context.hybrid_gi_enabled());
    assert!(context.virtual_geometry_enabled());
    assert!(context
        .advanced_provider_reports()
        .iter()
        .all(|report| report.degradations.is_empty()));
}

#[test]
fn frame_submission_context_exposes_view_visibility_by_key() {
    let context = context_with_advanced_plan(AdvancedProfileRuntimePlan::from_profile_bundle(
        &RenderProfileBundle::advanced_render(),
        &advanced_capabilities(),
        &AdvancedProviderAvailability::new(),
    ));

    assert!(context
        .view_visibility(&VisibilityViewKey::MainCamera)
        .is_some());
    assert!(context
        .view_visibility(&VisibilityViewKey::ShadowCascade {
            light: 99,
            cascade: 0,
        })
        .is_none());
}

#[test]
fn frame_submission_context_keeps_the_resolved_view_family_phase_contract() {
    let view_family_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_temporal_fractions(0.5, 0.75),
        RenderUpscalerKind::Temporal,
    );
    let context = context_with_advanced_plan_and_view_family(
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new(),
        ),
        view_family_pipeline,
    );

    let temporal_targets = context
        .view_family_pipeline()
        .phase_targets(RenderPipelinePhase::TemporalReconstruction)
        .expect("temporal reconstruction is enabled for a temporal view family");
    assert_eq!(
        temporal_targets
            .input()
            .expect("temporal reconstruction reads the primary scene")
            .viewport()
            .physical_size,
        UVec2::new(960, 540)
    );
    assert_eq!(
        temporal_targets.output().viewport().physical_size,
        UVec2::new(1440, 810)
    );
    assert_eq!(
        context
            .view_family_pipeline()
            .phase_targets(RenderPipelinePhase::DisplayPostProcess)
            .expect("display post process is enabled")
            .output()
            .viewport()
            .physical_size,
        UVec2::new(1440, 810)
    );
}

#[test]
fn virtual_geometry_payload_source_clears_when_plan_degrades_feature() {
    let context = context_with_advanced_plan_and_virtual_geometry(
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
        ),
        Some(RenderVirtualGeometryExtract::default()),
        RenderVirtualGeometryPayloadSource::Authored,
        native_view_family_pipeline(),
    );

    assert!(!context.virtual_geometry_enabled());
    assert!(context.virtual_geometry_extract().is_none());
    assert_eq!(
        context.virtual_geometry_payload_source(),
        RenderVirtualGeometryPayloadSource::None
    );
}

#[test]
fn virtual_geometry_payload_source_survives_for_provider_backed_extract() {
    let context = context_with_advanced_plan_and_virtual_geometry(
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        ),
        Some(RenderVirtualGeometryExtract::default()),
        RenderVirtualGeometryPayloadSource::Authored,
        native_view_family_pipeline(),
    );

    assert!(context.virtual_geometry_enabled());
    assert!(context.virtual_geometry_extract().is_some());
    assert_eq!(
        context.virtual_geometry_payload_source(),
        RenderVirtualGeometryPayloadSource::Authored
    );
}

#[test]
fn render_taa_jitter_zero_when_taa_inactive() {
    assert_eq!(
        temporal_jitter_for_submission(AntiAliasFallbackReport::exact(AntiAliasMode::Off), 4),
        TemporalJitterSample::default()
    );
    assert_eq!(
        temporal_jitter_for_submission(AntiAliasFallbackReport::exact(AntiAliasMode::Fxaa), 4),
        TemporalJitterSample::default()
    );
    assert_eq!(
        temporal_jitter_for_submission(
            AntiAliasFallbackReport::exact(AntiAliasMode::Msaa { samples: 4 }),
            4,
        ),
        TemporalJitterSample::default()
    );
    assert_eq!(
        temporal_jitter_for_submission(
            AntiAliasFallbackReport::fallback(
                AntiAliasMode::Taa,
                AntiAliasMode::Fxaa,
                AntiAliasFallbackReason::MissingHistory,
            ),
            4,
        ),
        TemporalJitterSample::default()
    );

    let jitter =
        temporal_jitter_for_submission(AntiAliasFallbackReport::exact(AntiAliasMode::Taa), 0);

    assert_eq!(jitter.sequence_index, 1);
    assert_ne!(jitter, TemporalJitterSample::default());
}

#[test]
fn hybrid_gi_payload_source_clears_when_plan_degrades_feature() {
    let context = context_with_advanced_plan_and_payloads(
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new().with_virtual_geometry_provider("vg"),
        ),
        Some(RenderHybridGiExtract::default()),
        RenderHybridGiPayloadSource::SceneRepresentation,
        None,
        RenderVirtualGeometryPayloadSource::None,
        native_view_family_pipeline(),
    );

    assert!(!context.hybrid_gi_enabled());
    assert!(context.hybrid_gi_extract().is_none());
    assert_eq!(
        context.hybrid_gi_payload_source(),
        RenderHybridGiPayloadSource::None
    );
}

#[test]
fn hybrid_gi_scene_representation_source_survives_for_provider_backed_settings() {
    let context = context_with_advanced_plan_and_payloads(
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        ),
        Some(RenderHybridGiExtract::default()),
        RenderHybridGiPayloadSource::SceneRepresentation,
        None,
        RenderVirtualGeometryPayloadSource::None,
        native_view_family_pipeline(),
    );

    assert!(context.hybrid_gi_enabled());
    assert!(context.hybrid_gi_extract().is_some());
    assert_eq!(
        context.hybrid_gi_payload_source(),
        RenderHybridGiPayloadSource::SceneRepresentation
    );
}

fn context_with_advanced_plan(
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
) -> FrameSubmissionContext {
    context_with_advanced_plan_and_view_family(advanced_runtime_plan, native_view_family_pipeline())
}

fn native_view_family_pipeline() -> RenderViewFamilyPipeline {
    RenderViewFamilyPipeline::resolve(
        UVec2::new(64, 64),
        RenderResolutionPolicy::default(),
        RenderUpscalerKind::Spatial,
    )
}

fn context_with_advanced_plan_and_view_family(
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
    view_family_pipeline: RenderViewFamilyPipeline,
) -> FrameSubmissionContext {
    context_with_advanced_plan_and_virtual_geometry(
        advanced_runtime_plan,
        None,
        RenderVirtualGeometryPayloadSource::None,
        view_family_pipeline,
    )
}

fn context_with_advanced_plan_and_virtual_geometry(
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
    virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    view_family_pipeline: RenderViewFamilyPipeline,
) -> FrameSubmissionContext {
    context_with_advanced_plan_and_payloads(
        advanced_runtime_plan,
        None,
        RenderHybridGiPayloadSource::None,
        virtual_geometry_extract,
        virtual_geometry_payload_source,
        view_family_pipeline,
    )
}

fn context_with_advanced_plan_and_payloads(
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
    hybrid_gi_extract: Option<RenderHybridGiExtract>,
    hybrid_gi_payload_source: RenderHybridGiPayloadSource,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
    virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    view_family_pipeline: RenderViewFamilyPipeline,
) -> FrameSubmissionContext {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    let source_extract = Arc::new(extract.clone());
    FrameSubmissionContext::new(
        UVec2::new(64, 64),
        UVec2::new(64, 64),
        RenderPipelineHandle::new(1),
        0,
        None,
        Default::default(),
        Arc::new(empty_pipeline()),
        RenderCapabilitySummary::default(),
        VisibilityContext::from_extract(&extract),
        None,
        ViewportCameraHistoryKey::from_camera(
            extract
                .view
                .selected_camera_descriptor()
                .expect("test extract has selected camera descriptor"),
        ),
        Default::default(),
        false,
        None,
        ViewportRenderOutputTarget::PrimarySurface,
        Default::default(),
        view_family_pipeline,
        None,
        Default::default(),
        Default::default(),
        Default::default(),
        advanced_runtime_plan,
        Default::default(),
        true,
        true,
        hybrid_gi_extract,
        hybrid_gi_payload_source,
        None,
        None,
        source_extract,
        0,
        0,
        0,
        virtual_geometry_extract,
        virtual_geometry_payload_source,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        None,
        1,
    )
}

fn advanced_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        virtual_geometry_supported: true,
        hybrid_global_illumination_supported: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        ..RenderCapabilitySummary::default()
    }
}

fn empty_pipeline() -> CompiledRenderPipeline {
    let graph = RenderGraphBuilder::new("advanced-runtime-plan-context-test")
        .compile()
        .unwrap();
    CompiledRenderPipeline::from_parts(crate::graphics::pipeline::CompiledRenderPipelineParts {
        handle: RenderPipelineHandle::new(1),
        name: "empty".to_string(),
        renderer_name: "empty".to_string(),
        execution_pass_metadata: Vec::new(),
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        environment_ibl_bake_request: None,
        ambient_occlusion_profile: None,
        half_resolution_transparency_depth_sigma:
            crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
        graph,
    })
    .expect("empty frame context pipeline execution packet")
}
