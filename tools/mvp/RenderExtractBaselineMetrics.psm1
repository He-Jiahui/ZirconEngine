Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'RenderExtractBaselineEvidence.psm1') -DisableNameChecking -ErrorAction Stop

function Get-RenderExtractPercentile {
    param(
        [Parameter(Mandatory)][double[]]$SortedValues,
        [Parameter(Mandatory)][ValidateRange(0, 100)][int]$Percentile
    )

    if ($SortedValues.Count -eq 0) {
        throw 'Cannot compute a percentile from zero samples.'
    }
    $index = [int][Math]::Ceiling((($SortedValues.Count - 1) * $Percentile) / 100.0)
    return $SortedValues[$index]
}

function Get-RenderExtractStatistics {
    param([Parameter(Mandatory)][double[]]$Values)

    if ($Values.Count -eq 0) {
        throw 'Cannot aggregate zero samples.'
    }
    $sorted = [double[]]$Values.Clone()
    [Array]::Sort($sorted)
    $total = 0.0
    foreach ($value in $sorted) {
        $total += $value
    }
    $minimum = $sorted[0]
    $maximum = $sorted[$sorted.Count - 1]
    return [ordered]@{
        sample_count = $sorted.Count
        min = $minimum
        median = Get-RenderExtractPercentile -SortedValues $sorted -Percentile 50
        p95 = Get-RenderExtractPercentile -SortedValues $sorted -Percentile 95
        p99 = Get-RenderExtractPercentile -SortedValues $sorted -Percentile 99
        max = $maximum
        mean = $total / $sorted.Count
        total = $total
        range = [ordered]@{
            min = $minimum
            max = $maximum
            delta = $maximum - $minimum
        }
    }
}

function Get-RenderExtractTimelineInteger {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $rawValue = Get-RenderExtractReportProperty -Value $Value -Name $Name -Label $Label
    if ($rawValue -isnot [ValueType]) {
        throw "$Label $Name must be a JSON number."
    }
    $number = [double]$rawValue
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or
        $number -lt 0 -or $number -ne [Math]::Truncate($number) -or
        $number -gt [Int64]::MaxValue) {
        throw "$Label $Name must be a finite nonnegative integer."
    }
    return [Int64]$number
}

function Get-RenderExtractSteadyMeasurementWindow {
    param(
        [Parameter(Mandatory)][object[]]$Frames,
        [Parameter(Mandatory)][ValidateRange(0, 1000000)][int]$WarmupPresentedFrameCount,
        [Parameter(Mandatory)][ValidateRange(1, 1000000)][int]$MeasuredPresentedFrameCount,
        [Parameter(Mandatory)][string]$Label
    )

    $primaryFrames = @($Frames | Where-Object {
            [string](Get-RenderExtractReportProperty -Value $_ -Name 'stream' -Label "$Label frame") -eq 'app' -and
            [string](Get-RenderExtractReportProperty -Value $_ -Name 'name' -Label "$Label frame") -eq 'runtime_redraw'
        })
    if ($primaryFrames.Count -eq 0) {
        throw "$Label has no app/runtime_redraw presented-frame samples."
    }
    $byIndex = @{}
    foreach ($frame in $primaryFrames) {
        $index = Get-RenderExtractTimelineInteger -Value $frame -Name 'frame_index' -Label "$Label app/runtime_redraw frame"
        if ($byIndex.ContainsKey($index)) {
            throw "$Label has duplicate app/runtime_redraw frame_index $index."
        }
        $byIndex[$index] = $frame
    }

    $firstMeasuredFrameIndex = [Int64]$WarmupPresentedFrameCount
    $lastMeasuredFrameIndex = $firstMeasuredFrameIndex + [Int64]$MeasuredPresentedFrameCount - 1
    $selectedFrames = [System.Collections.Generic.List[object]]::new()
    for ($frameIndex = $firstMeasuredFrameIndex; $frameIndex -le $lastMeasuredFrameIndex; $frameIndex++) {
        if (-not $byIndex.ContainsKey($frameIndex)) {
            throw "$Label is missing app/runtime_redraw frame_index $frameIndex in its measured window."
        }
        $selectedFrames.Add($byIndex[$frameIndex]) | Out-Null
    }

    $firstFrame = $selectedFrames[0]
    $lastFrame = $selectedFrames[$selectedFrames.Count - 1]
    $startUs = Get-RenderExtractTimelineInteger -Value $firstFrame -Name 'start_us' -Label "$Label first measured frame"
    $lastStartUs = Get-RenderExtractTimelineInteger -Value $lastFrame -Name 'start_us' -Label "$Label last measured frame"
    $lastDurationUs = Get-RenderExtractTimelineInteger -Value $lastFrame -Name 'duration_us' -Label "$Label last measured frame"
    if ($lastDurationUs -gt ([Int64]::MaxValue - $lastStartUs)) {
        throw "$Label measured timestamp window exceeds the supported integer range."
    }
    $endUs = $lastStartUs + $lastDurationUs
    if ($endUs -le $startUs) {
        throw "$Label has an empty measured timestamp window."
    }
    return [pscustomobject][ordered]@{
        primary_frame_stream = 'app'
        primary_frame_name = 'runtime_redraw'
        warmup_presented_frame_count = $WarmupPresentedFrameCount
        measured_presented_frame_count = $MeasuredPresentedFrameCount
        target_presented_frame_count = $WarmupPresentedFrameCount + $MeasuredPresentedFrameCount
        start_us = $startUs
        end_us = $endUs
        primary_frames = @($selectedFrames)
    }
}

function Select-RenderExtractTimelineWindowSamples {
    param(
        [Parameter(Mandatory)][object[]]$Frames,
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters,
        [Parameter(Mandatory)]$Window
    )

    $startUs = [Int64]$Window.start_us
    $endUs = [Int64]$Window.end_us
    $spans = @($Spans | Where-Object {
            $spanStartUs = Get-RenderExtractTimelineInteger -Value $_ -Name 'start_us' -Label 'Timeline span'
            $spanDurationUs = Get-RenderExtractTimelineInteger -Value $_ -Name 'duration_us' -Label 'Timeline span'
            $spanStartUs -ge $startUs -and ($spanStartUs + $spanDurationUs) -le $endUs
        })
    $counters = @($Counters | Where-Object {
            $timestampUs = Get-RenderExtractTimelineInteger -Value $_ -Name 'timestamp_us' -Label 'Timeline counter'
            $timestampUs -ge $startUs -and $timestampUs -le $endUs
        })
    return [pscustomobject][ordered]@{
        frames = @($Window.primary_frames)
        spans = $spans
        counters = $counters
    }
}

function ConvertTo-RenderExtractSpanAggregate {
    param([Parameter(Mandatory)][object[]]$Records)

    $first = $Records[0]
    return [pscustomobject][ordered]@{
        stream = $first.stream
        category = $first.category
        name = $first.name
        path = $first.path
        statistics_us = Get-RenderExtractStatistics -Values @($Records | ForEach-Object { [double]$_.duration_us })
    }
}

function ConvertTo-RenderExtractCounterAggregate {
    param([Parameter(Mandatory)][object[]]$Records)

    $first = $Records[0]
    return [pscustomobject][ordered]@{
        stream = $first.stream
        name = $first.name
        statistics = Get-RenderExtractStatistics -Values @($Records | ForEach-Object { [double]$_.value })
    }
}

function Get-RenderExtractAggregates {
    param(
        [Parameter(Mandatory)][object[]]$Records,
        [Parameter(Mandatory)][ValidateSet('span', 'counter')][string]$Kind
    )

    if ($Records.Count -eq 0) {
        return @()
    }
    $groups = @($Records | Group-Object -Property group_key)
    $aggregates = foreach ($group in $groups) {
        if ($Kind -eq 'span') {
            ConvertTo-RenderExtractSpanAggregate -Records @($group.Group)
        }
        else {
            ConvertTo-RenderExtractCounterAggregate -Records @($group.Group)
        }
    }
    if ($Kind -eq 'span') {
        return @($aggregates | Sort-Object @{ Expression = { $_.statistics_us.total }; Descending = $true }, path)
    }
    return @($aggregates | Sort-Object @{ Expression = { $_.statistics.total }; Descending = $true }, name)
}

function Get-RenderExtractInstrumentationCoverage {
    param(
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters,
        [string[]]$SpanCategory = @(),
        [string[]]$SpanNames = @(),
        [string[]]$CounterNames = @(),
        [switch]$RequireAllCounterNames
    )

    $matchingSpans = @(if ($SpanCategory.Count -eq 0 -or $SpanNames.Count -eq 0) {
            @()
        }
        else {
            $Spans | Where-Object {
                $SpanCategory -contains $_.category -and $SpanNames -contains $_.name
            }
        })
    $matchingCounters = @(if ($CounterNames.Count -eq 0) {
            @()
        }
        else {
            $Counters | Where-Object { $CounterNames -contains $_.name }
        })
    $missingCounterNames = @()
    if ($RequireAllCounterNames) {
        $matchedCounterNames = @($matchingCounters | ForEach-Object { [string]$_.name })
        $missingCounterNames = @($CounterNames | Where-Object { $_ -notin $matchedCounterNames })
    }
    $hasSamples = ($matchingSpans.Count + $matchingCounters.Count) -gt 0
    return [ordered]@{
        status = if (-not $hasSamples) {
            'not_emitted'
        }
        elseif ($RequireAllCounterNames -and $missingCounterNames.Count -gt 0) {
            'partial'
        }
        else {
            'measured'
        }
        spans = @($matchingSpans | Select-Object -First 20)
        counters = @($matchingCounters | Select-Object -First 20)
        missing_counter_names = $missingCounterNames
    }
}

function Get-RenderExtractSchedulerWorkerOccupancyAttempt {
    param(
        [Parameter(Mandatory)][object[]]$Counters,
        [Parameter(Mandatory)][string]$Label
    )

    $counterName = 'render_framework.scheduler.worker_utilization'
    $matching = @($Counters | Where-Object {
            $_.stream -eq 'runtime' -and $_.name -eq $counterName
        })
    if ($matching.Count -eq 0) {
        return [pscustomobject][ordered]@{
            status = 'not_emitted'
            reason = 'The scheduler worker occupancy counter was not emitted.'
            sample_count = 0
            observed_window_us = $null
            busy_duration_us = $null
            occupancy_ratio = $null
        }
    }

    $numericTypes = @(
        [sbyte], [byte], [short], [ushort], [int], [uint], [long], [ulong],
        [single], [double], [decimal]
    )
    $samples = [System.Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $matching.Count; $index++) {
        $counter = $matching[$index]
        $rawTimestamp = Get-RenderExtractReportProperty `
            -Value $counter `
            -Name 'timestamp_us' `
            -Label "$Label scheduler worker occupancy sample"
        if ($numericTypes -notcontains $rawTimestamp.GetType()) {
            throw "$Label scheduler worker occupancy timestamp_us must be a JSON number."
        }
        $timestamp = [double]$rawTimestamp
        if ([double]::IsNaN($timestamp) -or [double]::IsInfinity($timestamp) -or
            $timestamp -lt 0 -or $timestamp -ne [Math]::Truncate($timestamp) -or
            $timestamp -gt [Int64]::MaxValue) {
            throw "$Label scheduler worker occupancy timestamp_us must be a finite nonnegative integer."
        }

        $rawValue = Get-RenderExtractReportProperty `
            -Value $counter `
            -Name 'value' `
            -Label "$Label scheduler worker occupancy sample"
        if ($numericTypes -notcontains $rawValue.GetType()) {
            throw "$Label scheduler worker occupancy value must be a JSON number."
        }
        $value = [double]$rawValue
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value) -or
            ($value -ne 0 -and $value -ne 1)) {
            throw "$Label scheduler worker occupancy value must be either 0 or 1."
        }

        $samples.Add([pscustomobject][ordered]@{
                timestamp_us = [Int64]$timestamp
                sequence = $index
                value = [int]$value
            }) | Out-Null
    }

    $orderedSamples = @($samples | Sort-Object timestamp_us, sequence)
    if ($orderedSamples[0].value -ne 0 -or $orderedSamples[$orderedSamples.Count - 1].value -ne 0) {
        throw "$Label scheduler worker occupancy must contain a complete idle-busy-idle sequence."
    }
    if (@($orderedSamples | Where-Object { $_.value -eq 1 }).Count -eq 0) {
        return [pscustomobject][ordered]@{
            status = 'not_emitted'
            reason = 'The scheduler worker occupancy counter did not observe a busy submission.'
            sample_count = $orderedSamples.Count
            observed_window_us = $null
            busy_duration_us = $null
            occupancy_ratio = $null
        }
    }

    [Int64]$observedWindowUs = $orderedSamples[$orderedSamples.Count - 1].timestamp_us - $orderedSamples[0].timestamp_us
    if ($observedWindowUs -le 0) {
        throw "$Label scheduler worker occupancy must span a nonzero observation window."
    }
    [Int64]$busyDurationUs = 0
    for ($index = 0; $index -lt ($orderedSamples.Count - 1); $index++) {
        $current = $orderedSamples[$index]
        $next = $orderedSamples[$index + 1]
        if ($current.value -eq 1) {
            $busyDurationUs += $next.timestamp_us - $current.timestamp_us
        }
    }

    return [pscustomobject][ordered]@{
        status = 'measured'
        reason = $null
        sample_count = $orderedSamples.Count
        observed_window_us = $observedWindowUs
        busy_duration_us = $busyDurationUs
        occupancy_ratio = [double]$busyDurationUs / [double]$observedWindowUs
    }
}

Export-ModuleMember -Function @(
    'Get-RenderExtractPercentile',
    'Get-RenderExtractStatistics',
    'Get-RenderExtractTimelineInteger',
    'Get-RenderExtractSteadyMeasurementWindow',
    'Select-RenderExtractTimelineWindowSamples',
    'ConvertTo-RenderExtractSpanAggregate',
    'ConvertTo-RenderExtractCounterAggregate',
    'Get-RenderExtractAggregates',
    'Get-RenderExtractInstrumentationCoverage',
    'Get-RenderExtractSchedulerWorkerOccupancyAttempt'
)
