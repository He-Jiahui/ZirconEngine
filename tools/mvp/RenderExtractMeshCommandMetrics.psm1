Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'RenderExtractBaselineMetrics.psm1') `
    -Force `
    -DisableNameChecking `
    -ErrorAction Stop

function Get-RenderExtractMeshCommandDispatchObservation {
    param([Parameter(Mandatory)][object[]]$Counters)

    $dispatchCounters = @($Counters | Where-Object {
            $_.stream -eq 'render' -and $_.name -eq 'mesh_commands.parallel_enabled'
        })
    $observedEnabled = $false
    $observedDisabled = $false
    foreach ($counter in $dispatchCounters) {
        if ($counter.PSObject.Properties['value']) {
            $value = [double]$counter.value
            $observedEnabled = $observedEnabled -or $value -gt 0
            $observedDisabled = $observedDisabled -or $value -eq 0
            continue
        }
        if ($counter.PSObject.Properties['statistics']) {
            $statistics = $counter.statistics
            $maximum = if ($statistics -is [Collections.IDictionary] -and
                $statistics.Contains('max')) {
                $statistics['max']
            }
            elseif ($statistics.PSObject.Properties['max']) {
                $statistics.max
            }
            else {
                $null
            }
            $minimum = if ($statistics -is [Collections.IDictionary] -and
                $statistics.Contains('min')) {
                $statistics['min']
            }
            elseif ($statistics.PSObject.Properties['min']) {
                $statistics.min
            }
            else {
                $null
            }
            if ($null -ne $maximum) {
                $observedEnabled = $observedEnabled -or [double]$maximum -gt 0
            }
            if ($null -ne $minimum) {
                $observedDisabled = $observedDisabled -or [double]$minimum -eq 0
            }
        }
    }
    return [pscustomobject][ordered]@{
        emitted = $dispatchCounters.Count -gt 0
        enabled = $observedEnabled
        disabled = $observedDisabled
    }
}

function Get-RenderExtractMeshCommandPreparationCoverage {
    param(
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters
    )

    $dispatch = Get-RenderExtractMeshCommandDispatchObservation -Counters $Counters
    $requiredBranchSpans = if (-not $dispatch.emitted) {
        @('prepare_cached_serial', 'serial_prepare_and_project', 'seal_phase_buffers')
    }
    else {
        $branchSpanNames = [System.Collections.Generic.List[string]]::new()
        foreach ($name in @(
            'prepare_cached_dispatch',
            'normalize_source_order',
            'parallel_admission'
        )) {
            $branchSpanNames.Add($name)
        }
        if ($dispatch.disabled) {
            $branchSpanNames.Add('prepare_cached_serial')
            $branchSpanNames.Add('serial_prepare_and_project')
        }
        if ($dispatch.enabled) {
            $branchSpanNames.Add('owner_transaction')
            $branchSpanNames.Add('worker_projection_wait')
            $branchSpanNames.Add('ordered_merge')
        }
        $branchSpanNames.Add('seal_phase_buffers')
        @($branchSpanNames)
    }
    $requiredSpanNames = @(
        'extract_cached_pre_mesh_draw',
        'pre_mesh_materialize',
        'finalize_old_path',
        'indirect_plan',
        'replay_record'
    ) + $requiredBranchSpans

    Get-RenderExtractInstrumentationCoverage `
        -Spans $Spans `
        -Counters $Counters `
        -SpanCategory 'mesh_commands' `
        -SpanNames @(
            'extract_cached_pre_mesh_draw',
            'prepare_cached_dispatch',
            'prepare_cached_serial',
            'normalize_source_order',
            'parallel_admission',
            'owner_transaction',
            'worker_projection_wait',
            'ordered_merge',
            'serial_prepare_and_project',
            'seal_phase_buffers',
            'pre_mesh_materialize',
            'finalize_old_path',
            'indirect_plan',
            'replay_record'
        ) `
        -RequiredSpanNames $requiredSpanNames `
        -CounterNames @(
            'mesh_commands.cache_hit_count',
            'mesh_commands.cache_miss_count',
            'mesh_commands.command_rebuild_count',
            'mesh_commands.command_count',
            'mesh_commands.command_arena_grow_count',
            'mesh_commands.command_arena_peak_capacity',
            'mesh_commands.phase_bucket_grow_count',
            'mesh_commands.phase_bucket_capacity',
            'mesh_commands.command_build_count',
            'mesh_commands.partition_move_count',
            'mesh_commands.merge_move_count',
            'mesh_commands.finalize_count',
            'mesh_commands.sort_count',
            'mesh_commands.partition_command_visit_count',
            'mesh_commands.merge_command_visit_count',
            'mesh_commands.sort_command_visit_count',
            'mesh_commands.active_bucket_count',
            'mesh_commands.cache_hit_payload_arc_clone_count',
            'mesh_commands.command_generation_reuse_count',
            'mesh_commands.view_generation_reuse_count',
            'mesh_commands.bucket.depth_prepass_length',
            'mesh_commands.bucket.shadow_length',
            'mesh_commands.bucket.opaque_length',
            'mesh_commands.bucket.alpha_mask_length',
            'mesh_commands.bucket.advanced_pbr_opaque_length',
            'mesh_commands.bucket.transmission_length',
            'mesh_commands.bucket.transparent_length',
            'mesh_commands.bucket.half_resolution_transparent_length',
            'mesh_commands.bucket.velocity_length',
            'mesh_commands.bucket.taa_reactive_mask_length'
        ) `
        -Streams 'render' `
        -SampleLimit 40 `
        -RequireAllSpanNames `
        -RequireAllCounterNames
}

function Get-RenderExtractMeshCommandParallelDispatchCoverage {
    param(
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters
    )

    $dispatch = Get-RenderExtractMeshCommandDispatchObservation -Counters $Counters
    $requiredSpanNames = if ($dispatch.enabled) {
        @('parallel_admission', 'owner_transaction', 'worker_projection_wait', 'ordered_merge')
    }
    else {
        @('parallel_admission')
    }

    Get-RenderExtractInstrumentationCoverage `
        -Spans $Spans `
        -Counters $Counters `
        -SpanCategory 'mesh_commands' `
        -SpanNames @(
            'parallel_admission',
            'owner_transaction',
            'worker_projection_wait',
            'ordered_merge'
        ) `
        -RequiredSpanNames $requiredSpanNames `
        -CounterNames @(
            'mesh_commands.batch_count',
            'mesh_commands.worker_count',
            'mesh_commands.parallel_enabled',
            'mesh_commands.dispatch_reason_code'
        ) `
        -Streams 'render' `
        -RequireAllSpanNames `
        -RequireAllCounterNames
}

function Get-RenderExtractMeshCommandAttemptCoverage {
    param(
        [Parameter(Mandatory)][ValidateRange(1, [int]::MaxValue)][int]$Attempt,
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters
    )

    [pscustomobject][ordered]@{
        attempt = $Attempt
        preparation = Get-RenderExtractMeshCommandPreparationCoverage `
            -Spans $Spans `
            -Counters $Counters
        parallel_dispatch = Get-RenderExtractMeshCommandParallelDispatchCoverage `
            -Spans $Spans `
            -Counters $Counters
    }
}

function Get-RenderExtractMeshCommandScenarioCoverage {
    param(
        [Parameter(Mandatory)][object[]]$Attempts,
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters
    )

    [pscustomobject][ordered]@{
        preparation = Get-RenderExtractMeshCommandCombinedAttemptCoverage `
            -Attempts $Attempts `
            -Spans $Spans `
            -Counters $Counters `
            -Kind 'preparation'
        parallel_dispatch = Get-RenderExtractMeshCommandCombinedAttemptCoverage `
            -Attempts $Attempts `
            -Spans $Spans `
            -Counters $Counters `
            -Kind 'parallel_dispatch'
    }
}

function Get-RenderExtractMeshCommandCombinedAttemptCoverage {
    param(
        [Parameter(Mandatory)][object[]]$Attempts,
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters,
        [Parameter(Mandatory)][ValidateSet('preparation', 'parallel_dispatch')][string]$Kind
    )

    $attemptStatuses = @($Attempts | ForEach-Object { $_.$Kind.status })
    $aggregate = if ($Kind -eq 'preparation') {
        Get-RenderExtractMeshCommandPreparationCoverage -Spans $Spans -Counters $Counters
    }
    else {
        Get-RenderExtractMeshCommandParallelDispatchCoverage -Spans $Spans -Counters $Counters
    }
    $aggregate['status'] = if (@($attemptStatuses | Where-Object { $_ -ne 'measured' }).Count -eq 0) {
        'measured'
    }
    elseif ($attemptStatuses -contains 'measured' -or $attemptStatuses -contains 'partial') {
        'partial'
    }
    else {
        'not_emitted'
    }
    $aggregate['attempts'] = @($Attempts | ForEach-Object {
            [pscustomobject][ordered]@{
                attempt = [int]$_.attempt
                status = $_.$Kind.status
                missing_span_names = @($_.$Kind.missing_span_names)
                missing_counter_names = @($_.$Kind.missing_counter_names)
            }
        })
    return $aggregate
}

function Add-RenderExtractMeshCommandMeasurementCoverage {
    param([Parameter(Mandatory)]$Report)

    $preparationStatuses = @(
        $Report.scenarios | ForEach-Object { $_.mesh_command_preparation.status }
    )
    $Report.measurement_coverage.mesh_command_preparation = [ordered]@{
        status = Get-RenderExtractCombinedCoverageStatus -Statuses $preparationStatuses
        source = 'mesh-command preparation timeline stages and cache/result counters'
    }

    $parallelDispatchStatuses = @(
        $Report.scenarios | ForEach-Object { $_.mesh_command_parallel_dispatch.status }
    )
    $Report.measurement_coverage.mesh_command_parallel_dispatch = [ordered]@{
        status = Get-RenderExtractCombinedCoverageStatus -Statuses $parallelDispatchStatuses
        source = 'mesh-command parallel admission stages and dispatch decision counters'
    }
}

function Get-RenderExtractCombinedCoverageStatus {
    param([Parameter(Mandatory)][object[]]$Statuses)

    if ($Statuses -contains 'partial' -or
        ($Statuses -contains 'measured' -and $Statuses -contains 'not_emitted')) {
        return 'partial'
    }
    if ($Statuses -contains 'measured') {
        return 'measured'
    }
    return 'not_emitted'
}

Export-ModuleMember -Function @(
    'Add-RenderExtractMeshCommandMeasurementCoverage',
    'Get-RenderExtractMeshCommandAttemptCoverage',
    'Get-RenderExtractMeshCommandCombinedAttemptCoverage',
    'Get-RenderExtractMeshCommandParallelDispatchCoverage',
    'Get-RenderExtractMeshCommandPreparationCoverage',
    'Get-RenderExtractMeshCommandScenarioCoverage'
)
