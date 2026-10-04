param(
    [string]$ProfileDir = '',
    [string]$OutputPath = '',
    [ValidateRange(1, 3600)][int]$MinimumWindowSeconds = 60,
    [ValidateRange(1, 10000)][int]$ExpectedWidth = 1920,
    [ValidateRange(1, 10000)][int]$ExpectedHeight = 1080,
    [ValidateRange(1, 1000)][int]$ExpectedRefreshRateHz = 60,
    [ValidateRange(1, 1000000)][double]$P95BudgetUs = 16700,
    [ValidateRange(1, 1000000)][double]$P99BudgetUs = 33300
)

function Get-ZirconMvpCadencePercentile {
    param([double[]]$Values, [double]$Quantile)
    if ($Values.Count -eq 0) { return $null }
    $ordered = @($Values | Sort-Object)
    $index = [Math]::Max(0, [int][Math]::Ceiling($ordered.Count * $Quantile) - 1)
    return [double]$ordered[$index]
}

function Test-ZirconMvpCadenceRetention {
    param([object]$Timeline)
    $retention = @($Timeline.recorder_retention)
    if ($retention.Count -eq 0) { return $false }
    $totals = @{ frames = 0L; spans = 0L; counters = 0L }
    foreach ($recorder in $retention) {
        foreach ($name in @('frames', 'spans', 'counters')) {
            $stream = $recorder.$name
            if ($null -eq $stream) { return $false }
            foreach ($field in @('capacity', 'written', 'overwritten', 'retained', 'oldest_sequence', 'newest_sequence')) {
                if ($null -eq $stream.PSObject.Properties[$field]) { return $false }
            }
            $capacity = [long]$stream.capacity
            $written = [long]$stream.written
            $overwritten = [long]$stream.overwritten
            $retained = [long]$stream.retained
            if ($capacity -le 0 -or $written -lt 0 -or $overwritten -ne 0 -or
                $retained -ne $written -or $retained -gt $capacity) { return $false }
            if ($retained -eq 0) {
                if ($null -ne $stream.oldest_sequence -or $null -ne $stream.newest_sequence) { return $false }
            }
            elseif ($null -eq $stream.oldest_sequence -or $null -eq $stream.newest_sequence -or
                [long]$stream.oldest_sequence -ne 0 -or [long]$stream.newest_sequence -ne ($written - 1)) {
                return $false
            }
            $totals[$name] += $retained
        }
    }
    foreach ($name in @('frames', 'spans', 'counters')) {
        if ($null -eq $Timeline.PSObject.Properties[$name] -or
            $totals[$name] -ne @($Timeline.$name).Count) { return $false }
    }
    return $true
}

function Get-ZirconMvpCadencePngSize {
    param([string]$Path)
    if (-not [System.IO.File]::Exists($Path)) { return $null }
    $bytes = New-Object byte[] 24
    $reader = [System.IO.File]::OpenRead($Path)
    try {
        if ($reader.Read($bytes, 0, 24) -ne 24) { return $null }
    }
    finally { $reader.Dispose() }
    if (-not (@(137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82) -join ',').Equals(($bytes[0..15] -join ','))) {
        return $null
    }
    $width = [long]$bytes[16] * 16777216 + [long]$bytes[17] * 65536 + [long]$bytes[18] * 256 + $bytes[19]
    $height = [long]$bytes[20] * 16777216 + [long]$bytes[21] * 65536 + [long]$bytes[22] * 256 + $bytes[23]
    if ($width -le 0 -or $height -le 0) { return $null }
    return [pscustomobject]@{ width = $width; height = $height }
}

function Get-ZirconMvpCadenceEvidence {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileDir,
        [ValidateRange(1, 3600)][int]$MinimumWindowSeconds = 60,
        [ValidateRange(1, 10000)][int]$ExpectedWidth = 1920,
        [ValidateRange(1, 10000)][int]$ExpectedHeight = 1080,
        [ValidateRange(1, 1000)][int]$ExpectedRefreshRateHz = 60,
        [ValidateRange(1, 1000000)][double]$P95BudgetUs = 16700,
        [ValidateRange(1, 1000000)][double]$P99BudgetUs = 33300
    )

    $report = [ordered]@{
        schema_version = 1
        profile_dir = [System.IO.Path]::GetFullPath($ProfileDir)
        submit_cadence = [ordered]@{
            metric = 'successful_surface_submit_interval'
            source = 'timeline.zrtrace.json:editor/ui.surface.submitted_count'
            status = 'unavailable'
            interval_count = 0
            window_seconds = 0.0
            p95_us = $null
            p99_us = $null
            observed_submit_rate_hz = $null
            minimum_window_seconds = $MinimumWindowSeconds
            p95_budget_us = $P95BudgetUs
            p99_budget_us = $P99BudgetUs
            minimum_submit_rate_hz = 59.9
        }
        editor_tick_cpu = [ordered]@{
            metric = 'retained_host_tick_cpu_duration'
            source = 'timeline.zrtrace.json:editor/retained_host_tick'
            status = 'unavailable'
            sample_count = 0
            p95_us = $null
            p99_us = $null
        }
        window_client = [ordered]@{
            source = 'screenshot_gpu.png:IHDR'
            observation = 'single_screenshot_only'
            status = 'unavailable'
            width = $null
            height = $null
            expected_width = $ExpectedWidth
            expected_height = $ExpectedHeight
        }
        display = [ordered]@{
            source = 'machine_manifest.json:display_modes'
            status = 'unavailable'
            refresh_rate_hz = $null
            expected_refresh_rate_hz = $ExpectedRefreshRateHz
        }
        adapter = [ordered]@{
            installed_gpu_names = @()
            actual_wgpu_adapter_status = 'unavailable'
            actual_wgpu_adapter_name = $null
        }
        scene = [ordered]@{
            source = 'source_manifest.json:input_fixture'
            status = 'unavailable'
            declared_selectable_node_count = $null
            runtime_visible_node_count_status = 'unavailable'
        }
        display_scanout_status = 'unavailable'
        embedded_play_status = 'unavailable'
        product_acceptance = 'pending'
        limitations = @(
            'Successful application surface-submit intervals are not display scanout or input-to-present latency.',
            'Editor tick CPU durations are not full display frame time or presentation latency.',
            'The GPU screenshot proves client size at one instant, not throughout the measurement window.',
            'Installed GPU names do not identify the WGPU adapter chosen by this editor process.'
        )
    }

    $timelinePath = Join-Path $ProfileDir 'timeline.zrtrace.json'
    if (Test-Path -LiteralPath $timelinePath -PathType Leaf) {
        $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
        $retentionComplete = Test-ZirconMvpCadenceRetention -Timeline $timeline
        $tickFrames = @($timeline.frames | Where-Object {
                $_.stream -eq 'editor' -and $_.name -eq 'retained_host_tick'
            })
        if (-not $retentionComplete) {
            $report.editor_tick_cpu.status = 'invalid'
        }
        elseif ($tickFrames.Count -gt 0) {
            $tickDurations = New-Object 'System.Collections.Generic.List[double]'
            $validTicks = $true
            foreach ($frame in $tickFrames) {
                try { $duration = [double]$frame.duration_us }
                catch { $validTicks = $false; break }
                if ([double]::IsNaN($duration) -or [double]::IsInfinity($duration) -or $duration -lt 0) {
                    $validTicks = $false
                    break
                }
                $tickDurations.Add($duration)
            }
            if ($validTicks) {
                $report.editor_tick_cpu.sample_count = $tickDurations.Count
                $report.editor_tick_cpu.p95_us = Get-ZirconMvpCadencePercentile -Values $tickDurations.ToArray() -Quantile 0.95
                $report.editor_tick_cpu.p99_us = Get-ZirconMvpCadencePercentile -Values $tickDurations.ToArray() -Quantile 0.99
                $report.editor_tick_cpu.status = 'reported'
            }
            else { $report.editor_tick_cpu.status = 'invalid' }
        }
        $samples = @($timeline.counters | Where-Object {
                $_.stream -eq 'editor' -and $_.name -eq 'ui.surface.submitted_count'
            })
        if (-not $retentionComplete) {
            $report.submit_cadence.status = 'invalid'
        }
        elseif ($samples.Count -lt 2) {
            $report.submit_cadence.status = 'insufficient'
        }
        else {
            $intervals = New-Object 'System.Collections.Generic.List[double]'
            $valid = $true
            $previous = $null
            foreach ($sample in $samples) {
                try {
                    $timestamp = [long]$sample.timestamp_us
                    $value = [double]$sample.value
                }
                catch { $valid = $false; break }
                if ($value -ne 1 -or $timestamp -lt 0) { $valid = $false; break }
                if ($null -ne $previous) {
                    if ($timestamp -le $previous) { $valid = $false; break }
                    $intervals.Add([double]($timestamp - $previous))
                }
                else { $first = $timestamp }
                $previous = $timestamp
            }
            if (-not $valid) {
                $report.submit_cadence.status = 'invalid'
            }
            else {
                $windowSeconds = [double]($previous - $first) / 1000000.0
                $report.submit_cadence.interval_count = $intervals.Count
                $report.submit_cadence.window_seconds = $windowSeconds
                $report.submit_cadence.p95_us = Get-ZirconMvpCadencePercentile -Values $intervals.ToArray() -Quantile 0.95
                $report.submit_cadence.p99_us = Get-ZirconMvpCadencePercentile -Values $intervals.ToArray() -Quantile 0.99
                if ($windowSeconds -gt 0) {
                    $report.submit_cadence.observed_submit_rate_hz = $intervals.Count / $windowSeconds
                }
                if ($windowSeconds -lt $MinimumWindowSeconds) {
                    $report.submit_cadence.status = 'insufficient'
                }
                elseif ($report.submit_cadence.observed_submit_rate_hz -lt $report.submit_cadence.minimum_submit_rate_hz -or
                    $report.submit_cadence.p95_us -gt $P95BudgetUs -or
                    $report.submit_cadence.p99_us -gt $P99BudgetUs) {
                    $report.submit_cadence.status = 'fail'
                }
                else { $report.submit_cadence.status = 'pass' }
            }
        }
    }

    $size = Get-ZirconMvpCadencePngSize -Path (Join-Path $ProfileDir 'screenshot_gpu.png')
    if ($null -ne $size) {
        $report.window_client.width = $size.width
        $report.window_client.height = $size.height
        $report.window_client.status = if ($size.width -eq $ExpectedWidth -and $size.height -eq $ExpectedHeight) { 'pass' } else { 'mismatch' }
    }

    $machinePath = Join-Path $ProfileDir 'machine_manifest.json'
    if (Test-Path -LiteralPath $machinePath -PathType Leaf) {
        $machine = Get-Content -LiteralPath $machinePath -Raw | ConvertFrom-Json
        if ($machine.gpu.status -eq 'captured') {
            $report.adapter.installed_gpu_names = @($machine.gpu.data | ForEach-Object { [string]$_.name })
        }
        if ($machine.display_modes.status -eq 'captured') {
            $activeModes = @($machine.display_modes.data | Where-Object {
                    [int]$_.width -gt 0 -and [int]$_.height -gt 0 -and [int]$_.refresh_rate_hz -gt 0
                })
            if ($activeModes.Count -eq 1) {
                $report.display.refresh_rate_hz = [int]$activeModes[0].refresh_rate_hz
                $report.display.status = if ($report.display.refresh_rate_hz -eq $ExpectedRefreshRateHz) { 'pass' } else { 'mismatch' }
            }
            elseif ($activeModes.Count -gt 1) { $report.display.status = 'ambiguous' }
        }
    }

    $sourcePath = Join-Path $ProfileDir 'source_manifest.json'
    if (Test-Path -LiteralPath $sourcePath -PathType Leaf) {
        $source = Get-Content -LiteralPath $sourcePath -Raw | ConvertFrom-Json
        if ($source.scenario -eq 'viewport_pointer' -and $source.input_fixture.kind -eq 'viewport_pointer_scene' -and
            $source.capture.options.run_phase -eq 'measured') {
            $report.scene.declared_selectable_node_count = [int]$source.input_fixture.selectable_node_count
            $report.scene.status = if ($report.scene.declared_selectable_node_count -ge 1000) { 'declared' } else { 'insufficient' }
        }
    }

    if ($report.submit_cadence.status -eq 'fail' -or
        $report.window_client.status -eq 'mismatch' -or
        $report.display.status -eq 'mismatch' -or
        $report.display.status -eq 'ambiguous' -or
        $report.scene.status -eq 'insufficient') {
        $report.product_acceptance = 'fail'
    }

    return [pscustomobject]$report
}

function Export-ZirconMvpCadenceEvidence {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileDir,
        [string]$OutputPath = '',
        [int]$MinimumWindowSeconds = 60,
        [int]$ExpectedWidth = 1920,
        [int]$ExpectedHeight = 1080,
        [int]$ExpectedRefreshRateHz = 60,
        [double]$P95BudgetUs = 16700,
        [double]$P99BudgetUs = 33300
    )
    $report = Get-ZirconMvpCadenceEvidence -ProfileDir $ProfileDir `
        -MinimumWindowSeconds $MinimumWindowSeconds -ExpectedWidth $ExpectedWidth `
        -ExpectedHeight $ExpectedHeight -ExpectedRefreshRateHz $ExpectedRefreshRateHz `
        -P95BudgetUs $P95BudgetUs -P99BudgetUs $P99BudgetUs
    if ([string]::IsNullOrWhiteSpace($OutputPath)) {
        $OutputPath = Join-Path $ProfileDir 'mvp_cadence_evidence.json'
    }
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
    return $report
}

if ($MyInvocation.InvocationName -ne '.') {
    if ([string]::IsNullOrWhiteSpace($ProfileDir)) { throw 'ProfileDir is required.' }
    Export-ZirconMvpCadenceEvidence -ProfileDir $ProfileDir -OutputPath $OutputPath `
        -MinimumWindowSeconds $MinimumWindowSeconds -ExpectedWidth $ExpectedWidth `
        -ExpectedHeight $ExpectedHeight -ExpectedRefreshRateHz $ExpectedRefreshRateHz `
        -P95BudgetUs $P95BudgetUs -P99BudgetUs $P99BudgetUs | ConvertTo-Json -Depth 8
}
