param(
    [string]$ProfileDir,
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'

function Get-ZirconResizeCounterValues {
    param(
        [object]$Timeline,
        [string]$Name
    )

    return @($Timeline.counters | Where-Object { $_.name -eq $Name } | ForEach-Object {
            $_.value
        })
}

function Get-ZirconResizeLatencySummary {
    param([object[]]$Values)

    $samples = @()
    foreach ($value in @($Values)) {
        try {
            $sample = [double]$value
        }
        catch {
            return $null
        }
        if ([double]::IsNaN($sample) -or [double]::IsInfinity($sample) -or $sample -lt 0.0) {
            return $null
        }
        $samples += $sample
    }
    if ($samples.Count -eq 0) {
        return $null
    }

    $ordered = @($samples | Sort-Object)
    $nearestRank = {
        param([double]$Percentile)
        $index = [Math]::Max(0, [Math]::Ceiling($ordered.Count * $Percentile) - 1)
        return [double]$ordered[$index]
    }
    return [pscustomobject][ordered]@{
        sample_count = [int64]$ordered.Count
        p50 = & $nearestRank 0.50
        p95 = & $nearestRank 0.95
        p99 = & $nearestRank 0.99
        max = [double]$ordered[-1]
        percentile_policy = 'nearest_rank'
    }
}

function Test-ZirconWindowResizeGeometryEvidence {
    param(
        [object]$Timeline,
        [double]$MatchingGeometryP95BudgetUs = 16670.0,
        [double]$MatchingGeometryMaxBudgetUs = 33340.0,
        [double]$GeometryPrepareP95BudgetUs = 8000.0
    )

    $requiredCounters = @(
        'window_metrics_received_count',
        'duplicate_metrics_suppressed_count',
        'geometry_coalesced_input_count',
        'matching_geometry_presented_input_count',
        'geometry_fallback_input_count',
        'geometry_prepare_count',
        'geometry_commit_count',
        'geometry_noop_count',
        'geometry_fallback_transaction_count',
        'stale_geometry_present_count',
        'geometry_full_hit_index_rebuild_count',
        'geometry_full_command_rebuild_count'
    )
    $blockers = [System.Collections.Generic.List[object]]::new()
    $totals = [ordered]@{}
    foreach ($shortName in $requiredCounters) {
        $name = "ui.window_resize.$shortName"
        $values = @(Get-ZirconResizeCounterValues -Timeline $Timeline -Name $name)
        if ($values.Count -eq 0) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'missing_counter'
                    counter = $name
                })
            continue
        }

        $sum = 0.0
        $valid = $true
        foreach ($value in $values) {
            try {
                $number = [double]$value
            }
            catch {
                $valid = $false
                break
            }
            if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or
                $number -lt 0.0 -or [Math]::Floor($number) -ne $number) {
                $valid = $false
                break
            }
            $sum += $number
        }
        if (-not $valid) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'invalid_counter_value'
                    counter = $name
                })
            continue
        }
        $totals[$shortName] = [int64]$sum
    }

    $conservation = [ordered]@{
        non_duplicate_inputs = $null
        resolved_inputs = $null
        prepared_transactions = $null
        resolved_transactions = $null
    }
    if ($totals.Count -eq $requiredCounters.Count) {
        $received = [int64]$totals.window_metrics_received_count
        $duplicates = [int64]$totals.duplicate_metrics_suppressed_count
        if ($received -le 0 -or $duplicates -gt $received) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'invalid_resize_input_counts'
                    received = $received
                    duplicates = $duplicates
                })
        }
        else {
            $nonDuplicate = $received - $duplicates
            $resolvedInputs =
                [int64]$totals.geometry_coalesced_input_count +
                [int64]$totals.matching_geometry_presented_input_count +
                [int64]$totals.geometry_fallback_input_count
            $conservation.non_duplicate_inputs = $nonDuplicate
            $conservation.resolved_inputs = $resolvedInputs
            if ($nonDuplicate -ne $resolvedInputs) {
                $blockers.Add([pscustomobject][ordered]@{
                        code = 'resize_input_conservation_failed'
                        non_duplicate_inputs = $nonDuplicate
                        resolved_inputs = $resolvedInputs
                    })
            }
        }

        $prepared = [int64]$totals.geometry_prepare_count
        $resolvedTransactions =
            [int64]$totals.geometry_commit_count +
            [int64]$totals.geometry_noop_count +
            [int64]$totals.geometry_fallback_transaction_count
        $conservation.prepared_transactions = $prepared
        $conservation.resolved_transactions = $resolvedTransactions
        if ($prepared -ne $resolvedTransactions) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'geometry_transaction_conservation_failed'
                    prepared_transactions = $prepared
                    resolved_transactions = $resolvedTransactions
                })
        }
        if ([int64]$totals.matching_geometry_presented_input_count -ne
            [int64]$totals.geometry_commit_count) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'matching_geometry_commit_membership_failed'
                    matching_inputs = [int64]$totals.matching_geometry_presented_input_count
                    commits = [int64]$totals.geometry_commit_count
                })
        }
        if ([int64]$totals.geometry_fallback_input_count -ne 0 -or
            [int64]$totals.geometry_fallback_transaction_count -ne 0) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'ordinary_resize_fallback'
                    input_count = [int64]$totals.geometry_fallback_input_count
                    transaction_count = [int64]$totals.geometry_fallback_transaction_count
                })
        }
        if ([int64]$totals.stale_geometry_present_count -ne 0) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'stale_geometry_presented'
                    count = [int64]$totals.stale_geometry_present_count
                })
        }
        if ([int64]$totals.geometry_full_hit_index_rebuild_count -ne 0) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'full_hit_index_rebuild'
                    count = [int64]$totals.geometry_full_hit_index_rebuild_count
                })
        }
        if ([int64]$totals.geometry_full_command_rebuild_count -ne 0) {
            $blockers.Add([pscustomobject][ordered]@{
                    code = 'full_command_rebuild'
                    count = [int64]$totals.geometry_full_command_rebuild_count
                })
        }
    }

    $matchingValues = @(Get-ZirconResizeCounterValues -Timeline $Timeline `
            -Name 'ui.window_resize.input_to_matching_geometry_us')
    $prepareValues = @(Get-ZirconResizeCounterValues -Timeline $Timeline `
            -Name 'ui.window_resize.geometry_prepare_us')
    $matchingSummary = Get-ZirconResizeLatencySummary -Values $matchingValues
    $prepareSummary = Get-ZirconResizeLatencySummary -Values $prepareValues
    if ($null -eq $matchingSummary) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'missing_or_invalid_matching_geometry_latency'
                counter = 'ui.window_resize.input_to_matching_geometry_us'
            })
    }
    elseif ($totals.Contains('matching_geometry_presented_input_count') -and
        [int64]$matchingSummary.sample_count -ne
            [int64]$totals.matching_geometry_presented_input_count) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'matching_geometry_latency_membership_failed'
                samples = [int64]$matchingSummary.sample_count
                matching_inputs = [int64]$totals.matching_geometry_presented_input_count
            })
    }
    elseif ([double]$matchingSummary.p95 -gt $MatchingGeometryP95BudgetUs -or
        [double]$matchingSummary.max -gt $MatchingGeometryMaxBudgetUs) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'matching_geometry_latency_budget_exceeded'
                p95_us = [double]$matchingSummary.p95
                max_us = [double]$matchingSummary.max
            })
    }

    if ($null -eq $prepareSummary) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'missing_or_invalid_geometry_prepare_latency'
                counter = 'ui.window_resize.geometry_prepare_us'
            })
    }
    elseif ($totals.Contains('geometry_prepare_count') -and
        [int64]$prepareSummary.sample_count -ne [int64]$totals.geometry_prepare_count) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'geometry_prepare_latency_membership_failed'
                samples = [int64]$prepareSummary.sample_count
                prepares = [int64]$totals.geometry_prepare_count
            })
    }
    elseif ([double]$prepareSummary.p95 -gt $GeometryPrepareP95BudgetUs) {
        $blockers.Add([pscustomobject][ordered]@{
                code = 'geometry_prepare_latency_budget_exceeded'
                p95_us = [double]$prepareSummary.p95
            })
    }

    return [pscustomobject][ordered]@{
        schema = 'zircon.editor.window_resize_geometry_evidence.v1'
        ready = $blockers.Count -eq 0
        blockers = @($blockers)
        counter_totals = [pscustomobject]$totals
        conservation = [pscustomobject]$conservation
        latency = [pscustomobject][ordered]@{
            input_to_matching_geometry_us = $matchingSummary
            geometry_prepare_us = $prepareSummary
        }
        budgets_us = [pscustomobject][ordered]@{
            matching_geometry_p95 = $MatchingGeometryP95BudgetUs
            matching_geometry_max = $MatchingGeometryMaxBudgetUs
            geometry_prepare_p95 = $GeometryPrepareP95BudgetUs
        }
    }
}

function Assert-ZirconWindowResizeGeometryOutputPath {
    param([string]$Path)

    $resolved = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetPathRoot($resolved)
    if ($root -notmatch '^[D-F]:\\$') {
        throw 'performance artifacts must be written under the D, E, or F drive'
    }
    return $resolved
}

if ($MyInvocation.InvocationName -ne '.') {
    if ([string]::IsNullOrWhiteSpace($ProfileDir)) {
        throw 'ProfileDir is required'
    }
    $timelinePath = Join-Path $ProfileDir 'timeline.zrtrace.json'
    if (-not (Test-Path -LiteralPath $timelinePath -PathType Leaf)) {
        throw "timeline artifact is missing: $timelinePath"
    }

    $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
    $evidence = Test-ZirconWindowResizeGeometryEvidence -Timeline $timeline
    $bound = [ordered]@{}
    foreach ($property in $evidence.PSObject.Properties) {
        $bound[$property.Name] = $property.Value
    }
    $bound.profile_binding = [pscustomobject][ordered]@{
        timeline_path = [System.IO.Path]::GetFullPath($timelinePath)
        timeline_sha256 = (Get-FileHash -LiteralPath $timelinePath -Algorithm SHA256).Hash
        tool_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
        generated_utc = [DateTime]::UtcNow.ToString('o')
    }
    $payload = ([pscustomobject]$bound | ConvertTo-Json -Depth 12) + "`n"
    if ([string]::IsNullOrWhiteSpace($OutputPath)) {
        Write-Output $payload
    }
    else {
        $resolvedOutputPath = Assert-ZirconWindowResizeGeometryOutputPath -Path $OutputPath
        $parent = Split-Path -Parent $resolvedOutputPath
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
        [System.IO.File]::WriteAllText($resolvedOutputPath, $payload, [System.Text.UTF8Encoding]::new($false))
    }
    if (-not [bool]$evidence.ready) {
        exit 1
    }
}
