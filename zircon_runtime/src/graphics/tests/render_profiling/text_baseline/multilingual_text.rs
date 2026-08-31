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
        surface::{UiRichTextFormat, UiTextDirection, UiTextWritingMode},
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

const TEXT_NODE_COUNT: usize = 4;
const VIEWPORT_SIZE: UVec2 = UVec2::new(1_280, 640);
const ROOT_NODE_ID: u64 = 1;
const CJK_NODE_ID: u64 = 2;
const ARABIC_NODE_ID: u64 = 3;
const MARKDOWN_NODE_ID: u64 = 4;
const VERTICAL_NODE_ID: u64 = 5;

#[test]
fn multilingual_text_profile_contract_resolves_rich_rtl_and_vertical_layouts() {
    assert_eq!(TEXT_NODE_COUNT, 4);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert!(MEASURED_FRAMES * 64 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(3, Path::new(r"E:\managed-text-profile")).session_id,
        "runtime-text-multilingual-r3"
    );

    let mut surface = multilingual_surface();
    surface.rebuild();
    let commands = &surface.render_extract.list.commands;
    assert_eq!(commands.len(), TEXT_NODE_COUNT);

    let cjk = command_for_node(commands, CJK_NODE_ID);
    assert_eq!(cjk.style.language.as_deref(), Some("zh-Hans"));
    let arabic = command_for_node(commands, ARABIC_NODE_ID);
    assert_eq!(arabic.style.text_direction, UiTextDirection::RightToLeft);
    assert_eq!(arabic.style.language.as_deref(), Some("ar"));
    let markdown = command_for_node(commands, MARKDOWN_NODE_ID);
    assert_eq!(
        markdown.style.rich_text_format,
        UiRichTextFormat::MarkdownInlineV1
    );
    let vertical = command_for_node(commands, VERTICAL_NODE_ID);
    let vertical_layout = vertical
        .text_layout
        .as_ref()
        .expect("multilingual vertical command should retain its resolved layout");
    assert_eq!(vertical_layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert!(vertical_layout.lines.len() >= 2);
    assert!(
        vertical_layout
            .lines
            .windows(2)
            .all(|columns| columns[0].frame.x > columns[1].frame.x),
        "VerticalRl columns should progress from right to left"
    );
}

#[test]
#[ignore = "managed Windows WGPU multilingual text profiling baseline"]
fn runtime_text_multilingual_profile_baseline_exports_complete_frame_matrix() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(std::sync::Arc::new(ProjectAssetManager::default()))
            .expect("multilingual text baseline should create a WGPU framework");
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("multilingual text baseline requires synchronous WGPU timestamp collection");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("multilingual text baseline should create a viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-text-multilingual-baseline")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("multilingual text baseline should configure the UI-only quality profile");
    let output_root = managed_output_root();

    for repetition in 1..=REPETITIONS {
        let mut surface = multilingual_surface();
        warm_multilingual_text(&framework, viewport, &mut surface);
        capture_repetition(&framework, viewport, &mut surface, repetition, &output_root);
    }
}

fn warm_multilingual_text(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    for _ in 0..WARMUP_FRAMES {
        rebuild_and_submit(framework, viewport, surface);
    }
    let stats = framework
        .query_stats()
        .expect("multilingual text baseline should query warm WGPU stats");
    assert_multilingual_stats(&stats);
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
    for _ in 0..MEASURED_FRAMES {
        {
            crate::profile_frame!("runtime", "runtime_text.multilingual");
            rebuild_and_submit(framework, viewport, surface);
        }
        let stats = framework
            .query_stats()
            .expect("multilingual text baseline should query measured WGPU stats");
        assert_multilingual_stats(&stats);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("multilingual text baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("multilingual text baseline captures measured frames")
        .frame_generation;
    for _ in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        rebuild_and_submit(framework, viewport, surface);
        let stats = framework
            .query_stats()
            .expect("multilingual text timestamp flush should query stats");
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export multilingual-text profiler report");
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
        "scenario": "multilingual-rich-vertical-native-text",
        "text_node_count": TEXT_NODE_COUNT,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles)
            .expect("serialize multilingual-text render frame profiles"),
    )
    .expect("write multilingual-text render frame profiles beside the managed profiler export");
}

fn rebuild_and_submit(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    surface.rebuild();
    assert_eq!(surface.render_extract.list.commands.len(), TEXT_NODE_COUNT);
    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(UiRenderSubmission::single(std::sync::Arc::new(
                surface.render_extract.clone(),
            ))),
        )
        .expect("multilingual text baseline should submit a complete UI extract");
}

fn assert_multilingual_stats(stats: &RenderStats) {
    assert_eq!(stats.last_ui_text_payload_count, TEXT_NODE_COUNT);
    assert!(stats.last_ui_text_glyph_count > 0);
    assert!(stats.last_ui_text_visible_raster_glyph_count > 0);
    assert_eq!(stats.last_ui_text_unmapped_glyph_count, 0);
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
        "multilingual native text must settle before capture: {stats:#?}"
    );
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
        "resolved GPU profiles must correspond exactly to multilingual-text generations"
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
        snapshot.counters.len() >= MEASURED_FRAMES * (16 + 14),
        "the recorder must retain the multilingual counters alongside renderer counters"
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
}

fn capture_config(repetition: usize, output_root: &Path) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: format!("runtime-text-multilingual-r{repetition}"),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn multilingual_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.multilingual"));
    let root_id = UiNodeId::new(ROOT_NODE_ID);
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_frame(viewport_frame())
            .with_state_flags(visible_text_state()),
    );
    for (node_id, path, frame, attributes) in [
        (
            CJK_NODE_ID,
            "root/cjk",
            UiFrame::new(32.0, 32.0, 840.0, 72.0),
            native_text_attributes("中文排版：引擎文本与布局", "zh-Hans", "ltr"),
        ),
        (
            ARABIC_NODE_ID,
            "root/arabic",
            UiFrame::new(32.0, 128.0, 840.0, 72.0),
            native_text_attributes("العربية النصية: تخطيط ومحاذاة", "ar", "rtl"),
        ),
        (
            MARKDOWN_NODE_ID,
            "root/markdown",
            UiFrame::new(32.0, 224.0, 840.0, 96.0),
            markdown_text_attributes("Markdown **strong** 中文 `code` مرحبًا"),
        ),
        (
            VERTICAL_NODE_ID,
            "root/vertical",
            UiFrame::new(1_008.0, 32.0, 112.0, 320.0),
            vertical_text_attributes("竖排「标点」。第二列，验证。"),
        ),
    ] {
        surface
            .tree
            .insert_child(
                root_id,
                UiTreeNode::new(UiNodeId::new(node_id), UiNodePath::new(path))
                    .with_frame(frame)
                    .with_template_metadata(UiTemplateNodeMetadata {
                        component: "Text".to_string(),
                        attributes,
                        ..UiTemplateNodeMetadata::default()
                    })
                    .with_state_flags(visible_text_state()),
            )
            .expect("multilingual text node should attach to its root");
    }
    surface
}

fn native_text_attributes(
    text: &str,
    language: &str,
    direction: &str,
) -> BTreeMap<String, toml::Value> {
    common_text_attributes(text)
        .into_iter()
        .chain([
            (
                "language".to_string(),
                toml::Value::String(language.to_string()),
            ),
            (
                "text_direction".to_string(),
                toml::Value::String(direction.to_string()),
            ),
        ])
        .collect()
}

fn markdown_text_attributes(text: &str) -> BTreeMap<String, toml::Value> {
    common_text_attributes(text)
        .into_iter()
        .chain([(
            "rich_text_format".to_string(),
            toml::Value::String("markdown_inline_v1".to_string()),
        )])
        .collect()
}

fn vertical_text_attributes(text: &str) -> BTreeMap<String, toml::Value> {
    common_text_attributes(text)
        .into_iter()
        .chain([(
            "writing_mode".to_string(),
            toml::Value::String("vertical-rl".to_string()),
        )])
        .collect()
}

fn common_text_attributes(text: &str) -> BTreeMap<String, toml::Value> {
    [
        ("text".to_string(), toml::Value::String(text.to_string())),
        (
            "foreground_color".to_string(),
            toml::Value::String("#f5f7fb".to_string()),
        ),
        ("font_size".to_string(), toml::Value::Float(24.0)),
        ("line_height".to_string(), toml::Value::Float(30.0)),
        ("wrap".to_string(), toml::Value::String("word".to_string())),
        (
            "text_render_mode".to_string(),
            toml::Value::String("native".to_string()),
        ),
    ]
    .into_iter()
    .collect()
}

fn command_for_node(
    commands: &[zircon_runtime_interface::ui::surface::UiRenderCommand],
    node_id: u64,
) -> &zircon_runtime_interface::ui::surface::UiRenderCommand {
    commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(node_id))
        .unwrap_or_else(|| panic!("multilingual command for node {node_id} should exist"))
}

fn viewport_frame() -> UiFrame {
    UiFrame::new(0.0, 0.0, VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32)
}
