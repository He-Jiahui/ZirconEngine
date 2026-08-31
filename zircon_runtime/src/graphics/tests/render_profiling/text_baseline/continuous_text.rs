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
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::{
    ProfileSnapshot,
    ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiFrame,
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    },
};

use super::support::{
    assert_counter_equals, assert_counter_frame_count, assert_counter_is_positive,
    assert_counter_is_zero, assert_span_frame_count, managed_output_root,
};
use super::{
    FRAME_PROFILES_FILE, GPU_FLUSH_FRAMES, MAX_SAMPLES, MEASURED_FRAMES, REPETITIONS,
    WARMUP_FRAMES, assert_profile_file, collect_resolved_gpu_profile,
    native_text_raster_is_settled, test_extract, visible_text_state,
};

const GLYPH_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const GLYPHS_PER_HARD_LINE: usize = 100;
const FONT_SIZE: f32 = 18.0;
const LINE_HEIGHT: f32 = 20.0;
const VIEWPORT_SIZE: UVec2 = UVec2::new(2_048, 2_048);

#[test]
fn continuous_text_profile_contract_uses_one_visible_native_run_per_glyph_scale() {
    assert_eq!(GLYPH_COUNTS, [1, 100, 1_000, 10_000]);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert!(MEASURED_FRAMES * 64 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(10_000, 3, Path::new(r"E:\managed-text-profile")).session_id,
        "runtime-text-continuous-10000-r3"
    );

    for glyph_count in GLYPH_COUNTS {
        let text = continuous_text(glyph_count);
        assert_eq!(
            text.bytes().filter(|byte| *byte == b'x').count(),
            glyph_count
        );
        assert_eq!(
            text.bytes().filter(|byte| *byte == b'\n').count(),
            glyph_count.saturating_sub(1) / GLYPHS_PER_HARD_LINE
        );

        let mut surface = continuous_text_surface(glyph_count);
        surface.rebuild();
        assert_eq!(surface.render_extract.list.commands.len(), 1);
    }
}

#[test]
#[ignore = "managed Windows WGPU continuous-text profiling baseline"]
fn runtime_text_continuous_profile_baseline_exports_complete_frame_matrix() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(std::sync::Arc::new(ProjectAssetManager::default()))
            .expect("continuous text baseline should create a WGPU framework");
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("continuous text baseline requires synchronous WGPU timestamp collection");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("continuous text baseline should create a viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-text-continuous-baseline")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("continuous text baseline should configure the UI-only quality profile");
    let output_root = managed_output_root();

    for glyph_count in GLYPH_COUNTS {
        for repetition in 1..=REPETITIONS {
            let mut surface = continuous_text_surface(glyph_count);
            warm_continuous_text(&framework, viewport, &mut surface, glyph_count);
            capture_repetition(
                &framework,
                viewport,
                &mut surface,
                glyph_count,
                repetition,
                &output_root,
            );
        }
    }
}

fn warm_continuous_text(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    glyph_count: usize,
) {
    for _ in 0..WARMUP_FRAMES {
        rebuild_and_submit(framework, viewport, surface);
    }
    let stats = framework
        .query_stats()
        .expect("continuous text baseline should query warm WGPU stats");
    assert_continuous_text_stats(&stats, glyph_count);
}

fn capture_repetition(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    glyph_count: usize,
    repetition: usize,
    output_root: &Path,
) {
    start_capture(capture_config(glyph_count, repetition, output_root));

    let mut current_profiles = Vec::with_capacity(MEASURED_FRAMES);
    let mut resolved_gpu_profiles = BTreeMap::new();
    for _ in 0..MEASURED_FRAMES {
        {
            crate::profile_frame!("runtime", "runtime_text.continuous");
            rebuild_and_submit(framework, viewport, surface);
        }
        let stats = framework
            .query_stats()
            .expect("continuous text baseline should query measured WGPU stats");
        assert_continuous_text_stats(&stats, glyph_count);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("continuous text baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("continuous text baseline captures measured frames")
        .frame_generation;
    for _ in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        rebuild_and_submit(framework, viewport, surface);
        let stats = framework
            .query_stats()
            .expect("continuous text timestamp flush should query stats");
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export continuous-text profiler report");
    reset_capture();
    let resolved_gpu_profiles = resolved_gpu_profiles
        .into_values()
        .filter(|profile| (first_generation..=last_generation).contains(&profile.frame_generation))
        .collect::<Vec<_>>();
    assert_complete_capture(
        glyph_count,
        &report.snapshot,
        &current_profiles,
        &resolved_gpu_profiles,
    );
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
        "scenario": "continuous-native-text",
        "glyph_count": glyph_count,
        "hard_line_glyph_count": GLYPHS_PER_HARD_LINE,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles)
            .expect("serialize continuous-text render frame profiles"),
    )
    .expect("write continuous-text render frame profiles beside the managed profiler export");
}

fn rebuild_and_submit(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    surface.rebuild();
    assert_eq!(surface.render_extract.list.commands.len(), 1);
    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(UiRenderSubmission::single(std::sync::Arc::new(
                surface.render_extract.clone(),
            ))),
        )
        .expect("continuous text baseline should submit a complete UI extract");
}

fn assert_continuous_text_stats(stats: &RenderStats, glyph_count: usize) {
    assert_eq!(stats.last_ui_text_payload_count, 1);
    assert_eq!(stats.last_ui_text_glyph_count, glyph_count);
    assert_eq!(stats.last_ui_text_unmapped_glyph_count, 0);
    assert_eq!(stats.last_ui_text_visible_raster_glyph_count, glyph_count);
    assert_eq!(stats.last_ui_text_raster_source_cache_miss_count, 0);
    assert_eq!(stats.last_ui_text_visible_missing_raster_image_count, 0);
    assert_eq!(stats.last_ui_text_visible_raster_placeholder_count, 0);
    assert_eq!(stats.last_ui_text_raster_worker_pending_count, 0);
    assert_eq!(stats.last_ui_text_raster_worker_failed_count, 0);
    assert_eq!(stats.last_ui_text_raster_renderer_upload_requeued_count, 0);
    assert_eq!(stats.last_ui_text_raster_renderer_upload_failure_count, 0);
    assert_eq!(stats.last_ui_text_layout_fallback_count, 0);
    assert_eq!(stats.last_ui_text_invalid_font_size_count, 0);
    assert_eq!(stats.last_ui_text_invalid_language_count, 0);
    assert_eq!(stats.last_ui_text_other_layout_error_count, 0);
    assert!(
        native_text_raster_is_settled(stats),
        "continuous native text must settle before capture: {stats:#?}"
    );
}

fn assert_complete_capture(
    glyph_count: usize,
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
        "resolved GPU profiles must correspond exactly to continuous-text generations"
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
        snapshot.counters.len() >= MEASURED_FRAMES * (15 + 14),
        "the recorder must retain the continuous-text counters alongside renderer counters"
    );
    for counter in [
        "text.shape_batch.requested",
        "text.shape_batch.cache_hits",
        "text.shape_batch.cache_misses",
        "ui_text.extract.commands",
        "ui_text.prewarm.requested",
        "ui_text.prewarm.cache_hits",
        "ui_text.prewarm.cache_misses",
        "ui_text.layout_resolve.cache_hits",
        "ui_text.layout_resolve.cache_misses",
        "ui_text.prepare.input_batches",
        "ui_text.prepare.resolved_native_batches",
        "ui_text.native_raster_plan.source_cache_hits",
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.slot_cache_hits",
        "ui_text.native_raster_plan.slot_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_deferred",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.worker_request_backpressured",
        "ui_text.native_raster_plan.worker_font_resident_bytes",
        "ui_text.native_raster_plan.worker_completion_backlog",
        "ui_text.native_raster_plan.worker_completion_backlog_bytes",
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
    assert_counter_equals(snapshot, "ui_text.extract.commands", 1.0);
    assert_counter_equals(snapshot, "ui_text.prepare.input_batches", 1.0);
    assert_counter_equals(snapshot, "ui_text.prepare.resolved_native_batches", 1.0);
    assert_counter_equals(
        snapshot,
        "ui_text.atlas_upload.native_instances",
        glyph_count as f64,
    );
    assert_counter_equals(snapshot, "ui_text.atlas_upload.native_draws", 1.0);
    for counter in [
        "text.shape_batch.cache_misses",
        "ui_text.prewarm.cache_misses",
        "ui_text.layout_resolve.cache_misses",
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.visible_placeholders",
        "ui_text.atlas_upload.native_copy_count",
        "ui_text.atlas_upload.native_bytes",
        "ui_text.atlas_upload.native_requeues",
        "ui_text.atlas_upload.native_failures",
    ] {
        assert_counter_is_zero(snapshot, counter);
    }
    for counter in [
        "ui_text.atlas_upload.native_instances",
        "ui_text.atlas_upload.native_draws",
    ] {
        assert_counter_is_positive(snapshot, counter);
    }
}

fn capture_config(
    glyph_count: usize,
    repetition: usize,
    output_root: &Path,
) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: format!("runtime-text-continuous-{glyph_count}-r{repetition}"),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn continuous_text_surface(glyph_count: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.text.continuous-{glyph_count}"
    )));
    let root_id = UiNodeId::new(1);
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_frame(viewport_frame())
            .with_state_flags(visible_text_state()),
    );
    surface
        .tree
        .insert_child(
            root_id,
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/continuous-text"))
                .with_frame(viewport_frame())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Text".to_string(),
                    attributes: continuous_text_attributes(glyph_count),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_state_flags(visible_text_state()),
        )
        .expect("continuous-text node should attach to its root");
    surface
}

fn continuous_text_attributes(glyph_count: usize) -> BTreeMap<String, toml::Value> {
    [
        (
            "text".to_string(),
            toml::Value::String(continuous_text(glyph_count)),
        ),
        (
            "foreground_color".to_string(),
            toml::Value::String("#f5f7fb".to_string()),
        ),
        (
            "font_size".to_string(),
            toml::Value::Float(FONT_SIZE.into()),
        ),
        (
            "line_height".to_string(),
            toml::Value::Float(LINE_HEIGHT.into()),
        ),
        ("wrap".to_string(), toml::Value::String("none".to_string())),
        (
            "text_render_mode".to_string(),
            toml::Value::String("native".to_string()),
        ),
    ]
    .into_iter()
    .collect()
}

fn continuous_text(glyph_count: usize) -> String {
    assert!(
        glyph_count > 0,
        "continuous-text baselines require visible glyphs"
    );
    let hard_line_count = glyph_count.saturating_sub(1) / GLYPHS_PER_HARD_LINE;
    let mut text = String::with_capacity(glyph_count + hard_line_count);
    for glyph_index in 0..glyph_count {
        if glyph_index > 0 && glyph_index % GLYPHS_PER_HARD_LINE == 0 {
            text.push('\n');
        }
        text.push('x');
    }
    text
}

fn viewport_frame() -> UiFrame {
    UiFrame::new(0.0, 0.0, VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32)
}
