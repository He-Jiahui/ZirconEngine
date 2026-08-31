function Get-ZirconHierarchyFilterMetricCounterNames {
    return @(
        "hierarchy_filter_projection_invocation_count",
        "hierarchy_filter_source_row_count",
        "hierarchy_filter_name_match_count",
        "hierarchy_filter_ancestor_link_count",
        "hierarchy_filter_visible_row_count"
    )
}

function ConvertTo-ZirconHierarchyFilterMetricValue {
    param([object]$Value)

    if ($null -eq $Value -or ($Value -is [string] -and [string]::IsNullOrWhiteSpace($Value))) {
        return $null
    }
    try {
        $numericValue = [double]$Value
    }
    catch {
        return $null
    }
    if ([double]::IsNaN($numericValue) -or [double]::IsInfinity($numericValue) -or $numericValue -lt 0) {
        return $null
    }
    return $numericValue
}

function Get-ZirconHierarchyFilterMetricSummary {
    param([object[]]$Values)

    $orderedValues = @($Values |
            ForEach-Object { ConvertTo-ZirconHierarchyFilterMetricValue -Value $_ } |
            Where-Object { $null -ne $_ } |
            Sort-Object)
    if ($orderedValues.Count -eq 0) {
        return $null
    }
    $nearestRank = {
        param([double]$Percentile)

        $index = [Math]::Max(0, [Math]::Ceiling($orderedValues.Count * $Percentile) - 1)
        return $orderedValues[$index]
    }
    return [pscustomobject]@{
        sample_count = $orderedValues.Count
        min = $orderedValues[0]
        p50 = & $nearestRank 0.50
        p95 = & $nearestRank 0.95
        max = $orderedValues[$orderedValues.Count - 1]
        mean = ($orderedValues | Measure-Object -Average).Average
    }
}

function Export-ZirconHierarchyFilterMetrics {
    param([string]$ProfileDir)

    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path -LiteralPath $timelinePath)) {
        return $null
    }
    $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
    $projectionDurations = @($timeline.spans |
            Where-Object {
                $_.stream -eq "editor" -and
                $_.category -eq "hierarchy" -and
                $_.name -eq "filter_projection"
            } |
            ForEach-Object { $_.duration_us })
    $counterMetrics = @(
        foreach ($counterName in Get-ZirconHierarchyFilterMetricCounterNames) {
            $summary = Get-ZirconHierarchyFilterMetricSummary -Values @($timeline.counters |
                    Where-Object {
                        $_.stream -eq "editor" -and $_.name -eq $counterName
                    } |
                    ForEach-Object { $_.value })
            if ($null -ne $summary) {
                [pscustomobject]@{
                    name = $counterName
                    values = $summary
                }
            }
        }
    )
    $metrics = [pscustomobject]@{
        schema_version = 1
        projection_duration_us = Get-ZirconHierarchyFilterMetricSummary -Values $projectionDurations
        counters = $counterMetrics
    }
    $metrics | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $ProfileDir "hierarchy_filter_metrics.json") -Encoding UTF8
    return $metrics
}

function Test-ZirconHierarchyFilterMetricsGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "hierarchy_filter") {
        return $true
    }
    $metricsPath = Join-Path $ProfileDir "hierarchy_filter_metrics.json"
    if (-not (Test-Path -LiteralPath $metricsPath)) {
        Write-Warning "Hierarchy filter gate requires filter-projection metrics."
        return $false
    }
    $metrics = Get-Content -LiteralPath $metricsPath -Raw | ConvertFrom-Json
    if ($null -eq $metrics.projection_duration_us -or
        [int64]$metrics.projection_duration_us.sample_count -le 0) {
        Write-Warning "Hierarchy filter gate requires at least one filter-projection span."
        return $false
    }
    foreach ($counterName in Get-ZirconHierarchyFilterMetricCounterNames) {
        $counter = @($metrics.counters | Where-Object { $_.name -eq $counterName } | Select-Object -First 1)
        if ($counter.Count -ne 1 -or $null -eq $counter[0].values -or
            [int64]$counter[0].values.sample_count -le 0) {
            Write-Warning "Hierarchy filter gate is missing counter evidence: $counterName"
            return $false
        }
    }
    return $true
}
