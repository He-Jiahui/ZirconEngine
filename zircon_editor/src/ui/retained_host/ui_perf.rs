#[cfg(feature = "profiling")]
use std::cell::Cell;
#[cfg(feature = "profiling")]
use std::time::Instant;

mod counter_batch;
mod counter_catalog;

pub(crate) use counter_batch::record_current_ui_perf_counter_batch;
pub(crate) use counter_catalog::UiPerfCounter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiPerfScenario {
    Startup,
    IdleHover,
    Click,
    Drag,
    DrawerResize,
    WindowResize,
    AssetRefresh,
    SessionHeartbeat,
    ViewportImage,
    ShellContent,
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
            UiPerfCounter::ShellPresentationBuildCount => {
                concat!($prefix, ".shell_presentation_build_count")
            }
            UiPerfCounter::HostSceneBuildCount => {
                concat!($prefix, ".host_scene_build_count")
            }
            UiPerfCounter::PaneProjectionBuildCount => {
                concat!($prefix, ".pane_projection_build_count")
            }
            UiPerfCounter::PresentationStructureGenerationChangeCount => {
                concat!($prefix, ".presentation_structure_generation_change_count")
            }
            UiPerfCounter::TemplateProjectionLayoutMeasureProbeNodeCount => {
                concat!(
                    $prefix,
                    ".template_projection_layout_measure_probe_node_count"
                )
            }
            UiPerfCounter::TemplateProjectionLayoutArrangeProbeNodeCount => {
                concat!(
                    $prefix,
                    ".template_projection_layout_arrange_probe_node_count"
                )
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
            UiPerfCounter::RedrawDamageRectCount => {
                concat!($prefix, ".redraw_damage_rect_count")
            }
            UiPerfCounter::RedrawDamageSourceRectCount => {
                concat!($prefix, ".redraw_damage_source_rect_count")
            }
            UiPerfCounter::RedrawDamageSimplificationCount => {
                concat!($prefix, ".redraw_damage_simplification_count")
            }
            UiPerfCounter::RedrawDamageRepresentedArea => {
                concat!($prefix, ".redraw_damage_represented_area")
            }
            UiPerfCounter::RedrawDamageBoundingArea => {
                concat!($prefix, ".redraw_damage_bounding_area")
            }
            UiPerfCounter::RedrawDamageBoundingOverdrawArea => {
                concat!($prefix, ".redraw_damage_bounding_overdraw_area")
            }
            UiPerfCounter::DirtyLayout => concat!($prefix, ".dirty_layout"),
            UiPerfCounter::DirtyPresentation => concat!($prefix, ".dirty_presentation"),
            UiPerfCounter::DirtyRender => concat!($prefix, ".dirty_render"),
            UiPerfCounter::DirtyPaintOnly => concat!($prefix, ".dirty_paint_only"),
            UiPerfCounter::ChromeSnapshotCount => concat!($prefix, ".chrome_snapshot_count"),
            UiPerfCounter::WorkbenchModelBuildCount => {
                concat!($prefix, ".workbench_model_build_count")
            }
            UiPerfCounter::AssetPointerSnapshotCloneCount => {
                concat!($prefix, ".asset_pointer_snapshot_clone_count")
            }
            UiPerfCounter::AssetBrowserScrollDispatchCount => {
                concat!($prefix, ".asset_browser_scroll_dispatch_count")
            }
            UiPerfCounter::AssetBrowserLogicalItemCount => {
                concat!($prefix, ".asset_browser_logical_item_count")
            }
            UiPerfCounter::AssetBrowserMaterializedItemCount => {
                concat!($prefix, ".asset_browser_materialized_item_count")
            }
            UiPerfCounter::AssetBrowserMaterializedNodeCount => {
                concat!($prefix, ".asset_browser_materialized_node_count")
            }
            UiPerfCounter::AssetBrowserVisibleItemCount => {
                concat!($prefix, ".asset_browser_visible_item_count")
            }
            UiPerfCounter::AssetBrowserVisibleNodeCount => {
                concat!($prefix, ".asset_browser_visible_node_count")
            }
            UiPerfCounter::AssetBrowserProjectionBuildCount => {
                concat!($prefix, ".asset_browser_projection_build_count")
            }
            UiPerfCounter::AssetBrowserLogicalPaintChunkBuildCount => {
                concat!($prefix, ".asset_browser_logical_paint_chunk_build_count")
            }
            UiPerfCounter::AssetBrowserLogicalPaintChunkReuseCount => {
                concat!($prefix, ".asset_browser_logical_paint_chunk_reuse_count")
            }
            UiPerfCounter::AssetBrowserLogicalPaintItemProjectionCount => {
                concat!(
                    $prefix,
                    ".asset_browser_logical_paint_item_projection_count"
                )
            }
            UiPerfCounter::AssetContentGenerationIdentityParseCount => {
                concat!($prefix, ".asset_content_generation_identity_parse_count")
            }
            UiPerfCounter::AssetContentDescriptorLookupCount => {
                concat!($prefix, ".asset_content_descriptor_lookup_count")
            }
            UiPerfCounter::ConsoleLogicalLineCount => {
                concat!($prefix, ".console_logical_line_count")
            }
            UiPerfCounter::ConsoleMaterializedLineCount => {
                concat!($prefix, ".console_materialized_line_count")
            }
            UiPerfCounter::ConsoleMaterializedNodeCount => {
                concat!($prefix, ".console_materialized_node_count")
            }
            UiPerfCounter::ConsoleVisibleLineCount => {
                concat!($prefix, ".console_visible_line_count")
            }
            UiPerfCounter::ConsoleOverscanLineCount => {
                concat!($prefix, ".console_overscan_line_count")
            }
            UiPerfCounter::ConsoleProjectionClonedNodeCount => {
                concat!($prefix, ".console_projection_cloned_node_count")
            }
            UiPerfCounter::ConsoleProjectionFormattedIdCount => {
                concat!($prefix, ".console_projection_formatted_id_count")
            }
            UiPerfCounter::ConsoleEnteredLineCount => {
                concat!($prefix, ".console_entered_line_count")
            }
            UiPerfCounter::ConsoleExpiredLineCount => {
                concat!($prefix, ".console_expired_line_count")
            }
            UiPerfCounter::ConsoleSlotReboundCount => {
                concat!($prefix, ".console_slot_rebound_count")
            }
            UiPerfCounter::ConsoleProjectionGenerationReuseCount => {
                concat!($prefix, ".console_projection_generation_reuse_count")
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
            UiPerfCounter::VisualAssetAsyncEnqueuedCount => {
                concat!($prefix, ".visual_asset_async_enqueued_count")
            }
            UiPerfCounter::VisualAssetAsyncDeduplicatedCount => {
                concat!($prefix, ".visual_asset_async_deduplicated_count")
            }
            UiPerfCounter::VisualAssetAsyncCompletedCount => {
                concat!($prefix, ".visual_asset_async_completed_count")
            }
            UiPerfCounter::VisualAssetAsyncStaleDiscardCount => {
                concat!($prefix, ".visual_asset_async_stale_discard_count")
            }
            UiPerfCounter::VisualAssetAsyncSubmissionRejectedCount => {
                concat!($prefix, ".visual_asset_async_submission_rejected_count")
            }
            UiPerfCounter::VisualAssetAsyncCompletionRedrawCount => {
                concat!($prefix, ".visual_asset_async_completion_redraw_count")
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
            UiPerfCounter::GpuImageDeviceAllocationCount => {
                concat!($prefix, ".gpu_image_device_allocation_count")
            }
            UiPerfCounter::GpuImageDeviceAllocationBytes => {
                concat!($prefix, ".gpu_image_device_allocation_bytes")
            }
            UiPerfCounter::GpuImageRegistryEvictedPinnedBytes => {
                concat!($prefix, ".gpu_image_registry_evicted_pinned_bytes")
            }
            UiPerfCounter::GpuImageSurfacePinCount => {
                concat!($prefix, ".gpu_image_surface_pin_count")
            }
            UiPerfCounter::GpuImageInFlightPresentPinCount => {
                concat!($prefix, ".gpu_image_in_flight_present_pin_count")
            }
            UiPerfCounter::GpuImageEvictionCompletionCount => {
                concat!($prefix, ".gpu_image_eviction_completion_count")
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
        UiPerfScenario::SessionHeartbeat => {
            counter_name_for_prefix!(counter, "ui.session_heartbeat")
        }
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
    fn window_resize_structure_counters_use_the_active_scenario_prefix() {
        let cases = [
            (
                UiPerfCounter::ShellPresentationBuildCount,
                "ui.window_resize.shell_presentation_build_count",
            ),
            (
                UiPerfCounter::HostSceneBuildCount,
                "ui.window_resize.host_scene_build_count",
            ),
            (
                UiPerfCounter::PaneProjectionBuildCount,
                "ui.window_resize.pane_projection_build_count",
            ),
            (
                UiPerfCounter::PresentationStructureGenerationChangeCount,
                "ui.window_resize.presentation_structure_generation_change_count",
            ),
        ];

        for (counter, expected) in cases {
            assert_eq!(
                counter_name(UiPerfScenario::WindowResize, counter),
                expected
            );
        }
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
    fn asset_browser_scale_counters_use_the_scroll_scenario_prefix() {
        let cases = [
            (
                UiPerfCounter::AssetBrowserScrollDispatchCount,
                "ui.idle_hover.asset_browser_scroll_dispatch_count",
            ),
            (
                UiPerfCounter::AssetBrowserLogicalItemCount,
                "ui.idle_hover.asset_browser_logical_item_count",
            ),
            (
                UiPerfCounter::AssetBrowserMaterializedItemCount,
                "ui.idle_hover.asset_browser_materialized_item_count",
            ),
            (
                UiPerfCounter::AssetBrowserMaterializedNodeCount,
                "ui.idle_hover.asset_browser_materialized_node_count",
            ),
            (
                UiPerfCounter::AssetBrowserVisibleItemCount,
                "ui.idle_hover.asset_browser_visible_item_count",
            ),
            (
                UiPerfCounter::AssetBrowserVisibleNodeCount,
                "ui.idle_hover.asset_browser_visible_node_count",
            ),
            (
                UiPerfCounter::AssetBrowserProjectionBuildCount,
                "ui.idle_hover.asset_browser_projection_build_count",
            ),
            (
                UiPerfCounter::AssetBrowserLogicalPaintChunkBuildCount,
                "ui.idle_hover.asset_browser_logical_paint_chunk_build_count",
            ),
            (
                UiPerfCounter::AssetBrowserLogicalPaintChunkReuseCount,
                "ui.idle_hover.asset_browser_logical_paint_chunk_reuse_count",
            ),
            (
                UiPerfCounter::AssetBrowserLogicalPaintItemProjectionCount,
                "ui.idle_hover.asset_browser_logical_paint_item_projection_count",
            ),
            (
                UiPerfCounter::AssetContentGenerationIdentityParseCount,
                "ui.idle_hover.asset_content_generation_identity_parse_count",
            ),
            (
                UiPerfCounter::AssetContentDescriptorLookupCount,
                "ui.idle_hover.asset_content_descriptor_lookup_count",
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
