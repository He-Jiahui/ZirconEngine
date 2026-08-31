function Test-ZirconAssetRefreshCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName -ne "asset_refresh") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Asset refresh gate could not find timeline.zrtrace.json."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $changeCounters = @($snapshot.counters) | Where-Object {
        $_.name -in @(
            "ui.asset_refresh.asset_change_count",
            "ui.asset_refresh.editor_change_count",
            "ui.asset_refresh.resource_change_count"
        )
    }
    $changeCount = 0
    foreach ($counter in $changeCounters) {
        $changeCount += [int64][Math]::Max(0, [Math]::Round([double]$counter.value))
    }
    Write-Host ("- asset_refresh_change_count={0}" -f $changeCount)
    if ($changeCount -le 0) {
        Write-Warning "Scenario 'asset_refresh' did not record any asset/editor/resource change counter."
        return $false
    }
    $fullInvalidationCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.asset_refresh.visual_asset_full_invalidation_count")
    $targetedInvalidationCount = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @(
        "ui.asset_refresh.visual_asset_targeted_invalidation_count",
        "ui.asset_refresh.svg_tree_targeted_invalidation_count"
    )
    $reconcileVisitCount = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @(
        "ui.asset_refresh.visual_asset_reconcile_source_visit_count",
        "ui.asset_refresh.svg_tree_reconcile_source_visit_count"
    )
    $reconciledInvalidationCount = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @(
        "ui.asset_refresh.visual_asset_reconciled_invalidation_count",
        "ui.asset_refresh.svg_tree_reconciled_invalidation_count"
    )
    $paintChunkBuildCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.asset_refresh.asset_browser_logical_paint_chunk_build_count")
    $paintChunkReuseCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.asset_refresh.asset_browser_logical_paint_chunk_reuse_count")
    $paintItemProjectionCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.asset_refresh.asset_browser_logical_paint_item_projection_count")
    Write-Host ("- asset_refresh_targeted_invalidation={0} reconcile_visits={1} reconciled_invalidation={2} full_invalidation={3} paint_chunk_build={4} paint_chunk_reuse={5} paint_item_projection={6}" -f `
            $targetedInvalidationCount, $reconcileVisitCount,
            $reconciledInvalidationCount, $fullInvalidationCount,
            $paintChunkBuildCount, $paintChunkReuseCount, $paintItemProjectionCount)
    if ($fullInvalidationCount -gt 0) {
        Write-Warning "Scenario 'asset_refresh' cleared all visual asset caches for a non-visual project change."
        return $false
    }
    if ($paintItemProjectionCount -gt (64 * $paintChunkBuildCount) -or
        ($paintItemProjectionCount -gt 0 -and $paintChunkBuildCount -le 0) -or
        ($paintChunkBuildCount -gt 0 -and $paintItemProjectionCount -le 0)) {
        Write-Warning "Scenario 'asset_refresh' reported inconsistent Asset Browser logical paint chunk work."
        return $false
    }
    return $true
}

function Test-ZirconWindowResizeCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ((Resolve-InteractionScenarioName -ScenarioName $ScenarioName) -ne "window_resize") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Window resize gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_steps) `
            -MaxCpuMsPerOperation 35.0)) {
        Write-Warning "Window resize gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }
    $counterTotals = @{}
    foreach ($name in @(
            "ui.window_resize.command_snapshot_build_count",
            "ui.window_resize.command_snapshot_reuse_count",
            "ui.window_resize.surface_reconfigure_count",
            "ui.window_resize.duplicate_size_suppressed_count",
            "ui.window_resize.duplicate_scale_suppressed_count",
            "ui.window_resize.workbench_model_build_count",
            "ui.window_resize.chrome_snapshot_count",
            "ui.window_resize.presentation_rebuild_count",
            "ui.window_resize.shell_presentation_build_count",
            "ui.window_resize.host_scene_build_count",
            "ui.window_resize.pane_projection_build_count",
            "ui.window_resize.presentation_structure_generation_change_count",
            "ui.window_resize.template_projection_layout_measure_probe_node_count",
            "ui.window_resize.template_projection_layout_arrange_probe_node_count",
            "ui.window_resize.asset_pointer_snapshot_clone_count",
            "ui.window_resize.gpu_image_vertices",
            "ui.window_resize.gpu_image_prepare_cache_hits",
            "ui.window_resize.gpu_image_prepare_command_visits",
            "ui.window_resize.gpu_image_upload_writes",
            "ui.window_resize.gpu_image_cache_key_allocations",
            "ui.window_resize.gpu_image_cache_admission_rejects",
            "ui.window_resize.gpu_image_invalid_payloads",
            "ui.window_resize.visual_asset_cache_hit_count",
            "ui.window_resize.visual_asset_cache_miss_count",
            "ui.window_resize.svg_tree_cache_memory_hit_count",
            "ui.window_resize.svg_tree_cache_miss_count",
            "ui.window_resize.visual_asset_full_invalidation_count",
            "ui.window_resize.shell_drag_authority_rebuild_count",
            "ui.window_resize.shell_drag_node_insert_count",
            "ui.window_resize.shell_drag_geometry_patch_count",
            "ui.window_resize.shell_drag_node_patch_count",
            "ui.window_resize.shell_drag_dispatcher_rebuild_count",
            "ui.window_resize.shell_drag_route_map_rebuild_count",
            "ui.window_metrics.pane_payload_cache_hit_count",
            "ui.window_metrics.pane_payload_cache_miss_count"
        )) {
        $counterTotals[$name] = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @($name)
    }
    $buildCount = $counterTotals["ui.window_resize.command_snapshot_build_count"]
    $reuseCount = $counterTotals["ui.window_resize.command_snapshot_reuse_count"]
    $surfaceCount = $counterTotals["ui.window_resize.surface_reconfigure_count"]
    $duplicateSizeCount = $counterTotals["ui.window_resize.duplicate_size_suppressed_count"]
    $duplicateScaleCount = $counterTotals["ui.window_resize.duplicate_scale_suppressed_count"]
    $modelCount = $counterTotals["ui.window_resize.workbench_model_build_count"]
    $chromeCount = $counterTotals["ui.window_resize.chrome_snapshot_count"]
    $presentationRebuildCount = $counterTotals["ui.window_resize.presentation_rebuild_count"]
    $shellPresentationBuildCount = $counterTotals["ui.window_resize.shell_presentation_build_count"]
    $hostSceneBuildCount = $counterTotals["ui.window_resize.host_scene_build_count"]
    $paneProjectionBuildCount = $counterTotals["ui.window_resize.pane_projection_build_count"]
    $structureGenerationChangeCount = $counterTotals["ui.window_resize.presentation_structure_generation_change_count"]
    $layoutMeasureProbeCount = $counterTotals["ui.window_resize.template_projection_layout_measure_probe_node_count"]
    $layoutArrangeProbeCount = $counterTotals["ui.window_resize.template_projection_layout_arrange_probe_node_count"]
    $assetPointerSnapshotCloneCount = $counterTotals["ui.window_resize.asset_pointer_snapshot_clone_count"]
    $imageVertexCount = $counterTotals["ui.window_resize.gpu_image_vertices"]
    $imagePrepareCacheHitCount = $counterTotals["ui.window_resize.gpu_image_prepare_cache_hits"]
    $imagePrepareCommandVisitCount = $counterTotals["ui.window_resize.gpu_image_prepare_command_visits"]
    $imageUploadCount = $counterTotals["ui.window_resize.gpu_image_upload_writes"]
    $imageAllocationCount = $counterTotals["ui.window_resize.gpu_image_cache_key_allocations"]
    $imageAdmissionRejectCount = $counterTotals["ui.window_resize.gpu_image_cache_admission_rejects"]
    $imageInvalidPayloadCount = $counterTotals["ui.window_resize.gpu_image_invalid_payloads"]
    $visualHitCount = $counterTotals["ui.window_resize.visual_asset_cache_hit_count"]
    $visualMissCount = $counterTotals["ui.window_resize.visual_asset_cache_miss_count"]
    $svgHitCount = $counterTotals["ui.window_resize.svg_tree_cache_memory_hit_count"]
    $svgMissCount = $counterTotals["ui.window_resize.svg_tree_cache_miss_count"]
    $visualFullInvalidationCount = $counterTotals["ui.window_resize.visual_asset_full_invalidation_count"]
    $shellDragAuthorityRebuildCount = $counterTotals["ui.window_resize.shell_drag_authority_rebuild_count"]
    $shellDragNodeInsertCount = $counterTotals["ui.window_resize.shell_drag_node_insert_count"]
    $shellDragGeometryPatchCount = $counterTotals["ui.window_resize.shell_drag_geometry_patch_count"]
    $shellDragNodePatchCount = $counterTotals["ui.window_resize.shell_drag_node_patch_count"]
    $shellDragDispatcherRebuildCount = $counterTotals["ui.window_resize.shell_drag_dispatcher_rebuild_count"]
    $shellDragRouteMapRebuildCount = $counterTotals["ui.window_resize.shell_drag_route_map_rebuild_count"]
    $panePayloadCacheHitCount = $counterTotals["ui.window_metrics.pane_payload_cache_hit_count"]
    $panePayloadCacheMissCount = $counterTotals["ui.window_metrics.pane_payload_cache_miss_count"]
    Write-Host ("- resize_snapshot_build={0} reuse={1} surface_reconfigure={2} duplicate_size_suppressed={3} duplicate_scale_suppressed={4} model_build={5} chrome_snapshot={6} presentation_rebuild={7} image_vertices={8} image_prepare_cache_hits={9} image_prepare_command_visits={10} image_uploads={11} image_allocations={12} visual_hits={13} visual_misses={14} svg_tree_hits={15} svg_tree_misses={16} shell_drag_authority_rebuild={17} shell_drag_node_insert={18} shell_drag_geometry_patch={19} shell_drag_node_patch={20} shell_drag_dispatcher_rebuild={21} shell_drag_route_map_rebuild={22} shell_presentation_build={23} host_scene_build={24} pane_projection_build={25} structure_generation_change={26} layout_measure_probes={27} layout_arrange_probes={28} asset_pointer_snapshot_clones={29} pane_payload_cache_hits={30} pane_payload_cache_misses={31}" -f `
            $buildCount, $reuseCount, $surfaceCount, $duplicateSizeCount, $duplicateScaleCount,
            $modelCount, $chromeCount, $presentationRebuildCount, $imageVertexCount, $imagePrepareCacheHitCount,
            $imagePrepareCommandVisitCount, $imageUploadCount, $imageAllocationCount,
            $visualHitCount, $visualMissCount, $svgHitCount, $svgMissCount,
            $shellDragAuthorityRebuildCount, $shellDragNodeInsertCount,
            $shellDragGeometryPatchCount, $shellDragNodePatchCount,
            $shellDragDispatcherRebuildCount, $shellDragRouteMapRebuildCount,
            $shellPresentationBuildCount, $hostSceneBuildCount, $paneProjectionBuildCount,
            $structureGenerationChangeCount, $layoutMeasureProbeCount, $layoutArrangeProbeCount,
            $assetPointerSnapshotCloneCount, $panePayloadCacheHitCount, $panePayloadCacheMissCount)

    $expectedSteps = [int64]$evidence.interaction.requested_steps
    $completedSteps = [int64]$evidence.interaction.completed_steps
    return $expectedSteps -gt 1 -and
        $completedSteps -eq $expectedSteps -and
        [bool]$evidence.interaction.restored_original_extent -and
        $buildCount -eq 1 -and
        $reuseCount -gt 0 -and
        $surfaceCount -gt 0 -and
        $surfaceCount -le $completedSteps -and
        $modelCount -le 1 -and
        $chromeCount -le 1 -and
        $presentationRebuildCount -eq 0 -and
        $shellPresentationBuildCount -eq 0 -and
        $hostSceneBuildCount -eq 0 -and
        $paneProjectionBuildCount -eq 0 -and
        $structureGenerationChangeCount -eq 0 -and
        $layoutMeasureProbeCount -eq 0 -and
        $layoutArrangeProbeCount -gt 0 -and
        $layoutArrangeProbeCount -le ($completedSteps * 64) -and
        $assetPointerSnapshotCloneCount -eq 0 -and
        $imageVertexCount -gt 0 -and
        $imagePrepareCacheHitCount -gt 0 -and
        $imagePrepareCommandVisitCount -eq 0 -and
        $imageUploadCount -le 1 -and
        $imageAllocationCount -le 1 -and
        $imageAdmissionRejectCount -eq 0 -and
        $imageInvalidPayloadCount -eq 0 -and
        $visualHitCount -gt 0 -and
        $visualMissCount -eq 0 -and
        $svgHitCount -gt 0 -and
        $svgMissCount -eq 0 -and
        $visualFullInvalidationCount -eq 0 -and
        $shellDragAuthorityRebuildCount -eq 0 -and
        $shellDragNodeInsertCount -eq 0 -and
        $shellDragGeometryPatchCount -gt 0 -and
        $shellDragNodePatchCount -ge $shellDragGeometryPatchCount -and
        $shellDragDispatcherRebuildCount -eq 0 -and
        $shellDragRouteMapRebuildCount -eq 0 -and
        $panePayloadCacheMissCount -eq 0
}

function Test-ZirconInteractiveFrameCommitCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ((Resolve-InteractionScenarioName -ScenarioName $ScenarioName) -ne "click") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Interactive frame commit gate requires a timeline artifact."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $deferredCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.interactive_frame.maintenance_deferred_count")
    Write-Host ("- interactive_frame_maintenance_deferred={0}" -f $deferredCount)
    if ($deferredCount -le 0) {
        Write-Warning "Click scenario did not use the lightweight interactive frame commit."
        return $false
    }
    return $true
}

function Test-ZirconStableVisualAssetCacheCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "idle_hover") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Stable hover visual-cache gate requires a timeline artifact."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.visual_asset_cache_hit_count",
            "ui.idle_hover.visual_asset_cache_miss_count",
            "ui.idle_hover.visual_asset_cache_candidate_build_count",
            "ui.idle_hover.visual_asset_async_enqueued_count",
            "ui.idle_hover.visual_asset_async_deduplicated_count",
            "ui.idle_hover.visual_asset_async_completed_count",
            "ui.idle_hover.visual_asset_async_stale_discard_count",
            "ui.idle_hover.visual_asset_async_submission_rejected_count",
            "ui.idle_hover.visual_asset_async_completion_redraw_count",
            "ui.idle_hover.svg_tree_cache_memory_hit_count",
            "ui.idle_hover.svg_tree_cache_miss_count",
            "ui.idle_hover.gpu_image_prepare_cache_hits",
            "ui.idle_hover.gpu_image_prepare_command_visits",
            "ui.idle_hover.gpu_image_upload_writes",
            "ui.idle_hover.gpu_image_shared_upload_writes",
            "ui.idle_hover.gpu_image_cache_key_allocations"
        )) {
        $counterTotals[$name] = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @($name)
    }
    $visualHitCount = $counterTotals["ui.idle_hover.visual_asset_cache_hit_count"]
    $visualMissCount = $counterTotals["ui.idle_hover.visual_asset_cache_miss_count"]
    $candidateBuildCount = $counterTotals["ui.idle_hover.visual_asset_cache_candidate_build_count"]
    $asyncEnqueuedCount = $counterTotals["ui.idle_hover.visual_asset_async_enqueued_count"]
    $asyncDeduplicatedCount = $counterTotals["ui.idle_hover.visual_asset_async_deduplicated_count"]
    $asyncCompletedCount = $counterTotals["ui.idle_hover.visual_asset_async_completed_count"]
    $asyncStaleDiscardCount = $counterTotals["ui.idle_hover.visual_asset_async_stale_discard_count"]
    $asyncRejectedCount = $counterTotals["ui.idle_hover.visual_asset_async_submission_rejected_count"]
    $asyncCompletionRedrawCount = $counterTotals["ui.idle_hover.visual_asset_async_completion_redraw_count"]
    $svgHitCount = $counterTotals["ui.idle_hover.svg_tree_cache_memory_hit_count"]
    $svgMissCount = $counterTotals["ui.idle_hover.svg_tree_cache_miss_count"]
    $imagePrepareCacheHitCount = $counterTotals["ui.idle_hover.gpu_image_prepare_cache_hits"]
    $imagePrepareCommandVisitCount = $counterTotals["ui.idle_hover.gpu_image_prepare_command_visits"]
    $imageUploadCount = $counterTotals["ui.idle_hover.gpu_image_upload_writes"]
    $sharedImageUploadCount = $counterTotals["ui.idle_hover.gpu_image_shared_upload_writes"]
    $imageAllocationCount = $counterTotals["ui.idle_hover.gpu_image_cache_key_allocations"]
    Write-Host ("- hover_visual_hits={0} misses={1} candidate_builds={2} async_enqueued={3} async_deduplicated={4} async_completed={5} async_stale={6} async_rejected={7} async_redraws={8} svg_tree_hits={9} svg_tree_misses={10} image_prepare_cache_hits={11} image_prepare_command_visits={12} image_uploads={13} shared_image_uploads={14} image_allocations={15}" -f `
            $visualHitCount, $visualMissCount, $candidateBuildCount,
            $asyncEnqueuedCount, $asyncDeduplicatedCount, $asyncCompletedCount,
            $asyncStaleDiscardCount, $asyncRejectedCount, $asyncCompletionRedrawCount,
            $svgHitCount, $svgMissCount, $imagePrepareCacheHitCount,
            $imagePrepareCommandVisitCount, $imageUploadCount, $sharedImageUploadCount,
            $imageAllocationCount)

    if ($visualHitCount -le 0 -or
        $svgHitCount -le 0 -or
        $imagePrepareCacheHitCount -le 0 -or
        $imagePrepareCommandVisitCount -ne 0 -or
        $visualMissCount -ne 0 -or
        $candidateBuildCount -ne 0 -or
        $asyncEnqueuedCount -ne 0 -or
        $asyncDeduplicatedCount -ne 0 -or
        $asyncCompletedCount -ne 0 -or
        $asyncStaleDiscardCount -ne 0 -or
        $asyncRejectedCount -ne 0 -or
        $asyncCompletionRedrawCount -ne 0 -or
        $svgMissCount -ne 0 -or
        $imageUploadCount -ne 0 -or
        $sharedImageUploadCount -ne 0 -or
        $imageAllocationCount -ne 0) {
        Write-Warning "Stable hover repeated SVG resolution, rasterization, or GPU image materialization."
        return $false
    }
    return $true
}

function Test-ZirconIdleHoverPaintSubmissionCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "idle_hover") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $outcomePath = Join-Path $ProfileDir "ui_surface_present_outcomes.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $outcomePath)) {
        Write-Warning "Idle-hover paint gate requires timeline and surface outcome artifacts."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $outcome = Get-Content -Path $outcomePath -Raw | ConvertFrom-Json
    $submittedCount = [int64]$outcome.submitted_count
    $presentBatchCount = [int64]$outcome.present_batch_count
    $damagedInputCount = [int64]$outcome.damaged_input_outcome_count
    $presentBatchDamagedCount = [int64]$outcome.present_batch_damaged_count
    if ([int]$outcome.schema_version -lt 5 -or
        -not [bool]$outcome.typed_input_outcome_complete -or
        [int64]$outcome.retryable_no_submit_count -ne 0 -or
        $submittedCount -le 0 -or
        $submittedCount -ne $presentBatchCount -or
        $damagedInputCount -lt $presentBatchCount -or
        $presentBatchDamagedCount -ne $damagedInputCount) {
        Write-Warning "Idle-hover paint gate requires complete damaged-input batches with one successful submit per batch."
        return $false
    }

    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.presentation_rebuild_count",
            "ui.idle_hover.presentation_structure_generation_change_count",
            "ui.idle_hover.presentation_generation_read_count",
            "ui.idle_hover.presentation_snapshot_read_count",
            "ui.idle_hover.chrome_command_full_rebuild_count",
            "ui.idle_hover.chrome_command_patch_count",
            "ui.idle_hover.full_paint_count",
            "ui.idle_hover.region_paint_count",
            "ui.idle_hover.painted_pixels",
            "ui.idle_hover.presented_surface_pixels",
            "ui.idle_hover.workbench_paint_index_query_count",
            "ui.idle_hover.workbench_paint_index_candidate_count",
            "ui.idle_hover.template_node_visit_count",
            "ui.idle_hover.template_node_clone_count",
            "ui.idle_hover.template_node_damage_reject_count",
            "ui.idle_hover.fallback_sort_count",
            "ui.paint_index.query_scratch_growth_count",
            "ui.idle_hover.software_fallback_present_count",
            "ui.idle_hover.gpu_batch_plan_builds",
            "ui.idle_hover.gpu_batch_plan_cache_hits",
            "ui.idle_hover.gpu_command_visibility_scans",
            "ui.idle_hover.gpu_vertex_buffer_creates",
            "ui.idle_hover.gpu_text_shapes",
            "ui.idle_hover.gpu_text_renderer_builds",
            "ui.idle_hover.gpu_text_renderer_cache_hits",
            "ui.idle_hover.gpu_text_prepare_failures"
        )) {
        $counterTotals[$name] = [int64](Get-UiCounterTotal `
                -Counters @($snapshot.counters) `
                -Names @($name))
    }

    $presentationRebuildCount = $counterTotals["ui.idle_hover.presentation_rebuild_count"]
    $structureGenerationChangeCount = $counterTotals["ui.idle_hover.presentation_structure_generation_change_count"]
    $presentationGenerationReadCount = $counterTotals["ui.idle_hover.presentation_generation_read_count"]
    $presentationSnapshotReadCount = $counterTotals["ui.idle_hover.presentation_snapshot_read_count"]
    $fullCommandCount = $counterTotals["ui.idle_hover.chrome_command_full_rebuild_count"]
    $patchCommandCount = $counterTotals["ui.idle_hover.chrome_command_patch_count"]
    $fullPaintCount = $counterTotals["ui.idle_hover.full_paint_count"]
    $regionPaintCount = $counterTotals["ui.idle_hover.region_paint_count"]
    $paintedPixels = $counterTotals["ui.idle_hover.painted_pixels"]
    $presentedPixels = $counterTotals["ui.idle_hover.presented_surface_pixels"]
    $paintQueryCount = $counterTotals["ui.idle_hover.workbench_paint_index_query_count"]
    $paintCandidateCount = $counterTotals["ui.idle_hover.workbench_paint_index_candidate_count"]
    $templateVisitCount = $counterTotals["ui.idle_hover.template_node_visit_count"]
    $templateCloneCount = $counterTotals["ui.idle_hover.template_node_clone_count"]
    $templateDamageRejectCount = $counterTotals["ui.idle_hover.template_node_damage_reject_count"]
    $fallbackSortCount = $counterTotals["ui.idle_hover.fallback_sort_count"]
    $paintScratchGrowthCount = $counterTotals["ui.paint_index.query_scratch_growth_count"]
    $softwareFallbackCount = $counterTotals["ui.idle_hover.software_fallback_present_count"]
    $batchPlanBuildCount = $counterTotals["ui.idle_hover.gpu_batch_plan_builds"]
    $batchPlanCacheHitCount = $counterTotals["ui.idle_hover.gpu_batch_plan_cache_hits"]
    $visibilityScanCount = $counterTotals["ui.idle_hover.gpu_command_visibility_scans"]
    $vertexBufferCreateCount = $counterTotals["ui.idle_hover.gpu_vertex_buffer_creates"]
    $textShapeCount = $counterTotals["ui.idle_hover.gpu_text_shapes"]
    $textRendererBuildCount = $counterTotals["ui.idle_hover.gpu_text_renderer_builds"]
    $textRendererCacheHitCount = $counterTotals["ui.idle_hover.gpu_text_renderer_cache_hits"]
    $textPrepareFailureCount = $counterTotals["ui.idle_hover.gpu_text_prepare_failures"]
    $maximumCandidatesPerQuery = 256
    $maximumTemplateVisitsPerSubmit = 256
    $maximumVisibilityScansPerSubmit = 256
    $maximumVertexBufferCreates = 2

    Write-Host ("- idle_hover_paint submits={0} batches={1} region={2} full={3} command_patch={4} command_full={5} generation_reads={6} snapshot_reads={7}" -f `
            $submittedCount, $presentBatchCount, $regionPaintCount, $fullPaintCount,
            $patchCommandCount, $fullCommandCount, $presentationGenerationReadCount,
            $presentationSnapshotReadCount)
    Write-Host ("- idle_hover_damage painted_pixels={0} presented_pixels={1} paint_queries={2} candidates={3} visits={4} clones={5} damage_rejects={6} fallback_sorts={7} scratch_growth={8}" -f `
            $paintedPixels, $presentedPixels, $paintQueryCount, $paintCandidateCount,
            $templateVisitCount, $templateCloneCount, $templateDamageRejectCount,
            $fallbackSortCount, $paintScratchGrowthCount)
    Write-Host ("- idle_hover_gpu batch_builds={0} batch_cache_hits={1} visibility_scans={2} vertex_buffer_creates={3} text_shapes={4} text_builds={5} text_cache_hits={6} text_failures={7}" -f `
            $batchPlanBuildCount, $batchPlanCacheHitCount, $visibilityScanCount,
            $vertexBufferCreateCount, $textShapeCount, $textRendererBuildCount,
            $textRendererCacheHitCount, $textPrepareFailureCount)

    $hasBoundedPresentationWork =
        $presentationRebuildCount -eq 0 -and
        $structureGenerationChangeCount -eq 0 -and
        $presentationGenerationReadCount -eq $submittedCount -and
        $presentationSnapshotReadCount -eq 0
    $hasBoundedCommandAndPaintWork =
        $fullCommandCount -eq 0 -and
        $patchCommandCount -ge $submittedCount -and
        $patchCommandCount -le ($submittedCount * 2) -and
        $fullPaintCount -eq 0 -and
        $regionPaintCount -eq $submittedCount -and
        $paintedPixels -gt 0 -and
        $presentedPixels -gt 0 -and
        ($paintedPixels * 2) -le $presentedPixels -and
        $fallbackSortCount -eq 0
    $hasBoundedNodeWork =
        $paintQueryCount -gt 0 -and
        $paintCandidateCount -gt 0 -and
        $paintCandidateCount -le ($paintQueryCount * $maximumCandidatesPerQuery) -and
        $templateVisitCount -gt 0 -and
        $templateVisitCount -le ($submittedCount * $maximumTemplateVisitsPerSubmit) -and
        $templateCloneCount -le $templateVisitCount -and
        $templateDamageRejectCount -le $templateVisitCount -and
        $paintScratchGrowthCount -eq 0
    $hasBoundedGpuWork =
        $softwareFallbackCount -eq 0 -and
        $batchPlanBuildCount -le $submittedCount -and
        ($batchPlanBuildCount + $batchPlanCacheHitCount) -eq $submittedCount -and
        $visibilityScanCount -le ($submittedCount * $maximumVisibilityScansPerSubmit) -and
        $vertexBufferCreateCount -le $maximumVertexBufferCreates -and
        $textShapeCount -eq 0 -and
        $textRendererBuildCount -eq 0 -and
        $textRendererCacheHitCount -gt 0 -and
        $textPrepareFailureCount -eq 0
    if (-not ($hasBoundedPresentationWork -and
            $hasBoundedCommandAndPaintWork -and
            $hasBoundedNodeWork -and
            $hasBoundedGpuWork)) {
        Write-Warning "Idle hover exceeded the retained local-paint and GPU submission budgets."
        return $false
    }
    return $true
}

function Test-ZirconSurfaceFramePublicationCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "idle_hover") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Stable hover surface-frame publication gate requires a timeline artifact."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $counterTotals = @{}
    foreach ($name in @(
            "ui.surface_frame.publication_build_count",
            "ui.surface_frame.arranged_node_clone_count",
            "ui.surface_frame.render_command_clone_count",
            "ui.surface_frame.render_segment_clone_count",
            "ui.surface_frame.render_directory_node_clone_count",
            "ui.surface_frame.render_full_snapshot_build_count",
            "ui.surface_frame.hit_entry_clone_count",
            "ui.surface_frame.hit_cell_entry_clone_count",
            "ui.surface_frame.focus_state_build_count",
            "ui.surface_frame.focus_path_build_count",
            "ui.surface_frame.focus_path_validation_node_count_upper_bound",
            "ui.surface_frame.pipeline_stage_build_count"
        )) {
        $counterTotals[$name] = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @($name)
    }
    $publicationBuildCount = $counterTotals["ui.surface_frame.publication_build_count"]
    $arrangedNodeCloneCount = $counterTotals["ui.surface_frame.arranged_node_clone_count"]
    $renderCommandCloneCount = $counterTotals["ui.surface_frame.render_command_clone_count"]
    $renderSegmentCloneCount = $counterTotals["ui.surface_frame.render_segment_clone_count"]
    $renderDirectoryNodeCloneCount = $counterTotals["ui.surface_frame.render_directory_node_clone_count"]
    $renderFullSnapshotBuildCount = $counterTotals["ui.surface_frame.render_full_snapshot_build_count"]
    $hitEntryCloneCount = $counterTotals["ui.surface_frame.hit_entry_clone_count"]
    $hitCellEntryCloneCount = $counterTotals["ui.surface_frame.hit_cell_entry_clone_count"]
    $focusStateBuildCount = $counterTotals["ui.surface_frame.focus_state_build_count"]
    $focusPathBuildCount = $counterTotals["ui.surface_frame.focus_path_build_count"]
    $focusPathValidationNodeCountUpperBound = $counterTotals["ui.surface_frame.focus_path_validation_node_count_upper_bound"]
    $pipelineStageBuildCount = $counterTotals["ui.surface_frame.pipeline_stage_build_count"]
    Write-Host ("- surface_frame_publication builds={0} arranged_nodes={1} render_commands={2} render_segments={3} render_directory_nodes={4} render_full_snapshots={5} hit_entries={6} hit_cell_entries={7} focus_state_builds={8} focus_path_builds={9} focus_path_validation_nodes_upper_bound={10} pipeline_stages={11}" -f `
            $publicationBuildCount, $arrangedNodeCloneCount, $renderCommandCloneCount,
            $renderSegmentCloneCount, $renderDirectoryNodeCloneCount, $renderFullSnapshotBuildCount,
            $hitEntryCloneCount, $hitCellEntryCloneCount, $focusStateBuildCount, $focusPathBuildCount,
            $focusPathValidationNodeCountUpperBound, $pipelineStageBuildCount)

    if ($publicationBuildCount -ne 0 -or
        $arrangedNodeCloneCount -ne 0 -or
        $renderCommandCloneCount -ne 0 -or
        $renderSegmentCloneCount -ne 0 -or
        $renderDirectoryNodeCloneCount -ne 0 -or
        $renderFullSnapshotBuildCount -ne 0 -or
        $hitEntryCloneCount -ne 0 -or
        $hitCellEntryCloneCount -ne 0 -or
        $focusStateBuildCount -ne 0 -or
        $focusPathBuildCount -ne 0 -or
        $focusPathValidationNodeCountUpperBound -ne 0 -or
        $pipelineStageBuildCount -ne 0) {
        Write-Warning "Stable hover republished a UiSurfaceFrame or rebuilt one of its O(N) products."
        return $false
    }
    return $true
}

function Test-ZirconViewportToolbarCacheCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "viewport_toolbar_click") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Viewport toolbar cache gate requires a timeline artifact."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $prelayoutHitCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.prelayout_surface_frame_cache_hit_count")
    $prelayoutReprojectCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.prelayout_surface_frame_cache_reproject_count")
    $prelayoutSharedLayoutCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.prelayout_surface_frame_cache_shared_layout_count")
    $prelayoutRouteKeyUpdateCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.prelayout_surface_frame_cache_route_key_update_count")
    $prelayoutMissCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.prelayout_surface_frame_cache_miss_count")
    $surfaceFrameHitCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.surface_frame_cache_hit_count")
    $surfaceFrameMissCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.surface_frame_cache_miss_count")
    $hitControlProjectionBatchCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.hit_control_projection_batch_count")
    $hitControlProjectionVisitCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.viewport_toolbar.hit_control_projection_visit_count")
    $prelayoutDecisionCount = $prelayoutHitCount +
        $prelayoutReprojectCount +
        $prelayoutSharedLayoutCount
    $expectedProjectionBatchCount = $prelayoutReprojectCount +
        $prelayoutSharedLayoutCount +
        $prelayoutRouteKeyUpdateCount +
        $surfaceFrameHitCount +
        $surfaceFrameMissCount

    Write-Host ("- viewport_toolbar_cache hit={0} reproject={1} shared_layout={2} route_key_update={3} prelayout_miss={4} resolved_hit={5} resolved_miss={6} projection_batches={7}/{8} projection_visits={9}" -f `
            $prelayoutHitCount, $prelayoutReprojectCount, $prelayoutSharedLayoutCount,
            $prelayoutRouteKeyUpdateCount, $prelayoutMissCount, $surfaceFrameHitCount,
            $surfaceFrameMissCount, $hitControlProjectionBatchCount,
            $expectedProjectionBatchCount, $hitControlProjectionVisitCount)
    if ($prelayoutDecisionCount -le 0 -or
        $prelayoutMissCount -ne 0 -or
        $surfaceFrameMissCount -ne 0 -or
        $hitControlProjectionBatchCount -ne $expectedProjectionBatchCount -or
        $hitControlProjectionVisitCount -lt $hitControlProjectionBatchCount) {
        Write-Warning "Viewport toolbar interaction did not remain on the retained surface-frame cache path."
        return $false
    }
    return $true
}

function Test-ZirconHierarchyScrollCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "hierarchy_scroll") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Hierarchy scroll gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_wheel_events) `
            -MaxCpuMsPerOperation 0.25)) {
        Write-Warning "Hierarchy scroll gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.hierarchy_scroll_dispatch_count",
            "ui.idle_hover.hierarchy_surface_rebuild_count",
            "ui.idle_hover.hierarchy_row_insert_count",
            "ui.idle_hover.hierarchy_dispatcher_rebuild_count",
            "ui.idle_hover.hierarchy_route_map_rebuild_count"
        )) {
        $counterTotals[$name] = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @($name)
    }

    $dispatchCount = $counterTotals["ui.idle_hover.hierarchy_scroll_dispatch_count"]
    $surfaceRebuildCount = $counterTotals["ui.idle_hover.hierarchy_surface_rebuild_count"]
    $rowInsertCount = $counterTotals["ui.idle_hover.hierarchy_row_insert_count"]
    $dispatcherRebuildCount = $counterTotals["ui.idle_hover.hierarchy_dispatcher_rebuild_count"]
    $routeMapRebuildCount = $counterTotals["ui.idle_hover.hierarchy_route_map_rebuild_count"]
    Write-Host ("- hierarchy_scroll_dispatch={0} surface_rebuild={1} row_insert={2} dispatcher_rebuild={3} route_map_rebuild={4}" -f `
            $dispatchCount, $surfaceRebuildCount, $rowInsertCount,
            $dispatcherRebuildCount, $routeMapRebuildCount)

    $requestedWheelEvents = [int64]$evidence.interaction.requested_wheel_events
    $completedWheelEvents = [int64]$evidence.interaction.completed_wheel_events
    $hasNoRetainedAuthorityRebuildWork =
        $surfaceRebuildCount -eq 0 -and
        $rowInsertCount -eq 0 -and
        $dispatcherRebuildCount -eq 0 -and
        $routeMapRebuildCount -eq 0
    if ($dispatchCount -ne $completedWheelEvents -or
        $requestedWheelEvents -le 0 -or
        $completedWheelEvents -ne $requestedWheelEvents -or
        -not $hasNoRetainedAuthorityRebuildWork) {
        Write-Warning "Hierarchy scroll profiling counters are missing, inconsistent, or rebuilt retained authority."
        return $false
    }
    return $true
}

function Test-ZirconWelcomeRecentScrollCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "welcome_recent_scroll") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Welcome recent scroll gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_wheel_events) `
            -MaxCpuMsPerOperation 0.25)) {
        Write-Warning "Welcome recent scroll gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.welcome_recent_scroll_dispatch_count",
            "ui.idle_hover.welcome_recent_surface_rebuild_count",
            "ui.idle_hover.welcome_recent_authority_rebuild_count",
            "ui.idle_hover.welcome_recent_row_insert_count",
            "ui.idle_hover.welcome_recent_geometry_patch_count",
            "ui.idle_hover.welcome_recent_dispatcher_rebuild_count",
            "ui.idle_hover.welcome_recent_route_map_rebuild_count"
        )) {
        $counterTotals[$name] = Get-UiCounterTotal -Counters @($snapshot.counters) -Names @($name)
    }

    $dispatchCount = $counterTotals["ui.idle_hover.welcome_recent_scroll_dispatch_count"]
    $surfaceRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_surface_rebuild_count"]
    $authorityRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_authority_rebuild_count"]
    $rowInsertCount = $counterTotals["ui.idle_hover.welcome_recent_row_insert_count"]
    $geometryPatchCount = $counterTotals["ui.idle_hover.welcome_recent_geometry_patch_count"]
    $dispatcherRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_dispatcher_rebuild_count"]
    $routeMapRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_route_map_rebuild_count"]
    Write-Host ("- welcome_recent_scroll_dispatch={0} surface_rebuild={1} authority_rebuild={2} row_insert={3} geometry_patch={4} dispatcher_rebuild={5} route_map_rebuild={6}" -f `
            $dispatchCount, $surfaceRebuildCount, $authorityRebuildCount,
            $rowInsertCount, $geometryPatchCount, $dispatcherRebuildCount,
            $routeMapRebuildCount)

    $requestedWheelEvents = [int64]$evidence.interaction.requested_wheel_events
    $completedWheelEvents = [int64]$evidence.interaction.completed_wheel_events
    $hasNoRetainedAuthorityWork =
        $surfaceRebuildCount -eq 0 -and
        $authorityRebuildCount -eq 0 -and
        $rowInsertCount -eq 0 -and
        $geometryPatchCount -eq 0 -and
        $dispatcherRebuildCount -eq 0 -and
        $routeMapRebuildCount -eq 0
    if ($dispatchCount -ne $completedWheelEvents -or
        $requestedWheelEvents -le 0 -or
        $completedWheelEvents -ne $requestedWheelEvents -or
        -not $hasNoRetainedAuthorityWork) {
        Write-Warning "Welcome recent scroll profiling counters are missing, inconsistent, or rebuilt retained authority."
        return $false
    }
    return $true
}

function Test-ZirconRuntimeDiagnosticsRefreshCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "runtime_diagnostics") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Runtime Diagnostics gate requires timeline and interaction evidence artifacts."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    $targets = @($evidence.interaction.targets)
    $hasExpectedTarget = $targets.Count -eq 1 -and
        $targets[0].target_id -eq "editor.runtime_diagnostics#1" -and
        $targets[0].target_kind -eq "drawer_tab" -and
        $targets[0].target_surface -eq "bottom" -and
        $targets[0].source -eq "ui_profile_geometry.json"
    if ($evidence.interaction.scenario -ne "runtime_diagnostics_tab_click" -or
        -not ([bool]$evidence.interaction.used_geometry) -or
        -not ([bool]$evidence.geometry_refreshed_after_interaction) -or
        -not $hasExpectedTarget) {
        Write-Warning "Runtime Diagnostics interaction was not bound to the freshly published bottom drawer geometry."
        return $false
    }

    $refreshCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.runtime_diagnostics.shell_content_refresh_count")
    $fullPresentationFallbackCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.runtime_diagnostics.full_presentation_fallback_count")
    $fullHostTargetCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @(
            "ui.click.host_invalidation_full_target_count",
            "ui.viewport_image.host_invalidation_full_target_count",
            "ui.shell_content.host_invalidation_full_target_count"
        )
    Write-Host ("- runtime_diagnostics_shell_refresh={0} full_presentation_fallback={1} full_host_target={2}" -f `
            $refreshCount, $fullPresentationFallbackCount, $fullHostTargetCount)
    if ($refreshCount -le 0 -or
        $fullPresentationFallbackCount -ne 0 -or
        $fullHostTargetCount -ne 0) {
        Write-Warning "Runtime Diagnostics refresh did not stay on the incremental shell-content path."
        return $false
    }
    return $true
}

function Get-ZirconUiCounterMaximum {
    param(
        [object[]]$Counters,
        [string]$Name
    )

    $values = @($Counters |
            Where-Object { $_.name -eq $Name } |
            ForEach-Object { [double]$_.value })
    if ($values.Count -eq 0) {
        return 0.0
    }
    return [double](($values | Measure-Object -Maximum).Maximum)
}

function Test-ZirconAssetBrowserScrollCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "asset_browser_scroll") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    $manifestPath = Join-Path $ProfileDir "source_manifest.json"
    if (-not (Test-Path $timelinePath) -or
        -not (Test-Path $evidencePath) -or
        -not (Test-Path $manifestPath)) {
        Write-Warning "Asset Browser scroll gate requires timeline, interaction, and source-manifest artifacts."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    $manifest = Get-Content -Path $manifestPath -Raw | ConvertFrom-Json
    $completedWheelEvents = [int64]$evidence.interaction.completed_wheel_events
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount $completedWheelEvents `
            -MaxCpuMsPerOperation 0.25)) {
        Write-Warning "Asset Browser scroll gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    $expectedItemCount = [int64]$manifest.capture.options.asset_catalog_item_count
    $fixtureKind = [string]$manifest.input_fixture.kind
    $fixtureItemCount = [int64]$manifest.input_fixture.asset_item_count
    $counters = @($snapshot.counters)
    $dispatchCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_browser_scroll_dispatch_count")
    $logicalItemCount = Get-ZirconUiCounterMaximum -Counters $counters `
        -Name "ui.idle_hover.asset_browser_logical_item_count"
    $materializedItemCount = Get-ZirconUiCounterMaximum -Counters $counters `
        -Name "ui.idle_hover.asset_browser_materialized_item_count"
    $materializedNodeCount = Get-ZirconUiCounterMaximum -Counters $counters `
        -Name "ui.idle_hover.asset_browser_materialized_node_count"
    $visibleItemCount = Get-ZirconUiCounterMaximum -Counters $counters `
        -Name "ui.idle_hover.asset_browser_visible_item_count"
    $visibleNodeCount = Get-ZirconUiCounterMaximum -Counters $counters `
        -Name "ui.idle_hover.asset_browser_visible_node_count"
    $projectionBuildCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_browser_projection_build_count")
    $paintChunkBuildCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_browser_logical_paint_chunk_build_count")
    $paintChunkReuseCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_browser_logical_paint_chunk_reuse_count")
    $paintItemProjectionCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_browser_logical_paint_item_projection_count")
    $generationIdentityParseCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_content_generation_identity_parse_count")
    $descriptorLookupCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.asset_content_descriptor_lookup_count")
    $templateNodeVisitCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.template_node_visit_count")
    $asyncStaleDiscardCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.visual_asset_async_stale_discard_count")
    $asyncRejectedCount = Get-UiCounterTotal -Counters $counters `
        -Names @("ui.idle_hover.visual_asset_async_submission_rejected_count")
    Write-Host ("- asset_browser_scroll dispatch={0} logical={1} materialized_items={2} materialized_nodes={3} visible_items={4} visible_nodes={5} projection_build={6} paint_chunk_build={7} paint_chunk_reuse={8} paint_item_projection={9} generation_identity_parse={10} descriptor_lookup={11} template_node_visits={12} async_stale={13} async_rejected={14}" -f `
            $dispatchCount, $logicalItemCount, $materializedItemCount,
            $materializedNodeCount, $visibleItemCount, $visibleNodeCount,
            $projectionBuildCount, $paintChunkBuildCount, $paintChunkReuseCount,
            $paintItemProjectionCount, $generationIdentityParseCount, $descriptorLookupCount,
            $templateNodeVisitCount, $asyncStaleDiscardCount, $asyncRejectedCount)

    $requestedWheelEvents = [int64]$evidence.interaction.requested_wheel_events
    if ($expectedItemCount -le 0 -or
        $fixtureKind -ne "asset_catalog_json" -or
        $fixtureItemCount -ne $expectedItemCount -or
        $requestedWheelEvents -le 0 -or
        $completedWheelEvents -ne $requestedWheelEvents -or
        $dispatchCount -ne $completedWheelEvents -or
        $logicalItemCount -ne $expectedItemCount -or
        $materializedItemCount -le 0 -or
        $materializedItemCount -gt $logicalItemCount -or
        ($logicalItemCount -gt $visibleItemCount -and
            $materializedItemCount -ge $logicalItemCount) -or
        $materializedNodeCount -lt $materializedItemCount -or
        $visibleItemCount -le 0 -or
        $visibleItemCount -gt $materializedItemCount -or
        $visibleNodeCount -lt $visibleItemCount -or
        $projectionBuildCount -ne 0 -or
        $paintChunkBuildCount -ne 0 -or
        $paintChunkReuseCount -ne 0 -or
        $paintItemProjectionCount -ne 0 -or
        $generationIdentityParseCount -ne 0 -or
        $descriptorLookupCount -le 0 -or
        $descriptorLookupCount -gt $templateNodeVisitCount -or
        $asyncStaleDiscardCount -ne 0 -or
        $asyncRejectedCount -ne 0) {
        Write-Warning "Asset Browser scroll counters are missing, source-scale inconsistent, or rebuilt projection state."
        return $false
    }
    return $true
}

function Get-ZirconDamageRegionCounterSamples {
    param(
        [object[]]$Counters,
        [string]$Name
    )

    return @($Counters |
            Where-Object { $_.name -eq $Name } |
            ForEach-Object { [double]$_.value })
}

function Export-ZirconDamageRegionEvidence {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName,
        [string]$CounterScenarioName
    )

    if ([string]::IsNullOrWhiteSpace($ScenarioName) -or
        $CounterScenarioName -notmatch '^[a-z0-9_]+$') {
        throw 'Damage-region evidence requires explicit capture and counter scenario names.'
    }

    $timelinePath = Join-Path $ProfileDir 'timeline.zrtrace.json'
    if (-not (Test-Path -LiteralPath $timelinePath)) {
        throw "Damage-region evidence could not find timeline.zrtrace.json in '$ProfileDir'."
    }
    $snapshot = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
    $counters = @($snapshot.counters)
    $metricSuffixes = [ordered]@{
        rect_count = 'rect_count'
        source_rect_count = 'source_rect_count'
        simplification_count = 'simplification_count'
        represented_area = 'represented_area'
        bounding_area = 'bounding_area'
        bounding_overdraw_area = 'bounding_overdraw_area'
    }
    $samples = @{}
    foreach ($metric in $metricSuffixes.Keys) {
        $counterName = "ui.$CounterScenarioName.redraw_damage_$($metricSuffixes[$metric])"
        $samples[$metric] = @(Get-ZirconDamageRegionCounterSamples `
                -Counters $counters `
                -Name $counterName)
    }

    $sampleCounts = @($metricSuffixes.Keys | ForEach-Object { $samples[$_].Count })
    $nonEmptyMetricCount = @($sampleCounts | Where-Object { $_ -gt 0 }).Count
    if ($nonEmptyMetricCount -ne 0 -and $nonEmptyMetricCount -ne $metricSuffixes.Count) {
        throw 'Damage-region evidence contains only part of the six-counter schema.'
    }
    if (@($sampleCounts | Select-Object -Unique).Count -ne 1) {
        throw 'Damage-region evidence counter series have inconsistent sample counts.'
    }

    $sampleCount = [int]$sampleCounts[0]
    $totalRectCount = 0.0
    $totalSourceRectCount = 0.0
    $totalSimplificationCount = 0.0
    $totalRepresentedArea = 0.0
    $totalBoundingArea = 0.0
    $totalBoundingOverdrawArea = 0.0
    for ($index = 0; $index -lt $sampleCount; $index++) {
        $rectCount = [double]$samples.rect_count[$index]
        $sourceRectCount = [double]$samples.source_rect_count[$index]
        $simplificationCount = [double]$samples.simplification_count[$index]
        $representedArea = [double]$samples.represented_area[$index]
        $boundingArea = [double]$samples.bounding_area[$index]
        $boundingOverdrawArea = [double]$samples.bounding_overdraw_area[$index]

        $numericValues = @(
            $rectCount,
            $sourceRectCount,
            $simplificationCount,
            $representedArea,
            $boundingArea,
            $boundingOverdrawArea
        )
        if (@($numericValues | Where-Object {
                    [double]::IsNaN($_) -or [double]::IsInfinity($_)
                }).Count -gt 0) {
            throw "Damage-region evidence sample $index contains a non-finite value."
        }
        if ($rectCount -lt 1 -or $rectCount -gt 3 -or
            [Math]::Abs($rectCount - [Math]::Round($rectCount)) -gt 0.000000001 -or
            $sourceRectCount -lt $rectCount -or
            [Math]::Abs($sourceRectCount - [Math]::Round($sourceRectCount)) -gt 0.000000001 -or
            $simplificationCount -lt 0 -or
            $simplificationCount -gt $sourceRectCount -or
            [Math]::Abs($simplificationCount - [Math]::Round($simplificationCount)) -gt 0.000000001) {
            throw "Damage-region evidence sample $index contains invalid count values."
        }
        if ($representedArea -le 0 -or
            $boundingArea -lt $representedArea -or
            $boundingOverdrawArea -lt 0) {
            throw "Damage-region evidence sample $index contains invalid logical areas."
        }
        $expectedOverdrawArea = [Math]::Max(0.0, $boundingArea - $representedArea)
        $areaTolerance = [Math]::Max(0.001, [Math]::Abs($boundingArea) * 0.000001)
        if ([Math]::Abs($expectedOverdrawArea - $boundingOverdrawArea) -gt $areaTolerance) {
            throw "Damage-region evidence sample $index contradicts its bounding overdraw area."
        }

        $totalRectCount += $rectCount
        $totalSourceRectCount += $sourceRectCount
        $totalSimplificationCount += $simplificationCount
        $totalRepresentedArea += $representedArea
        $totalBoundingArea += $boundingArea
        $totalBoundingOverdrawArea += $boundingOverdrawArea
    }

    $boundingOverdrawRatio = if ($totalBoundingArea -gt 0) {
        $totalBoundingOverdrawArea / $totalBoundingArea
    }
    else {
        0.0
    }
    $simplificationRatio = if ($totalSourceRectCount -gt 0) {
        $totalSimplificationCount / $totalSourceRectCount
    }
    else {
        0.0
    }
    $minimumSampleCount = 100
    $minimumBoundingOverdrawRatio = 0.20
    $maximumSimplificationRatio = 0.05
    $eligibleForMultiRegionTrial = $sampleCount -ge $minimumSampleCount -and
        $boundingOverdrawRatio -ge $minimumBoundingOverdrawRatio -and
        $simplificationRatio -le $maximumSimplificationRatio
    $result = [ordered]@{
        schema_version = 1
        capture_scenario = $ScenarioName
        counter_scenario = $CounterScenarioName
        has_region_samples = $sampleCount -gt 0
        sample_count = $sampleCount
        total_rect_count = [int64][Math]::Round($totalRectCount)
        total_source_rect_count = [int64][Math]::Round($totalSourceRectCount)
        total_simplification_count = [int64][Math]::Round($totalSimplificationCount)
        total_represented_area = $totalRepresentedArea
        total_bounding_area = $totalBoundingArea
        total_bounding_overdraw_area = $totalBoundingOverdrawArea
        bounding_overdraw_ratio = $boundingOverdrawRatio
        simplification_ratio = $simplificationRatio
        trial_thresholds = [ordered]@{
            minimum_sample_count = $minimumSampleCount
            minimum_bounding_overdraw_ratio = $minimumBoundingOverdrawRatio
            maximum_simplification_ratio = $maximumSimplificationRatio
        }
        eligible_for_multi_region_trial = $eligibleForMultiRegionTrial
        performance_accepted = $false
    }
    $outputPath = Join-Path $ProfileDir 'ui_damage_region_evidence.json'
    $result | ConvertTo-Json -Depth 5 |
        Set-Content -LiteralPath $outputPath -Encoding UTF8
    return [pscustomobject]$result
}

function Get-ZirconDamageRegionSha256 {
    param([string]$Value)

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-ZirconDamageRegionFileSetSignature {
    param(
        [object[]]$Files,
        [string]$Description
    )

    if ($Files.Count -eq 0) {
        throw "Damage-region trial evidence requires a non-empty $Description fingerprint set."
    }
    $entries = @($Files |
            Sort-Object -Property relative_path |
            ForEach-Object {
                $relativePath = [string]$_.relative_path
                $sha256 = ([string]$_.sha256).ToLowerInvariant()
                $byteLength = [int64]$_.byte_length
                if ([string]::IsNullOrWhiteSpace($relativePath) -or
                    $sha256 -notmatch '^[0-9a-f]{64}$' -or
                    $byteLength -lt 0) {
                    throw "Damage-region trial evidence contains an invalid $Description fingerprint."
                }
                "{0}|{1}|{2}" -f $relativePath.Replace('\\', '/'), $sha256, $byteLength
            })
    if (@($entries | Select-Object -Unique).Count -ne $entries.Count) {
        throw "Damage-region trial evidence contains duplicate $Description fingerprints."
    }
    return $entries -join "`n"
}

function Get-ZirconDamageRegionSourceBindingId {
    param([object]$Manifest)

    $git = $Manifest.repository.git
    $editor = $Manifest.binaries.editor
    $runtime = $Manifest.binaries.runtime
    if ([int]$Manifest.schema_version -ne 2 -or
        [string]::IsNullOrWhiteSpace([string]$git.revision) -or
        [string]$git.dirty_tree_sha256 -notmatch '^[0-9a-fA-F]{64}$' -or
        [string]$editor.sha256 -notmatch '^[0-9a-fA-F]{64}$' -or
        [string]$runtime.sha256 -notmatch '^[0-9a-fA-F]{64}$' -or
        [int64]$editor.byte_length -lt 0 -or
        [int64]$runtime.byte_length -lt 0) {
        throw 'Damage-region trial evidence contains an invalid source manifest binding.'
    }
    $criticalSourceSignature = Get-ZirconDamageRegionFileSetSignature `
        -Files @($Manifest.repository.critical_source_files) `
        -Description 'critical-source'
    $toolSignature = Get-ZirconDamageRegionFileSetSignature `
        -Files @($Manifest.capture.tool_files) `
        -Description 'capture-tool'
    $binding = @(
        ([string]$git.revision).ToLowerInvariant(),
        ([string][bool]$git.dirty).ToLowerInvariant(),
        [string][int64]$git.dirty_entry_count,
        ([string]$git.dirty_tree_sha256).ToLowerInvariant(),
        ([string]$editor.sha256).ToLowerInvariant(),
        [string][int64]$editor.byte_length,
        ([string]$runtime.sha256).ToLowerInvariant(),
        [string][int64]$runtime.byte_length,
        $criticalSourceSignature,
        $toolSignature
    ) -join "`n"
    return Get-ZirconDamageRegionSha256 -Value $binding
}

function Get-ZirconDamageRegionCaptureContractId {
    param([object]$Manifest)

    $options = $Manifest.capture.options
    if ($null -eq $options) {
        throw 'Damage-region trial evidence requires capture options in every source manifest.'
    }
    $optionSignature = @($options.PSObject.Properties |
            Where-Object { $_.Name -ne 'run_ordinal' } |
            Sort-Object -Property Name |
            ForEach-Object {
                "{0}={1}" -f $_.Name, ($_.Value | ConvertTo-Json -Compress -Depth 8)
            }) -join "`n"
    return Get-ZirconDamageRegionSha256 `
        -Value (([string]$Manifest.scenario) + "`n" + $optionSignature)
}

function Export-ZirconDamageRegionTrialEvidence {
    param(
        [string[]]$ProfileDirs,
        [string]$OutputDir,
        [string]$ScenarioName
    )

    if ($ProfileDirs.Count -eq 0 -or
        [string]::IsNullOrWhiteSpace($OutputDir) -or
        [string]::IsNullOrWhiteSpace($ScenarioName)) {
        throw 'Damage-region trial evidence requires profile directories, an output directory, and a scenario.'
    }
    $resolvedProfileDirs = @($ProfileDirs | ForEach-Object {
            [System.IO.Path]::GetFullPath($_).TrimEnd([char[]]@('\', '/'))
        })
    if (@($resolvedProfileDirs | Select-Object -Unique).Count -ne $resolvedProfileDirs.Count) {
        throw 'Damage-region trial evidence requires unique profile directories.'
    }

    $runs = @()
    $sourceBindingIds = @()
    $captureContractIds = @()
    $counterScenarios = @()
    foreach ($profileDir in $resolvedProfileDirs) {
        $evidencePath = Join-Path $profileDir 'ui_damage_region_evidence.json'
        $manifestPath = Join-Path $profileDir 'source_manifest.json'
        if (-not (Test-Path -LiteralPath $evidencePath -PathType Leaf) -or
            -not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
            throw "Damage-region trial evidence requires per-run evidence and source manifest artifacts in '$profileDir'."
        }
        $evidence = Get-Content -LiteralPath $evidencePath -Raw | ConvertFrom-Json
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $options = $manifest.capture.options
        if ([int]$evidence.schema_version -ne 1 -or
            [string]$evidence.capture_scenario -ne $ScenarioName -or
            [string]$manifest.scenario -ne $ScenarioName -or
            [string]$options.run_phase -ne 'measured' -or
            [int]$options.run_ordinal -le 0 -or
            [int]$options.measured_run_count -le 0 -or
            [bool]$evidence.performance_accepted) {
            throw "Damage-region trial evidence found an invalid measured run in '$profileDir'."
        }
        $minimumSampleCount = [int]$evidence.trial_thresholds.minimum_sample_count
        $minimumOverdrawRatio = [double]$evidence.trial_thresholds.minimum_bounding_overdraw_ratio
        $maximumSimplificationRatio = [double]$evidence.trial_thresholds.maximum_simplification_ratio
        $computedEligibility = [int]$evidence.sample_count -ge $minimumSampleCount -and
            [double]$evidence.bounding_overdraw_ratio -ge $minimumOverdrawRatio -and
            [double]$evidence.simplification_ratio -le $maximumSimplificationRatio
        if ([bool]$evidence.eligible_for_multi_region_trial -ne $computedEligibility) {
            throw "Damage-region trial evidence found inconsistent per-run eligibility in '$profileDir'."
        }

        $sourceBindingId = Get-ZirconDamageRegionSourceBindingId -Manifest $manifest
        $captureContractId = Get-ZirconDamageRegionCaptureContractId -Manifest $manifest
        $sourceBindingIds += $sourceBindingId
        $captureContractIds += $captureContractId
        $counterScenarios += [string]$evidence.counter_scenario
        $runs += [pscustomobject]@{
            run_ordinal = [int]$options.run_ordinal
            session_id = [string]$manifest.session_id
            profile_directory = $profileDir
            sample_count = [int]$evidence.sample_count
            total_source_rect_count = [int64]$evidence.total_source_rect_count
            total_simplification_count = [int64]$evidence.total_simplification_count
            total_represented_area = [double]$evidence.total_represented_area
            total_bounding_area = [double]$evidence.total_bounding_area
            total_bounding_overdraw_area = [double]$evidence.total_bounding_overdraw_area
            eligible = [bool]$evidence.eligible_for_multi_region_trial
        }
    }

    if (@($sourceBindingIds | Select-Object -Unique).Count -ne 1) {
        throw 'Damage-region trial evidence requires every run to use the same source binding.'
    }
    if (@($captureContractIds | Select-Object -Unique).Count -ne 1) {
        throw 'Damage-region trial evidence requires every run to use the same capture contract.'
    }
    if (@($counterScenarios | Select-Object -Unique).Count -ne 1) {
        throw 'Damage-region trial evidence requires every run to use the same counter scenario.'
    }

    $runs = @($runs | Sort-Object -Property run_ordinal)
    $declaredRunCounts = @($resolvedProfileDirs | ForEach-Object {
            $manifestPath = Join-Path $_ 'source_manifest.json'
            $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
            [int]$manifest.capture.options.measured_run_count
        } | Select-Object -Unique)
    if ($declaredRunCounts.Count -ne 1 -or $declaredRunCounts[0] -ne $runs.Count) {
        throw 'Damage-region trial evidence does not contain the complete declared measured run set.'
    }
    for ($index = 0; $index -lt $runs.Count; $index++) {
        if ($runs[$index].run_ordinal -ne ($index + 1)) {
            throw 'Damage-region trial evidence requires a contiguous measured run ordinal set.'
        }
    }

    $totalSampleCount = [int64](($runs | Measure-Object -Property sample_count -Sum).Sum)
    $totalSourceRectCount = [int64](($runs | Measure-Object -Property total_source_rect_count -Sum).Sum)
    $totalSimplificationCount = [int64](($runs | Measure-Object -Property total_simplification_count -Sum).Sum)
    $totalRepresentedArea = [double](($runs | Measure-Object -Property total_represented_area -Sum).Sum)
    $totalBoundingArea = [double](($runs | Measure-Object -Property total_bounding_area -Sum).Sum)
    $totalBoundingOverdrawArea = [double](($runs | Measure-Object -Property total_bounding_overdraw_area -Sum).Sum)
    $everyRunEligible = @($runs | Where-Object { -not $_.eligible }).Count -eq 0
    $minimumRunCount = 3
    $result = [ordered]@{
        schema_version = 1
        capture_scenario = $ScenarioName
        counter_scenario = $counterScenarios[0]
        source_binding_id = $sourceBindingIds[0]
        capture_contract_id = $captureContractIds[0]
        run_count = $runs.Count
        minimum_run_count = $minimumRunCount
        total_sample_count = $totalSampleCount
        total_source_rect_count = $totalSourceRectCount
        total_simplification_count = $totalSimplificationCount
        total_represented_area = $totalRepresentedArea
        total_bounding_area = $totalBoundingArea
        total_bounding_overdraw_area = $totalBoundingOverdrawArea
        bounding_overdraw_ratio = if ($totalBoundingArea -gt 0) {
            $totalBoundingOverdrawArea / $totalBoundingArea
        }
        else {
            0.0
        }
        simplification_ratio = if ($totalSourceRectCount -gt 0) {
            $totalSimplificationCount / $totalSourceRectCount
        }
        else {
            0.0
        }
        every_run_eligible = $everyRunEligible
        trial_recommended = $runs.Count -ge $minimumRunCount -and $everyRunEligible
        performance_accepted = $false
        runs = @($runs)
    }
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    $outputPath = Join-Path $OutputDir 'ui_damage_region_trial_evidence.json'
    $result | ConvertTo-Json -Depth 7 |
        Set-Content -LiteralPath $outputPath -Encoding UTF8
    return [pscustomobject]$result
}
