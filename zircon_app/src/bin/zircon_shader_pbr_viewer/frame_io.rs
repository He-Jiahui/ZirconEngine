use crate::evidence_identity::{
    evidence_transport_path, fingerprint_file, EvidenceFileFingerprint, ReadyFrameEvidenceIdentity,
    READY_FRAME_EVIDENCE_IDENTITY_SCHEMA,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

use zircon_runtime::core::framework::render::ShaderVariantMissReport;
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

const READY_FRAME_EVIDENCE_SCHEMA: &str = "zircon_shader_pbr_viewer_ready_frame_evidence_v17";
// This reports only reuse inside the viewer's MeshPipelineCache, never a persisted driver PSO.
pub(crate) const ENVIRONMENT_ONLY_BASE_PREWARM_CACHE_SCOPE: &str =
    "process_local_mesh_pipeline_cache";
pub(crate) const ENVIRONMENT_ONLY_BASE_PREWARM_NOT_REQUESTED_CACHE_SCOPE: &str = "not_requested";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReadyFrameEvidenceMetadata {
    pub(crate) backend: String,
    pub(crate) evidence_identity: ReadyFrameEvidenceIdentity,
    pub(crate) interactive_direct_present_enabled: bool,
    pub(crate) host_mode: String,
    pub(crate) host_composition_id: String,
    pub(crate) scene_id: String,
    pub(crate) capture_target: String,
    pub(crate) gpu_scene_surface_present_count: u32,
    pub(crate) hdri_path: String,
    pub(crate) requested_source_face_size: Option<u32>,
    pub(crate) requested_pmrem_face_size: Option<u32>,
    pub(crate) active_source_cubemap_face_size: u32,
    pub(crate) active_source_cubemap_mip_count: u32,
    pub(crate) active_pmrem_face_size: u32,
    pub(crate) active_pmrem_mip_count: u32,
    pub(crate) render_profile: String,
    pub(crate) material_fixture: String,
    pub(crate) required_material_base_pipeline_kind: String,
    pub(crate) required_material_base_pipeline_ready_at_capture: bool,
    pub(crate) environment_only_base_prewarm_requested: bool,
    pub(crate) environment_only_base_prewarm_pipeline_ready: bool,
    pub(crate) environment_only_base_pipeline_ready_at_capture: bool,
    pub(crate) environment_only_base_prewarm_cache_hit: bool,
    pub(crate) environment_only_base_prewarm_cache_scope: String,
    pub(crate) environment_only_base_prewarm_shader_source_resolution: Duration,
    pub(crate) environment_only_base_prewarm_pipeline_creation: Duration,
    pub(crate) environment_only_base_prewarm_elapsed: Duration,
    pub(crate) camera_yaw_degrees: f32,
    pub(crate) camera_pitch_degrees: f32,
    pub(crate) ibl_bake_algorithm_version: u64,
    pub(crate) ibl_staging_status: String,
    pub(crate) ibl_staging_elapsed: Duration,
    pub(crate) ibl_staging_source_decode: Duration,
    pub(crate) ibl_staging_cubemap_build: Duration,
    pub(crate) ibl_staging_equirect_projection: Duration,
    pub(crate) ibl_staging_source_mip_build: Duration,
    pub(crate) ibl_staging_pmrem_build: Duration,
    pub(crate) ibl_staging_sh9_build: Duration,
    pub(crate) ibl_staging_irradiance_cube_build: Duration,
    pub(crate) ibl_staging_bundle_write: Duration,
    pub(crate) ibl_staging_source_zcube_bytes: u64,
    pub(crate) ibl_staging_asset_derived_bytes: u64,
    pub(crate) ibl_staging_parallel_executor_work_items: u64,
    pub(crate) ibl_staging_equirect_projection_parallel_work_items: u64,
    pub(crate) ibl_staging_source_mip_build_parallel_work_items: u64,
    pub(crate) ibl_staging_pmrem_build_parallel_work_items: u64,
    pub(crate) ibl_staging_irradiance_cube_build_parallel_work_items: u64,
    pub(crate) ibl_staging_irradiance_cube_source_sample_visits: u64,
    pub(crate) ibl_total_elapsed: Duration,
    pub(crate) scene_startup_hdri_decode: Duration,
    pub(crate) scene_startup_project_assets: Duration,
    pub(crate) scene_startup_runtime_bootstrap: Duration,
    pub(crate) scene_startup_project_open: Duration,
    pub(crate) scene_startup_world_load: Duration,
    pub(crate) scene_startup_renderer_initialization: Duration,
    pub(crate) scene_startup_renderer_backend_initialization: Duration,
    pub(crate) scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized: bool,
    pub(crate) scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait: Duration,
    pub(crate) scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization:
        Duration,
    pub(crate) scene_startup_renderer_environment_brdf_lut_texture_upload_submission: Duration,
    pub(crate) scene_startup_renderer_deferred_initialization: Duration,
    pub(crate) scene_startup_renderer_deferred_standard_pipeline: Duration,
    pub(crate) scene_startup_resource_streamer_initialization: Duration,
    pub(crate) scene_startup_ibl_restore: Duration,
    pub(crate) scene_startup_total: Duration,
    pub(crate) one_shot_base_pipeline_wait_elapsed: Duration,
    pub(crate) viewer_scene_load_elapsed: Duration,
    // Captured after the Ready frame renders, so async Base PSO admission is included.
    pub(crate) viewer_ready_elapsed: Duration,
    pub(crate) ready_frame_render_elapsed: Duration,
    pub(crate) ready_frame_render_extract: Duration,
    pub(crate) ready_frame_renderer_call: Duration,
    pub(crate) ready_frame_readback_and_completion: Duration,
    pub(crate) shader_variant_miss_report: ShaderVariantMissReport,
}

pub(crate) fn startup_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [10, 15, 21], [35, 59, 80])
}

pub(crate) fn error_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [42, 12, 18], [94, 30, 38])
}

pub(crate) fn write_ready_frame_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .map(usize::try_from)
        .transpose()
        .map_err(|error| format!("screenshot dimensions do not fit usize: {error}"))?
        .ok_or_else(|| "screenshot dimensions overflow".to_owned())?;
    if rgba.len() != expected_len {
        return Err(format!(
            "frame RGBA length {} does not match {width}x{height} output",
            rgba.len()
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!("create screenshot directory {}: {error}", parent.display())
        })?;
    }
    image::save_buffer_with_format(
        path,
        rgba,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| format!("encode screenshot {}: {error}", path.display()))
}

/// Writes the CPU-readback PNG and a matching provenance sidecar as one evidence unit.
pub(crate) fn write_ready_frame_evidence(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
    metadata: &ReadyFrameEvidenceMetadata,
) -> Result<PathBuf, String> {
    write_ready_frame_png(path, width, height, rgba)?;
    let screenshot_fingerprint = match fingerprint_file(path, "Ready-frame screenshot") {
        Ok(fingerprint) => fingerprint,
        Err(error) => {
            let _ = std::fs::remove_file(path);
            return Err(error);
        }
    };
    let metadata_path = match ready_frame_evidence_metadata_path(path) {
        Ok(path) => path,
        Err(error) => {
            let _ = std::fs::remove_file(path);
            return Err(error);
        }
    };
    if let Err(error) = write_ready_frame_metadata(
        &metadata_path,
        path,
        &screenshot_fingerprint,
        width,
        height,
        metadata,
    ) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(&metadata_path);
        return Err(error);
    }
    Ok(metadata_path)
}

fn ready_frame_evidence_metadata_path(path: &Path) -> Result<PathBuf, String> {
    let mut name = path
        .file_name()
        .ok_or_else(|| format!("screenshot path has no file name: {}", path.display()))?
        .to_os_string();
    name.push(".txt");
    Ok(path.with_file_name(name))
}

fn write_ready_frame_metadata(
    metadata_path: &Path,
    screenshot_path: &Path,
    screenshot_fingerprint: &EvidenceFileFingerprint,
    width: u32,
    height: u32,
    metadata: &ReadyFrameEvidenceMetadata,
) -> Result<(), String> {
    let screenshot_name = screenshot_path
        .file_name()
        .ok_or_else(|| {
            format!(
                "screenshot path has no file name: {}",
                screenshot_path.display()
            )
        })?
        .to_string_lossy();
    let contents = format!(
        "schema={READY_FRAME_EVIDENCE_SCHEMA}\n\
         screenshot={screenshot_name}\n\
         screenshot_sha256={}\n\
         screenshot_byte_length={}\n\
         evidence_identity_schema={}\n\
         evidence_run_id={}\n\
         evidence_validation_policy={}\n\
         evidence_identity_path={}\n\
         evidence_identity_sha256={}\n\
         evidence_identity_byte_length={}\n\
         viewer_binary_path={}\n\
         viewer_binary_sha256={}\n\
         viewer_binary_byte_length={}\n\
         hdri_sha256={}\n\
         hdri_byte_length={}\n\
         build_provenance_path={}\n\
         build_provenance_sha256={}\n\
         build_provenance_byte_length={}\n\
         source_manifest_sha256={}\n\
         screenshot_presentation=cpu_readback\n\
         interactive_direct_present_enabled={}\n\
         host_mode={}\n\
         host_composition_id={}\n\
         scene_id={}\n\
         capture_target={}\n\
         gpu_scene_surface_present_count={}\n\
         backend={}\n\
         hdri_path={}\n\
         requested_source_face_size={}\n\
         requested_pmrem_face_size={}\n\
         active_source_cubemap_face_size={}\n\
         active_source_cubemap_mip_count={}\n\
         active_pmrem_face_size={}\n\
         active_pmrem_mip_count={}\n\
         render_profile={}\n\
         material_fixture={}\n\
         required_material_base_pipeline_kind={}\n\
         required_material_base_pipeline_ready_at_capture={}\n\
         environment_only_base_prewarm_requested={}\n\
         environment_only_base_prewarm_pipeline_ready={}\n\
         environment_only_base_pipeline_ready_at_capture={}\n\
         environment_only_base_prewarm_cache_hit={}\n\
         environment_only_base_prewarm_cache_scope={}\n\
         environment_only_base_prewarm_shader_source_resolution_ns={}\n\
         environment_only_base_prewarm_pipeline_creation_ns={}\n\
         environment_only_base_prewarm_elapsed_ns={}\n\
         viewport={}x{}\n\
         camera_yaw_degrees={:.3}\n\
         camera_pitch_degrees={:.3}\n\
         ibl_bake_algorithm_version={}\n\
         ibl_staging_status={}\n\
         ibl_staging_elapsed_ns={}\n\
         ibl_staging_source_decode_ns={}\n\
         ibl_staging_cubemap_build_ns={}\n\
         ibl_staging_equirect_projection_ns={}\n\
         ibl_staging_source_mip_build_ns={}\n\
         ibl_staging_pmrem_build_ns={}\n\
         ibl_staging_sh9_build_ns={}\n\
         ibl_staging_irradiance_cube_build_ns={}\n\
         ibl_staging_bundle_write_ns={}\n\
         ibl_staging_source_zcube_bytes={}\n\
         ibl_staging_asset_derived_bytes={}\n\
         ibl_staging_parallel_executor_work_items={}\n\
         ibl_staging_equirect_projection_parallel_work_items={}\n\
         ibl_staging_source_mip_build_parallel_work_items={}\n\
         ibl_staging_pmrem_build_parallel_work_items={}\n\
         ibl_staging_irradiance_cube_build_parallel_work_items={}\n\
         ibl_staging_irradiance_cube_source_sample_visits={}\n\
         ibl_total_elapsed_ns={}\n\
         scene_startup_hdri_decode_ns={}\n\
         scene_startup_project_assets_ns={}\n\
         scene_startup_runtime_bootstrap_ns={}\n\
         scene_startup_project_open_ns={}\n\
         scene_startup_world_load_ns={}\n\
         scene_startup_renderer_initialization_ns={}\n\
         scene_startup_renderer_backend_initialization_ns={}\n\
         scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized={}\n\
         scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns={}\n\
         scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns={}\n\
         scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns={}\n\
         scene_startup_renderer_deferred_initialization_ns={}\n\
         scene_startup_renderer_deferred_standard_pipeline_ns={}\n\
         scene_startup_resource_streamer_initialization_ns={}\n\
         scene_startup_ibl_restore_ns={}\n\
         scene_startup_total_ns={}\n\
         one_shot_base_pipeline_wait_elapsed_ns={}\n\
         viewer_scene_load_elapsed_ns={}\n\
         viewer_ready_elapsed_ns={}\n\
         ready_frame_render_elapsed_ns={}\n\
         ready_frame_extract_ns={}\n\
         ready_frame_renderer_call_ns={}\n\
         ready_frame_readback_and_completion_ns={}\n\
         registered_pipeline_variant_count={}\n\
         registered_shader_variant_count={}\n\
         texture_presence_normalized_pipeline_variant_count={}\n\
         texture_presence_equivalent_pipeline_variant_count={}\n\
         cached_render_pipeline_count={}\n\
         cached_shader_module_count={}\n\
         render_pipeline_creation_count={}\n\
         shader_module_creation_count={}\n\
         render_pipeline_creation_cpu_microseconds={}\n\
         shader_module_creation_cpu_microseconds={}\n\
         async_base_pipeline_queue_wait_count={}\n\
         async_base_pipeline_queue_wait_microseconds={}\n",
        screenshot_fingerprint.sha256,
        screenshot_fingerprint.byte_length,
        READY_FRAME_EVIDENCE_IDENTITY_SCHEMA,
        metadata.evidence_identity.run_id,
        metadata.evidence_identity.validation_policy,
        evidence_transport_path(&metadata.evidence_identity.identity_manifest.path).display(),
        metadata.evidence_identity.identity_manifest.sha256,
        metadata.evidence_identity.identity_manifest.byte_length,
        evidence_transport_path(&metadata.evidence_identity.viewer_binary.path).display(),
        metadata.evidence_identity.viewer_binary.sha256,
        metadata.evidence_identity.viewer_binary.byte_length,
        metadata.evidence_identity.hdri.sha256,
        metadata.evidence_identity.hdri.byte_length,
        evidence_transport_path(&metadata.evidence_identity.build_provenance.path).display(),
        metadata.evidence_identity.build_provenance.sha256,
        metadata.evidence_identity.build_provenance.byte_length,
        metadata.evidence_identity.source_manifest_sha256,
        metadata.interactive_direct_present_enabled,
        metadata.host_mode,
        metadata.host_composition_id,
        metadata.scene_id,
        metadata.capture_target,
        metadata.gpu_scene_surface_present_count,
        metadata.backend,
        metadata.hdri_path,
        face_size_label(metadata.requested_source_face_size),
        face_size_label(metadata.requested_pmrem_face_size),
        metadata.active_source_cubemap_face_size,
        metadata.active_source_cubemap_mip_count,
        metadata.active_pmrem_face_size,
        metadata.active_pmrem_mip_count,
        metadata.render_profile,
        metadata.material_fixture,
        metadata.required_material_base_pipeline_kind,
        metadata.required_material_base_pipeline_ready_at_capture,
        metadata.environment_only_base_prewarm_requested,
        metadata.environment_only_base_prewarm_pipeline_ready,
        metadata.environment_only_base_pipeline_ready_at_capture,
        metadata.environment_only_base_prewarm_cache_hit,
        metadata.environment_only_base_prewarm_cache_scope,
        metadata
            .environment_only_base_prewarm_shader_source_resolution
            .as_nanos(),
        metadata
            .environment_only_base_prewarm_pipeline_creation
            .as_nanos(),
        metadata.environment_only_base_prewarm_elapsed.as_nanos(),
        width,
        height,
        metadata.camera_yaw_degrees,
        metadata.camera_pitch_degrees,
        metadata.ibl_bake_algorithm_version,
        metadata.ibl_staging_status,
        metadata.ibl_staging_elapsed.as_nanos(),
        metadata.ibl_staging_source_decode.as_nanos(),
        metadata.ibl_staging_cubemap_build.as_nanos(),
        metadata.ibl_staging_equirect_projection.as_nanos(),
        metadata.ibl_staging_source_mip_build.as_nanos(),
        metadata.ibl_staging_pmrem_build.as_nanos(),
        metadata.ibl_staging_sh9_build.as_nanos(),
        metadata.ibl_staging_irradiance_cube_build.as_nanos(),
        metadata.ibl_staging_bundle_write.as_nanos(),
        metadata.ibl_staging_source_zcube_bytes,
        metadata.ibl_staging_asset_derived_bytes,
        metadata.ibl_staging_parallel_executor_work_items,
        metadata.ibl_staging_equirect_projection_parallel_work_items,
        metadata.ibl_staging_source_mip_build_parallel_work_items,
        metadata.ibl_staging_pmrem_build_parallel_work_items,
        metadata.ibl_staging_irradiance_cube_build_parallel_work_items,
        metadata.ibl_staging_irradiance_cube_source_sample_visits,
        metadata.ibl_total_elapsed.as_nanos(),
        metadata.scene_startup_hdri_decode.as_nanos(),
        metadata.scene_startup_project_assets.as_nanos(),
        metadata.scene_startup_runtime_bootstrap.as_nanos(),
        metadata.scene_startup_project_open.as_nanos(),
        metadata.scene_startup_world_load.as_nanos(),
        metadata.scene_startup_renderer_initialization.as_nanos(),
        metadata
            .scene_startup_renderer_backend_initialization
            .as_nanos(),
        metadata.scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized,
        metadata
            .scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait
            .as_nanos(),
        metadata
            .scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization
            .as_nanos(),
        metadata
            .scene_startup_renderer_environment_brdf_lut_texture_upload_submission
            .as_nanos(),
        metadata
            .scene_startup_renderer_deferred_initialization
            .as_nanos(),
        metadata
            .scene_startup_renderer_deferred_standard_pipeline
            .as_nanos(),
        metadata
            .scene_startup_resource_streamer_initialization
            .as_nanos(),
        metadata.scene_startup_ibl_restore.as_nanos(),
        metadata.scene_startup_total.as_nanos(),
        metadata.one_shot_base_pipeline_wait_elapsed.as_nanos(),
        metadata.viewer_scene_load_elapsed.as_nanos(),
        metadata.viewer_ready_elapsed.as_nanos(),
        metadata.ready_frame_render_elapsed.as_nanos(),
        metadata.ready_frame_render_extract.as_nanos(),
        metadata.ready_frame_renderer_call.as_nanos(),
        metadata.ready_frame_readback_and_completion.as_nanos(),
        metadata
            .shader_variant_miss_report
            .registered_pipeline_variant_count,
        metadata
            .shader_variant_miss_report
            .registered_shader_variant_count,
        metadata
            .shader_variant_miss_report
            .texture_presence_normalized_pipeline_variant_count,
        metadata
            .shader_variant_miss_report
            .texture_presence_equivalent_pipeline_variant_count,
        metadata
            .shader_variant_miss_report
            .cached_render_pipeline_count,
        metadata
            .shader_variant_miss_report
            .cached_shader_module_count,
        metadata
            .shader_variant_miss_report
            .render_pipeline_creation_count,
        metadata
            .shader_variant_miss_report
            .shader_module_creation_count,
        metadata
            .shader_variant_miss_report
            .render_pipeline_creation_cpu_microseconds,
        metadata
            .shader_variant_miss_report
            .shader_module_creation_cpu_microseconds,
        metadata
            .shader_variant_miss_report
            .async_base_pipeline_queue_wait_count,
        metadata
            .shader_variant_miss_report
            .async_base_pipeline_queue_wait_microseconds,
    );
    std::fs::write(metadata_path, contents).map_err(|error| {
        format!(
            "write screenshot metadata {}: {error}",
            metadata_path.display()
        )
    })
}

fn face_size_label(face_size: Option<u32>) -> String {
    face_size.map_or_else(|| "automatic".to_owned(), |size| size.to_string())
}

fn status_frame(size: UVec2, top: [u8; 3], bottom: [u8; 3]) -> ViewportFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let t = y as f32 / height.saturating_sub(1).max(1) as f32;
        for x in 0..width {
            let shimmer = if ((x / 18) + (y / 18)) & 1 == 0 { 6 } else { 0 };
            rgba.push(lerp_u8(top[0], bottom[0], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[1], bottom[1], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[2], bottom[2], t).saturating_add(shimmer));
            rgba.push(255);
        }
    }
    ViewportFrame {
        width,
        height,
        rgba,
        generation: 0,
        capture_report: Default::default(),
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{
        error_frame, ready_frame_evidence_metadata_path, startup_frame, write_ready_frame_evidence,
        write_ready_frame_png, ReadyFrameEvidenceMetadata,
    };
    use std::time::Duration;
    use zircon_runtime::core::framework::render::{
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
        assert!(metadata_text.contains("schema=zircon_shader_pbr_viewer_ready_frame_evidence_v17"));
        assert!(metadata_text
            .contains("evidence_identity_schema=zircon_shader_pbr_viewer_evidence_identity_v1"));
        assert!(metadata_text.contains("evidence_run_id=shader-pbr-test-ready-frame"));
        assert!(metadata_text
            .contains("evidence_validation_policy=zircon_shader_pbr_viewer_ready_frame_v17"));
        assert!(metadata_text.contains("material_fixture=metal-mirror"));
        assert!(metadata_text
            .contains("required_material_base_pipeline_kind=environment-only-pbr-base"));
        assert!(metadata_text.contains("required_material_base_pipeline_ready_at_capture=true"));
        assert!(metadata_text.contains("source_manifest_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"));
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
        assert!(metadata_text.contains("environment_only_base_pipeline_ready_at_capture=true"));
        assert!(metadata_text.contains("environment_only_base_prewarm_cache_hit=false"));
        assert!(metadata_text.contains(
            "environment_only_base_prewarm_cache_scope=process_local_mesh_pipeline_cache"
        ));
        assert!(metadata_text
            .contains("environment_only_base_prewarm_shader_source_resolution_ns=2000000"));
        assert!(
            metadata_text.contains("environment_only_base_prewarm_pipeline_creation_ns=11000000")
        );
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
        assert!(
            metadata_text.contains("scene_startup_renderer_backend_initialization_ns=377000000")
        );
        assert!(metadata_text.contains(
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized=true"
        ));
        assert!(metadata_text.contains(
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns=8000000"
        ));
        assert!(metadata_text
            .contains("scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns=7000000"));
        assert!(metadata_text.contains(
            "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns=1000000"
        ));
        assert!(
            metadata_text.contains("scene_startup_renderer_deferred_initialization_ns=1600000000")
        );
        assert!(metadata_text
            .contains("scene_startup_renderer_deferred_standard_pipeline_ns=987000000"));
        assert!(
            metadata_text.contains("scene_startup_resource_streamer_initialization_ns=1597000000")
        );
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
        let path =
            viewer_test_artifact_root("invalid-ready-evidence").join(format!("{unique}.png"));
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
            scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization:
                Duration::ZERO,
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
            scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization:
                Duration::ZERO,
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
}
