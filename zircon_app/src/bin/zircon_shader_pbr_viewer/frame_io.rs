use crate::evidence_identity::{
    evidence_transport_path, fingerprint_file, EvidenceFileFingerprint, ReadyFrameEvidenceIdentity,
    READY_FRAME_EVIDENCE_IDENTITY_SCHEMA,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

use zircon_runtime::core::framework::render::{
    EnvironmentRuntimeSnapshot, RenderEnvironmentCaptureHandle, ShaderVariantMissReport,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

const READY_FRAME_EVIDENCE_SCHEMA: &str = "zircon_shader_pbr_viewer_ready_frame_evidence_v18";
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
    pub(crate) environment_runtime_snapshot: EnvironmentRuntimeSnapshot,
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
         environment_runtime_frame_generation={}\n\
         environment_reflection_probe_extracted_count={}\n\
         environment_reflection_probe_active_count={}\n\
         environment_reflection_probe_capacity_dropped_candidate_count={}\n\
         environment_reflection_probe_scheduled_cubemap_upload_bytes={}\n\
         environment_reflection_probe_asset_load_call_count={}\n\
         environment_reflection_probe_asset_load_cpu_time_us={}\n\
         environment_reflection_probe_fragment_visit_upper_bound={}\n\
         environment_hydration_observation_epoch={}\n\
         environment_hydration_resident_payload_bytes={}\n\
         environment_capture_observation_epoch={}\n\
         environment_capture_residency_observation_epoch={}\n\
         environment_capture_residency_last_published_handle={}\n\
         environment_capture_residency_last_published_output_generation={}\n\
         environment_capture_residency_resident_gpu_bytes={}\n\
         environment_capture_residency_eviction_count={}\n\
         environment_cubemap_upload_observation_epoch={}\n\
         environment_cubemap_upload_resident_texture_bytes={}\n\
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
        optional_u64_label(metadata.environment_runtime_snapshot.frame_generation),
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .extracted_probe_count,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .active_probe_count,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .capacity_dropped_candidate_count,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .scheduled_cubemap_upload_bytes,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .asset_load_call_count,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .asset_load_cpu_time_us,
        metadata
            .environment_runtime_snapshot
            .reflection_probes
            .full_resolution_fragment_probe_visit_upper_bound,
        metadata
            .environment_runtime_snapshot
            .hydration
            .observation_epoch,
        metadata
            .environment_runtime_snapshot
            .hydration
            .resident_payload_bytes,
        metadata
            .environment_runtime_snapshot
            .capture
            .observation_epoch,
        metadata
            .environment_runtime_snapshot
            .capture_residency
            .observation_epoch,
        optional_capture_handle_label(
            metadata
                .environment_runtime_snapshot
                .capture_residency
                .last_published_handle,
        ),
        optional_u64_label(
            metadata
                .environment_runtime_snapshot
                .capture_residency
                .last_published_output_generation,
        ),
        metadata
            .environment_runtime_snapshot
            .capture_residency
            .resident_gpu_bytes,
        metadata
            .environment_runtime_snapshot
            .capture_residency
            .eviction_count,
        metadata
            .environment_runtime_snapshot
            .cubemap_upload
            .observation_epoch,
        metadata
            .environment_runtime_snapshot
            .cubemap_upload
            .resident_texture_bytes,
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

fn optional_u64_label(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_capture_handle_label(value: Option<RenderEnvironmentCaptureHandle>) -> String {
    value.map_or_else(|| "none".to_owned(), |handle| handle.get().to_string())
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
#[path = "tests/frame_io.rs"]
mod tests;
