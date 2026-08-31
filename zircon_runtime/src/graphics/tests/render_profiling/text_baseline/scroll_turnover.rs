use std::{collections::BTreeMap, path::Path};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::diagnostics::profiling::{
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, ProfileCaptureConfig, export_report, reset_capture,
    start_capture, stop_capture, test_capture_lock,
};
use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameProfile, RenderFramework, RenderPipelineHandle,
    RenderQualityProfile, RenderSubmissionConfig, RenderViewportDescriptor, RenderViewportHandle,
    UiRenderSubmission,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeScrollExt};
use zircon_runtime_interface::{
    ProfileSnapshot,
    ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{
            AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiScrollState,
            UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
            UiVirtualListWindow,
        },
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

const ROW_COUNT: usize = 1_000;
const VISIBLE_ROW_COUNT: usize = 100;
const TURNOVER_ROW_COUNT: usize = 10;
const ROW_HEIGHT: f32 = 20.0;
const VIEWPORT_SIZE: UVec2 = UVec2::new(960, 2_000);
const SCROLL_NODE_ID: u64 = 1;

#[test]
fn scroll_turnover_profile_contract_keeps_entering_rows_as_the_only_cold_shapes() {
    assert_eq!(ROW_COUNT, 1_000);
    assert_eq!(VISIBLE_ROW_COUNT, 100);
    assert_eq!(TURNOVER_ROW_COUNT, VISIBLE_ROW_COUNT / 10);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert!(MEASURED_FRAMES * 128 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(3, Path::new(r"E:\managed-text-profile")).session_id,
        "runtime-text-scroll-turnover-r3"
    );

    let mut surface = scroll_turnover_surface();
    surface
        .compute_layout(viewport_size())
        .expect("scroll turnover fixture should compute its initial virtual window");
    assert_virtual_window(&surface, 0);
    assert_eq!(
        surface.render_extract.list.commands.len(),
        VISIBLE_ROW_COUNT
    );

    let rebuild = move_window_and_rebuild(&mut surface, TURNOVER_ROW_COUNT);
    assert_virtual_window(&surface, TURNOVER_ROW_COUNT);
    assert_eq!(rebuild.text_shape_cache_miss_count, TURNOVER_ROW_COUNT);
    assert_eq!(rebuild.text_layout_cache_miss_count, VISIBLE_ROW_COUNT);
    assert_eq!(rebuild.text_layout_cache_hit_count, 0);
    assert!(
        rebuild.text_shape_cache_hit_count >= VISIBLE_ROW_COUNT - TURNOVER_ROW_COUNT,
        "the overlapping rows should reuse their shaped source entries: {rebuild:#?}"
    );
    assert_eq!(
        surface.render_extract.list.commands.len(),
        VISIBLE_ROW_COUNT
    );
}

#[test]
#[ignore = "managed Windows WGPU text scroll-turnover profiling baseline"]
fn runtime_text_scroll_turnover_profile_baseline_exports_complete_frame_matrix() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(std::sync::Arc::new(ProjectAssetManager::default()))
            .expect("scroll turnover baseline should create a WGPU framework");
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("scroll turnover baseline requires synchronous WGPU timestamp collection");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("scroll turnover baseline should create a viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-text-scroll-turnover-baseline")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .expect("scroll turnover baseline should configure the UI-only quality profile");
    let output_root = managed_output_root();

    for repetition in 1..=REPETITIONS {
        let mut surface = scroll_turnover_surface();
        surface
            .compute_layout(viewport_size())
            .expect("scroll turnover baseline should build its first virtual window");
        warm_scroll_windows(&framework, viewport, &mut surface);
        let warm_stats = framework
            .query_stats()
            .expect("scroll turnover baseline should query warm WGPU stats");
        assert!(
            native_text_raster_is_settled(&warm_stats),
            "scroll turnover raster must settle before capture: {warm_stats:#?}"
        );
        capture_repetition(&framework, viewport, &mut surface, repetition, &output_root);
    }
}

fn warm_scroll_windows(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
) {
    for frame_index in 0..WARMUP_FRAMES {
        let first_visible = if frame_index % 2 == 0 {
            0
        } else {
            TURNOVER_ROW_COUNT
        };
        let _ = move_window_and_rebuild(surface, first_visible);
        framework
            .submit_frame_extract_with_ui(
                viewport,
                test_extract(),
                Some(UiRenderSubmission::single(std::sync::Arc::new(
                    surface.render_extract.clone(),
                ))),
            )
            .expect("scroll turnover warm-up frame should submit");
    }
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
        let first_visible = if frame_index % 2 == 0 {
            0
        } else {
            TURNOVER_ROW_COUNT
        };
        {
            crate::profile_frame!("runtime", "runtime_text.scroll_turnover");
            let rebuild = {
                crate::profile_scope!("runtime", "ui_text.scroll_turnover", "move_visible_window");
                move_window_and_rebuild(surface, first_visible)
            };
            assert_virtual_window(surface, first_visible);
            assert_eq!(rebuild.text_shape_cache_miss_count, 0);
            assert_eq!(rebuild.text_layout_cache_miss_count, 0);
            assert_eq!(rebuild.text_layout_cache_hit_count, VISIBLE_ROW_COUNT);
            assert_eq!(
                surface.render_extract.list.commands.len(),
                VISIBLE_ROW_COUNT
            );
            record_scroll_turnover_profile(rebuild, first_visible);
            framework
                .submit_frame_extract_with_ui(
                    viewport,
                    test_extract(),
                    Some(UiRenderSubmission::single(std::sync::Arc::new(
                        surface.render_extract.clone(),
                    ))),
                )
                .expect("scroll turnover measured frame should submit");
        }
        let stats = framework
            .query_stats()
            .expect("scroll turnover baseline should query measured WGPU stats");
        assert_eq!(stats.last_ui_text_payload_count, VISIBLE_ROW_COUNT);
        assert!(
            native_text_raster_is_settled(&stats),
            "scroll turnover measured frame must not retain raster work: {stats:#?}"
        );
        assert_eq!(stats.last_ui_text_raster_source_cache_miss_count, 0);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("scroll turnover baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("scroll turnover baseline captures measured frames")
        .frame_generation;
    for flush_index in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        let first_visible = if (MEASURED_FRAMES + flush_index) % 2 == 0 {
            0
        } else {
            TURNOVER_ROW_COUNT
        };
        let _ = move_window_and_rebuild(surface, first_visible);
        framework
            .submit_frame_extract_with_ui(
                viewport,
                test_extract(),
                Some(UiRenderSubmission::single(std::sync::Arc::new(
                    surface.render_extract.clone(),
                ))),
            )
            .expect("scroll turnover timestamp flush should submit");
        let stats = framework
            .query_stats()
            .expect("scroll turnover timestamp flush should query stats");
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export scroll-turnover profiler report");
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
        "scenario": "virtualized-scroll-turnover",
        "row_count": ROW_COUNT,
        "visible_row_count": VISIBLE_ROW_COUNT,
        "turnover_row_count": TURNOVER_ROW_COUNT,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles).expect("serialize scroll-turnover profiles"),
    )
    .expect("write scroll-turnover profiles beside the managed profiler export");
}

fn record_scroll_turnover_profile(
    rebuild: crate::ui::surface::UiSurfaceRebuildReport,
    first_visible: usize,
) {
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.first_visible_row",
        first_visible
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.entering_rows",
        TURNOVER_ROW_COUNT
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.overlapping_rows",
        VISIBLE_ROW_COUNT - TURNOVER_ROW_COUNT
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.layout_visited_nodes",
        rebuild.layout_visited_node_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.render_commands",
        rebuild.render_command_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.layout_cache_hits",
        rebuild.text_layout_cache_hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.layout_cache_misses",
        rebuild.text_layout_cache_miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.shape_cache_hits",
        rebuild.text_shape_cache_hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.scroll_turnover.shape_cache_misses",
        rebuild.text_shape_cache_miss_count
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
        "resolved GPU profiles must correspond exactly to scroll-turnover generations"
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
        ("ui_text.scroll_turnover", "move_visible_window"),
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
        snapshot.counters.len() >= MEASURED_FRAMES * (9 + 40),
        "the recorder must retain the scroll scenario counters alongside renderer counters"
    );

    for counter in [
        "ui_text.scroll_turnover.first_visible_row",
        "ui_text.scroll_turnover.entering_rows",
        "ui_text.scroll_turnover.overlapping_rows",
        "ui_text.scroll_turnover.layout_visited_nodes",
        "ui_text.scroll_turnover.render_commands",
        "ui_text.scroll_turnover.layout_cache_hits",
        "ui_text.scroll_turnover.layout_cache_misses",
        "ui_text.scroll_turnover.shape_cache_hits",
        "ui_text.scroll_turnover.shape_cache_misses",
        "ui_text.prepare.input_batches",
        "ui_text.native_raster_plan.source_cache_hits",
        "ui_text.native_raster_plan.source_cache_misses",
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
        "ui_text.scroll_turnover.entering_rows",
        TURNOVER_ROW_COUNT as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.scroll_turnover.overlapping_rows",
        (VISIBLE_ROW_COUNT - TURNOVER_ROW_COUNT) as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.scroll_turnover.render_commands",
        VISIBLE_ROW_COUNT as f64,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.scroll_turnover.layout_cache_hits",
        VISIBLE_ROW_COUNT as f64,
    );
    assert_counter_equals(snapshot, "ui_text.scroll_turnover.layout_cache_misses", 0.0);
    assert_counter_equals(snapshot, "ui_text.scroll_turnover.shape_cache_misses", 0.0);
    for counter in [
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.slot_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_deferred",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.worker_request_backpressured",
        "ui_text.native_raster_plan.worker_font_copied_bytes",
        "ui_text.native_raster_plan.worker_cancelled",
        "ui_text.native_raster_plan.worker_completion_applied_bytes",
        "ui_text.native_raster_plan.worker_completion_drained_bytes",
        "ui_text.native_raster_plan.worker_completion_budget_deferred",
        "ui_text.native_raster_plan.worker_completion_oversized_accepted",
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
        session_id: format!("runtime-text-scroll-turnover-r{repetition}"),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn scroll_turnover_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.scroll-turnover"));
    let root_id = UiNodeId::new(SCROLL_NODE_ID);
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root/scroll"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(VIEWPORT_SIZE.x as f32),
                height: fixed_constraint(VIEWPORT_SIZE.y as f32),
            })
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                gap: 0.0,
                scrollbar_visibility: UiScrollbarVisibility::Never,
                virtualization: Some(UiVirtualListConfig {
                    item_extent: ROW_HEIGHT,
                    overscan: 0,
                }),
            }))
            .with_scroll_state(UiScrollState::default())
            .with_state_flags(visible_text_state()),
    );
    for row in 0..ROW_COUNT {
        surface
            .tree
            .insert_child(
                root_id,
                UiTreeNode::new(
                    UiNodeId::new((row + 2) as u64),
                    UiNodePath::new(format!("root/scroll/row-{row:04}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(VIEWPORT_SIZE.x as f32),
                    height: fixed_constraint(ROW_HEIGHT),
                })
                .with_template_metadata(row_text_metadata(row))
                .with_state_flags(visible_text_state()),
            )
            .expect("scroll turnover rows should attach to their scroll owner");
    }
    surface
}

fn row_text_metadata(row: usize) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(&format!(
            r##"
text = "scroll-profile-row-{row:04}"
foreground_color = "#f5f7fb"
font_size = 18.0
line_height = 20.0
wrap = "none"
text_render_mode = "native"
"##
        ))
        .expect("scroll turnover row metadata should parse"),
        ..UiTemplateNodeMetadata::default()
    }
}

fn move_window_and_rebuild(
    surface: &mut UiSurface,
    first_visible: usize,
) -> crate::ui::surface::UiSurfaceRebuildReport {
    let offset = first_visible as f32 * ROW_HEIGHT;
    let _ = surface
        .tree
        .set_scroll_offset(UiNodeId::new(SCROLL_NODE_ID), offset)
        .expect("scroll turnover fixture should accept a virtual-window offset");
    surface
        .rebuild_dirty(viewport_size())
        .expect("scroll turnover fixture should rebuild its virtual window")
}

fn assert_virtual_window(surface: &UiSurface, first_visible: usize) {
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(SCROLL_NODE_ID))
            .expect("scroll turnover fixture should retain its root")
            .layout_cache
            .virtual_window,
        Some(UiVirtualListWindow {
            first_visible,
            last_visible_exclusive: first_visible + VISIBLE_ROW_COUNT,
        })
    );
}

fn viewport_size() -> UiSize {
    UiSize::new(VIEWPORT_SIZE.x as f32, VIEWPORT_SIZE.y as f32)
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 0,
        weight: 0.0,
        stretch_mode: StretchMode::Fixed,
    }
}
