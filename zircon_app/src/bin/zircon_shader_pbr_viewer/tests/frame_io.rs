use super::{
    error_frame, ready_frame_evidence_metadata_path, startup_frame, write_ready_frame_evidence,
    write_ready_frame_png, ReadyFrameEvidenceMetadata,
};
use std::time::Duration;
use zircon_runtime::core::framework::render::{
    EnvironmentRuntimeSnapshot, RealtimeIblReadiness, RealtimeIblStatusReport,
    ShaderVariantMissReport, IBL_BAKE_ALGORITHM_VERSION,
};

use crate::evidence_identity::{
    EvidenceFileFingerprint, ReadyFrameEvidenceIdentity, READY_FRAME_EVIDENCE_VALIDATION_POLICY,
};
use crate::work_paths::viewer_test_artifact_root;

#[test]
fn status_frames_clamp_zero_dimensions_and_remain_opaque() {
    let frame = startup_frame(zircon_runtime::core::math::UVec2::new(0, 0));

    assert_eq!((frame.width, frame.height), (1, 1));
    assert_eq!(frame.rgba.len(), 4);
    assert_eq!(frame.rgba[3], 255);
    assert_ne!(
        frame.rgba,
        error_frame(zircon_runtime::core::math::UVec2::new(1, 1)).rgba
    );
}

#[test]
fn ready_frame_png_encoder_roundtrips_rgba_pixels() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path = viewer_test_artifact_root("ready-frame-png").join(format!("{unique}.png"));
    let rgba = [
        255, 0, 0, 255, // red
        0, 255, 0, 128, // green with alpha
    ];

    write_ready_frame_png(&path, 2, 1, &rgba).expect("PNG encoding should succeed");
    let decoded = image::open(&path)
        .expect("written Ready-frame PNG should decode")
        .to_rgba8();
    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");

    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.as_raw(), &rgba);
}

#[test]
fn ready_frame_png_encoder_rejects_mismatched_rgba_without_creating_evidence() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path = viewer_test_artifact_root("invalid-ready-frame").join(format!("{unique}.png"));

    let error = write_ready_frame_png(&path, 2, 1, &[255, 0, 0, 255])
        .expect_err("a truncated RGBA frame must not produce evidence");

    assert!(error.contains("does not match 2x1 output"));
    assert!(!path.exists());
    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");
}

#[test]
fn ready_frame_evidence_writes_png_with_provenance_sidecar() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path = viewer_test_artifact_root("ready-frame-evidence").join(format!("{unique}.png"));
    let metadata = ReadyFrameEvidenceMetadata {
        backend: "Dx12".to_owned(),
        evidence_identity: ready_frame_evidence_identity_fixture(),
        interactive_direct_present_enabled: false,
        host_mode: "offscreen-diagnostic".to_owned(),
        host_composition_id: "zircon_shader_pbr_viewer_standalone_diagnostic_v1".to_owned(),
        scene_id: "single_pbr_mirror_sphere".to_owned(),
        capture_target: "offscreen-scene-renderer-cpu-readback".to_owned(),
        gpu_scene_surface_present_count: 0,
        hdri_path: "polyhaven_lakes_2k.hdr".to_owned(),
        requested_source_face_size: None,
        requested_pmrem_face_size: Some(256),
        active_source_cubemap_face_size: 512,
        active_source_cubemap_mip_count: 10,
        active_pmrem_face_size: 256,
        active_pmrem_mip_count: 9,
        render_profile: "environment_only_pbr_preview".to_owned(),
        material_fixture: "metal-mirror".to_owned(),
        required_material_base_pipeline_kind: "environment-only-pbr-base".to_owned(),
        required_material_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_requested: true,
        environment_only_base_prewarm_pipeline_ready: false,
        environment_only_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_cache_hit: false,
        environment_only_base_prewarm_cache_scope: ENVIRONMENT_ONLY_BASE_PREWARM_CACHE_SCOPE
            .to_owned(),
        environment_only_base_prewarm_shader_source_resolution: Duration::from_millis(2),
        environment_only_base_prewarm_pipeline_creation: Duration::from_millis(11),
        environment_only_base_prewarm_elapsed: Duration::from_millis(13),
        camera_yaw_degrees: 12.5,
        camera_pitch_degrees: -7.0,
        ibl_bake_algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
        ibl_staging_status: "Written".to_owned(),
        ibl_staging_elapsed: Duration::from_millis(34),
        ibl_staging_source_decode: Duration::from_millis(1),
        ibl_staging_cubemap_build: Duration::from_millis(18),
        ibl_staging_equirect_projection: Duration::from_millis(3),
        ibl_staging_source_mip_build: Duration::from_millis(4),
        ibl_staging_pmrem_build: Duration::from_millis(5),
        ibl_staging_sh9_build: Duration::from_millis(6),
        ibl_staging_irradiance_cube_build: Duration::from_millis(7),
        ibl_staging_bundle_write: Duration::from_millis(8),
        ibl_staging_source_zcube_bytes: 1_024,
        ibl_staging_asset_derived_bytes: 2_048,
        ibl_staging_parallel_executor_work_items: 42,
        ibl_staging_equirect_projection_parallel_work_items: 6,
        ibl_staging_source_mip_build_parallel_work_items: 12,
        ibl_staging_pmrem_build_parallel_work_items: 24,
        ibl_staging_irradiance_cube_build_parallel_work_items: 0,
        ibl_staging_irradiance_cube_source_sample_visits: 37_748_736,
        ibl_total_elapsed: Duration::from_millis(40),
        scene_startup_hdri_decode: Duration::from_millis(21),
        scene_startup_project_assets: Duration::from_millis(34),
        scene_startup_runtime_bootstrap: Duration::from_millis(55),
        scene_startup_project_open: Duration::from_millis(89),
        scene_startup_world_load: Duration::from_millis(144),
        scene_startup_renderer_initialization: Duration::from_millis(3_600),
        scene_startup_renderer_backend_initialization: Duration::from_millis(377),
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized: true,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait:
            Duration::from_millis(8),
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization:
            Duration::from_millis(7),
        scene_startup_renderer_environment_brdf_lut_texture_upload_submission:
            Duration::from_millis(1),
        scene_startup_renderer_deferred_initialization: Duration::from_millis(1_600),
        scene_startup_renderer_deferred_standard_pipeline: Duration::from_millis(987),
        scene_startup_resource_streamer_initialization: Duration::from_millis(1_597),
        scene_startup_ibl_restore: Duration::from_millis(2_584),
        scene_startup_total: Duration::from_millis(7_000),
        one_shot_base_pipeline_wait_elapsed: Duration::from_millis(75),
        viewer_scene_load_elapsed: Duration::from_millis(7_120),
        viewer_ready_elapsed: Duration::from_millis(7_250),
        ready_frame_render_elapsed: Duration::from_millis(16),
        ready_frame_render_extract: Duration::from_millis(2),
        ready_frame_renderer_call: Duration::from_millis(11),
        ready_frame_readback_and_completion: Duration::from_millis(3),
        environment_runtime_snapshot: environment_runtime_snapshot_fixture(),
        shader_variant_miss_report: shader_variant_miss_report_fixture(),
    };

    let metadata_path = write_ready_frame_evidence(&path, 1, 1, &[128, 64, 32, 255], &metadata)
        .expect("Ready-frame evidence should write");
    let metadata_text =
        std::fs::read_to_string(&metadata_path).expect("evidence metadata should be readable");

    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");

    assert_eq!(
        metadata_path,
        ready_frame_evidence_metadata_path(&path).unwrap()
    );
    assert!(metadata_text.contains("schema=zircon_shader_pbr_viewer_ready_frame_evidence_v18"));
    assert!(metadata_text
        .contains("evidence_identity_schema=zircon_shader_pbr_viewer_evidence_identity_v1"));
    assert!(metadata_text.contains("evidence_run_id=shader-pbr-test-ready-frame"));
    assert!(metadata_text
        .contains("evidence_validation_policy=zircon_shader_pbr_viewer_ready_frame_v18"));
    assert!(metadata_text.contains("material_fixture=metal-mirror"));
    assert!(
        metadata_text.contains("required_material_base_pipeline_kind=environment-only-pbr-base")
    );
    assert!(metadata_text.contains("required_material_base_pipeline_ready_at_capture=true"));
    assert!(metadata_text.contains(
        "source_manifest_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    ));
    assert!(metadata_text.contains("screenshot_presentation=cpu_readback"));
    assert!(metadata_text.contains("host_mode=offscreen-diagnostic"));
    assert!(metadata_text
        .contains("host_composition_id=zircon_shader_pbr_viewer_standalone_diagnostic_v1"));
    assert!(metadata_text.contains("scene_id=single_pbr_mirror_sphere"));
    assert!(metadata_text.contains("capture_target=offscreen-scene-renderer-cpu-readback"));
    assert!(metadata_text.contains("gpu_scene_surface_present_count=0"));
    assert!(metadata_text.contains("backend=Dx12"));
    assert!(metadata_text.contains("hdri_path=polyhaven_lakes_2k.hdr"));
    assert!(metadata_text.contains("requested_source_face_size=automatic"));
    assert!(metadata_text.contains("requested_pmrem_face_size=256"));
    assert!(metadata_text.contains("active_source_cubemap_face_size=512"));
    assert!(metadata_text.contains("active_source_cubemap_mip_count=10"));
    assert!(metadata_text.contains("active_pmrem_face_size=256"));
    assert!(metadata_text.contains("active_pmrem_mip_count=9"));
    assert!(metadata_text.contains("render_profile=environment_only_pbr_preview"));
    assert!(metadata_text.contains("environment_only_base_prewarm_pipeline_ready=false"));
    assert!(metadata_text.contains("environment_runtime_frame_generation=none"));
    assert!(metadata_text.contains("environment_reflection_probe_active_count=0"));
    assert!(metadata_text.contains("environment_reflection_probe_fragment_visit_upper_bound=0"));
    assert!(metadata_text.contains("environment_hydration_observation_epoch=0"));
    assert!(metadata_text.contains("environment_capture_observation_epoch=0"));
    assert!(metadata_text.contains("environment_capture_residency_observation_epoch=0"));
    assert!(metadata_text.contains("environment_capture_residency_last_published_handle=none"));
    assert!(metadata_text.contains("environment_cubemap_upload_resident_texture_bytes=0"));
    assert!(metadata_text.contains("environment_only_base_pipeline_ready_at_capture=true"));
    assert!(metadata_text.contains("environment_only_base_prewarm_cache_hit=false"));
    assert!(metadata_text
        .contains("environment_only_base_prewarm_cache_scope=process_local_mesh_pipeline_cache"));
    assert!(
        metadata_text.contains("environment_only_base_prewarm_shader_source_resolution_ns=2000000")
    );
    assert!(metadata_text.contains("environment_only_base_prewarm_pipeline_creation_ns=11000000"));
    assert!(metadata_text.contains("environment_only_base_prewarm_elapsed_ns=13000000"));
    assert!(metadata_text.contains("interactive_direct_present_enabled=false"));
    assert!(metadata_text.contains(&format!(
        "ibl_bake_algorithm_version={IBL_BAKE_ALGORITHM_VERSION}"
    )));
    assert!(metadata_text.contains("ibl_staging_status=Written"));
    assert!(metadata_text.contains("ibl_staging_source_decode_ns=1000000"));
    assert!(metadata_text.contains("ibl_staging_cubemap_build_ns=18000000"));
    assert!(metadata_text.contains("ibl_staging_equirect_projection_ns=3000000"));
    assert!(metadata_text.contains("ibl_staging_source_mip_build_ns=4000000"));
    assert!(metadata_text.contains("ibl_staging_pmrem_build_ns=5000000"));
    assert!(metadata_text.contains("ibl_staging_sh9_build_ns=6000000"));
    assert!(metadata_text.contains("ibl_staging_irradiance_cube_build_ns=7000000"));
    assert!(metadata_text.contains("ibl_staging_bundle_write_ns=8000000"));
    assert!(metadata_text.contains("ibl_staging_source_zcube_bytes=1024"));
    assert!(metadata_text.contains("ibl_staging_asset_derived_bytes=2048"));
    assert!(metadata_text.contains("ibl_staging_parallel_executor_work_items=42"));
    assert!(metadata_text.contains("ibl_staging_equirect_projection_parallel_work_items=6"));
    assert!(metadata_text.contains("ibl_staging_source_mip_build_parallel_work_items=12"));
    assert!(metadata_text.contains("ibl_staging_pmrem_build_parallel_work_items=24"));
    assert!(metadata_text.contains("ibl_staging_irradiance_cube_build_parallel_work_items=0"));
    assert!(metadata_text.contains("ibl_staging_irradiance_cube_source_sample_visits=37748736"));
    assert!(metadata_text.contains("scene_startup_hdri_decode_ns=21000000"));
    assert!(metadata_text.contains("scene_startup_project_assets_ns=34000000"));
    assert!(metadata_text.contains("scene_startup_runtime_bootstrap_ns=55000000"));
    assert!(metadata_text.contains("scene_startup_project_open_ns=89000000"));
    assert!(metadata_text.contains("scene_startup_world_load_ns=144000000"));
    assert!(metadata_text.contains("scene_startup_renderer_initialization_ns=3600000000"));
    assert!(metadata_text.contains("scene_startup_renderer_backend_initialization_ns=377000000"));
    assert!(metadata_text
        .contains("scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized=true"));
    assert!(metadata_text.contains(
        "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns=8000000"
    ));
    assert!(metadata_text.contains(
        "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns=7000000"
    ));
    assert!(metadata_text.contains(
        "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns=1000000"
    ));
    assert!(metadata_text.contains("scene_startup_renderer_deferred_initialization_ns=1600000000"));
    assert!(
        metadata_text.contains("scene_startup_renderer_deferred_standard_pipeline_ns=987000000")
    );
    assert!(metadata_text.contains("scene_startup_resource_streamer_initialization_ns=1597000000"));
    assert!(metadata_text.contains("scene_startup_ibl_restore_ns=2584000000"));
    assert!(metadata_text.contains("scene_startup_total_ns=7000000000"));
    assert!(metadata_text.contains("one_shot_base_pipeline_wait_elapsed_ns=75000000"));
    assert!(metadata_text.contains("viewer_scene_load_elapsed_ns=7120000000"));
    assert!(metadata_text.contains("viewer_ready_elapsed_ns=7250000000"));
    assert!(metadata_text.contains("registered_pipeline_variant_count=16"));
    assert!(metadata_text.contains("registered_shader_variant_count=1"));
    assert!(metadata_text.contains("texture_presence_normalized_pipeline_variant_count=1"));
    assert!(metadata_text.contains("texture_presence_equivalent_pipeline_variant_count=15"));
    assert!(metadata_text.contains("cached_render_pipeline_count=8"));
    assert!(metadata_text.contains("cached_shader_module_count=2"));
    assert!(metadata_text.contains("render_pipeline_creation_count=8"));
    assert!(metadata_text.contains("shader_module_creation_count=2"));
    assert!(metadata_text.contains("render_pipeline_creation_cpu_microseconds=42000"));
    assert!(metadata_text.contains("shader_module_creation_cpu_microseconds=17000"));
    assert!(metadata_text.contains("async_base_pipeline_queue_wait_count=1"));
    assert!(metadata_text.contains("async_base_pipeline_queue_wait_microseconds=88"));
}

#[test]
fn ready_frame_evidence_rejects_invalid_pixels_without_leaving_a_sidecar() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path = viewer_test_artifact_root("invalid-ready-evidence").join(format!("{unique}.png"));
    let metadata = ReadyFrameEvidenceMetadata {
        backend: "Dx12".to_owned(),
        evidence_identity: ready_frame_evidence_identity_fixture(),
        interactive_direct_present_enabled: false,
        host_mode: "offscreen-diagnostic".to_owned(),
        host_composition_id: "zircon_shader_pbr_viewer_standalone_diagnostic_v1".to_owned(),
        scene_id: "single_pbr_mirror_sphere".to_owned(),
        capture_target: "offscreen-scene-renderer-cpu-readback".to_owned(),
        gpu_scene_surface_present_count: 0,
        hdri_path: "test.hdr".to_owned(),
        requested_source_face_size: Some(64),
        requested_pmrem_face_size: Some(64),
        active_source_cubemap_face_size: 64,
        active_source_cubemap_mip_count: 7,
        active_pmrem_face_size: 64,
        active_pmrem_mip_count: 7,
        render_profile: "environment_only_pbr_preview".to_owned(),
        material_fixture: "metal-mirror".to_owned(),
        required_material_base_pipeline_kind: "environment-only-pbr-base".to_owned(),
        required_material_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_requested: true,
        environment_only_base_prewarm_pipeline_ready: true,
        environment_only_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_cache_hit: true,
        environment_only_base_prewarm_cache_scope: ENVIRONMENT_ONLY_BASE_PREWARM_CACHE_SCOPE
            .to_owned(),
        environment_only_base_prewarm_shader_source_resolution: Duration::ZERO,
        environment_only_base_prewarm_pipeline_creation: Duration::ZERO,
        environment_only_base_prewarm_elapsed: Duration::ZERO,
        camera_yaw_degrees: 0.0,
        camera_pitch_degrees: 0.0,
        ibl_bake_algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
        ibl_staging_status: "Written".to_owned(),
        ibl_staging_elapsed: Duration::ZERO,
        ibl_staging_source_decode: Duration::ZERO,
        ibl_staging_cubemap_build: Duration::ZERO,
        ibl_staging_equirect_projection: Duration::ZERO,
        ibl_staging_source_mip_build: Duration::ZERO,
        ibl_staging_pmrem_build: Duration::ZERO,
        ibl_staging_sh9_build: Duration::ZERO,
        ibl_staging_irradiance_cube_build: Duration::ZERO,
        ibl_staging_bundle_write: Duration::ZERO,
        ibl_staging_source_zcube_bytes: 0,
        ibl_staging_asset_derived_bytes: 0,
        ibl_staging_parallel_executor_work_items: 0,
        ibl_staging_equirect_projection_parallel_work_items: 0,
        ibl_staging_source_mip_build_parallel_work_items: 0,
        ibl_staging_pmrem_build_parallel_work_items: 0,
        ibl_staging_irradiance_cube_build_parallel_work_items: 0,
        ibl_staging_irradiance_cube_source_sample_visits: 0,
        ibl_total_elapsed: Duration::ZERO,
        scene_startup_hdri_decode: Duration::ZERO,
        scene_startup_project_assets: Duration::ZERO,
        scene_startup_runtime_bootstrap: Duration::ZERO,
        scene_startup_project_open: Duration::ZERO,
        scene_startup_world_load: Duration::ZERO,
        scene_startup_renderer_initialization: Duration::ZERO,
        scene_startup_renderer_backend_initialization: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized: false,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_texture_upload_submission: Duration::ZERO,
        scene_startup_renderer_deferred_initialization: Duration::ZERO,
        scene_startup_renderer_deferred_standard_pipeline: Duration::ZERO,
        scene_startup_resource_streamer_initialization: Duration::ZERO,
        scene_startup_ibl_restore: Duration::ZERO,
        scene_startup_total: Duration::ZERO,
        one_shot_base_pipeline_wait_elapsed: Duration::ZERO,
        viewer_scene_load_elapsed: Duration::ZERO,
        viewer_ready_elapsed: Duration::ZERO,
        ready_frame_render_elapsed: Duration::ZERO,
        ready_frame_render_extract: Duration::ZERO,
        ready_frame_renderer_call: Duration::ZERO,
        ready_frame_readback_and_completion: Duration::ZERO,
        environment_runtime_snapshot: environment_runtime_snapshot_fixture(),
        shader_variant_miss_report: ShaderVariantMissReport::default(),
    };

    let error = write_ready_frame_evidence(&path, 2, 1, &[0, 0, 0, 255], &metadata)
        .expect_err("invalid pixels must not create screenshot evidence");

    assert!(error.contains("does not match 2x1 output"));
    assert!(!path.exists());
    assert!(!ready_frame_evidence_metadata_path(&path).unwrap().exists());
    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");
}

#[test]
fn ready_frame_evidence_removes_png_when_sidecar_write_fails() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path =
        viewer_test_artifact_root("ready-frame-sidecar-failure").join(format!("{unique}.png"));
    let metadata_path = ready_frame_evidence_metadata_path(&path).unwrap();
    let metadata = ReadyFrameEvidenceMetadata {
        backend: "Dx12".to_owned(),
        evidence_identity: ready_frame_evidence_identity_fixture(),
        interactive_direct_present_enabled: false,
        host_mode: "offscreen-diagnostic".to_owned(),
        host_composition_id: "zircon_shader_pbr_viewer_standalone_diagnostic_v1".to_owned(),
        scene_id: "single_pbr_mirror_sphere".to_owned(),
        capture_target: "offscreen-scene-renderer-cpu-readback".to_owned(),
        gpu_scene_surface_present_count: 0,
        hdri_path: "test.hdr".to_owned(),
        requested_source_face_size: Some(64),
        requested_pmrem_face_size: Some(64),
        active_source_cubemap_face_size: 64,
        active_source_cubemap_mip_count: 7,
        active_pmrem_face_size: 64,
        active_pmrem_mip_count: 7,
        render_profile: "environment_only_pbr_preview".to_owned(),
        material_fixture: "metal-mirror".to_owned(),
        required_material_base_pipeline_kind: "environment-only-pbr-base".to_owned(),
        required_material_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_requested: true,
        environment_only_base_prewarm_pipeline_ready: true,
        environment_only_base_pipeline_ready_at_capture: true,
        environment_only_base_prewarm_cache_hit: true,
        environment_only_base_prewarm_cache_scope: ENVIRONMENT_ONLY_BASE_PREWARM_CACHE_SCOPE
            .to_owned(),
        environment_only_base_prewarm_shader_source_resolution: Duration::ZERO,
        environment_only_base_prewarm_pipeline_creation: Duration::ZERO,
        environment_only_base_prewarm_elapsed: Duration::ZERO,
        camera_yaw_degrees: 0.0,
        camera_pitch_degrees: 0.0,
        ibl_bake_algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
        ibl_staging_status: "Written".to_owned(),
        ibl_staging_elapsed: Duration::ZERO,
        ibl_staging_source_decode: Duration::ZERO,
        ibl_staging_cubemap_build: Duration::ZERO,
        ibl_staging_equirect_projection: Duration::ZERO,
        ibl_staging_source_mip_build: Duration::ZERO,
        ibl_staging_pmrem_build: Duration::ZERO,
        ibl_staging_sh9_build: Duration::ZERO,
        ibl_staging_irradiance_cube_build: Duration::ZERO,
        ibl_staging_bundle_write: Duration::ZERO,
        ibl_staging_source_zcube_bytes: 0,
        ibl_staging_asset_derived_bytes: 0,
        ibl_staging_parallel_executor_work_items: 0,
        ibl_staging_equirect_projection_parallel_work_items: 0,
        ibl_staging_source_mip_build_parallel_work_items: 0,
        ibl_staging_pmrem_build_parallel_work_items: 0,
        ibl_staging_irradiance_cube_build_parallel_work_items: 0,
        ibl_staging_irradiance_cube_source_sample_visits: 0,
        ibl_total_elapsed: Duration::ZERO,
        scene_startup_hdri_decode: Duration::ZERO,
        scene_startup_project_assets: Duration::ZERO,
        scene_startup_runtime_bootstrap: Duration::ZERO,
        scene_startup_project_open: Duration::ZERO,
        scene_startup_world_load: Duration::ZERO,
        scene_startup_renderer_initialization: Duration::ZERO,
        scene_startup_renderer_backend_initialization: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized: false,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization: Duration::ZERO,
        scene_startup_renderer_environment_brdf_lut_texture_upload_submission: Duration::ZERO,
        scene_startup_renderer_deferred_initialization: Duration::ZERO,
        scene_startup_renderer_deferred_standard_pipeline: Duration::ZERO,
        scene_startup_resource_streamer_initialization: Duration::ZERO,
        scene_startup_ibl_restore: Duration::ZERO,
        scene_startup_total: Duration::ZERO,
        one_shot_base_pipeline_wait_elapsed: Duration::ZERO,
        viewer_scene_load_elapsed: Duration::ZERO,
        viewer_ready_elapsed: Duration::ZERO,
        ready_frame_render_elapsed: Duration::ZERO,
        ready_frame_render_extract: Duration::ZERO,
        ready_frame_renderer_call: Duration::ZERO,
        ready_frame_readback_and_completion: Duration::ZERO,
        environment_runtime_snapshot: environment_runtime_snapshot_fixture(),
        shader_variant_miss_report: ShaderVariantMissReport::default(),
    };
    std::fs::create_dir(&metadata_path).expect("sidecar path directory should be created");

    let error = write_ready_frame_evidence(&path, 1, 1, &[0, 0, 0, 255], &metadata)
        .expect_err("a sidecar directory must reject metadata output");

    assert!(metadata_path.is_dir());
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&metadata_path);

    assert!(error.contains("write screenshot metadata"));
    assert!(!path.exists());
    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");
}

fn ready_frame_evidence_identity_fixture() -> ReadyFrameEvidenceIdentity {
    ReadyFrameEvidenceIdentity {
        identity_manifest: evidence_file_fingerprint_fixture("E:/evidence/identity.json", 'a'),
        run_id: "shader-pbr-test-ready-frame".to_owned(),
        validation_policy: READY_FRAME_EVIDENCE_VALIDATION_POLICY.to_owned(),
        source_manifest_sha256: "c".repeat(64),
        viewer_binary: evidence_file_fingerprint_fixture("E:/evidence/viewer.exe", 'd'),
        hdri: evidence_file_fingerprint_fixture("E:/evidence/lakes.hdr", 'e'),
        build_provenance: evidence_file_fingerprint_fixture(
            "E:/evidence/build-provenance.json",
            'f',
        ),
    }
}

fn evidence_file_fingerprint_fixture(path: &str, marker: char) -> EvidenceFileFingerprint {
    EvidenceFileFingerprint {
        path: std::path::PathBuf::from(path),
        sha256: marker.to_string().repeat(64),
        byte_length: 4096,
    }
}

fn shader_variant_miss_report_fixture() -> ShaderVariantMissReport {
    let mut report = ShaderVariantMissReport::default();
    report.record_registered_variant_counts(16, 1, 1);
    report.record_cached_gpu_object_counts(8, 2);
    report.record_gpu_object_creation_totals(8, 2, 42_000, 17_000);
    report.record_async_base_pipeline_queue_wait_totals(1, 88);
    report
}

fn environment_runtime_snapshot_fixture() -> EnvironmentRuntimeSnapshot {
    EnvironmentRuntimeSnapshot {
        frame_generation: None,
        frame_profile: None,
        scene_submission: Default::default(),
        reflection_probes: Default::default(),
        realtime_ibl: RealtimeIblStatusReport {
            readiness: RealtimeIblReadiness::Fallback,
            current_frame_number: 0,
            published_key: None,
            pending_key: None,
            queued_key: None,
            published_generation_frame_number: None,
            last_good_age_frame_count: None,
            active_generation_start_frame_number: None,
            active_generation_elapsed_frame_count: None,
            active_generation_coalesced_source_change_count: 0,
            failure: None,
        },
        hydration: Default::default(),
        capture: Default::default(),
        capture_residency: Default::default(),
        cubemap_upload: Default::default(),
    }
}
