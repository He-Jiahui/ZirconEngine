use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::diagnostics::profiling::{
    export_report, reset_capture, start_capture, stop_capture, test_capture_lock,
    ProfileCaptureConfig, PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE,
    PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE,
};
use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameProfile, RenderFramework, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderSubmissionConfig, RenderViewportDescriptor,
    RenderViewportHandle,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::text::font::{
    force_publish_shared_font_database, shared_font_database_generation,
    shared_font_database_test_read_guard, shared_font_database_test_serial_guard,
};
use crate::ui::{
    dispatch::UiInputManager, platform_input::translate_winit_window_event, surface::UiSurface,
};
use zircon_runtime_interface::{
    ui::{
        dispatch::{UiInputSequence, UiInputTimestamp, UiWindowId},
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{UiFrame, UiSize},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
        window::{UiWindowEventMetadata, UiWindowInputContext},
    },
    ProfileSnapshot,
};

use super::support::{
    assert_counter_equals, assert_counter_frame_count, assert_counter_is_positive,
    assert_counter_is_zero, assert_span_frame_count, managed_output_root,
};
use super::{
    assert_profile_file, collect_resolved_gpu_profile, native_text_raster_is_settled, test_extract,
    visible_text_state, FRAME_PROFILES_FILE, GPU_FLUSH_FRAMES, MAX_SAMPLES, MEASURED_FRAMES,
    REPETITIONS, WARMUP_FRAMES,
};

const VIEWPORT_SIZE: UVec2 = UVec2::new(960, 360);
const TEXT_NODE_COUNT: usize = 1;
const SCALE_ONE: f64 = 1.0;
const SCALE_TWO: f64 = 2.0;
const ROOT_NODE_ID: u64 = 1;
const TEXT_NODE_ID: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationScenario {
    DpiScaleTwo,
    FontDatabasePublish,
}

impl GenerationScenario {
    const ALL: [Self; 2] = [Self::DpiScaleTwo, Self::FontDatabasePublish];

    const fn name(self) -> &'static str {
        match self {
            Self::DpiScaleTwo => "dpi-2x",
            Self::FontDatabasePublish => "font-generation",
        }
    }
}

#[test]
fn dpi_and_font_generation_profile_contract_uses_production_invalidation_inputs() {
    assert_eq!(GenerationScenario::ALL.len(), 2);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert!(MEASURED_FRAMES * 48 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(
            GenerationScenario::DpiScaleTwo,
            3,
            Path::new(r"E:\managed-text-profile"),
        )
        .session_id,
        "runtime-text-dpi-2x-r3"
    );
    assert_eq!(
        capture_config(
            GenerationScenario::FontDatabasePublish,
            3,
            Path::new(r"E:\managed-text-profile"),
        )
        .session_id,
        "runtime-text-font-generation-r3"
    );

    let mut surface = generation_surface();
    let mut input_manager = UiInputManager::default();
    apply_window_scale(&mut surface, &mut input_manager, SCALE_ONE, 1);
    apply_window_scale(&mut surface, &mut input_manager, SCALE_TWO, 2);
    assert_eq!(surface.render_extract.list.commands.len(), TEXT_NODE_COUNT);
    assert_eq!(surface.render_extract.raster_scale, SCALE_TWO as f32);
}

#[test]
#[ignore = "managed Windows WGPU DPI text profiling baseline"]
fn runtime_text_dpi_scale_generation_profile_baseline_exports_complete_frame_matrix() {
    let _capture_guard = test_capture_lock();
    let _font_serial_guard = shared_font_database_test_serial_guard();
    let output_root = managed_output_root();

    for repetition in 1..=REPETITIONS {
        let (framework, viewport) = generation_framework("runtime-text-dpi-generation-baseline");
        let mut surface = generation_surface();
        let mut input_manager = UiInputManager::default();
        apply_window_scale(&mut surface, &mut input_manager, SCALE_ONE, 1);
        warm_stable_surface(&framework, viewport, &mut surface);

        apply_window_scale(&mut surface, &mut input_manager, SCALE_TWO, 2);
        let transition = submit_current_surface(&framework, viewport, &surface);
        assert_transition_stats(&transition);
        assert!(
            transition.last_ui_text_raster_source_cache_miss_count > 0,
            "the same renderer must rasterize a new source entry after a 1x to 2x DPI event: {transition:#?}"
        );
        warm_stable_surface(&framework, viewport, &mut surface);
        capture_repetition(
            GenerationScenario::DpiScaleTwo,
            &framework,
            viewport,
            &mut surface,
            repetition,
            SCALE_TWO,
            shared_font_database_generation(),
            transition.last_ui_text_raster_source_cache_miss_count,
            &output_root,
        );
    }
}

#[test]
#[ignore = "managed Windows WGPU font-generation text profiling baseline"]
fn runtime_text_font_generation_profile_baseline_exports_complete_frame_matrix() {
    let _capture_guard = test_capture_lock();
    let _font_serial_guard = shared_font_database_test_serial_guard();
    let database = {
        let (_, database) = shared_font_database_test_read_guard();
        database.clone()
    };
    let output_root = managed_output_root();

    for repetition in 1..=REPETITIONS {
        let (framework, viewport) = generation_framework("runtime-text-font-generation-baseline");
        let mut surface = generation_surface();
        let mut input_manager = UiInputManager::default();
        apply_window_scale(
            &mut surface,
            &mut input_manager,
            SCALE_TWO,
            repetition as u64,
        );
        warm_stable_surface(&framework, viewport, &mut surface);

        let generation_before = shared_font_database_generation();
        let generation_after = force_publish_shared_font_database(&database);
        assert!(generation_after > generation_before);
        let transition = submit_rebuilt_surface(&framework, viewport, &mut surface);
        assert_transition_stats(&transition);
        assert!(
            transition.last_ui_text_raster_source_cache_miss_count > 0,
            "the same renderer must invalidate source entries after a font-database publication: {transition:#?}"
        );
        warm_stable_surface(&framework, viewport, &mut surface);
        capture_repetition(
            GenerationScenario::FontDatabasePublish,
            &framework,
            viewport,
            &mut surface,
            repetition,
            SCALE_TWO,
            generation_after,
            transition.last_ui_text_raster_source_cache_miss_count,
            &output_root,
        );
    }
}

fn generation_framework(profile_name: &str) -> (WgpuRenderFramework, RenderViewportHandle) {
    let framework =
        WgpuRenderFramework::new_for_test(std::sync::Arc::new(ProjectAssetManager::default()))
            .expect("generation baseline should create a WGPU framework");
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("generation baseline requires synchronous WGPU timestamp collection");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("generation baseline should create a viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new(profile_name)
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("generation baseline should configure a UI-only quality profile");
    (framework, viewport)
}

fn warm_stable_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    let mut consecutive_settled_frames = 0;
    for _ in 0..WARMUP_FRAMES {
        let stats = submit_rebuilt_surface(framework, viewport, surface);
        if native_text_raster_is_settled(&stats) {
            consecutive_settled_frames += 1;
        } else {
            consecutive_settled_frames = 0;
        }
    }
    assert!(
        consecutive_settled_frames >= 2,
        "generation baseline requires two consecutive settled frames before capture"
    );
}

fn capture_repetition(
    scenario: GenerationScenario,
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    repetition: usize,
    raster_scale: f64,
    font_generation: u64,
    transition_source_cache_misses: usize,
    output_root: &Path,
) {
    start_capture(capture_config(scenario, repetition, output_root));

    let mut current_profiles = Vec::with_capacity(MEASURED_FRAMES);
    let mut resolved_gpu_profiles = BTreeMap::new();
    for _ in 0..MEASURED_FRAMES {
        {
            crate::profile_frame!("runtime", "runtime_text.dpi_font_generation");
            crate::profile_counter!(
                "runtime",
                "ui_text.dpi_font_generation.raster_scale_milli",
                (raster_scale * 1_000.0) as u64
            );
            crate::profile_counter!(
                "runtime",
                "ui_text.dpi_font_generation.font_generation",
                font_generation
            );
            submit_rebuilt_surface(framework, viewport, surface);
        }
        let stats = framework
            .query_stats()
            .expect("generation baseline should query measured WGPU stats");
        assert_stable_stats(&stats);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("generation baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("generation baseline captures measured frames")
        .frame_generation;
    for _ in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        let _ = submit_rebuilt_surface(framework, viewport, surface);
        let stats = framework
            .query_stats()
            .expect("generation timestamp flush should query WGPU stats");
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export DPI/font-generation profiler report");
    reset_capture();
    let resolved_gpu_profiles = resolved_gpu_profiles
        .into_values()
        .filter(|profile| (first_generation..=last_generation).contains(&profile.frame_generation))
        .collect::<Vec<_>>();
    assert_complete_capture(
        scenario,
        raster_scale,
        font_generation,
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
        "scenario": scenario.name(),
        "raster_scale": raster_scale,
        "font_generation": font_generation,
        "transition_source_cache_misses": transition_source_cache_misses,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles)
            .expect("serialize DPI/font-generation render frame profiles"),
    )
    .expect("write DPI/font-generation profiles beside the managed profiler export");
}

fn submit_rebuilt_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) -> RenderStats {
    surface.rebuild();
    submit_current_surface(framework, viewport, surface)
}

fn submit_current_surface(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &UiSurface,
) -> RenderStats {
    assert_eq!(surface.render_extract.list.commands.len(), TEXT_NODE_COUNT);
    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(surface.render_extract.clone()),
        )
        .expect("generation baseline should submit a complete UI extract");
    framework
        .query_stats()
        .expect("generation baseline should query submitted WGPU stats")
}

fn assert_transition_stats(stats: &RenderStats) {
    assert_eq!(stats.last_ui_text_payload_count, TEXT_NODE_COUNT);
    assert!(stats.last_ui_text_glyph_count > 0);
    assert_eq!(stats.last_ui_text_unmapped_glyph_count, 0);
    assert_eq!(stats.last_ui_text_layout_fallback_count, 0);
    assert_eq!(stats.last_ui_text_invalid_font_size_count, 0);
    assert_eq!(stats.last_ui_text_invalid_language_count, 0);
    assert_eq!(stats.last_ui_text_other_layout_error_count, 0);
}

fn assert_stable_stats(stats: &RenderStats) {
    assert_transition_stats(stats);
    assert!(
        stats.last_ui_text_visible_raster_glyph_count > 0,
        "DPI/font generation stable frame must retain visible native glyphs: {stats:#?}"
    );
    assert_eq!(stats.last_ui_text_raster_source_cache_miss_count, 0);
    assert_eq!(stats.last_ui_text_visible_missing_raster_image_count, 0);
    assert_eq!(stats.last_ui_text_visible_raster_placeholder_count, 0);
    assert_eq!(stats.last_ui_text_raster_worker_pending_count, 0);
    assert_eq!(stats.last_ui_text_raster_worker_failed_count, 0);
    assert_eq!(stats.last_ui_text_raster_renderer_upload_requeued_count, 0);
    assert_eq!(stats.last_ui_text_raster_renderer_upload_failure_count, 0);
    assert!(
        native_text_raster_is_settled(stats),
        "DPI/font generation stable frame must not retain raster work: {stats:#?}"
    );
}

fn assert_complete_capture(
    scenario: GenerationScenario,
    raster_scale: f64,
    font_generation: u64,
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
        "resolved GPU profiles must correspond exactly to DPI/font-generation frames"
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
        snapshot.counters.len() >= MEASURED_FRAMES * (2 + 28),
        "the recorder must retain DPI/font-generation counters alongside renderer counters"
    );
    for counter in [
        "ui_text.dpi_font_generation.raster_scale_milli",
        "ui_text.dpi_font_generation.font_generation",
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
        "ui_text.dpi_font_generation.raster_scale_milli",
        raster_scale * 1_000.0,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.dpi_font_generation.font_generation",
        font_generation as f64,
    );
    assert_counter_equals(snapshot, "ui_text.extract.commands", TEXT_NODE_COUNT as f64);
    assert_counter_equals(
        snapshot,
        "ui_text.prepare.input_batches",
        TEXT_NODE_COUNT as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.prepare.resolved_native_batches",
        TEXT_NODE_COUNT as f64,
    );
    for counter in [
        "text.shape_batch.cache_misses",
        "ui_text.prewarm.cache_misses",
        "ui_text.layout_resolve.cache_misses",
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.slot_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_deferred",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.worker_request_backpressured",
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
    assert_eq!(
        raster_scale,
        SCALE_TWO,
        "{} capture must retain the 2x physical raster scale",
        scenario.name()
    );
}

fn capture_config(
    scenario: GenerationScenario,
    repetition: usize,
    output_root: &Path,
) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: format!("runtime-text-{}-r{repetition}", scenario.name()),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn generation_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.dpi-font-generation"));
    let root_id = UiNodeId::new(ROOT_NODE_ID);
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_frame(viewport_frame())
            .with_state_flags(visible_text_state()),
    );
    surface
        .tree
        .insert_child(
            root_id,
            UiTreeNode::new(UiNodeId::new(TEXT_NODE_ID), UiNodePath::new("root/text"))
                .with_frame(UiFrame::new(32.0, 96.0, 720.0, 96.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Text".to_string(),
                    attributes: text_attributes(),
                    ..UiTemplateNodeMetadata::default()
                })
                .with_state_flags(visible_text_state()),
        )
        .expect("generation text node should attach to its root");
    surface
}

fn apply_window_scale(
    surface: &mut UiSurface,
    input_manager: &mut UiInputManager,
    scale: f64,
    sequence: u64,
) {
    let metadata = |event_sequence| {
        UiWindowEventMetadata::for_window(
            UiWindowId::new("runtime-text-profile-window"),
            UiInputTimestamp::from_micros(event_sequence),
            UiInputSequence::new(event_sequence),
        )
        .synthetic(true)
    };
    let physical_size = winit::dpi::PhysicalSize::new(
        (VIEWPORT_SIZE.x as f64 * scale) as u32,
        (VIEWPORT_SIZE.y as f64 * scale) as u32,
    );
    if surface.window_state.metrics.is_some() {
        let suggested_surface_size = Arc::new(Mutex::new(physical_size));
        let scale_event = translate_winit_window_event(
            UiWindowInputContext::from_window_metadata(&metadata(sequence.saturating_mul(2))),
            &winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor: scale,
                surface_size_writer: winit::event::SurfaceSizeWriter::new(Arc::downgrade(
                    &suggested_surface_size,
                )),
            },
        )
        .expect("Winit scale-factor change should translate");
        let scale_dispatch = surface
            .dispatch_window_input_pump_event(input_manager, scale_event)
            .expect("generation baseline should dispatch a translated DPI event");
        assert!(
            scale_dispatch
                .diagnostics
                .notes
                .iter()
                .any(|note| note == "window_scale_factor_updated"),
            "the DPI transition must use the scale-factor input route: {scale_dispatch:#?}"
        );

        let resized_context = UiWindowInputContext::from_window_metadata(&metadata(
            sequence.saturating_mul(2).saturating_add(1),
        ))
        .with_window_metrics(
            surface
                .window_state
                .metrics
                .expect("scale-factor dispatch must establish current metrics"),
        );
        let resized_event = translate_winit_window_event(
            resized_context,
            &winit::event::WindowEvent::SurfaceResized(physical_size),
        )
        .expect("Winit surface resize should translate after a DPI event");
        let resized_dispatch = surface
            .dispatch_window_input_pump_event(input_manager, resized_event)
            .expect("generation baseline should dispatch a translated resize event");
        assert!(
            resized_dispatch
                .diagnostics
                .notes
                .iter()
                .any(|note| note == "window_layout_metrics_dirty"),
            "the physical resize must retain the DPI-aware metrics: {resized_dispatch:#?}"
        );
    } else {
        let resized_event = translate_winit_window_event(
            UiWindowInputContext::from_window_metadata(&metadata(sequence)),
            &winit::event::WindowEvent::SurfaceResized(physical_size),
        )
        .expect("Winit initial surface resize should translate");
        let dispatch = surface
            .dispatch_window_input_pump_event(input_manager, resized_event)
            .expect("generation baseline should dispatch a translated initial resize event");
        assert!(
            dispatch
                .diagnostics
                .notes
                .iter()
                .any(|note| note == "window_layout_metrics_dirty"),
            "the initial resize must establish window metrics: {dispatch:#?}"
        );
    }
    surface
        .rebuild_dirty(viewport_size())
        .expect("generation baseline should rebuild after a window metric event");
    assert_eq!(
        surface
            .window_state
            .metrics
            .expect("window metric bootstrap must precede a scale change")
            .scale_factor,
        scale
    );
    assert_eq!(surface.render_extract.raster_scale, scale as f32);
}

fn text_attributes() -> BTreeMap<String, toml::Value> {
    [
        (
            "text".to_string(),
            toml::Value::String("DPI and font generation native text".to_string()),
        ),
        (
            "foreground_color".to_string(),
            toml::Value::String("#f5f7fb".to_string()),
        ),
        ("font_size".to_string(), toml::Value::Float(28.0)),
        ("line_height".to_string(), toml::Value::Float(34.0)),
        ("wrap".to_string(), toml::Value::String("none".to_string())),
        (
            "text_render_mode".to_string(),
            toml::Value::String("native".to_string()),
        ),
    ]
    .into_iter()
    .collect()
}

fn viewport_frame() -> UiFrame {
    UiFrame::new(0.0, 0.0, VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32)
}

fn viewport_size() -> UiSize {
    UiSize::new(VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32)
}
