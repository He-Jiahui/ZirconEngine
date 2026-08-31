use std::{collections::BTreeMap, path::Path};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::diagnostics::profiling::{
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, ProfileCaptureConfig, export_report, reset_capture,
    start_capture, stop_capture, test_capture_lock,
};
use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameProfile, RenderFramework, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderSubmissionConfig, RenderViewportDescriptor,
    RenderViewportHandle, UiRenderSubmission,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::ui::surface::{UiInvalidationReason, UiSurface};
use zircon_runtime_interface::{
    ProfileSnapshot,
    ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{UiFrame, UiSize},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    },
};

use super::support::{
    assert_any_counter_is_positive, assert_counter_does_not_exceed, assert_counter_equals,
    assert_counter_frame_count, assert_counter_peak_at_least, assert_span_frame_count,
    managed_output_root,
};
use super::{
    FRAME_PROFILES_FILE, GPU_FLUSH_FRAMES, MAX_SAMPLES, MEASURED_FRAMES, REPETITIONS,
    WARMUP_FRAMES, assert_profile_file, collect_resolved_gpu_profile, test_extract,
    visible_text_state,
};

const PRESSURE_NODE_COUNT: usize = 512;
const GRID_COLUMNS: usize = 32;
const PRESSURE_GLYPH_VARIANT_COUNT: usize = 64;
const PRESSURE_FONT_SIZE_VARIANT_COUNT: usize = 8;
const PRESSURE_EPOCH_FRAME_COUNT: usize = 16;
const PRESSURE_SETTLE_FRAME_LIMIT: usize = 240;
const VIEWPORT_SIZE: UVec2 = UVec2::new(960, 360);
const ROOT_NODE_ID: u64 = 1;
const TEXT_NODE_ID_START: u64 = 2;
const FONT_SIZE_BASE: f32 = 12.0;
const FONT_SIZE_EPOCH_STEP: f32 = 0.01;
const PRESSURE_GLYPHS: &[u8; PRESSURE_GLYPH_VARIANT_COUNT] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!?";

#[test]
fn queue_pressure_profile_contract_uses_production_cache_key_pressure() {
    assert_eq!(PRESSURE_NODE_COUNT, 512);
    assert_eq!(GRID_COLUMNS, 32);
    assert_eq!(PRESSURE_GLYPHS.len(), PRESSURE_GLYPH_VARIANT_COUNT);
    assert_eq!(
        PRESSURE_NODE_COUNT,
        PRESSURE_GLYPH_VARIANT_COUNT * PRESSURE_FONT_SIZE_VARIANT_COUNT
    );
    assert!(WARMUP_FRAMES >= PRESSURE_EPOCH_FRAME_COUNT);
    assert!(MEASURED_FRAMES >= PRESSURE_EPOCH_FRAME_COUNT);
    assert!(
        PRESSURE_NODE_COUNT
            > crate::text::native_bitmap_atlas::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME,
        "the queue-pressure workload must exceed the production per-frame request budget"
    );
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert!(MEASURED_FRAMES * 64 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(3, Path::new(r"E:\managed-text-profile")).session_id,
        "runtime-text-native-queue-pressure-r3"
    );

    let mut surface = queue_pressure_surface();
    let rebuild = apply_pressure_epoch(&mut surface, 0);
    assert_eq!(rebuild.render_command_count, PRESSURE_NODE_COUNT);
    assert_eq!(
        surface.render_extract.list.commands.len(),
        PRESSURE_NODE_COUNT
    );
    let first_font_size = font_size_for(0, 0);
    let next_font_size = font_size_for(1, 0);
    assert_ne!(first_font_size.to_bits(), next_font_size.to_bits());
    assert_ne!(glyph_for(0), glyph_for(PRESSURE_GLYPH_VARIANT_COUNT - 1));
    assert_ne!(
        font_size_for(0, 0).round() as u32,
        font_size_for(0, PRESSURE_GLYPH_VARIANT_COUNT).round() as u32,
        "each glyph group must occupy a distinct persistent physical-pixel bucket"
    );
}

#[test]
#[ignore = "managed Windows native text cache and queue pressure profiling baseline"]
fn runtime_text_native_queue_pressure_profile_baseline_exports_complete_frame_matrix() {
    let _capture_guard = test_capture_lock();
    let output_root = managed_output_root();

    for repetition in 1..=REPETITIONS {
        let (framework, viewport) = queue_pressure_framework();
        let mut surface = queue_pressure_surface();
        warm_pressure_surface(&framework, viewport, &mut surface);
        capture_repetition(&framework, viewport, &mut surface, repetition, &output_root);
    }
}

fn queue_pressure_framework() -> (WgpuRenderFramework, RenderViewportHandle) {
    let framework =
        WgpuRenderFramework::new_for_test(std::sync::Arc::new(ProjectAssetManager::default()))
            .expect("queue-pressure baseline should create a WGPU framework");
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("queue-pressure baseline requires synchronous WGPU timestamp collection");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("queue-pressure baseline should create a viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-text-native-queue-pressure")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("queue-pressure baseline should configure a UI-only quality profile");
    (framework, viewport)
}

fn warm_pressure_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    for frame_index in 0..WARMUP_FRAMES {
        let rebuild = apply_pressure_epoch(surface, pressure_epoch_for_frame(frame_index));
        assert_eq!(rebuild.render_command_count, PRESSURE_NODE_COUNT);
        submit_surface(framework, viewport, surface);
    }
    settle_pressure_epoch(
        framework,
        viewport,
        surface,
        pressure_epoch_for_frame(WARMUP_FRAMES),
    );
}

fn settle_pressure_epoch(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    epoch: usize,
) {
    for settle_frame in 0..PRESSURE_SETTLE_FRAME_LIMIT {
        let rebuild = apply_pressure_epoch(surface, epoch);
        assert_eq!(rebuild.render_command_count, PRESSURE_NODE_COUNT);
        submit_surface(framework, viewport, surface);
        let stats = framework
            .query_stats()
            .expect("queue-pressure settle should query native raster diagnostics");
        if stats.last_ui_text_raster_persistent_key_count >= PRESSURE_NODE_COUNT {
            return;
        }
    }
    panic!(
        "queue-pressure settle did not bind {} persistent raster keys within {} frames",
        PRESSURE_NODE_COUNT, PRESSURE_SETTLE_FRAME_LIMIT
    );
}

fn capture_repetition(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    repetition: usize,
    output_root: &Path,
) {
    start_capture(capture_config(repetition, output_root));

    let mut current_profiles = Vec::with_capacity(MEASURED_FRAMES);
    let mut resolved_gpu_profiles = BTreeMap::new();
    for frame_index in 0..MEASURED_FRAMES {
        let epoch = pressure_epoch_for_frame(WARMUP_FRAMES + frame_index);
        {
            crate::profile_frame!("runtime", "runtime_text.native_queue_pressure");
            let rebuild = {
                crate::profile_scope!("runtime", "ui_text.queue_pressure", "update_font_size_keys");
                apply_pressure_epoch(surface, epoch)
            };
            assert_eq!(rebuild.render_command_count, PRESSURE_NODE_COUNT);
            assert_eq!(
                surface.render_extract.list.commands.len(),
                PRESSURE_NODE_COUNT
            );
            crate::profile_counter!("runtime", "ui_text.queue_pressure.epoch", epoch);
            crate::profile_counter!(
                "runtime",
                "ui_text.queue_pressure.text_batches",
                surface.render_extract.list.commands.len()
            );
            submit_surface(framework, viewport, surface);
        }
        let stats = framework
            .query_stats()
            .expect("queue-pressure baseline should query measured WGPU stats");
        assert_pressure_stats(&stats);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("queue-pressure baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("queue-pressure baseline captures measured frames")
        .frame_generation;
    for flush_index in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        let _ = apply_pressure_epoch(
            surface,
            pressure_epoch_for_frame(WARMUP_FRAMES + MEASURED_FRAMES + flush_index),
        );
        submit_surface(framework, viewport, surface);
        let stats = framework
            .query_stats()
            .expect("queue-pressure timestamp flush should query WGPU stats");
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export native queue-pressure profiler report");
    reset_capture();
    let resolved_gpu_profiles = resolved_gpu_profiles
        .into_values()
        .filter(|profile| (first_generation..=last_generation).contains(&profile.frame_generation))
        .collect::<Vec<_>>();
    assert_complete_capture(&report.snapshot, &current_profiles, &resolved_gpu_profiles);
    for expected_file in [
        PROFILE_TIMELINE_NATIVE_FILE,
        PROFILE_TIMELINE_PERFETTO_FILE,
        PROFILE_HOTSPOTS_FILE,
        PROFILE_SUMMARY_FILE,
    ] {
        assert_profile_file(&report.files, expected_file);
    }
    let export_dir = std::path::PathBuf::from(&report.export_dir);
    assert!(export_dir.starts_with(output_root));
    let frame_profiles = serde_json::json!({
        "scenario": "native-cache-and-queue-pressure",
        "pressure_node_count": PRESSURE_NODE_COUNT,
        "font_size_epoch_step": FONT_SIZE_EPOCH_STEP,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles)
            .expect("serialize native queue-pressure render frame profiles"),
    )
    .expect("write native queue-pressure profiles beside the managed profiler export");
}

fn submit_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &UiSurface,
) {
    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(UiRenderSubmission::single(std::sync::Arc::new(
                surface.render_extract.clone(),
            ))),
        )
        .expect("queue-pressure baseline should submit a complete UI extract");
}

fn assert_pressure_stats(stats: &RenderStats) {
    assert_eq!(stats.last_ui_text_payload_count, PRESSURE_NODE_COUNT);
    assert_eq!(stats.last_ui_text_glyph_count, PRESSURE_NODE_COUNT);
    assert_eq!(stats.last_ui_text_unmapped_glyph_count, 0);
    assert_eq!(stats.last_ui_text_layout_fallback_count, 0);
    assert_eq!(stats.last_ui_text_invalid_font_size_count, 0);
    assert_eq!(stats.last_ui_text_invalid_language_count, 0);
    assert_eq!(stats.last_ui_text_other_layout_error_count, 0);
}

fn assert_complete_capture(
    snapshot: &ProfileSnapshot,
    current_profiles: &[RenderFrameProfile],
    resolved_gpu_profiles: &[RenderFrameProfile],
) {
    assert_eq!(snapshot.frames.len(), MEASURED_FRAMES);
    assert_eq!(current_profiles.len(), MEASURED_FRAMES);
    assert_eq!(resolved_gpu_profiles.len(), MEASURED_FRAMES);
    assert_eq!(
        current_profiles
            .iter()
            .map(|profile| profile.frame_generation)
            .collect::<Vec<_>>(),
        resolved_gpu_profiles
            .iter()
            .map(|profile| profile.frame_generation)
            .collect::<Vec<_>>(),
        "resolved GPU profiles must correspond exactly to native queue-pressure frames"
    );
    for profile in resolved_gpu_profiles {
        assert!(profile.gpu_frame_time_us.is_some());
        assert!(profile.passes.iter().any(|pass| {
            pass.pass_name == "runtime-ui"
                && pass.executor_id == "ui.screen-space"
                && pass.budget_key == RenderBudgetKey::Ui
                && pass.gpu_time_us.is_some()
        }));
    }

    for (category, name) in [
        ("ui_text.queue_pressure", "update_font_size_keys"),
        ("text.shape_batch", "shape_paragraphs_with_cache"),
        ("ui_text.extract", "owner_prewarm_request_collection"),
        ("ui_text.extract", "owner_prewarm_overlap_admission"),
        ("ui_text.extract", "render_command_collection"),
        ("ui_text.prewarm", "render_command_text"),
        ("ui_text.layout_resolve", "render_command_text"),
        ("ui_text.prepare", "screen_space_ui_text"),
        ("ui_text.native_raster_plan", "native_text_prepare"),
    ] {
        assert_span_frame_count(snapshot, category, name);
    }
    assert!(
        snapshot.counters.len() >= MEASURED_FRAMES * 56,
        "the recorder must retain native cache and worker-queue diagnostics"
    );

    for counter in [
        "ui_text.queue_pressure.epoch",
        "ui_text.queue_pressure.text_batches",
        "ui_text.prepare.input_batches",
        "ui_text.prepare.resolved_native_batches",
        "ui_text.native_raster_plan.source_cache_hits",
        "ui_text.native_raster_plan.source_cache_approximate_hits",
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.source_cache_inserts",
        "ui_text.native_raster_plan.source_cache_capacity",
        "ui_text.native_raster_plan.source_cache_entries",
        "ui_text.native_raster_plan.source_cache_persistent_raster_keys",
        "ui_text.native_raster_plan.source_cache_resident_bytes",
        "ui_text.native_raster_plan.source_cache_max_bytes",
        "ui_text.native_raster_plan.source_cache_approximate_probes",
        "ui_text.native_raster_plan.source_cache_lru_repairs",
        "ui_text.native_raster_plan.source_cache_lru_touches",
        "ui_text.native_raster_plan.source_cache_evicted",
        "ui_text.native_raster_plan.source_cache_evicted_bytes",
        "ui_text.native_raster_plan.source_cache_budget_linked_evictions",
        "ui_text.native_raster_plan.source_cache_linked_invalidations",
        "ui_text.native_raster_plan.source_cache_budget_rejections",
        "ui_text.native_raster_plan.source_cache_invalidated",
        "ui_text.native_raster_plan.slot_cache_hits",
        "ui_text.native_raster_plan.slot_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_deferred",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.worker_request_backpressured",
        "ui_text.native_raster_plan.worker_font_copied_bytes",
        "ui_text.native_raster_plan.worker_font_resident_bytes",
        "ui_text.native_raster_plan.worker_font_resident_entries",
        "ui_text.native_raster_plan.worker_cancelled",
        "ui_text.native_raster_plan.worker_completion_applied_bytes",
        "ui_text.native_raster_plan.worker_completion_drained_bytes",
        "ui_text.native_raster_plan.worker_completion_budget_deferred",
        "ui_text.native_raster_plan.worker_completion_oversized_accepted",
        "ui_text.native_raster_plan.worker_pool_threads",
        "ui_text.native_raster_plan.worker_pool_in_flight",
        "ui_text.native_raster_plan.worker_pool_queued",
        "ui_text.native_raster_plan.worker_pool_queued_bytes",
        "ui_text.native_raster_plan.worker_pool_running",
        "ui_text.native_raster_plan.worker_pool_completed_total",
        "ui_text.native_raster_plan.worker_pool_failed_total",
        "ui_text.native_raster_plan.worker_pool_queue_peak",
        "ui_text.native_raster_plan.worker_completion_backlog",
        "ui_text.native_raster_plan.worker_completion_backlog_bytes",
        "ui_text.native_raster_plan.worker_completion_backpressured_total",
        "ui_text.native_raster_plan.worker_completion_budget_rejected_total",
        "ui_text.native_raster_plan.worker_completion_rejected_bytes_total",
        "ui_text.native_raster_plan.worker_request_backpressured_total",
        "ui_text.native_raster_plan.worker_cancelled_total",
        "ui_text.native_raster_plan.visible_placeholders",
        "ui_text.atlas_upload.native_copy_count",
        "ui_text.atlas_upload.native_bytes",
        "ui_text.atlas_upload.native_requeues",
        "ui_text.atlas_upload.native_failures",
        "ui_text.atlas_upload.native_instances",
        "ui_text.atlas_upload.native_draws",
    ] {
        assert_counter_frame_count(snapshot, counter);
    }
    assert_counter_equals(
        snapshot,
        "ui_text.queue_pressure.text_batches",
        PRESSURE_NODE_COUNT as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.prepare.input_batches",
        PRESSURE_NODE_COUNT as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.prepare.resolved_native_batches",
        PRESSURE_NODE_COUNT as f64,
    );
    for counter in [
        "ui_text.native_raster_plan.source_cache_capacity",
        "ui_text.native_raster_plan.source_cache_max_bytes",
        "ui_text.native_raster_plan.worker_pool_threads",
    ] {
        assert_counter_is_positive(snapshot, counter);
    }
    assert_counter_does_not_exceed(
        snapshot,
        "ui_text.native_raster_plan.source_cache_entries",
        "ui_text.native_raster_plan.source_cache_capacity",
    );
    assert_counter_does_not_exceed(
        snapshot,
        "ui_text.native_raster_plan.source_cache_resident_bytes",
        "ui_text.native_raster_plan.source_cache_max_bytes",
    );
    assert_counter_does_not_exceed(
        snapshot,
        "ui_text.native_raster_plan.worker_pool_queued",
        "ui_text.native_raster_plan.worker_pool_in_flight",
    );
    assert_counter_does_not_exceed(
        snapshot,
        "ui_text.native_raster_plan.worker_pool_running",
        "ui_text.native_raster_plan.worker_pool_in_flight",
    );
    assert_counter_peak_at_least(
        snapshot,
        "ui_text.native_raster_plan.source_cache_misses",
        PRESSURE_NODE_COUNT as f64,
    );
    assert_counter_peak_at_least(
        snapshot,
        "ui_text.native_raster_plan.source_cache_persistent_raster_keys",
        PRESSURE_NODE_COUNT as f64,
    );
    assert_any_counter_is_positive(
        snapshot,
        &[
            "ui_text.native_raster_plan.worker_deferred",
            "ui_text.native_raster_plan.worker_request_backpressured",
        ],
    );
}

fn capture_config(repetition: usize, output_root: &Path) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: format!("runtime-text-native-queue-pressure-r{repetition}"),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn queue_pressure_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.native-queue-pressure"));
    let root_id = UiNodeId::new(ROOT_NODE_ID);
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_frame(UiFrame::new(
                0.0,
                0.0,
                VIEWPORT_SIZE.x as f32,
                VIEWPORT_SIZE.y as f32,
            ))
            .with_state_flags(visible_text_state()),
    );
    for index in 0..PRESSURE_NODE_COUNT {
        let column = index % GRID_COLUMNS;
        let row = index / GRID_COLUMNS;
        surface
            .tree
            .insert_child(
                root_id,
                UiTreeNode::new(
                    UiNodeId::new(TEXT_NODE_ID_START + index as u64),
                    UiNodePath::new(format!("root/text-{index}")),
                )
                .with_frame(UiFrame::new(
                    (column * 30) as f32,
                    (row * 22) as f32,
                    30.0,
                    22.0,
                ))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Text".to_string(),
                    attributes: text_attributes(index),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_state_flags(visible_text_state()),
            )
            .expect("queue-pressure text node should attach to its root");
    }
    surface
}

fn apply_pressure_epoch(
    surface: &mut UiSurface,
    epoch: usize,
) -> crate::ui::surface::UiSurfaceRebuildReport {
    for index in 0..PRESSURE_NODE_COUNT {
        let node_id = UiNodeId::new(TEXT_NODE_ID_START + index as u64);
        {
            let node = surface
                .tree
                .node_mut(node_id)
                .expect("queue-pressure text node should exist");
            let metadata = node
                .template_metadata
                .as_mut()
                .expect("queue-pressure text node should retain metadata");
            metadata.attributes.insert(
                "font_size".to_string(),
                toml::Value::Float(font_size_for(epoch, index) as f64),
            );
        }
        surface
            .invalidate_node(node_id, UiInvalidationReason::Text)
            .expect("queue-pressure text invalidation should succeed");
    }
    surface
        .rebuild_dirty(UiSize::new(VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32))
        .expect("queue-pressure text invalidations should rebuild")
}

fn text_attributes(index: usize) -> BTreeMap<String, toml::Value> {
    [
        (
            "text".to_string(),
            toml::Value::String(glyph_for(index).to_string()),
        ),
        (
            "foreground_color".to_string(),
            toml::Value::String("#f5f7fb".to_string()),
        ),
        (
            "font_size".to_string(),
            toml::Value::Float(font_size_for(0, index) as f64),
        ),
        ("line_height".to_string(), toml::Value::Float(20.0)),
        ("wrap".to_string(), toml::Value::String("none".to_string())),
        (
            "text_render_mode".to_string(),
            toml::Value::String("native".to_string()),
        ),
    ]
    .into_iter()
    .collect()
}

fn font_size_for(epoch: usize, index: usize) -> f32 {
    let bucket = index / PRESSURE_GLYPH_VARIANT_COUNT;
    FONT_SIZE_BASE + bucket as f32 + epoch as f32 * FONT_SIZE_EPOCH_STEP
}

const fn pressure_epoch_for_frame(frame_index: usize) -> usize {
    frame_index / PRESSURE_EPOCH_FRAME_COUNT
}

fn glyph_for(index: usize) -> char {
    char::from(PRESSURE_GLYPHS[index % PRESSURE_GLYPH_VARIANT_COUNT])
}
