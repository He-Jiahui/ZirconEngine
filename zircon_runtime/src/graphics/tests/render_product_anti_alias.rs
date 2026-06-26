use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasFallbackReason, AntiAliasMode, AntiAliasSettings, CapturedFrame, FallbackSkyboxKind,
    PreviewEnvironmentExtract, ProjectionMode, RenderCapabilitySummary, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderPipelineHandle,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::anti_alias::fxaa::{FXAA_EXECUTOR_ID, FXAA_PASS_NAME};
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderPassStage, RenderPipelineAsset,
    WgpuRenderFramework,
};
use crate::scene::world::World;

mod particle;
mod reactive_mask;

const TAA_RESOLVE_EXECUTOR_ID: &str = "temporal.taa-resolve";
const TAA_REACTIVE_MASK_CLEAR_EXECUTOR_ID: &str = "temporal.taa-reactive-mask-clear";
const TAA_REACTIVE_MASK_MESH_EXECUTOR_ID: &str = "temporal.taa-reactive-mask-mesh";

#[test]
fn anti_alias_settings_report_structured_fallbacks() {
    let fxaa_capable = RenderCapabilitySummary {
        backend_name: "aa-test".to_string(),
        supports_fxaa: true,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    };
    let no_screen_space_aa = RenderCapabilitySummary {
        backend_name: "aa-test".to_string(),
        supports_fxaa: false,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    };

    let auto = AntiAliasSettings::auto().resolve(&fxaa_capable, false);
    assert_eq!(auto.requested_mode, AntiAliasMode::Auto);
    assert_eq!(auto.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(
        auto.reason,
        Some(AntiAliasFallbackReason::AutoResolvedToFxaa)
    );

    let dlss = AntiAliasSettings::dlss().resolve(&fxaa_capable, false);
    assert_eq!(dlss.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(dlss.reason, Some(AntiAliasFallbackReason::UnsupportedDlss));

    let msaa = AntiAliasSettings::msaa(8).resolve(&fxaa_capable, false);
    assert_eq!(msaa.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(
        msaa.reason,
        Some(AntiAliasFallbackReason::UnsupportedMsaaSampleCount)
    );

    let taa = AntiAliasSettings::taa().resolve(&fxaa_capable, false);
    assert_eq!(taa.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(taa.reason, Some(AntiAliasFallbackReason::MissingHistory));

    let unsupported_auto = AntiAliasSettings::auto().resolve(&no_screen_space_aa, false);
    assert_eq!(unsupported_auto.effective_mode, AntiAliasMode::Off);
    assert_eq!(
        unsupported_auto.reason,
        Some(AntiAliasFallbackReason::UnsupportedFxaa)
    );
}

#[test]
fn render_product_anti_alias_compiles_fxaa_pass_for_default_3d() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&perspective_extract())
        .unwrap();

    assert!(compiled
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::AntiAlias)));
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias));

    let post = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == "uber")
        .expect("uber pass should compile");
    let fxaa = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == FXAA_PASS_NAME)
        .expect("FXAA pass should compile");
    let runtime_ui = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == "runtime-ui")
        .expect("runtime UI should remain after postprocess");

    assert!(post < fxaa && fxaa < runtime_ui);
    assert_eq!(
        compiled.pass_stages[fxaa].stage,
        RenderPassStage::PostProcess
    );
    let fxaa_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == FXAA_PASS_NAME)
        .expect("compiled graph should contain FXAA pass");
    assert_eq!(fxaa_pass.executor_id.as_deref(), Some(FXAA_EXECUTOR_ID));
}

#[test]
fn render_product_anti_alias_submit_records_fxaa_stats_and_graph_node() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-aa-product")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, perspective_extract())
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        stats.last_anti_alias_fallback.requested_mode,
        AntiAliasMode::Auto
    );
    assert_eq!(
        stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Fxaa
    );
    assert_eq!(
        stats.last_anti_alias_fallback.reason,
        Some(AntiAliasFallbackReason::AutoResolvedToFxaa)
    );
    assert_eq!(stats.last_anti_alias_graph_executed_pass_count, 1);
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&FXAA_EXECUTOR_ID.to_string()));
    assert!(stats
        .last_post_process_graph_executed_nodes
        .contains(&FXAA_PASS_NAME.to_string()));
}

#[test]
fn render_product_temporal_off_matches_anti_alias_feature_disabled_product() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let explicit_off_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let feature_disabled_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            explicit_off_viewport,
            anti_alias_product_profile("runtime-taa-off-explicit", true),
        )
        .unwrap();
    framework
        .set_quality_profile(
            feature_disabled_viewport,
            anti_alias_product_profile("runtime-taa-off-feature-disabled", false),
        )
        .unwrap();

    let (explicit_off_frame, explicit_off_stats) = submit_and_capture_anti_alias_product(
        &framework,
        explicit_off_viewport,
        temporal_off_product_extract(viewport_size),
    );
    let (feature_disabled_frame, feature_disabled_stats) = submit_and_capture_anti_alias_product(
        &framework,
        feature_disabled_viewport,
        temporal_off_product_extract(viewport_size),
    );

    assert_eq!(
        explicit_off_stats.last_anti_alias_fallback.requested_mode,
        AntiAliasMode::Off
    );
    assert_eq!(
        explicit_off_stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Off
    );
    assert_eq!(explicit_off_stats.last_anti_alias_fallback.reason, None);
    assert_eq!(
        explicit_off_stats.last_anti_alias_graph_executed_pass_count,
        0
    );
    assert!(!explicit_off_stats
        .last_graph_executed_executor_ids
        .contains(&FXAA_EXECUTOR_ID.to_string()));
    assert!(!explicit_off_stats
        .last_post_process_graph_executed_nodes
        .contains(&FXAA_PASS_NAME.to_string()));

    assert_eq!(
        feature_disabled_stats
            .last_anti_alias_fallback
            .requested_mode,
        AntiAliasMode::Off
    );
    assert_eq!(
        feature_disabled_stats
            .last_anti_alias_fallback
            .effective_mode,
        AntiAliasMode::Off
    );
    assert_eq!(feature_disabled_stats.last_anti_alias_fallback.reason, None);
    assert_eq!(
        feature_disabled_stats.last_anti_alias_graph_executed_pass_count,
        0
    );

    assert_captured_frames_equal(
        &explicit_off_frame,
        &feature_disabled_frame,
        "TAA/AA off product output should match the feature-disabled baseline",
    );
}

#[test]
fn render_product_taa_uses_temporal_resolve_seed_frame_when_requested() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            viewport,
            anti_alias_product_profile("runtime-taa-product", true).with_temporal_history(true),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, empty_temporal_taa_product_extract(viewport_size))
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_taa_resolve_product_stats(&stats);
}

#[test]
fn render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            viewport,
            anti_alias_product_profile("runtime-taa-static-history", true)
                .with_temporal_history(true),
        )
        .unwrap();

    let (seed_frame, seed_stats) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        empty_temporal_taa_product_extract(viewport_size),
    );
    let (history_frame_a, history_stats_a) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        empty_temporal_taa_product_extract(viewport_size),
    );
    let (history_frame_b, history_stats_b) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        empty_temporal_taa_product_extract(viewport_size),
    );

    assert_taa_resolve_product_stats(&seed_stats);
    assert_taa_resolve_product_stats(&history_stats_a);
    assert_taa_resolve_product_stats(&history_stats_b);
    assert_eq!(seed_frame.width, history_frame_a.width);
    assert_eq!(seed_frame.height, history_frame_a.height);
    assert_captured_frames_equal(
        &history_frame_a,
        &history_frame_b,
        "static empty-scene TAA history output should stay pixel-stable after the seed frame",
    );
}

#[test]
fn render_product_taa_dynamic_occlusion_change_converges_after_history_seed() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            viewport,
            anti_alias_product_profile("runtime-taa-dynamic-occlusion", true)
                .with_temporal_history(true),
        )
        .unwrap();

    let (visible_frame, visible_stats) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        dynamic_occlusion_taa_product_extract(viewport_size, false),
    );
    let (occluded_frame_a, occluded_stats_a) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        dynamic_occlusion_taa_product_extract(viewport_size, true),
    );
    let (occluded_frame_b, occluded_stats_b) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        dynamic_occlusion_taa_product_extract(viewport_size, true),
    );
    let (occluded_frame_c, occluded_stats_c) = submit_and_capture_anti_alias_product(
        &framework,
        viewport,
        dynamic_occlusion_taa_product_extract(viewport_size, true),
    );

    assert_taa_resolve_product_stats(&visible_stats);
    assert_taa_resolve_product_stats(&occluded_stats_a);
    assert_taa_resolve_product_stats(&occluded_stats_b);
    assert_taa_resolve_product_stats(&occluded_stats_c);
    assert_frame_delta_decreases_after_occlusion(
        &visible_frame,
        &occluded_frame_a,
        &occluded_frame_b,
        &occluded_frame_c,
    );
}

fn anti_alias_product_profile(name: &str, anti_alias_enabled: bool) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_anti_alias(anti_alias_enabled)
}

fn temporal_off_product_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = World::new().to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.anti_alias = AntiAliasSettings::off();
    extract
}

fn empty_temporal_taa_product_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(801),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Perspective,
        ),
    );
    extract.apply_viewport_size(viewport_size);
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
}

fn dynamic_occlusion_taa_product_extract(
    viewport_size: UVec2,
    occluder_visible: bool,
) -> RenderFrameExtract {
    let mut meshes = Vec::new();
    if occluder_visible {
        meshes.push(dynamic_occlusion_mesh(
            811,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.6, 2.8, 0.25),
            "builtin://material/taa-occlusion-wall",
            Vec4::new(0.75, 0.78, 0.85, 1.0),
            Mobility::Static,
        ));
    }
    meshes.push(dynamic_occlusion_mesh(
        812,
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.65, 0.65, 0.65),
        "builtin://material/taa-hidden-target",
        Vec4::new(0.0, 0.9, 0.5, 1.0),
        Mobility::Dynamic,
    ));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(802),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                    ..ViewportCameraSnapshot::default()
                },
                meshes,
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract.apply_viewport_size(viewport_size);
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
}

fn dynamic_occlusion_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    material_label: &str,
    tint: Vec4,
    mobility: Mobility,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            translation,
            scale,
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            material_label,
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
    }
}

fn submit_and_capture_anti_alias_product(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("product frame should be available for AA parity");
    let stats = framework.query_stats().unwrap();
    (frame, stats)
}

fn assert_taa_resolve_product_stats(stats: &RenderStats) {
    assert!(stats.capabilities.supports_taa);
    assert_eq!(
        stats.last_anti_alias_fallback.requested_mode,
        AntiAliasMode::Taa
    );
    assert_eq!(
        stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Taa
    );
    assert_eq!(stats.last_anti_alias_fallback.reason, None);
    assert_eq!(
        stats.last_anti_alias_graph_executed_pass_count, 1,
        "expected one TAA anti-alias graph pass, executed executor ids: {:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&TAA_RESOLVE_EXECUTOR_ID.to_string()),
        "expected TAA executor `{}` in executed executor ids: {:?}",
        TAA_RESOLVE_EXECUTOR_ID,
        stats.last_graph_executed_executor_ids
    );
    assert!(!stats
        .last_graph_executed_executor_ids
        .contains(&FXAA_EXECUTOR_ID.to_string()));
    assert!(!stats
        .last_post_process_graph_executed_nodes
        .contains(&FXAA_PASS_NAME.to_string()));
}

fn assert_executor_order(stats: &RenderStats, before: &str, after: &str) {
    let before_index = stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executor_id| executor_id == before)
        .unwrap_or_else(|| {
            panic!(
                "expected executor `{}` in executed executor ids: {:?}",
                before, stats.last_graph_executed_executor_ids
            )
        });
    let after_index = stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executor_id| executor_id == after)
        .unwrap_or_else(|| {
            panic!(
                "expected executor `{}` in executed executor ids: {:?}",
                after, stats.last_graph_executed_executor_ids
            )
        });
    assert!(
        before_index < after_index,
        "expected executor `{before}` to run before `{after}`, executed executor ids: {:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_captured_frames_equal(actual: &CapturedFrame, expected: &CapturedFrame, label: &str) {
    assert_eq!(actual.width, expected.width, "{label}: width mismatch");
    assert_eq!(actual.height, expected.height, "{label}: height mismatch");
    if let Some(index) = first_pixel_mismatch(&actual.rgba, &expected.rgba) {
        let byte = index * 4;
        panic!(
            "{label}: pixel {index} mismatch, actual={:?}, expected={:?}",
            &actual.rgba[byte..byte + 4],
            &expected.rgba[byte..byte + 4]
        );
    }
}

fn assert_frame_delta_decreases_after_occlusion(
    visible_frame: &CapturedFrame,
    occluded_frame_a: &CapturedFrame,
    occluded_frame_b: &CapturedFrame,
    occluded_frame_c: &CapturedFrame,
) {
    let transition_delta = frame_rgba_abs_delta(visible_frame, occluded_frame_a);
    let settled_delta = frame_rgba_abs_delta(occluded_frame_b, occluded_frame_c);

    assert!(
        transition_delta > 0,
        "dynamic occlusion setup should change captured output"
    );
    assert!(
        settled_delta < transition_delta,
        "TAA repeated occluded frames should converge below the visible->occluded transition delta; transition={transition_delta}, settled={settled_delta}"
    );
}

fn frame_rgba_abs_delta(actual: &CapturedFrame, expected: &CapturedFrame) -> u64 {
    assert_eq!(actual.width, expected.width);
    assert_eq!(actual.height, expected.height);
    assert_eq!(actual.rgba.len(), expected.rgba.len());
    actual
        .rgba
        .iter()
        .zip(expected.rgba.iter())
        .map(|(actual, expected)| actual.abs_diff(*expected) as u64)
        .sum()
}

fn first_pixel_mismatch(actual: &[u8], expected: &[u8]) -> Option<usize> {
    assert_eq!(
        actual.len(),
        expected.len(),
        "captured frame byte lengths should match"
    );
    actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .position(|(actual, expected)| actual != expected)
}

fn perspective_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(800),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Perspective,
        ),
    )
}
