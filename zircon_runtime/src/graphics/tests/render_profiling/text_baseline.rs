use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

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
use crate::text::cache::DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY;
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ProfileSnapshot;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{LayoutBoundary, UiFrame, UiSize},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use super::{assert_profile_file, native_text_raster_is_settled, test_extract};

const LABEL_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const LARGE_LABEL_BASELINE_COUNT: usize = 10_000;
const LARGE_LABEL_STABLE_TEXT_COUNT: usize = 512;
const WARMUP_FRAMES: usize = 60;
const MEASURED_FRAMES: usize = 300;
const REPETITIONS: usize = 3;
const MAX_SAMPLES: usize = 65_536;
const GPU_FLUSH_FRAMES: usize = 8;
const FRAME_PROFILES_FILE: &str = "render-frame-profiles.json";

#[path = "text_baseline/continuous_text.rs"]
mod continuous_text;
#[path = "text_baseline/dpi_font_generation.rs"]
mod dpi_font_generation;
#[path = "text_baseline/layout_cache_pressure.rs"]
mod layout_cache_pressure;
#[path = "text_baseline/localized_text_dirty.rs"]
mod localized_text_dirty;
#[path = "text_baseline/multilingual_text.rs"]
mod multilingual_text;
#[path = "text_baseline/queue_pressure.rs"]
mod queue_pressure;
#[path = "text_baseline/scroll_turnover.rs"]
mod scroll_turnover;
#[path = "text_baseline/support.rs"]
mod support;

use support::{
    assert_counter_equals, assert_counter_frame_count, assert_counter_is_absent,
    assert_counter_is_positive, assert_counter_is_zero, assert_span_frame_count,
    assert_span_is_absent, managed_output_root,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StaticLabelScenario {
    ForcedFullRebuild,
    RetainedSteady,
    LocalizedTextDirty,
}

impl StaticLabelScenario {
    const ALL: [Self; 3] = [
        Self::ForcedFullRebuild,
        Self::RetainedSteady,
        Self::LocalizedTextDirty,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::ForcedFullRebuild => "forced-full-rebuild",
            Self::RetainedSteady => "retained-steady",
            Self::LocalizedTextDirty => "localized-text-dirty",
        }
    }
}

#[test]
fn static_label_profile_baseline_contract_matches_plan() {
    assert_eq!(LABEL_COUNTS, [1, 100, 1_000, 10_000]);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert_eq!(static_label_text_identity(1_000, 999), 999);
    assert_eq!(static_label_text_identity(10_000, 511), 511);
    assert_eq!(static_label_text_identity(10_000, 512), 0);
    assert!(!static_label_exceeds_layout_cache_capacity(1_000));
    assert!(static_label_exceeds_layout_cache_capacity(10_000));

    let config = capture_config(
        StaticLabelScenario::ForcedFullRebuild,
        1_000,
        3,
        Path::new(r"E:\managed-text-profile"),
    );
    assert_eq!(config.session_id, "runtime-text-static-labels-1000-r3");
    assert_eq!(config.max_frames, MEASURED_FRAMES);
    assert_eq!(config.max_spans, MAX_SAMPLES);
    assert_eq!(config.max_counters, 65_536);
    assert_eq!(
        capture_config(
            StaticLabelScenario::RetainedSteady,
            1_000,
            3,
            Path::new(r"E:\managed-text-profile"),
        )
        .session_id,
        "runtime-text-static-labels-retained-1000-r3"
    );
    assert_eq!(
        capture_config(
            StaticLabelScenario::LocalizedTextDirty,
            1_000,
            3,
            Path::new(r"E:\managed-text-profile"),
        )
        .session_id,
        "runtime-text-static-labels-localized-dirty-1000-r3"
    );
}

#[test]
fn static_label_root_keeps_localized_text_invalidation_at_the_leaf() {
    let viewport_size = UVec2::new(320, 160);
    let label_count = 100;
    let mut surface = static_label_surface(label_count, viewport_size);
    let root = surface
        .tree
        .node(UiNodeId::new(1))
        .expect("static-label surface should have a root");

    assert_eq!(root.layout_boundary, LayoutBoundary::ParentDirected);
    surface.rebuild();
    surface.clear_dirty_flags();
    localized_text_dirty::mutate(&mut surface, 1);

    let report = surface
        .rebuild_dirty(UiSize::new(viewport_size.x as f32, viewport_size.y as f32))
        .expect("localized text invalidation should rebuild");

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert!(report.hit_grid_outer_node_visit_count <= 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert!(report.render_command_rebuilt_count <= 1);
}

#[test]
#[ignore = "managed Windows WGPU profiling baseline"]
fn runtime_text_static_label_profile_baseline_exports_complete_frame_matrix() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    framework
        .set_submission_config(RenderSubmissionConfig::synchronous().with_gpu_timing())
        .expect("static-label baseline requires synchronous WGPU timestamp collection");
    let viewport_size = UVec2::new(1_920, 720);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-text-static-label-baseline")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();
    let output_root = managed_output_root();

    for scenario in StaticLabelScenario::ALL {
        for label_count in LABEL_COUNTS {
            let mut surface = static_label_surface(label_count, viewport_size);
            for repetition in 1..=REPETITIONS {
                for frame_index in 0..WARMUP_FRAMES {
                    let _ = rebuild_and_submit(
                        &framework,
                        viewport,
                        &mut surface,
                        viewport_size,
                        scenario,
                        frame_index,
                    );
                }
                let warm_stats = framework.query_stats().unwrap();
                assert!(
                    native_text_raster_is_settled(&warm_stats),
                    "{} static-label baseline did not settle after repetition {repetition} warm-up: {warm_stats:#?}",
                    scenario.name()
                );
                capture_repetition(
                    &framework,
                    viewport,
                    &mut surface,
                    viewport_size,
                    scenario,
                    label_count,
                    repetition,
                    &output_root,
                );
            }
        }
    }
}

fn capture_repetition(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    viewport_size: UVec2,
    scenario: StaticLabelScenario,
    label_count: usize,
    repetition: usize,
    output_root: &Path,
) {
    start_capture(capture_config(
        scenario,
        label_count,
        repetition,
        output_root,
    ));

    let mut current_profiles = Vec::with_capacity(MEASURED_FRAMES);
    let mut resolved_gpu_profiles = BTreeMap::new();
    for frame_index in 0..MEASURED_FRAMES {
        {
            crate::profile_frame!("runtime", "runtime_text.static_labels");
            let rebuild = rebuild_and_submit(
                framework,
                viewport,
                surface,
                viewport_size,
                scenario,
                frame_index,
            );
            match (scenario, rebuild) {
                (StaticLabelScenario::RetainedSteady, Some(rebuild)) => {
                    assert_retained_surface_skipped_text_work(rebuild);
                }
                (StaticLabelScenario::LocalizedTextDirty, Some(rebuild)) => {
                    localized_text_dirty::assert_patch(rebuild, label_count);
                    localized_text_dirty::record_profile(rebuild);
                }
                (StaticLabelScenario::ForcedFullRebuild, None) => {}
                _ => panic!(
                    "static-label baseline scenario did not produce its expected rebuild report"
                ),
            }
        }
        let stats = framework.query_stats().unwrap();
        assert_eq!(stats.last_ui_text_payload_count, label_count);
        assert!(
            native_text_raster_is_settled(&stats),
            "measured static-label frame left native raster work unsettled: {stats:#?}"
        );
        assert_eq!(stats.last_ui_text_raster_source_cache_miss_count, 0);
        current_profiles.push(stats.last_frame_profile.as_ref().clone());
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }
    stop_capture();

    let first_generation = current_profiles
        .first()
        .expect("the baseline captures measured frames")
        .frame_generation;
    let last_generation = current_profiles
        .last()
        .expect("the baseline captures measured frames")
        .frame_generation;
    for flush_index in 0..GPU_FLUSH_FRAMES {
        if resolved_gpu_profiles.contains_key(&last_generation) {
            break;
        }
        let _ = rebuild_and_submit(
            framework,
            viewport,
            surface,
            viewport_size,
            scenario,
            MEASURED_FRAMES + flush_index,
        );
        let stats = framework.query_stats().unwrap();
        collect_resolved_gpu_profile(&mut resolved_gpu_profiles, &stats);
    }

    let report = export_report().expect("export static-label text profiling baseline");
    reset_capture();
    let resolved_gpu_profiles = resolved_gpu_profiles
        .into_values()
        .filter(|profile| (first_generation..=last_generation).contains(&profile.frame_generation))
        .collect::<Vec<_>>();

    assert_complete_capture(
        scenario,
        label_count,
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
    let export_dir = PathBuf::from(&report.export_dir);
    assert!(export_dir.starts_with(output_root));
    let frame_profiles = serde_json::json!({
        "scenario": scenario.name(),
        "label_count": label_count,
        "warmup_frames": WARMUP_FRAMES,
        "measured_frames": MEASURED_FRAMES,
        "repetition": repetition,
        "current": current_profiles,
        "resolved_gpu": resolved_gpu_profiles,
    });
    std::fs::write(
        export_dir.join(FRAME_PROFILES_FILE),
        serde_json::to_vec_pretty(&frame_profiles).expect("serialize render frame profiles"),
    )
    .expect("write render frame profiles beside the managed profiler export");
}

fn capture_config(
    scenario: StaticLabelScenario,
    label_count: usize,
    repetition: usize,
    output_root: &Path,
) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: match scenario {
            StaticLabelScenario::ForcedFullRebuild => {
                format!("runtime-text-static-labels-{label_count}-r{repetition}")
            }
            StaticLabelScenario::RetainedSteady => {
                format!("runtime-text-static-labels-retained-{label_count}-r{repetition}")
            }
            StaticLabelScenario::LocalizedTextDirty => {
                format!("runtime-text-static-labels-localized-dirty-{label_count}-r{repetition}")
            }
        },
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn rebuild_and_submit(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    surface: &mut UiSurface,
    viewport_size: UVec2,
    scenario: StaticLabelScenario,
    mutation_frame_index: usize,
) -> Option<crate::ui::surface::UiSurfaceRebuildReport> {
    let rebuild = match scenario {
        StaticLabelScenario::ForcedFullRebuild => {
            surface.rebuild();
            None
        }
        StaticLabelScenario::RetainedSteady => Some(
            surface
                .rebuild_dirty(UiSize::new(viewport_size.x as f32, viewport_size.y as f32))
                .expect("retained static-label surface rebuild should succeed"),
        ),
        StaticLabelScenario::LocalizedTextDirty => {
            localized_text_dirty::mutate(surface, mutation_frame_index);
            Some(
                surface
                    .rebuild_dirty(UiSize::new(viewport_size.x as f32, viewport_size.y as f32))
                    .expect("localized text-dirty surface rebuild should succeed"),
            )
        }
    };
    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(UiRenderSubmission::single(std::sync::Arc::new(
                surface.render_extract.clone(),
            ))),
        )
        .unwrap();
    rebuild
}

fn collect_resolved_gpu_profile(
    profiles: &mut BTreeMap<u64, RenderFrameProfile>,
    stats: &RenderStats,
) {
    if let Some(profile) = &stats.last_resolved_gpu_frame_profile {
        profiles.insert(profile.frame_generation, profile.as_ref().clone());
    }
}

fn assert_complete_capture(
    scenario: StaticLabelScenario,
    label_count: usize,
    snapshot: &ProfileSnapshot,
    current_profiles: &[RenderFrameProfile],
    resolved_gpu_profiles: &[RenderFrameProfile],
) {
    assert_eq!(snapshot.frames.len(), MEASURED_FRAMES);
    assert_eq!(current_profiles.len(), MEASURED_FRAMES);
    assert_eq!(
        resolved_gpu_profiles.len(),
        MEASURED_FRAMES,
        "every measured frame requires a resolved WGPU timestamp profile"
    );
    assert_eq!(
        current_profiles
            .iter()
            .map(|profile| profile.frame_generation)
            .collect::<Vec<_>>(),
        resolved_gpu_profiles
            .iter()
            .map(|profile| profile.frame_generation)
            .collect::<Vec<_>>(),
        "resolved GPU profiles must correspond exactly to the measured frame generations"
    );
    assert!(
        resolved_gpu_profiles
            .iter()
            .all(|profile| profile.gpu_frame_time_us.is_some()),
        "the Windows baseline requires real WGPU frame timestamps"
    );
    for profile in resolved_gpu_profiles {
        let ui_pass = profile
            .passes
            .iter()
            .find(|pass| {
                pass.pass_name == "runtime-ui"
                    && pass.executor_id == "ui.screen-space"
                    && pass.budget_key == RenderBudgetKey::Ui
            })
            .unwrap_or_else(|| {
                panic!(
                    "measured generation {} omitted the runtime UI pass profile",
                    profile.frame_generation
                )
            });
        assert!(
            ui_pass.gpu_time_us.is_some(),
            "measured generation {} omitted the runtime UI pass timestamp",
            profile.frame_generation
        );
    }
    let always_present_spans = [
        ("ui_text.prepare", "screen_space_ui_text"),
        ("ui_text.native_raster_plan", "native_text_prepare"),
    ];
    for (category, name) in always_present_spans {
        assert_span_frame_count(snapshot, category, name);
    }
    for counter in [
        "ui_text.prepare.input_batches",
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.atlas_upload.native_instances",
        "ui_text.atlas_upload.native_draws",
        "ui_text.atlas_upload.native_bytes",
    ] {
        assert_counter_frame_count(snapshot, counter);
    }
    match scenario {
        StaticLabelScenario::ForcedFullRebuild => {
            assert!(
                snapshot.counters.len() >= MEASURED_FRAMES * (19 + 14),
                "the 65,536-sample recorder must retain complete UI and shape counter trajectories"
            );
            for (category, name) in [
                ("text.shape_batch", "shape_paragraphs_with_cache"),
                ("ui_text.extract", "owner_prewarm_request_collection"),
                ("ui_text.extract", "owner_prewarm_overlap_admission"),
                ("ui_text.extract", "render_command_collection"),
                ("ui_text.prewarm", "render_command_text"),
                ("ui_text.layout_resolve", "render_command_text"),
            ] {
                assert_span_frame_count(snapshot, category, name);
            }
            for counter in [
                "text.shape_batch.requested",
                "ui_text.extract.commands",
                "ui_text.font_handles.registration_batches",
                "ui_text.prewarm.requested",
                "ui_text.layout_resolve.cache_hits",
                "ui_text.layout_resolve.uncached_document_resolves",
            ] {
                assert_counter_frame_count(snapshot, counter);
            }
            for zero_counter in [
                "text.shape_batch.cache_misses",
                "text.shape_batch.shaped",
                "text.shape_batch.inserted",
                "ui_text.prewarm.cache_misses",
                "ui_text.layout_resolve.shape_cache_misses",
            ] {
                assert_counter_is_zero(snapshot, zero_counter);
            }
            assert_counter_equals(
                snapshot,
                "ui_text.layout_resolve.uncached_document_resolves",
                0.0,
            );
            if static_label_exceeds_layout_cache_capacity(label_count) {
                assert_counter_is_zero(snapshot, "ui_text.layout_resolve.cache_hits");
                assert_counter_equals(
                    snapshot,
                    "ui_text.layout_resolve.cache_misses",
                    label_count as f64,
                );
            } else {
                assert_counter_equals(
                    snapshot,
                    "ui_text.layout_resolve.cache_hits",
                    label_count as f64,
                );
                assert_counter_is_zero(snapshot, "ui_text.layout_resolve.cache_misses");
            }
        }
        StaticLabelScenario::RetainedSteady => {
            for (category, name) in [
                ("text.shape_batch", "shape_paragraphs_with_cache"),
                ("ui_text.extract", "owner_prewarm_request_collection"),
                ("ui_text.extract", "owner_prewarm_overlap_admission"),
                ("ui_text.extract", "render_command_collection"),
                ("ui_text.prewarm", "render_command_text"),
                ("ui_text.layout_resolve", "render_command_text"),
            ] {
                assert_span_is_absent(snapshot, category, name);
            }
            for counter in [
                "text.shape_batch.requested",
                "ui_text.extract.commands",
                "ui_text.prewarm.requested",
                "ui_text.layout_resolve.cache_hits",
                "ui_text.layout_resolve.uncached_document_resolves",
            ] {
                assert_counter_is_absent(snapshot, counter);
            }
        }
        StaticLabelScenario::LocalizedTextDirty => {
            localized_text_dirty::assert_complete_capture(snapshot);
        }
    }
    for counter in [
        "ui_text.atlas_upload.native_instances",
        "ui_text.atlas_upload.native_draws",
    ] {
        assert_counter_is_positive(snapshot, counter);
    }
    for zero_counter in [
        "ui_text.native_raster_plan.source_cache_misses",
        "ui_text.native_raster_plan.worker_pending",
        "ui_text.native_raster_plan.worker_failed",
        "ui_text.native_raster_plan.visible_placeholders",
        "ui_text.atlas_upload.native_copy_count",
        "ui_text.atlas_upload.native_bytes",
        "ui_text.atlas_upload.native_requeues",
        "ui_text.atlas_upload.native_failures",
    ] {
        assert_counter_is_zero(snapshot, zero_counter);
    }
}

fn assert_retained_surface_skipped_text_work(rebuild: crate::ui::surface::UiSurfaceRebuildReport) {
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
    assert!(!rebuild.render_rebuilt);
    assert_eq!(rebuild.arranged_outer_node_visit_count, 0);
    assert_eq!(rebuild.hit_grid_outer_node_visit_count, 0);
    assert_eq!(rebuild.render_outer_node_visit_count, 0);
    assert_eq!(rebuild.layout_visited_node_count, 0);
    assert_eq!(rebuild.text_layout_cache_hit_count, 0);
    assert_eq!(rebuild.text_layout_cache_miss_count, 0);
    assert_eq!(rebuild.text_shape_cache_hit_count, 0);
    assert_eq!(rebuild.text_shape_cache_miss_count, 0);
}

fn static_label_surface(label_count: usize, viewport_size: UVec2) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.text.static-labels-{label_count}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(
                0.0,
                0.0,
                viewport_size.x as f32,
                viewport_size.y as f32,
            ))
            // Keep a leaf text mutation from promoting the measurement to the whole label tree.
            .with_layout_boundary(LayoutBoundary::ParentDirected)
            .with_state_flags(visible_text_state()),
    );
    for index in 0..label_count {
        let column = index % 30;
        let row = index / 30;
        let text_identity = static_label_text_identity(label_count, index);
        let text = format!("L{text_identity:04}");
        let attributes = toml::from_str(&format!(
            r##"
text = "{text}"
foreground_color = "#f5f7fb"
font_size = 18.0
line_height = 20.0
wrap = "none"
text_render_mode = "native"
"##
        ))
        .expect("static label metadata should parse");
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new((index + 2) as u64),
                    UiNodePath::new(format!("root/label-{index}")),
                )
                .with_frame(UiFrame::new(
                    (column * 64) as f32,
                    (row * 21) as f32,
                    64.0,
                    20.0,
                ))
                .with_state_flags(visible_text_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Text".to_string(),
                    attributes,
                    ..UiTemplateNodeMetadata::default()
                }),
            )
            .expect("static label should be inserted into the profile surface");
    }
    surface
}

fn static_label_text_identity(label_count: usize, index: usize) -> usize {
    if label_count == LARGE_LABEL_BASELINE_COUNT {
        index % LARGE_LABEL_STABLE_TEXT_COUNT
    } else {
        index
    }
}

fn static_label_exceeds_layout_cache_capacity(label_count: usize) -> bool {
    label_count > DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY
}

fn visible_text_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}
