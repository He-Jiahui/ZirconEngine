$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$reporter = Join-Path $repoRoot 'tools\mvp\Write-RenderExtractBaselineReport.ps1'
$evidenceModule = Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineEvidence.psm1'
$metricsModule = Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineMetrics.psm1'
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -DisableNameChecking -ErrorAction Stop
$originalTestMode = $env:RENDER_EXTRACT_BASELINE_REPORT_TEST_MODE

try {
    $env:RENDER_EXTRACT_BASELINE_REPORT_TEST_MODE = '1'
    . $reporter
}
finally {
    $env:RENDER_EXTRACT_BASELINE_REPORT_TEST_MODE = $originalTestMode
}

Import-Module $evidenceModule -Force -DisableNameChecking -ErrorAction Stop
Import-Module $metricsModule -Force -DisableNameChecking -ErrorAction Stop

$assertEvidenceDirectoryContract = (Get-Command Assert-RenderExtractBaselineEvidenceDirectory -CommandType Function).ScriptBlock

function New-RenderExtractTimelineFixture {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$SessionId,
        [Parameter(Mandatory)][int]$FrameDurationUs,
        [Parameter(Mandatory)][int]$QueueDurationUs,
        [Parameter(Mandatory)][int]$QueueDepth,
        [Parameter(Mandatory)][int]$TotalPresentedFrameCount,
        [Parameter(Mandatory)][int]$WarmupPresentedFrameCount
    )

    $frames = [System.Collections.Generic.List[object]]::new()
    for ($frameIndex = 0; $frameIndex -lt $TotalPresentedFrameCount; $frameIndex++) {
        $frames.Add([ordered]@{
                stream = 'app'
                name = 'runtime_redraw'
                frame_index = $frameIndex
                start_us = $frameIndex * $FrameDurationUs
                duration_us = $FrameDurationUs
                budget_ms = 16.67
                over_budget = $false
            }) | Out-Null
    }
    $sampleFrameIndex = [Math]::Min($WarmupPresentedFrameCount, $TotalPresentedFrameCount - 1)
    $sampleStartUs = ($sampleFrameIndex * $FrameDurationUs)
    $timeline = [ordered]@{
        session_id = $SessionId
        output_root = 'E:\ZirconBuilds\mvp-perf'
        active = $true
        feature_enabled = $true
        frame_budget_ms = 16.67
        frames = @($frames)
        spans = @(
            [ordered]@{
                id = 1
                parent_id = $null
                frame_index = $sampleFrameIndex
                stream = 'runtime'
                category = 'render'
                name = 'submit'
                path = 'runtime/render:submit'
                start_us = $sampleStartUs
                duration_us = $FrameDurationUs
                depth = 0
            },
            [ordered]@{
                id = 2
                parent_id = $null
                frame_index = $sampleFrameIndex
                stream = 'runtime'
                category = 'scheduler'
                name = 'queue_wait'
                path = 'runtime/schedule:queue_wait'
                start_us = $sampleStartUs + 10
                duration_us = $QueueDurationUs
                depth = 0
            },
            [ordered]@{
                id = 3
                parent_id = $null
                frame_index = $sampleFrameIndex
                stream = 'runtime'
                category = 'render_framework.wait'
                name = 'operation_lock'
                path = 'runtime/render_framework.wait:operation_lock'
                start_us = $sampleStartUs + 20
                duration_us = 5
                depth = 0
            }
        )
        counters = @(
            [ordered]@{
                stream = 'runtime'
                name = 'render_framework.scheduler.pending_depth'
                value = $QueueDepth
                timestamp_us = $sampleStartUs + 30
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'runtime'
                name = 'scene.ecs.native_system.worker_utilization'
                value = 0.5
                timestamp_us = $sampleStartUs + 31
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.frame_pump'
                value = 1
                timestamp_us = $sampleStartUs + 32
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.frame_pump_suppressed'
                value = 1
                timestamp_us = $sampleStartUs + 33
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.runtime_tick'
                value = 1
                timestamp_us = $sampleStartUs + 34
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.redraw_request'
                value = 1
                timestamp_us = $sampleStartUs + 35
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.native_present'
                value = 1
                timestamp_us = $sampleStartUs + 36
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.presented_frame'
                value = 1
                timestamp_us = $sampleStartUs + 37
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.explicit_frame_capture_request'
                value = 1
                timestamp_us = $sampleStartUs + 38
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'app'
                name = 'runtime_entry.explicit_frame_capture_rgba_bytes'
                value = 8294400
                timestamp_us = $sampleStartUs + 39
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                stream = 'runtime'
                name = 'resource_management.scan.instances'
                value = 5
                timestamp_us = $sampleStartUs + 40
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.scan.matching_rows'
                 value = 128
                 timestamp_us = $sampleStartUs + 40
                 frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.scan.rows_emitted'
                 value = 128
                 timestamp_us = $sampleStartUs + 40
                 frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.scan.shard_candidate_checks'
                 value = 8192
                 timestamp_us = $sampleStartUs + 40
                 frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.scan.filtered_rows_skipped'
                 value = 96
                 timestamp_us = $sampleStartUs + 40
                 frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.instances'
                 value = 3
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.matching_rows'
                 value = 128
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.candidate_rows'
                 value = 96
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.rows_returned'
                 value = 50
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.shard_candidate_checks'
                 value = 6144
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            },
            [ordered]@{
                 stream = 'runtime'
                 name = 'resource_management.page.filtered_rows_skipped'
                 value = 96
                timestamp_us = $sampleStartUs + 41
                frame_index = $sampleFrameIndex
            }
        )
    }
    [IO.File]::WriteAllText($Path, ($timeline | ConvertTo-Json -Depth 6), [Text.UTF8Encoding]::new($false))
}

function New-RenderExtractBaselineFixture {
    param(
        [Parameter(Mandatory)][string]$Directory,
        [Parameter(Mandatory)][int[]]$FrameDurationsUs,
        [Parameter(Mandatory)][int[]]$ProcessDurationsMs
    )

    [IO.Directory]::CreateDirectory($Directory) | Out-Null
    $invocationId = 'A' * 32
    $profilesDirectory = Join-Path (Join-Path $Directory 'profiles') $invocationId
    $capturesDirectory = Join-Path (Join-Path $Directory 'captures') $invocationId
    [IO.Directory]::CreateDirectory($profilesDirectory) | Out-Null
    [IO.Directory]::CreateDirectory($capturesDirectory) | Out-Null
    $runs = [System.Collections.Generic.List[object]]::new()
    $scenarioPlans = @(
        [pscustomobject]@{ logical_id = 'pipelined-first-frame'; product = 'runtime'; runtime_profile = 'runtime-pipelined'; warmup_presented_frame_count = 0; measured_presented_frame_count = 1; target_presented_frame_count = 1 },
        [pscustomobject]@{ logical_id = 'pipelined-steady'; product = 'runtime'; runtime_profile = 'runtime-pipelined'; warmup_presented_frame_count = 60; measured_presented_frame_count = 300; target_presented_frame_count = 360 },
        [pscustomobject]@{ logical_id = 'synchronous-steady'; product = 'runtime'; runtime_profile = 'runtime'; warmup_presented_frame_count = 60; measured_presented_frame_count = 300; target_presented_frame_count = 360 },
        [pscustomobject]@{ logical_id = 'editor-first-frame'; product = 'editor'; runtime_profile = 'editor'; warmup_presented_frame_count = 0; measured_presented_frame_count = 1; target_presented_frame_count = 1 }
    )
    for ($scenarioIndex = 0; $scenarioIndex -lt $scenarioPlans.Count; $scenarioIndex++) {
        $scenario = $scenarioPlans[$scenarioIndex]
        for ($index = 0; $index -lt $FrameDurationsUs.Count; $index++) {
            $attempt = $index + 1
            $sessionId = "$($scenario.logical_id)-$attempt"
            $profileDirectory = Join-Path $profilesDirectory $sessionId
            [IO.Directory]::CreateDirectory($profileDirectory) | Out-Null
            New-RenderExtractTimelineFixture `
                -Path (Join-Path $profileDirectory 'timeline.zrtrace.json') `
                -SessionId $sessionId `
                -FrameDurationUs $FrameDurationsUs[$index] `
                -QueueDurationUs ($attempt * 10) `
                -QueueDepth $attempt `
                -TotalPresentedFrameCount $scenario.target_presented_frame_count `
                -WarmupPresentedFrameCount $scenario.warmup_presented_frame_count
            [IO.File]::WriteAllText((Join-Path $profileDirectory 'hotspots.json'), '{}', [Text.UTF8Encoding]::new($false))
            [IO.File]::WriteAllText((Join-Path $profileDirectory 'counter_hotspots.json'), '{}', [Text.UTF8Encoding]::new($false))
            [IO.File]::WriteAllText((Join-Path $profileDirectory 'summary.md'), '# fixture', [Text.UTF8Encoding]::new($false))
            $capturePath = Join-Path $capturesDirectory "$sessionId.png"
            [IO.File]::WriteAllBytes($capturePath, [byte[]](137, 80, 78, 71, $attempt))
            $startedAt = [DateTimeOffset]::Parse('2026-08-11T00:00:00.0000000+00:00').AddMilliseconds((($scenarioIndex * 10) + $index) * 100)
            $runs.Add([ordered]@{
                    logical_id = $scenario.logical_id
                    product = $scenario.product
                    attempt = $attempt
                    invocation_id = $invocationId
                    runtime_profile = $scenario.runtime_profile
                    warmup_presented_frame_count = $scenario.warmup_presented_frame_count
                    measured_presented_frame_count = $scenario.measured_presented_frame_count
                    target_presented_frame_count = $scenario.target_presented_frame_count
                    exit_code = 0
                    peak_working_set_bytes = 104857600 + ($attempt * 1048576)
                    total_processor_time_ms = 5 + $attempt
                    process_id = 1000 + ($scenarioIndex * 100) + $attempt
                    process_elapsed_ms = $ProcessDurationsMs[$index]
                    started_at_utc = $startedAt.ToString('o')
                    ended_at_utc = $startedAt.AddMilliseconds(999).ToString('o')
                    stdout = (Join-Path $Directory "$sessionId.stdout.log")
                    stderr = (Join-Path $Directory "$sessionId.stderr.log")
                    profile_directory = $profileDirectory
                    frame_capture_png = $capturePath
                    system_trace_etl = $null
                    profiling_input = [ordered]@{
                        manifest_sha256 = 'B' * 64
                        executable_sha256 = if ($scenario.product -eq 'runtime') { 'C' * 64 } else { 'E' * 64 }
                        library_sha256 = if ($scenario.product -eq 'runtime') { 'D' * 64 } else { 'F' * 64 }
                        asset_manifest_sha256 = if ($scenario.product -eq 'runtime') { '1' * 64 } else { '2' * 64 }
                        asset_file_count = 628
                        asset_bytes = 4465771
                    }
                }) | Out-Null
        }
    }
    $summary = [ordered]@{
        schema_version = 4
        generated_at_utc = '2026-08-11T00:00:00.0000000+00:00'
        source_fingerprint = ('A' * 64)
        profiling_input_manifest_sha256 = ('B' * 64)
        invocation_id = $invocationId
        project = [ordered]@{
            runtime_argument = '.'
            physical_identity = 'E:\fixture-project'
            scale_project = $null
        }
        runs = @($runs)
    }
    $summaryPath = Join-Path $Directory 'render-extract-baseline.json'
    [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
    return $summaryPath
}

function Add-RenderSchedulerWorkerOccupancyCounters {
    param(
        [Parameter(Mandatory)][string]$TimelinePath,
        [Parameter(Mandatory)][int]$IdleAtUs,
        [Parameter(Mandatory)][int]$BusyAtUs,
        [Parameter(Mandatory)][int]$CompleteAtUs
    )

    $timeline = Get-Content -LiteralPath $TimelinePath -Raw | ConvertFrom-Json
    $primaryFrames = @($timeline.frames | Where-Object {
            $_.stream -eq 'app' -and $_.name -eq 'runtime_redraw'
        })
    $measurementStartUs = [Int64]$primaryFrames[[Math]::Min(60, $primaryFrames.Count - 1)].start_us
    $timeline.counters += @(
        [pscustomobject][ordered]@{
            stream = 'runtime'
            name = 'render_framework.scheduler.worker_utilization'
            value = 0
            timestamp_us = $measurementStartUs + $IdleAtUs
            frame_index = $null
        },
        [pscustomobject][ordered]@{
            stream = 'runtime'
            name = 'render_framework.scheduler.worker_utilization'
            value = 1
            timestamp_us = $measurementStartUs + $BusyAtUs
            frame_index = $null
        },
        [pscustomobject][ordered]@{
            stream = 'runtime'
            name = 'render_framework.scheduler.worker_utilization'
            value = 0
            timestamp_us = $measurementStartUs + $CompleteAtUs
            frame_index = $null
        }
    )
    [IO.File]::WriteAllText($TimelinePath, ($timeline | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
}

