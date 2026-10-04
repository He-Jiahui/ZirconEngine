function Get-ZirconUiSurfacePipelineMetricSummary {
    param([object[]]$Values)

    $orderedValues = @($Values |
            Where-Object { $null -ne $_ } |
            ForEach-Object { [double]$_ } |
            Where-Object { -not [double]::IsNaN($_) -and -not [double]::IsInfinity($_) } |
            Sort-Object)
    if ($orderedValues.Count -eq 0) {
        return [pscustomobject][ordered]@{
            sample_count = 0
            p50 = $null
            p95 = $null
            max = $null
            sum = 0
            mean = $null
        }
    }

    $nearestRank = {
        param([double]$Percentile)

        $index = [Math]::Max(0, [Math]::Ceiling($orderedValues.Count * $Percentile) - 1)
        return $orderedValues[$index]
    }
    $measure = $orderedValues | Measure-Object -Sum -Average
    return [pscustomobject][ordered]@{
        sample_count = $orderedValues.Count
        p50 = & $nearestRank 0.50
        p95 = & $nearestRank 0.95
        max = $orderedValues[$orderedValues.Count - 1]
        sum = $measure.Sum
        mean = $measure.Average
    }
}

function Get-ZirconUiSurfacePipelineCounterSummary {
    param(
        [object[]]$Counters,
        [string]$Name
    )

    return Get-ZirconUiSurfacePipelineMetricSummary -Values @($Counters |
            Where-Object { $_.stream -eq "runtime" -and $_.name -eq $Name } |
            ForEach-Object { $_.value })
}

function Export-ZirconUiSurfacePipelineMetrics {
    param([string]$ProfileDir)

    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path -LiteralPath $timelinePath)) {
        return $null
    }

    $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
    $counters = @($timeline.counters)
    $stageCounters = [ordered]@{
        surface_rebuild = "ui.surface_rebuild.total_elapsed_us"
        layout = "ui.surface_rebuild.layout_elapsed_us"
        post_layout = "ui.surface_rebuild.post_layout_elapsed_us"
        base_picking = "ui.surface_rebuild.base_picking_elapsed_us"
        render_extract = "ui.surface_rebuild.render_extract_elapsed_us"
        projected_hit_rebuild = "ui.surface_projected_hit.rebuild_elapsed_us"
        projected_hit_patch = "ui.surface_projected_hit.patch_elapsed_us"
        navigation_rebuild = "ui.navigation_index.rebuild_elapsed_us"
        frame_publication = "ui.surface_frame.publication_elapsed_us"
    }
    $workCounters = [ordered]@{
        dirty_node_count = "ui.surface_rebuild.dirty_node_count"
        layout_visited_node_count = "ui.surface_rebuild.layout_visited_node_count"
        arranged_outer_node_visit_count = "ui.surface_rebuild.arranged_outer_node_visit_count"
        hit_grid_outer_node_visit_count = "ui.surface_rebuild.hit_grid_outer_node_visit_count"
        render_outer_node_visit_count = "ui.surface_rebuild.render_outer_node_visit_count"
        render_command_reused_count = "ui.surface_rebuild.render_command_reused_count"
        render_command_rebuilt_count = "ui.surface_rebuild.render_command_rebuilt_count"
        projected_hit_affected_entry_count = "ui.surface_projected_hit.affected_entry_count"
        projected_hit_patch_fallback_count = "ui.surface_projected_hit.patch_fallback_count"
    }

    $stageMetrics = [ordered]@{}
    foreach ($entry in $stageCounters.GetEnumerator()) {
        $stageMetrics[$entry.Key] = Get-ZirconUiSurfacePipelineCounterSummary `
            -Counters $counters `
            -Name $entry.Value
    }
    $workMetrics = [ordered]@{}
    foreach ($entry in $workCounters.GetEnumerator()) {
        $workMetrics[$entry.Key] = Get-ZirconUiSurfacePipelineCounterSummary `
            -Counters $counters `
            -Name $entry.Value
    }

    $metrics = [pscustomobject][ordered]@{
        schema_version = 1
        stage_duration_us = [pscustomobject]$stageMetrics
        work = [pscustomobject]$workMetrics
    }
    $outputPath = Join-Path $ProfileDir "ui_surface_pipeline_metrics.json"
    $metrics | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath $outputPath -Encoding UTF8
    return $outputPath
}
