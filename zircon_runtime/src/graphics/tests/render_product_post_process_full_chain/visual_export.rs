use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, RenderFrameProfile, RenderFramework, RenderGpuTimingStatus, RenderStats,
    RenderSubmissionConfig, RenderViewportDescriptor,
};
use crate::core::math::UVec2;
use crate::graphics::backend::configure_renderdoc_capture_file_path_template;
use crate::graphics::debug_markers;

use super::fixture::{
    full_chain_material, full_chain_product_extract, insert_user_lut_texture,
    register_full_chain_material,
};
use super::{
    assert_terminal_signal_covers_frame, assert_terminal_signal_has_chromatic_content,
    assert_transient_texture_pool_aliases_logical_resources, create_full_chain_product_viewport,
    full_chain_product_framework,
};

#[test]
#[ignore = "writes Render17 cold/warm WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_render17_pfm1_render_graph_cold_warm_wgpu_png() {
    let viewport_size = UVec2::new(320, 240);
    let output_directory = render_evidence_directory();
    std::fs::create_dir_all(&output_directory)
        .expect("Render17 RenderDoc evidence directory must be creatable");
    let rdc_template = renderdoc_capture_template(&output_directory);
    assert!(
        configure_renderdoc_capture_file_path_template(&rdc_template)
            .expect("Render17 capture output template must be valid UTF-8"),
        "Render17 evidence export must run with RenderDoc injected before WGPU initialization"
    );
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/plan01_full_chain_receiver.zmaterial",
        full_chain_material(
            "Plan01FullChainReceiver",
            [0.03, 0.04, 0.055, 1.0],
            1.0,
            0.04,
            [0.0, 0.0, 0.0],
            false,
            true,
        ),
    );
    let caster_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/plan01_full_chain_caster.zmaterial",
        full_chain_material(
            "Plan01FullChainCaster",
            [1.0, 0.18, 0.04, 1.0],
            0.0,
            0.22,
            [4.2, 0.32, 0.10],
            false,
            false,
        ),
    );
    let user_lut = insert_user_lut_texture(
        asset_manager.as_ref(),
        "res://textures/plan01_full_chain_lut.png",
    );
    let framework = full_chain_product_framework(asset_manager);
    framework
        .set_submission_config(
            RenderSubmissionConfig::synchronous()
                .with_parallel_recording(1)
                .with_gpu_timing(),
        )
        .expect("Render17 evidence must explicitly enable parallel recording and GPU timing");
    let viewport = create_full_chain_product_viewport(
        &framework,
        viewport_size,
        "plan17-pfm1-render-graph-cold-warm",
        true,
    );
    let extract = full_chain_product_extract(
        viewport_size,
        receiver_material,
        caster_material,
        user_lut,
        true,
    );

    assert!(
        !framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending,
        "the cold/warm exporter arms its own non-contiguous RenderDoc captures; leave ZR_RENDERDOC_CAPTURE_FRAME_COUNT and ZR_RENDERDOC_CAPTURE_NEXT unset (or set ZR_RENDERDOC_CAPTURE_FRAME_COUNT=0)"
    );
    framework
        .request_graphics_debugger_capture(viewport)
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first = framework.query_stats().unwrap();
    let cold_capture_status = framework.query_graphics_debugger_status().unwrap();
    assert!(!cold_capture_status.capture_pending);
    assert_eq!(
        cold_capture_status.last_capture_frame,
        first.last_generation
    );
    assert_eq!(
        cold_capture_status.last_error, None,
        "Render17 cold RenderDoc capture must stop without an error"
    );
    let cold = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Render17 PF-M1 cold frame should be capturable");
    let cold_profile = captured_frame_profile(&cold, "cold");
    assert_eq!(cold_profile.frame_generation, cold.generation);
    assert_eq!(first.last_generation, Some(cold_profile.frame_generation));
    assert_capture_profile_observability(&cold_profile, &first, "cold");
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let history_ready = framework.query_stats().unwrap();

    // The first full-chain frame has no temporal history. The second frame
    // intentionally compiles the one history-enabled graph variant; the third
    // frame below must reuse that settled variant.
    assert_eq!(
        history_ready.last_graph_compiled_cache_miss_count,
        first.last_graph_compiled_cache_miss_count + 1,
        "the initial temporal-history transition must compile exactly one final graph variant"
    );
    assert!(
        history_ready.last_graph_compiled_cache_hit_count
            > first.last_graph_compiled_cache_hit_count,
        "the history transition must still reuse the unchanged base graph"
    );
    // The history-ready frame intentionally compiles the final variant. Capture the following
    // frame so the RenderDoc pair contains a true cold frame and a settled warm frame.
    framework
        .request_graphics_debugger_capture(viewport)
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
    framework.submit_frame_extract(viewport, extract).unwrap();
    let warm = framework.query_stats().unwrap();
    let capture_status = framework.query_graphics_debugger_status().unwrap();
    assert!(!capture_status.capture_pending);
    assert_eq!(capture_status.last_capture_frame, warm.last_generation);
    assert_eq!(
        capture_status.last_error, None,
        "Render17 warm RenderDoc capture must stop without an error"
    );
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Render17 PF-M1 full-chain frame should be capturable");
    let graph_dump = frame
        .graph_dump
        .as_deref()
        .expect("Render17 capture must retain the compiled graph dump");
    let profile = captured_frame_profile(&frame, "settled warm");
    assert_capture_profile_observability(&profile, &warm, "settled warm");

    assert_eq!(
        warm.last_graph_compiled_cache_miss_count,
        history_ready.last_graph_compiled_cache_miss_count,
        "the settled warm frame must not compile another graph variant"
    );
    assert!(
        warm.last_graph_compiled_cache_hit_count
            > history_ready.last_graph_compiled_cache_hit_count,
        "the settled warm frame must reuse at least one compiled graph: history_ready_hits={}, warm_hits={}",
        history_ready.last_graph_compiled_cache_hit_count,
        warm.last_graph_compiled_cache_hit_count,
    );
    let pool = warm
        .last_graph_execution_resource_report
        .transient_pool_report;
    assert!(pool.texture_reused_count > 0 || pool.buffer_reused_count > 0);
    assert_transient_texture_pool_aliases_logical_resources(&warm);
    assert_eq!(
        warm.last_graph_executed_debug_markers.len(),
        warm.last_graph_executed_pass_count
    );
    assert_eq!(
        profile
            .passes
            .iter()
            .map(|pass| debug_markers::marker_for_render_graph_pass(&pass.pass_name))
            .collect::<Vec<_>>(),
        warm.last_graph_executed_debug_markers,
        "the capture profile pass names and emitted RenderDoc graph markers must stay aligned",
    );
    assert_eq!(profile.frame_generation, frame.generation);
    assert_eq!(warm.last_generation, Some(profile.frame_generation));
    assert_eq!(profile.passes.len(), warm.last_graph_executed_pass_count);
    assert_eq!(
        profile
            .passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        warm.last_graph_executed_passes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    assert!(profile
        .passes
        .iter()
        .all(|pass| graph_dump.contains(&pass.pass_name)));
    assert_terminal_signal_covers_frame(&frame);
    assert_terminal_signal_has_chromatic_content(
        &frame,
        None,
        Some(format!("warm={:?}", warm.last_exposure_readback_report)),
    );

    let cold_output = output_directory
        .join("plan17_pfm1_render_graph_cold_warm_wgpu_current_source_20260801_cold.png");
    let output = output_directory
        .join("plan17_pfm1_render_graph_cold_warm_wgpu_current_source_20260801.png");
    write_png(&cold_output, &cold.rgba, cold.width, cold.height);
    write_png(&output, &frame.rgba, frame.width, frame.height);
    assert!(
        cold_output.is_file(),
        "missing cold visual evidence: {}",
        cold_output.display()
    );
    assert!(
        output.is_file(),
        "missing warm visual evidence: {}",
        output.display()
    );
    let rdc_files = renderdoc_capture_files(&output_directory, &rdc_template);
    assert_eq!(
        rdc_files.len(),
        2,
        "expected one cold and one settled-warm RenderDoc capture for template {}: {rdc_files:?}",
        rdc_template.display(),
    );
    let evidence_output = output_directory
        .join("plan17_pfm1_render_graph_cold_warm_wgpu_current_source_20260801.json");
    write_capture_evidence(
        &evidence_output,
        viewport_size,
        &cold_output,
        &cold_profile,
        &first,
        &output,
        &profile,
        &warm,
        &rdc_files,
    );
    assert!(
        evidence_output.is_file(),
        "missing Render17 profile evidence: {}",
        evidence_output.display()
    );
}

fn captured_frame_profile(captured_frame: &CapturedFrame, phase: &str) -> RenderFrameProfile {
    serde_json::from_str(
        captured_frame
            .frame_profile_json
            .as_deref()
            .unwrap_or_else(|| {
                panic!("Render17 {phase} capture must retain the frame profile JSON")
            }),
    )
    .unwrap_or_else(|error| {
        panic!("Render17 {phase} capture frame profile JSON must remain decodable: {error}")
    })
}

fn assert_capture_profile_observability(
    profile: &RenderFrameProfile,
    stats: &RenderStats,
    phase: &str,
) {
    assert_ne!(
        profile.gpu_timing_status,
        RenderGpuTimingStatus::Disabled,
        "Render17 {phase} evidence explicitly enables GPU timing and must report its observation state"
    );
    let parallel = &stats.last_graph_parallel_recording_report;
    assert_eq!(
        profile.parallel_recording_eligible_stage_count,
        parallel.eligible_stage_count.min(u32::MAX as usize) as u32,
        "Render17 {phase} evidence must retain the graph's parallel-recording eligibility"
    );
    assert_eq!(
        profile.parallel_recording_eligible_bucket_count,
        parallel.eligible_bucket_count.min(u32::MAX as usize) as u32,
        "Render17 {phase} evidence must retain the graph's parallel-recording buckets"
    );
    assert_eq!(
        profile.parallel_recording_executed_stage_count,
        parallel.executed_stage_count.min(u32::MAX as usize) as u32,
        "Render17 {phase} evidence must distinguish eligible from executed parallel stages"
    );
    assert_eq!(
        profile.parallel_recording_executed_bucket_count,
        parallel.executed_bucket_count.min(u32::MAX as usize) as u32,
        "Render17 {phase} evidence must distinguish eligible from executed parallel buckets"
    );

    let recorded_passes = &stats.last_graph_execution_profile_report.pass_profiles;
    assert_eq!(
        profile.passes.len(),
        recorded_passes.len(),
        "Render17 {phase} evidence must retain every graph pass profile"
    );
    for (captured, recorded) in profile.passes.iter().zip(recorded_passes) {
        assert_eq!(captured.pass_name, recorded.pass_name);
        assert_eq!(captured.executor_id, recorded.executor_id);
        assert_eq!(
            captured.cpu_elapsed_micros, recorded.cpu_elapsed_micros,
            "Render17 {phase} evidence must retain per-pass CPU recording time for {}",
            captured.pass_name,
        );
    }
}

const RENDER17_CAPTURE_EVIDENCE_SCHEMA_VERSION: u32 = 2;

#[derive(Serialize)]
struct Render17CaptureEvidence {
    schema_version: u32,
    workload: &'static str,
    viewport_width: u32,
    viewport_height: u32,
    cold: Render17CaptureFrameEvidence,
    settled_warm: Render17CaptureFrameEvidence,
    renderdoc_capture_files: Vec<String>,
    renderdoc_replay_audit: Render17ReplayAuditStatus,
}

#[derive(Serialize)]
struct Render17CaptureFrameEvidence {
    generation: u64,
    png_file: String,
    graph_executed_pass_count: usize,
    graph_executed_passes: Vec<String>,
    graph_cache_total_hit_count: usize,
    graph_cache_total_miss_count: usize,
    graph_cache_total_eviction_count: usize,
    readback_in_flight_count: usize,
    readback_bytes: u64,
    transient_texture_pool_created_count: usize,
    transient_texture_pool_reused_count: usize,
    transient_buffer_pool_created_count: usize,
    transient_buffer_pool_reused_count: usize,
    frame_profile: RenderFrameProfile,
    resolved_gpu_frame_profile: Option<RenderFrameProfile>,
}

#[derive(Serialize)]
struct Render17ReplayAuditStatus {
    status: &'static str,
    audit_script: &'static str,
    reason: &'static str,
}

fn write_capture_evidence(
    output: &Path,
    viewport_size: UVec2,
    cold_output: &Path,
    cold_profile: &RenderFrameProfile,
    cold_stats: &RenderStats,
    warm_output: &Path,
    warm_profile: &RenderFrameProfile,
    warm_stats: &RenderStats,
    rdc_files: &[PathBuf],
) {
    let evidence = Render17CaptureEvidence {
        schema_version: RENDER17_CAPTURE_EVIDENCE_SCHEMA_VERSION,
        workload: "WGPU full-chain temporal cold and settled-warm capture",
        viewport_width: viewport_size.x,
        viewport_height: viewport_size.y,
        cold: Render17CaptureFrameEvidence::from_stats(cold_output, cold_profile, cold_stats),
        settled_warm: Render17CaptureFrameEvidence::from_stats(
            warm_output,
            warm_profile,
            warm_stats,
        ),
        renderdoc_capture_files: rdc_files
            .iter()
            .map(|path| evidence_file_name(path.as_path()))
            .collect(),
        renderdoc_replay_audit: Render17ReplayAuditStatus {
            status: "unavailable_pending_renderdoc_replay",
            audit_script: "docs/plans/performance/01/renderdoc_capture_audit.py",
            reason: "The exporter writes source-frame evidence; action counts and GPU event durations require replaying each emitted RDC.",
        },
    };
    let encoded = serde_json::to_vec_pretty(&evidence)
        .expect("Render17 profile evidence must be serializable");
    std::fs::write(output, encoded).expect("Render17 profile evidence must be writable");
}

impl Render17CaptureFrameEvidence {
    fn from_stats(output: &Path, profile: &RenderFrameProfile, stats: &RenderStats) -> Self {
        let transient_pool = &stats
            .last_graph_execution_resource_report
            .transient_pool_report;
        Self {
            generation: profile.frame_generation,
            png_file: evidence_file_name(output),
            graph_executed_pass_count: stats.last_graph_executed_pass_count,
            graph_executed_passes: stats.last_graph_executed_passes.clone(),
            graph_cache_total_hit_count: stats.last_graph_compiled_cache_hit_count,
            graph_cache_total_miss_count: stats.last_graph_compiled_cache_miss_count,
            graph_cache_total_eviction_count: stats.last_graph_compiled_cache_eviction_count,
            readback_in_flight_count: stats.last_readback_in_flight_count,
            readback_bytes: stats.last_readback_bytes,
            transient_texture_pool_created_count: transient_pool.texture_created_count,
            transient_texture_pool_reused_count: transient_pool.texture_reused_count,
            transient_buffer_pool_created_count: transient_pool.buffer_created_count,
            transient_buffer_pool_reused_count: transient_pool.buffer_reused_count,
            frame_profile: profile.clone(),
            resolved_gpu_frame_profile: stats.last_resolved_gpu_frame_profile.as_deref().cloned(),
        }
    }
}

fn evidence_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .expect("Render17 evidence path must have a UTF-8 file name")
        .to_owned()
}

fn write_png(output: &Path, rgba: &[u8], width: u32, height: u32) {
    std::fs::create_dir_all(output.parent().expect("PNG path must have a parent")).unwrap();
    image::save_buffer_with_format(
        output,
        rgba,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
}

fn render_evidence_directory() -> PathBuf {
    repository_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}

fn renderdoc_capture_template(output_directory: &Path) -> PathBuf {
    let capture_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    output_directory.join(format!(
        "plan17_pfm1_render_graph_cold_warm_wgpu_current_source_{}_{}",
        std::process::id(),
        capture_id,
    ))
}

fn renderdoc_capture_files(output_directory: &Path, template: &Path) -> Vec<PathBuf> {
    let prefix = format!(
        "{}_frame",
        template
            .file_name()
            .and_then(|name| name.to_str())
            .expect("RenderDoc capture template must have a UTF-8 file name"),
    );
    let mut captures = std::fs::read_dir(output_directory)
        .expect("RenderDoc must create the configured evidence directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rdc")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
        })
        .collect::<Vec<_>>();
    captures.sort();
    captures
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below the repository root")
        .to_path_buf()
}
