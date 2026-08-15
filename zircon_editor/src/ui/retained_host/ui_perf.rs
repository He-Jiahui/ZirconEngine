#[cfg(feature = "profiling")]
use std::cell::Cell;
#[cfg(feature = "profiling")]
use std::time::Instant;

mod counter_batch;

pub(crate) use counter_batch::record_current_ui_perf_counter_batch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiPerfScenario {
    Startup,
    IdleHover,
    Click,
    Drag,
    DrawerResize,
    WindowResize,
    AssetRefresh,
    ViewportImage,
    ShellContent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiPerfCounter {
    FrameDurationUs,
    InputToDamageUs,
    DamageToSubmitUs,
    HostInvalidationTransactionCount,
    HostInvalidationScopeCount,
    HostInvalidationLegacyDirtyTransactionCount,
    HostInvalidationFullTargetCount,
    HostInvalidationShellContentTargetCount,
    HostInvalidationWorkbenchProjectionTargetCount,
    HostInvalidationViewPresentationTargetCount,
    HostInvalidationWindowMetricsTargetCount,
    HostInvalidationPaintOnlyTargetCount,
    SlowPathRebuildCount,
    ScopedPresentationPatchCount,
    ScopedPresentationFloatingWindowRowsVisited,
    ScopedPresentationFloatingWindowRowsCloned,
    ScopedPresentationNativePresenterVisitCount,
    ScopedPresentationDamageRegionCount,
    ScopedPresentationProjectionMissingCount,
    ScopedPresentationPresenterCoverageFallbackCount,
    RenderPathCount,
    PresentationRebuildCount,
    AssetEditorPanePresentationBuildCount,
    AssetEditorPaneReflectionBuildCount,
    AssetEditorPanePreviewBuildCount,
    AssetEditorPaneSourceBuildCount,
    AssetEditorPaneInspectorBuildCount,
    AssetEditorPaneStyleBuildCount,
    AssetEditorPaneThemeBuildCount,
    AssetEditorPaneCommandAvailabilityBuildCount,
    FullPaintCount,
    RegionPaintCount,
    PaintedPixels,
    PresentedSurfacePixels,
    RedrawFullFrame,
    RedrawRegion,
    DirtyLayout,
    DirtyPresentation,
    DirtyRender,
    DirtyPaintOnly,
    ChromeSnapshotCount,
    WorkbenchModelBuildCount,
    HierarchyScrollDispatchCount,
    HierarchySurfaceRebuildCount,
    HierarchyRowInsertCount,
    HierarchyDispatcherRebuildCount,
    HierarchyRouteMapRebuildCount,
    WelcomeRecentScrollDispatchCount,
    WelcomeRecentSurfaceRebuildCount,
    WelcomeRecentAuthorityRebuildCount,
    WelcomeRecentRowInsertCount,
    WelcomeRecentGeometryPatchCount,
    WelcomeRecentDispatcherRebuildCount,
    WelcomeRecentRouteMapRebuildCount,
    ShellDragAuthorityRebuildCount,
    ShellDragNodeInsertCount,
    ShellDragGeometryPatchCount,
    ShellDragNodePatchCount,
    ShellDragDispatcherRebuildCount,
    ShellDragRouteMapRebuildCount,
    WorkbenchHitIndexBuildCount,
    WorkbenchHitIndexQueryCount,
    PanePopupIndexQueryCount,
    PanePopupIndexCandidateCount,
    VisualAssetCacheHitCount,
    VisualAssetCacheMissCount,
    VisualAssetCacheCandidateBuildCount,
    SvgTreeCacheMemoryHitCount,
    SvgTreeCacheMissCount,
    WorkbenchPaintIndexQueryCount,
    WorkbenchPaintIndexCandidateCount,
    ChromeCommandFullRebuildCount,
    ChromeCommandPatchCount,
    PresentationGenerationReadCount,
    PresentationSnapshotReadCount,
    TemplateNodeVisitCount,
    TemplateNodeCloneCount,
    TemplateNodeDamageRejectCount,
    FallbackSortCount,
    ArtifactExportCount,
    SoftwareFallbackPresentCount,
    GpuUploadBytes,
    GpuImageUploadWrites,
    GpuImageSharedResolves,
    GpuImageSharedUploadWrites,
    GpuImageSharedUploadBytes,
    GpuImageSharedResidentBytes,
    GpuImageCacheKeyAllocations,
    GpuImageCachePruneVisits,
    GpuImageCacheAdmissionRejects,
    GpuImageInvalidPayloads,
    GpuImageCacheResidentBytes,
    GpuDrawCalls,
    GpuCompiledDrawCalls,
    GpuRenderPasses,
    GpuTimestampSupportedPresentCount,
    GpuTimeUs,
    GpuProfileLatencyFrames,
    GpuVisibleCommands,
    GpuVisibleCommandPayloadBytes,
    GpuVisibleCommandStyles,
    GpuVisibleDrawItems,
    GpuCompiledDrawItems,
    GpuCommandVisibilityScans,
    GpuCommandStatsCacheHits,
    GpuSolidVertices,
    GpuCompiledSolidVertices,
    GpuSolidInstances,
    GpuCompiledSolidInstances,
    GpuImageVertices,
    GpuCompiledImageVertices,
    GpuBatchLayers,
    GpuCompiledBatchLayers,
    GpuBatchDependencies,
    GpuCompiledBatchDependencies,
    GpuBatchMerges,
    GpuCompiledBatchMerges,
    GpuOverlapCandidates,
    GpuBatchPlanBuilds,
    GpuBatchPlanCacheHits,
    GpuVertexBufferCreates,
    GpuVertexUploadBytes,
    GpuRetainedCacheCopyBytes,
    GpuTextShapes,
    GpuTextRendererBuilds,
    GpuTextRendererCacheHits,
    GpuTextPrepareFailures,
    GpuImagePrepareCommandVisits,
    GpuImagePrepareCacheHits,
}

#[cfg(feature = "profiling")]
thread_local! {
    static CURRENT_SCENARIO: Cell<UiPerfScenario> = const { Cell::new(UiPerfScenario::Startup) };
}

#[cfg(feature = "profiling")]
pub(crate) struct UiPerfScenarioGuard {
    previous: UiPerfScenario,
}

#[cfg(not(feature = "profiling"))]
pub(crate) struct UiPerfScenarioGuard;

#[cfg(feature = "profiling")]
impl Drop for UiPerfScenarioGuard {
    fn drop(&mut self) {
        CURRENT_SCENARIO.with(|current| current.set(self.previous));
    }
}

#[cfg(feature = "profiling")]
pub(crate) struct UiPerfScenarioTimer {
    scenario: UiPerfScenario,
    start: Instant,
}

#[cfg(not(feature = "profiling"))]
pub(crate) struct UiPerfScenarioTimer;

#[cfg(feature = "profiling")]
impl Drop for UiPerfScenarioTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros().min(u64::MAX as u128) as f64;
        record_ui_perf_counter(self.scenario, UiPerfCounter::FrameDurationUs, elapsed);
    }
}

pub(crate) fn enter_ui_perf_scenario(scenario: UiPerfScenario) -> UiPerfScenarioGuard {
    #[cfg(feature = "profiling")]
    {
        let previous = CURRENT_SCENARIO.with(|current| {
            let previous = current.get();
            current.set(scenario);
            previous
        });
        UiPerfScenarioGuard { previous }
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = scenario;
        UiPerfScenarioGuard
    }
}

pub(crate) fn time_ui_perf_scenario(scenario: UiPerfScenario) -> UiPerfScenarioTimer {
    #[cfg(feature = "profiling")]
    {
        UiPerfScenarioTimer {
            scenario,
            start: Instant::now(),
        }
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = UiPerfCounter::FrameDurationUs;
        let _ = scenario;
        UiPerfScenarioTimer
    }
}

pub(crate) fn current_ui_perf_scenario() -> UiPerfScenario {
    #[cfg(feature = "profiling")]
    {
        CURRENT_SCENARIO.with(|current| current.get())
    }
    #[cfg(not(feature = "profiling"))]
    {
        UiPerfScenario::Startup
    }
}

pub(crate) fn record_current_ui_perf_counter(counter: UiPerfCounter, value: f64) {
    record_ui_perf_counter(current_ui_perf_scenario(), counter, value);
}

pub(crate) fn record_ui_perf_counter(scenario: UiPerfScenario, counter: UiPerfCounter, value: f64) {
    #[cfg(feature = "profiling")]
    {
        zircon_runtime::profile_counter!("editor", counter_name(scenario, counter), value);
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = (scenario, counter, value);
    }
}

#[cfg(feature = "profiling")]
macro_rules! counter_name_for_prefix {
    ($counter:expr, $prefix:literal) => {
        match $counter {
            UiPerfCounter::FrameDurationUs => concat!($prefix, ".frame_duration_us"),
            UiPerfCounter::InputToDamageUs => concat!($prefix, ".input_to_damage_us"),
            UiPerfCounter::DamageToSubmitUs => concat!($prefix, ".damage_to_submit_us"),
            UiPerfCounter::HostInvalidationTransactionCount => {
                concat!($prefix, ".host_invalidation_transaction_count")
            }
            UiPerfCounter::HostInvalidationScopeCount => {
                concat!($prefix, ".host_invalidation_scope_count")
            }
            UiPerfCounter::HostInvalidationLegacyDirtyTransactionCount => {
                concat!($prefix, ".host_invalidation_legacy_dirty_transaction_count")
            }
            UiPerfCounter::HostInvalidationFullTargetCount => {
                concat!($prefix, ".host_invalidation_full_target_count")
            }
            UiPerfCounter::HostInvalidationShellContentTargetCount => {
                concat!($prefix, ".host_invalidation_shell_content_target_count")
            }
            UiPerfCounter::HostInvalidationWorkbenchProjectionTargetCount => {
                concat!(
                    $prefix,
                    ".host_invalidation_workbench_projection_target_count"
                )
            }
            UiPerfCounter::HostInvalidationViewPresentationTargetCount => {
                concat!($prefix, ".host_invalidation_view_presentation_target_count")
            }
            UiPerfCounter::HostInvalidationWindowMetricsTargetCount => {
                concat!($prefix, ".host_invalidation_window_metrics_target_count")
            }
            UiPerfCounter::HostInvalidationPaintOnlyTargetCount => {
                concat!($prefix, ".host_invalidation_paint_only_target_count")
            }
            UiPerfCounter::SlowPathRebuildCount => {
                concat!($prefix, ".slow_path_rebuild_count")
            }
            UiPerfCounter::ScopedPresentationPatchCount => {
                concat!($prefix, ".scoped_presentation_patch_count")
            }
            UiPerfCounter::ScopedPresentationFloatingWindowRowsVisited => {
                concat!($prefix, ".scoped_presentation_floating_window_rows_visited")
            }
            UiPerfCounter::ScopedPresentationFloatingWindowRowsCloned => {
                concat!($prefix, ".scoped_presentation_floating_window_rows_cloned")
            }
            UiPerfCounter::ScopedPresentationNativePresenterVisitCount => {
                concat!($prefix, ".scoped_presentation_native_presenter_visit_count")
            }
            UiPerfCounter::ScopedPresentationDamageRegionCount => {
                concat!($prefix, ".scoped_presentation_damage_region_count")
            }
            UiPerfCounter::ScopedPresentationProjectionMissingCount => {
                concat!($prefix, ".scoped_presentation_projection_missing_count")
            }
            UiPerfCounter::ScopedPresentationPresenterCoverageFallbackCount => {
                concat!(
                    $prefix,
                    ".scoped_presentation_presenter_coverage_fallback_count"
                )
            }
            UiPerfCounter::RenderPathCount => concat!($prefix, ".render_path_count"),
            UiPerfCounter::PresentationRebuildCount => {
                concat!($prefix, ".presentation_rebuild_count")
            }
            UiPerfCounter::AssetEditorPanePresentationBuildCount => {
                concat!($prefix, ".asset_editor_pane_presentation_build_count")
            }
            UiPerfCounter::AssetEditorPaneReflectionBuildCount => {
                concat!($prefix, ".asset_editor_pane_reflection_build_count")
            }
            UiPerfCounter::AssetEditorPanePreviewBuildCount => {
                concat!($prefix, ".asset_editor_pane_preview_build_count")
            }
            UiPerfCounter::AssetEditorPaneSourceBuildCount => {
                concat!($prefix, ".asset_editor_pane_source_build_count")
            }
            UiPerfCounter::AssetEditorPaneInspectorBuildCount => {
                concat!($prefix, ".asset_editor_pane_inspector_build_count")
            }
            UiPerfCounter::AssetEditorPaneStyleBuildCount => {
                concat!($prefix, ".asset_editor_pane_style_build_count")
            }
            UiPerfCounter::AssetEditorPaneThemeBuildCount => {
                concat!($prefix, ".asset_editor_pane_theme_build_count")
            }
            UiPerfCounter::AssetEditorPaneCommandAvailabilityBuildCount => {
                concat!(
                    $prefix,
                    ".asset_editor_pane_command_availability_build_count"
                )
            }
            UiPerfCounter::FullPaintCount => concat!($prefix, ".full_paint_count"),
            UiPerfCounter::RegionPaintCount => concat!($prefix, ".region_paint_count"),
            UiPerfCounter::PaintedPixels => concat!($prefix, ".painted_pixels"),
            UiPerfCounter::PresentedSurfacePixels => {
                concat!($prefix, ".presented_surface_pixels")
            }
            UiPerfCounter::RedrawFullFrame => concat!($prefix, ".redraw_full_frame"),
            UiPerfCounter::RedrawRegion => concat!($prefix, ".redraw_region"),
            UiPerfCounter::DirtyLayout => concat!($prefix, ".dirty_layout"),
            UiPerfCounter::DirtyPresentation => concat!($prefix, ".dirty_presentation"),
            UiPerfCounter::DirtyRender => concat!($prefix, ".dirty_render"),
            UiPerfCounter::DirtyPaintOnly => concat!($prefix, ".dirty_paint_only"),
            UiPerfCounter::ChromeSnapshotCount => concat!($prefix, ".chrome_snapshot_count"),
            UiPerfCounter::WorkbenchModelBuildCount => {
                concat!($prefix, ".workbench_model_build_count")
            }
            UiPerfCounter::HierarchyScrollDispatchCount => {
                concat!($prefix, ".hierarchy_scroll_dispatch_count")
            }
            UiPerfCounter::HierarchySurfaceRebuildCount => {
                concat!($prefix, ".hierarchy_surface_rebuild_count")
            }
            UiPerfCounter::HierarchyRowInsertCount => {
                concat!($prefix, ".hierarchy_row_insert_count")
            }
            UiPerfCounter::HierarchyDispatcherRebuildCount => {
                concat!($prefix, ".hierarchy_dispatcher_rebuild_count")
            }
            UiPerfCounter::HierarchyRouteMapRebuildCount => {
                concat!($prefix, ".hierarchy_route_map_rebuild_count")
            }
            UiPerfCounter::WelcomeRecentScrollDispatchCount => {
                concat!($prefix, ".welcome_recent_scroll_dispatch_count")
            }
            UiPerfCounter::WelcomeRecentSurfaceRebuildCount => {
                concat!($prefix, ".welcome_recent_surface_rebuild_count")
            }
            UiPerfCounter::WelcomeRecentAuthorityRebuildCount => {
                concat!($prefix, ".welcome_recent_authority_rebuild_count")
            }
            UiPerfCounter::WelcomeRecentRowInsertCount => {
                concat!($prefix, ".welcome_recent_row_insert_count")
            }
            UiPerfCounter::WelcomeRecentGeometryPatchCount => {
                concat!($prefix, ".welcome_recent_geometry_patch_count")
            }
            UiPerfCounter::WelcomeRecentDispatcherRebuildCount => {
                concat!($prefix, ".welcome_recent_dispatcher_rebuild_count")
            }
            UiPerfCounter::WelcomeRecentRouteMapRebuildCount => {
                concat!($prefix, ".welcome_recent_route_map_rebuild_count")
            }
            UiPerfCounter::ShellDragAuthorityRebuildCount => {
                concat!($prefix, ".shell_drag_authority_rebuild_count")
            }
            UiPerfCounter::ShellDragNodeInsertCount => {
                concat!($prefix, ".shell_drag_node_insert_count")
            }
            UiPerfCounter::ShellDragGeometryPatchCount => {
                concat!($prefix, ".shell_drag_geometry_patch_count")
            }
            UiPerfCounter::ShellDragNodePatchCount => {
                concat!($prefix, ".shell_drag_node_patch_count")
            }
            UiPerfCounter::ShellDragDispatcherRebuildCount => {
                concat!($prefix, ".shell_drag_dispatcher_rebuild_count")
            }
            UiPerfCounter::ShellDragRouteMapRebuildCount => {
                concat!($prefix, ".shell_drag_route_map_rebuild_count")
            }
            UiPerfCounter::WorkbenchHitIndexBuildCount => {
                concat!($prefix, ".workbench_hit_index_build_count")
            }
            UiPerfCounter::WorkbenchHitIndexQueryCount => {
                concat!($prefix, ".workbench_hit_index_query_count")
            }
            UiPerfCounter::PanePopupIndexQueryCount => {
                concat!($prefix, ".pane_popup_index_query_count")
            }
            UiPerfCounter::PanePopupIndexCandidateCount => {
                concat!($prefix, ".pane_popup_index_candidate_count")
            }
            UiPerfCounter::VisualAssetCacheHitCount => {
                concat!($prefix, ".visual_asset_cache_hit_count")
            }
            UiPerfCounter::VisualAssetCacheMissCount => {
                concat!($prefix, ".visual_asset_cache_miss_count")
            }
            UiPerfCounter::VisualAssetCacheCandidateBuildCount => {
                concat!($prefix, ".visual_asset_cache_candidate_build_count")
            }
            UiPerfCounter::SvgTreeCacheMemoryHitCount => {
                concat!($prefix, ".svg_tree_cache_memory_hit_count")
            }
            UiPerfCounter::SvgTreeCacheMissCount => {
                concat!($prefix, ".svg_tree_cache_miss_count")
            }
            UiPerfCounter::WorkbenchPaintIndexQueryCount => {
                concat!($prefix, ".workbench_paint_index_query_count")
            }
            UiPerfCounter::WorkbenchPaintIndexCandidateCount => {
                concat!($prefix, ".workbench_paint_index_candidate_count")
            }
            UiPerfCounter::ChromeCommandFullRebuildCount => {
                concat!($prefix, ".chrome_command_full_rebuild_count")
            }
            UiPerfCounter::ChromeCommandPatchCount => {
                concat!($prefix, ".chrome_command_patch_count")
            }
            UiPerfCounter::PresentationGenerationReadCount => {
                concat!($prefix, ".presentation_generation_read_count")
            }
            UiPerfCounter::PresentationSnapshotReadCount => {
                concat!($prefix, ".presentation_snapshot_read_count")
            }
            UiPerfCounter::TemplateNodeVisitCount => {
                concat!($prefix, ".template_node_visit_count")
            }
            UiPerfCounter::TemplateNodeCloneCount => {
                concat!($prefix, ".template_node_clone_count")
            }
            UiPerfCounter::TemplateNodeDamageRejectCount => {
                concat!($prefix, ".template_node_damage_reject_count")
            }
            UiPerfCounter::FallbackSortCount => concat!($prefix, ".fallback_sort_count"),
            UiPerfCounter::ArtifactExportCount => concat!($prefix, ".artifact_export_count"),
            UiPerfCounter::SoftwareFallbackPresentCount => {
                concat!($prefix, ".software_fallback_present_count")
            }
            UiPerfCounter::GpuUploadBytes => concat!($prefix, ".gpu_upload_bytes"),
            UiPerfCounter::GpuImageUploadWrites => concat!($prefix, ".gpu_image_upload_writes"),
            UiPerfCounter::GpuImageSharedResolves => {
                concat!($prefix, ".gpu_image_shared_resolves")
            }
            UiPerfCounter::GpuImageSharedUploadWrites => {
                concat!($prefix, ".gpu_image_shared_upload_writes")
            }
            UiPerfCounter::GpuImageSharedUploadBytes => {
                concat!($prefix, ".gpu_image_shared_upload_bytes")
            }
            UiPerfCounter::GpuImageSharedResidentBytes => {
                concat!($prefix, ".gpu_image_shared_resident_bytes")
            }
            UiPerfCounter::GpuImageCacheKeyAllocations => {
                concat!($prefix, ".gpu_image_cache_key_allocations")
            }
            UiPerfCounter::GpuImageCachePruneVisits => {
                concat!($prefix, ".gpu_image_cache_prune_visits")
            }
            UiPerfCounter::GpuImageCacheAdmissionRejects => {
                concat!($prefix, ".gpu_image_cache_admission_rejects")
            }
            UiPerfCounter::GpuImageInvalidPayloads => {
                concat!($prefix, ".gpu_image_invalid_payloads")
            }
            UiPerfCounter::GpuImageCacheResidentBytes => {
                concat!($prefix, ".gpu_image_cache_resident_bytes")
            }
            UiPerfCounter::GpuDrawCalls => concat!($prefix, ".gpu_draw_calls"),
            UiPerfCounter::GpuCompiledDrawCalls => concat!($prefix, ".gpu_compiled_draw_calls"),
            UiPerfCounter::GpuRenderPasses => concat!($prefix, ".gpu_render_passes"),
            UiPerfCounter::GpuTimestampSupportedPresentCount => {
                concat!($prefix, ".gpu_timestamp_supported_present_count")
            }
            UiPerfCounter::GpuTimeUs => concat!($prefix, ".gpu_time_us"),
            UiPerfCounter::GpuProfileLatencyFrames => {
                concat!($prefix, ".gpu_profile_latency_frames")
            }
            UiPerfCounter::GpuVisibleCommands => concat!($prefix, ".gpu_visible_commands"),
            UiPerfCounter::GpuVisibleCommandPayloadBytes => {
                concat!($prefix, ".gpu_visible_command_payload_bytes")
            }
            UiPerfCounter::GpuVisibleCommandStyles => {
                concat!($prefix, ".gpu_visible_command_styles")
            }
            UiPerfCounter::GpuVisibleDrawItems => concat!($prefix, ".gpu_visible_draw_items"),
            UiPerfCounter::GpuCompiledDrawItems => concat!($prefix, ".gpu_compiled_draw_items"),
            UiPerfCounter::GpuCommandVisibilityScans => {
                concat!($prefix, ".gpu_command_visibility_scans")
            }
            UiPerfCounter::GpuCommandStatsCacheHits => {
                concat!($prefix, ".gpu_command_stats_cache_hits")
            }
            UiPerfCounter::GpuSolidVertices => concat!($prefix, ".gpu_solid_vertices"),
            UiPerfCounter::GpuCompiledSolidVertices => {
                concat!($prefix, ".gpu_compiled_solid_vertices")
            }
            UiPerfCounter::GpuSolidInstances => concat!($prefix, ".gpu_solid_instances"),
            UiPerfCounter::GpuCompiledSolidInstances => {
                concat!($prefix, ".gpu_compiled_solid_instances")
            }
            UiPerfCounter::GpuImageVertices => concat!($prefix, ".gpu_image_vertices"),
            UiPerfCounter::GpuCompiledImageVertices => {
                concat!($prefix, ".gpu_compiled_image_vertices")
            }
            UiPerfCounter::GpuBatchLayers => concat!($prefix, ".gpu_batch_layers"),
            UiPerfCounter::GpuCompiledBatchLayers => {
                concat!($prefix, ".gpu_compiled_batch_layers")
            }
            UiPerfCounter::GpuBatchDependencies => concat!($prefix, ".gpu_batch_dependencies"),
            UiPerfCounter::GpuCompiledBatchDependencies => {
                concat!($prefix, ".gpu_compiled_batch_dependencies")
            }
            UiPerfCounter::GpuBatchMerges => concat!($prefix, ".gpu_batch_merges"),
            UiPerfCounter::GpuCompiledBatchMerges => {
                concat!($prefix, ".gpu_compiled_batch_merges")
            }
            UiPerfCounter::GpuOverlapCandidates => concat!($prefix, ".gpu_overlap_candidates"),
            UiPerfCounter::GpuBatchPlanBuilds => concat!($prefix, ".gpu_batch_plan_builds"),
            UiPerfCounter::GpuBatchPlanCacheHits => {
                concat!($prefix, ".gpu_batch_plan_cache_hits")
            }
            UiPerfCounter::GpuVertexBufferCreates => {
                concat!($prefix, ".gpu_vertex_buffer_creates")
            }
            UiPerfCounter::GpuVertexUploadBytes => concat!($prefix, ".gpu_vertex_upload_bytes"),
            UiPerfCounter::GpuRetainedCacheCopyBytes => {
                concat!($prefix, ".gpu_retained_cache_copy_bytes")
            }
            UiPerfCounter::GpuTextShapes => concat!($prefix, ".gpu_text_shapes"),
            UiPerfCounter::GpuTextRendererBuilds => {
                concat!($prefix, ".gpu_text_renderer_builds")
            }
            UiPerfCounter::GpuTextRendererCacheHits => {
                concat!($prefix, ".gpu_text_renderer_cache_hits")
            }
            UiPerfCounter::GpuTextPrepareFailures => {
                concat!($prefix, ".gpu_text_prepare_failures")
            }
            UiPerfCounter::GpuImagePrepareCommandVisits => {
                concat!($prefix, ".gpu_image_prepare_command_visits")
            }
            UiPerfCounter::GpuImagePrepareCacheHits => {
                concat!($prefix, ".gpu_image_prepare_cache_hits")
            }
        }
    };
}

#[cfg(feature = "profiling")]
fn counter_name(scenario: UiPerfScenario, counter: UiPerfCounter) -> &'static str {
    match scenario {
        UiPerfScenario::Startup => counter_name_for_prefix!(counter, "ui.startup"),
        UiPerfScenario::IdleHover => counter_name_for_prefix!(counter, "ui.idle_hover"),
        UiPerfScenario::Click => counter_name_for_prefix!(counter, "ui.click"),
        UiPerfScenario::Drag => counter_name_for_prefix!(counter, "ui.drag"),
        UiPerfScenario::DrawerResize => counter_name_for_prefix!(counter, "ui.drawer_resize"),
        UiPerfScenario::WindowResize => counter_name_for_prefix!(counter, "ui.window_resize"),
        UiPerfScenario::AssetRefresh => counter_name_for_prefix!(counter, "ui.asset_refresh"),
        UiPerfScenario::ViewportImage => counter_name_for_prefix!(counter, "ui.viewport_image"),
        UiPerfScenario::ShellContent => counter_name_for_prefix!(counter, "ui.shell_content"),
    }
}

#[cfg(all(test, feature = "profiling"))]
mod tests {
    use super::{counter_name, UiPerfCounter, UiPerfScenario};

    #[test]
    fn shell_content_timer_uses_a_dedicated_counter_prefix() {
        assert_eq!(
            counter_name(UiPerfScenario::ShellContent, UiPerfCounter::FrameDurationUs),
            "ui.shell_content.frame_duration_us"
        );
    }

    #[test]
    fn overlap_candidates_counter_uses_the_active_scenario_prefix() {
        assert_eq!(
            counter_name(UiPerfScenario::Startup, UiPerfCounter::GpuOverlapCandidates),
            "ui.startup.gpu_overlap_candidates"
        );
        assert_eq!(
            counter_name(
                UiPerfScenario::ViewportImage,
                UiPerfCounter::GpuOverlapCandidates,
            ),
            "ui.viewport_image.gpu_overlap_candidates"
        );
        assert_eq!(
            counter_name(UiPerfScenario::Startup, UiPerfCounter::GpuSolidVertices),
            "ui.startup.gpu_solid_vertices"
        );
        assert_eq!(
            counter_name(UiPerfScenario::Startup, UiPerfCounter::GpuSolidInstances),
            "ui.startup.gpu_solid_instances"
        );
        assert_eq!(
            counter_name(UiPerfScenario::Startup, UiPerfCounter::GpuBatchMerges),
            "ui.startup.gpu_batch_merges"
        );
        assert_eq!(
            counter_name(UiPerfScenario::Startup, UiPerfCounter::GpuImageUploadWrites),
            "ui.startup.gpu_image_upload_writes"
        );
    }

    #[test]
    fn compiled_ui_presenter_reuse_counters_use_the_active_scenario_prefix() {
        let cases = [
            (
                UiPerfCounter::HostInvalidationTransactionCount,
                "ui.startup.host_invalidation_transaction_count",
            ),
            (
                UiPerfCounter::HostInvalidationScopeCount,
                "ui.startup.host_invalidation_scope_count",
            ),
            (
                UiPerfCounter::HostInvalidationLegacyDirtyTransactionCount,
                "ui.startup.host_invalidation_legacy_dirty_transaction_count",
            ),
            (
                UiPerfCounter::HostInvalidationFullTargetCount,
                "ui.startup.host_invalidation_full_target_count",
            ),
            (
                UiPerfCounter::HostInvalidationShellContentTargetCount,
                "ui.startup.host_invalidation_shell_content_target_count",
            ),
            (
                UiPerfCounter::HostInvalidationWorkbenchProjectionTargetCount,
                "ui.startup.host_invalidation_workbench_projection_target_count",
            ),
            (
                UiPerfCounter::HostInvalidationViewPresentationTargetCount,
                "ui.startup.host_invalidation_view_presentation_target_count",
            ),
            (
                UiPerfCounter::HostInvalidationWindowMetricsTargetCount,
                "ui.startup.host_invalidation_window_metrics_target_count",
            ),
            (
                UiPerfCounter::HostInvalidationPaintOnlyTargetCount,
                "ui.startup.host_invalidation_paint_only_target_count",
            ),
            (
                UiPerfCounter::PresentedSurfacePixels,
                "ui.startup.presented_surface_pixels",
            ),
            (
                UiPerfCounter::GpuTimestampSupportedPresentCount,
                "ui.startup.gpu_timestamp_supported_present_count",
            ),
            (
                UiPerfCounter::GpuImageSharedResidentBytes,
                "ui.startup.gpu_image_shared_resident_bytes",
            ),
            (
                UiPerfCounter::ScopedPresentationFloatingWindowRowsVisited,
                "ui.startup.scoped_presentation_floating_window_rows_visited",
            ),
            (
                UiPerfCounter::ScopedPresentationFloatingWindowRowsCloned,
                "ui.startup.scoped_presentation_floating_window_rows_cloned",
            ),
            (
                UiPerfCounter::ScopedPresentationNativePresenterVisitCount,
                "ui.startup.scoped_presentation_native_presenter_visit_count",
            ),
            (
                UiPerfCounter::ScopedPresentationDamageRegionCount,
                "ui.startup.scoped_presentation_damage_region_count",
            ),
            (
                UiPerfCounter::ScopedPresentationProjectionMissingCount,
                "ui.startup.scoped_presentation_projection_missing_count",
            ),
            (
                UiPerfCounter::ScopedPresentationPresenterCoverageFallbackCount,
                "ui.startup.scoped_presentation_presenter_coverage_fallback_count",
            ),
            (
                UiPerfCounter::AssetEditorPanePresentationBuildCount,
                "ui.startup.asset_editor_pane_presentation_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneReflectionBuildCount,
                "ui.startup.asset_editor_pane_reflection_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPanePreviewBuildCount,
                "ui.startup.asset_editor_pane_preview_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneSourceBuildCount,
                "ui.startup.asset_editor_pane_source_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneInspectorBuildCount,
                "ui.startup.asset_editor_pane_inspector_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneStyleBuildCount,
                "ui.startup.asset_editor_pane_style_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneThemeBuildCount,
                "ui.startup.asset_editor_pane_theme_build_count",
            ),
            (
                UiPerfCounter::AssetEditorPaneCommandAvailabilityBuildCount,
                "ui.startup.asset_editor_pane_command_availability_build_count",
            ),
            (
                UiPerfCounter::InputToDamageUs,
                "ui.startup.input_to_damage_us",
            ),
            (
                UiPerfCounter::DamageToSubmitUs,
                "ui.startup.damage_to_submit_us",
            ),
            (
                UiPerfCounter::WorkbenchHitIndexBuildCount,
                "ui.startup.workbench_hit_index_build_count",
            ),
            (
                UiPerfCounter::WorkbenchHitIndexQueryCount,
                "ui.startup.workbench_hit_index_query_count",
            ),
            (
                UiPerfCounter::PanePopupIndexQueryCount,
                "ui.startup.pane_popup_index_query_count",
            ),
            (
                UiPerfCounter::PanePopupIndexCandidateCount,
                "ui.startup.pane_popup_index_candidate_count",
            ),
            (
                UiPerfCounter::VisualAssetCacheHitCount,
                "ui.startup.visual_asset_cache_hit_count",
            ),
            (
                UiPerfCounter::VisualAssetCacheMissCount,
                "ui.startup.visual_asset_cache_miss_count",
            ),
            (
                UiPerfCounter::VisualAssetCacheCandidateBuildCount,
                "ui.startup.visual_asset_cache_candidate_build_count",
            ),
            (
                UiPerfCounter::SvgTreeCacheMemoryHitCount,
                "ui.startup.svg_tree_cache_memory_hit_count",
            ),
            (
                UiPerfCounter::SvgTreeCacheMissCount,
                "ui.startup.svg_tree_cache_miss_count",
            ),
            (
                UiPerfCounter::WorkbenchPaintIndexQueryCount,
                "ui.startup.workbench_paint_index_query_count",
            ),
            (
                UiPerfCounter::WorkbenchPaintIndexCandidateCount,
                "ui.startup.workbench_paint_index_candidate_count",
            ),
            (
                UiPerfCounter::GpuRenderPasses,
                "ui.startup.gpu_render_passes",
            ),
            (
                UiPerfCounter::GpuCommandVisibilityScans,
                "ui.startup.gpu_command_visibility_scans",
            ),
            (
                UiPerfCounter::GpuCommandStatsCacheHits,
                "ui.startup.gpu_command_stats_cache_hits",
            ),
            (
                UiPerfCounter::GpuRetainedCacheCopyBytes,
                "ui.startup.gpu_retained_cache_copy_bytes",
            ),
            (
                UiPerfCounter::GpuImagePrepareCommandVisits,
                "ui.startup.gpu_image_prepare_command_visits",
            ),
            (
                UiPerfCounter::GpuImagePrepareCacheHits,
                "ui.startup.gpu_image_prepare_cache_hits",
            ),
            (
                UiPerfCounter::PresentationGenerationReadCount,
                "ui.startup.presentation_generation_read_count",
            ),
            (
                UiPerfCounter::PresentationSnapshotReadCount,
                "ui.startup.presentation_snapshot_read_count",
            ),
            (
                UiPerfCounter::TemplateNodeVisitCount,
                "ui.startup.template_node_visit_count",
            ),
            (
                UiPerfCounter::TemplateNodeCloneCount,
                "ui.startup.template_node_clone_count",
            ),
            (
                UiPerfCounter::TemplateNodeDamageRejectCount,
                "ui.startup.template_node_damage_reject_count",
            ),
            (
                UiPerfCounter::FallbackSortCount,
                "ui.startup.fallback_sort_count",
            ),
            (
                UiPerfCounter::ArtifactExportCount,
                "ui.startup.artifact_export_count",
            ),
        ];

        for (counter, expected) in cases {
            assert_eq!(counter_name(UiPerfScenario::Startup, counter), expected);
        }
    }

    #[test]
    fn hierarchy_scroll_work_counters_use_the_active_scenario_prefix() {
        let cases = [
            (
                UiPerfCounter::HierarchyScrollDispatchCount,
                "ui.idle_hover.hierarchy_scroll_dispatch_count",
            ),
            (
                UiPerfCounter::HierarchySurfaceRebuildCount,
                "ui.idle_hover.hierarchy_surface_rebuild_count",
            ),
            (
                UiPerfCounter::HierarchyRowInsertCount,
                "ui.idle_hover.hierarchy_row_insert_count",
            ),
            (
                UiPerfCounter::HierarchyDispatcherRebuildCount,
                "ui.idle_hover.hierarchy_dispatcher_rebuild_count",
            ),
            (
                UiPerfCounter::HierarchyRouteMapRebuildCount,
                "ui.idle_hover.hierarchy_route_map_rebuild_count",
            ),
        ];

        for (counter, expected) in cases {
            assert_eq!(counter_name(UiPerfScenario::IdleHover, counter), expected);
        }
    }

    #[test]
    fn welcome_recent_scroll_work_counters_use_the_active_scenario_prefix() {
        let cases = [
            (
                UiPerfCounter::WelcomeRecentScrollDispatchCount,
                "ui.idle_hover.welcome_recent_scroll_dispatch_count",
            ),
            (
                UiPerfCounter::WelcomeRecentSurfaceRebuildCount,
                "ui.idle_hover.welcome_recent_surface_rebuild_count",
            ),
            (
                UiPerfCounter::WelcomeRecentAuthorityRebuildCount,
                "ui.idle_hover.welcome_recent_authority_rebuild_count",
            ),
            (
                UiPerfCounter::WelcomeRecentRowInsertCount,
                "ui.idle_hover.welcome_recent_row_insert_count",
            ),
            (
                UiPerfCounter::WelcomeRecentGeometryPatchCount,
                "ui.idle_hover.welcome_recent_geometry_patch_count",
            ),
            (
                UiPerfCounter::WelcomeRecentDispatcherRebuildCount,
                "ui.idle_hover.welcome_recent_dispatcher_rebuild_count",
            ),
            (
                UiPerfCounter::WelcomeRecentRouteMapRebuildCount,
                "ui.idle_hover.welcome_recent_route_map_rebuild_count",
            ),
        ];

        for (counter, expected) in cases {
            assert_eq!(counter_name(UiPerfScenario::IdleHover, counter), expected);
        }
    }

    #[test]
    fn shell_drag_patch_counters_use_the_window_resize_prefix() {
        let cases = [
            (
                UiPerfCounter::ShellDragAuthorityRebuildCount,
                "ui.window_resize.shell_drag_authority_rebuild_count",
            ),
            (
                UiPerfCounter::ShellDragNodeInsertCount,
                "ui.window_resize.shell_drag_node_insert_count",
            ),
            (
                UiPerfCounter::ShellDragGeometryPatchCount,
                "ui.window_resize.shell_drag_geometry_patch_count",
            ),
            (
                UiPerfCounter::ShellDragNodePatchCount,
                "ui.window_resize.shell_drag_node_patch_count",
            ),
            (
                UiPerfCounter::ShellDragDispatcherRebuildCount,
                "ui.window_resize.shell_drag_dispatcher_rebuild_count",
            ),
            (
                UiPerfCounter::ShellDragRouteMapRebuildCount,
                "ui.window_resize.shell_drag_route_map_rebuild_count",
            ),
        ];

        for (counter, expected) in cases {
            assert_eq!(
                counter_name(UiPerfScenario::WindowResize, counter),
                expected
            );
        }
    }
}
