use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use zircon_runtime::asset::{
    texture_asset_from_ibl_bake_artifact_pmrem, AlphaMode, AssetReference, AssetUri, MaterialAsset,
    ProjectAssetManager, TextureAsset,
};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, CapturedFrame, DisplayMode, EnvironmentExtract,
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, ProbeInfluenceShape, ProceduralSkyParams, ProjectionMode,
    ReflectionProbeData, RenderFrameExtract, RenderFramework, RenderLayerSet, RenderMeshSnapshot,
    RenderOverlayExtract, RenderQualityProfile, RenderReflectionProbeWorkloadReport,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Quat, Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, WgpuRenderFramework,
};
use zircon_runtime::render_graph::QueueLane;

mod support;

const OUTPUT_SIZE: UVec2 = UVec2::new(320, 240);
const PROFILE_OUTPUT_SIZE: UVec2 = UVec2::new(1_920, 1_080);
const PROFILE_PROBE_COUNTS: [usize; 4] = [1, 8, 32, 64];
const PROFILE_WARMUP_FRAME_COUNT: usize = 16;
const PROFILE_GPU_SAMPLE_COUNT: usize = 120;
const PROFILE_MAX_SUBMISSION_COUNT: usize =
    PROFILE_WARMUP_FRAME_COUNT + PROFILE_GPU_SAMPLE_COUNT * 2;
const PROBE_PMREM_WRITE_COUNT_PER_CUBEMAP: usize = 8;
const PROFILE_OUTPUT_NAME: &str =
    "runtime_environment_reflection_probe_linear_scan_before_profile_20260829.json";
const PROFILE_SCREENSHOT_NAME: &str =
    "runtime_environment_reflection_probe_linear_scan_64_20260829.png";
const RENDERDOC_CAPTURE_PROFILE_ENV: &str = "ZR_RENDERDOC_CAPTURE_REFLECTION_PROBE_64";
const LEFT_PMREM_COLOR: [f32; 4] = [0.9, 0.01, 0.01, 1.0];
const RIGHT_PMREM_COLOR: [f32; 4] = [0.01, 0.01, 0.9, 1.0];

#[test]
fn reflection_probe_feature_off_matches_skybox_and_enabled_probes_change_pixels() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_mirror_material(&asset_manager);
    let left = register_probe_pmrem(
        &asset_manager,
        "res://generated/integration-probe-left.zpmrem",
        LEFT_PMREM_COLOR,
    );
    let right = register_probe_pmrem(
        &asset_manager,
        "res://generated/integration-probe-right.zpmrem",
        RIGHT_PMREM_COLOR,
    );
    let asset_runtime = support::ProjectAssetTestRuntime::new(Arc::clone(&asset_manager));
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [reflection_probe_render_feature_descriptor()],
        Vec::new(),
        Vec::new(),
        asset_runtime.worker_pool(),
    )
    .expect("reflection-probe product framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(OUTPUT_SIZE))
        .expect("reflection-probe product viewport");

    let probes = vec![
        reflection_probe(1, -2.0, left),
        reflection_probe(2, 2.0, right),
    ];
    let enabled = capture(
        &framework,
        viewport,
        material,
        true,
        EnvironmentExtract::procedural_default().with_reflection_probes(probes.clone()),
        1,
    );
    let disabled = capture(
        &framework,
        viewport,
        material,
        false,
        EnvironmentExtract::procedural_default().with_reflection_probes(probes),
        2,
    );
    let sky_only = capture(
        &framework,
        viewport,
        material,
        true,
        EnvironmentExtract::procedural_default(),
        3,
    );

    let fallback_error = mean_absolute_rgb_error(&disabled.rgba, &sky_only.rgba);
    let probe_difference = mean_absolute_rgb_error(&enabled.rgba, &sky_only.rgba);
    println!(
        "reflection_probe_product fallback_mae={fallback_error:.6} enabled_vs_sky_mae={probe_difference:.6}"
    );
    assert!(
        fallback_error <= 0.25,
        "feature-off probe frame must match sky fallback, MAE={fallback_error}"
    );
    assert!(
        probe_difference >= 4.0,
        "enabled probes must visibly differ from sky fallback, MAE={probe_difference}"
    );
}

#[test]
#[ignore = "manual 1080p GPU before-profile for Shader 06 reflection-probe scaling"]
fn export_reflection_probe_linear_scan_before_profile() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_mirror_material(&asset_manager);
    let cubemap = register_probe_pmrem(
        &asset_manager,
        "res://generated/profile-probe-shared.zpmrem",
        LEFT_PMREM_COLOR,
    );
    let asset_runtime = support::ProjectAssetTestRuntime::new(Arc::clone(&asset_manager));
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [reflection_probe_render_feature_descriptor()],
        Vec::new(),
        Vec::new(),
        asset_runtime.worker_pool(),
    )
    .expect("reflection-probe before-profile framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PROFILE_OUTPUT_SIZE))
        .expect("reflection-probe before-profile viewport");
    framework
        .set_quality_profile(viewport, quality_profile(true))
        .expect("set reflection-probe before-profile quality");

    let mut next_world_snapshot = 10_000_u64;
    let mut cases = Vec::with_capacity(PROFILE_PROBE_COUNTS.len());
    for probe_count in PROFILE_PROBE_COUNTS {
        let environment = profile_environment(cubemap, probe_count);
        let mut sample_generations = HashSet::new();
        let mut recorded_generations = HashSet::new();
        let mut profiles = Vec::with_capacity(PROFILE_GPU_SAMPLE_COUNT);
        let mut latest_workload = None;

        for submission_index in 0..PROFILE_MAX_SUBMISSION_COUNT {
            let stats = submit_probe_frame(
                &framework,
                viewport,
                material,
                environment.clone(),
                next_world_snapshot,
            );
            next_world_snapshot = next_world_snapshot.saturating_add(1);
            let workload = stats.last_reflection_probe_workload;
            assert_profile_workload(workload, probe_count);
            latest_workload = Some(workload);

            if submission_index >= PROFILE_WARMUP_FRAME_COUNT {
                sample_generations.insert(
                    stats
                        .last_generation
                        .expect("profile submission must publish a frame generation"),
                );
            }
            if let Some(profile) = stats.last_resolved_gpu_frame_profile {
                if sample_generations.contains(&profile.frame_generation)
                    && recorded_generations.insert(profile.frame_generation)
                    && profile.gpu_frame_time_us.is_some()
                {
                    profiles.push(profile.as_ref().clone());
                }
            }
            if profiles.len() == PROFILE_GPU_SAMPLE_COUNT {
                break;
            }
        }

        assert_eq!(
            profiles.len(),
            PROFILE_GPU_SAMPLE_COUNT,
            "probe_count={probe_count} must resolve the requested independent GPU samples"
        );
        let gpu_frame_times = profiles
            .iter()
            .map(|profile| {
                profile
                    .gpu_frame_time_us
                    .expect("recorded profile must contain GPU frame time")
            })
            .collect::<Vec<_>>();
        let workload = latest_workload.expect("profile case must submit at least one frame");
        cases.push(serde_json::json!({
            "probe_count": probe_count,
            "workload": {
                "extracted_probe_count": workload.extracted_probe_count,
                "camera_layer_candidate_count": workload.camera_layer_candidate_count,
                "attempted_candidate_count": workload.attempted_candidate_count,
                "active_probe_count": workload.active_probe_count,
                "capacity_dropped_candidate_count": workload.capacity_dropped_candidate_count,
                "scheduled_cubemap_upload_count": workload.scheduled_cubemap_upload_count,
                "scheduled_cubemap_upload_bytes": workload.scheduled_cubemap_upload_bytes,
                "scheduled_texture_write_count": workload.scheduled_texture_write_count,
                "asset_load_call_count": workload.asset_load_call_count,
                "asset_load_cpu_time_us": workload.asset_load_cpu_time_us,
                "rejected_cubemap_count": workload.rejected_cubemap_count,
                "full_resolution_fragment_probe_visit_upper_bound": workload.full_resolution_fragment_probe_visit_upper_bound,
            },
            "gpu_frame_time_us": {
                "sample_count": gpu_frame_times.len(),
                "min": gpu_frame_times.iter().copied().min(),
                "p50": nearest_rank_percentile(&gpu_frame_times, 50),
                "p95": nearest_rank_percentile(&gpu_frame_times, 95),
                "p99": nearest_rank_percentile(&gpu_frame_times, 99),
                "max": gpu_frame_times.iter().copied().max(),
            },
            "profiles": profiles,
        }));
    }

    let capture_renderdoc =
        std::env::var(RENDERDOC_CAPTURE_PROFILE_ENV).is_ok_and(|value| value == "1");
    if capture_renderdoc {
        framework
            .request_graphics_debugger_capture(viewport)
            .expect("request RenderDoc capture for the 64-probe profile frame");
    }
    let final_stats = submit_probe_frame(
        &framework,
        viewport,
        material,
        profile_environment(cubemap, 64),
        next_world_snapshot,
    );
    assert_profile_workload(final_stats.last_reflection_probe_workload, 64);
    if capture_renderdoc {
        let capture_status = framework
            .query_graphics_debugger_status()
            .expect("query 64-probe RenderDoc capture status");
        assert!(!capture_status.capture_pending);
        assert_eq!(capture_status.last_error, None);
    }
    let frame = framework
        .capture_frame(viewport)
        .expect("capture 64-probe before-profile frame")
        .expect("64-probe before-profile frame should be available");

    let output_dir = shader_test_output_dir();
    std::fs::create_dir_all(&output_dir).expect("create Shader 06 evidence directory");
    let screenshot_output = output_dir.join(PROFILE_SCREENSHOT_NAME);
    save_captured_frame_png(&frame, &screenshot_output);
    let profile_output = output_dir.join(PROFILE_OUTPUT_NAME);
    let report = serde_json::json!({
        "schema": 1,
        "kind": "reflection_probe_linear_scan_before_profile",
        "render_size": [PROFILE_OUTPUT_SIZE.x, PROFILE_OUTPUT_SIZE.y],
        "warmup_frame_count": PROFILE_WARMUP_FRAME_COUNT,
        "gpu_sample_count_per_case": PROFILE_GPU_SAMPLE_COUNT,
        "probe_counts": PROFILE_PROBE_COUNTS,
        "shared_scene": true,
        "shared_material": true,
        "shared_pmrem": true,
        "renderdoc_capture_requested": capture_renderdoc,
        "cases": cases,
    });
    std::fs::write(
        &profile_output,
        serde_json::to_vec_pretty(&report).expect("serialize reflection-probe profile"),
    )
    .expect("write reflection-probe before-profile report");
    assert!(profile_output.starts_with(&output_dir));
    assert!(screenshot_output.starts_with(&output_dir));
}

fn capture(
    framework: &WgpuRenderFramework,
    viewport: zircon_runtime::core::framework::render::RenderViewportHandle,
    material: ResourceId,
    reflection_probes: bool,
    environment: EnvironmentExtract,
    world_snapshot: u64,
) -> CapturedFrame {
    let expected_extracted_probe_count = environment.reflection_probes().len();
    framework
        .set_quality_profile(viewport, quality_profile(reflection_probes))
        .expect("set reflection-probe product quality");
    let stats = submit_probe_frame(framework, viewport, material, environment, world_snapshot);
    let compiled_probe_feature = stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "reflection_probes");
    assert_eq!(compiled_probe_feature, reflection_probes);
    assert!(stats.last_visibility_visible_count > 0);
    assert!(stats.last_mesh_draw_count > 0);
    assert!(stats.last_material_ready_count > 0);
    let probe_workload = stats.last_reflection_probe_workload;
    assert_eq!(
        probe_workload.extracted_probe_count,
        expected_extracted_probe_count
    );
    let expected_active_probe_count = if reflection_probes {
        expected_extracted_probe_count
    } else {
        0
    };
    assert_eq!(
        probe_workload.camera_layer_candidate_count,
        expected_active_probe_count
    );
    assert_eq!(
        probe_workload.attempted_candidate_count,
        expected_active_probe_count
    );
    assert_eq!(
        probe_workload.active_probe_count,
        expected_active_probe_count
    );
    assert_eq!(probe_workload.capacity_dropped_candidate_count, 0);
    assert_eq!(probe_workload.rejected_cubemap_count, 0);
    assert_eq!(
        probe_workload.asset_load_call_count,
        probe_workload.scheduled_cubemap_upload_count
    );
    assert_eq!(
        probe_workload.scheduled_texture_write_count,
        probe_workload.scheduled_cubemap_upload_count * PROBE_PMREM_WRITE_COUNT_PER_CUBEMAP
    );
    assert_eq!(
        probe_workload.full_resolution_fragment_probe_visit_upper_bound,
        u64::from(OUTPUT_SIZE.x)
            * u64::from(OUTPUT_SIZE.y)
            * u64::try_from(expected_active_probe_count).unwrap_or(u64::MAX)
    );
    if expected_active_probe_count == 0 {
        assert_eq!(probe_workload.scheduled_cubemap_upload_count, 0);
        assert_eq!(probe_workload.scheduled_cubemap_upload_bytes, 0);
        assert_eq!(probe_workload.scheduled_texture_write_count, 0);
        assert_eq!(probe_workload.asset_load_cpu_time_us, 0);
    } else {
        assert_eq!(
            probe_workload.scheduled_cubemap_upload_count,
            expected_active_probe_count
        );
        assert!(probe_workload.scheduled_cubemap_upload_bytes > 0);
    }

    framework
        .capture_frame(viewport)
        .expect("capture reflection-probe product frame")
        .expect("reflection-probe product frame should be available")
}

fn submit_probe_frame(
    framework: &WgpuRenderFramework,
    viewport: zircon_runtime::core::framework::render::RenderViewportHandle,
    material: ResourceId,
    environment: EnvironmentExtract,
    world_snapshot: u64,
) -> RenderStats {
    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(world_snapshot),
                scene_snapshot(material, environment),
            ),
        )
        .expect("submit reflection-probe product frame");
    framework
        .query_stats()
        .expect("reflection-probe product stats")
}

fn scene_snapshot(material: ResourceId, environment: EnvironmentExtract) -> RenderSceneSnapshot {
    let camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 8.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 1.2,
        z_near: 0.1,
        z_far: 100.0,
        ..ViewportCameraSnapshot::default()
    };
    let preview =
        zircon_runtime::core::framework::render::PreviewEnvironmentExtract::from_environment(
            &environment,
            true,
            Vec4::ZERO,
        );

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: Transform::default(),
                model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                    "builtin://cube",
                )),
                mesh: None,
                material: ResourceHandle::<MaterialMarker>::new(material),
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                    is_static: false,
                    ..RendererCommon::default()
                },
            }],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment,
        preview,
        virtual_geometry_debug: None,
    }
}

fn register_mirror_material(asset_manager: &ProjectAssetManager) -> ResourceId {
    let uri = AssetUri::parse("res://generated/integration-probe-mirror.zmaterial")
        .expect("mirror material URI");
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some("Reflection Probe Integration Mirror".to_string()),
        shader: AssetReference::from_locator(
            AssetUri::parse("builtin://shader/pbr.wgsl").expect("PBR shader URI"),
        ),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.92, 0.92, 0.92, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 1.0,
        roughness: 0.08,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri),
            material,
        )
        .expect("register mirror material");
    id
}

fn register_probe_pmrem(
    asset_manager: &ProjectAssetManager,
    uri_text: &str,
    color: [f32; 4],
) -> ResourceId {
    let uri = AssetUri::parse(uri_text).expect("probe PMREM URI");
    let id = ResourceId::from_locator(&uri);
    let source = build_source_cubemap_from_equirect(128, |_, _| color);
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
        .expect("probe PMREM payload");
    let texture = texture_asset_from_ibl_bake_artifact_pmrem(
        uri.clone(),
        &IblBakeArtifactBlob::from_payload(payload),
    )
    .expect("probe PMREM texture");
    asset_manager
        .assets::<TextureAsset>()
        .insert(ResourceRecord::new(id, ResourceKind::Texture, uri), texture)
        .expect("register probe PMREM texture");
    id
}

fn reflection_probe(probe_id: u64, center_x: f32, cubemap: ResourceId) -> ReflectionProbeData {
    ReflectionProbeData::try_new(
        probe_id,
        Vec3::new(center_x, 0.0, 0.0),
        Quat::IDENTITY,
        ProbeInfluenceShape::box_shape(Vec3::new(4.0, 10.0, 10.0), 4.0)
            .expect("reflection-probe product influence"),
        Vec3::new(4.0, 10.0, 10.0),
    )
    .expect("reflection-probe product contract")
    .with_baked_cubemap(Some(cubemap))
}

fn profile_environment(cubemap: ResourceId, probe_count: usize) -> EnvironmentExtract {
    EnvironmentExtract::procedural_default().with_reflection_probes(
        (0..probe_count)
            .map(|index| reflection_probe(index as u64 + 1, 0.0, cubemap))
            .collect(),
    )
}

fn assert_profile_workload(report: RenderReflectionProbeWorkloadReport, probe_count: usize) {
    assert_eq!(report.extracted_probe_count, probe_count);
    assert_eq!(report.camera_layer_candidate_count, probe_count);
    assert_eq!(report.attempted_candidate_count, probe_count);
    assert_eq!(report.active_probe_count, probe_count);
    assert_eq!(report.capacity_dropped_candidate_count, 0);
    assert_eq!(report.rejected_cubemap_count, 0);
    assert_eq!(
        report.full_resolution_fragment_probe_visit_upper_bound,
        u64::from(PROFILE_OUTPUT_SIZE.x)
            .saturating_mul(u64::from(PROFILE_OUTPUT_SIZE.y))
            .saturating_mul(u64::try_from(probe_count).unwrap_or(u64::MAX))
    );
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let rank = (percentile * samples.len() + 99) / 100;
    let index = rank.clamp(1, samples.len()) - 1;
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[index]
}

fn shader_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime must live below the repository root")
        .join("docs/tests/runtime/shader")
}

fn save_captured_frame_png(frame: &CapturedFrame, output: &std::path::Path) {
    image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        frame.width,
        frame.height,
        frame.rgba.clone(),
    )
    .expect("reflection-probe profile frame dimensions")
    .save_with_format(output, image::ImageFormat::Png)
    .expect("write reflection-probe profile screenshot");
}

fn quality_profile(reflection_probes: bool) -> RenderQualityProfile {
    RenderQualityProfile::new("reflection-probe-public-product")
        .with_reflection_probes(reflection_probes)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_clustered_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(false)
        .with_solari(false)
}

fn reflection_probe_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn mean_absolute_rgb_error(left: &[u8], right: &[u8]) -> f32 {
    assert_eq!(left.len(), right.len());
    let mut total = 0_u64;
    let mut samples = 0_u64;
    for (left_pixel, right_pixel) in left.chunks_exact(4).zip(right.chunks_exact(4)) {
        for channel in 0..3 {
            total += (i16::from(left_pixel[channel]) - i16::from(right_pixel[channel]))
                .unsigned_abs() as u64;
            samples += 1;
        }
    }
    total as f32 / samples.max(1) as f32
}
