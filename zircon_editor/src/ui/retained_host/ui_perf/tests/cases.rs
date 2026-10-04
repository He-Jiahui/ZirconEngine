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
