$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$reporter = Join-Path $repoRoot 'tools\mvp\Write-RenderExtractBaselineReport.ps1'
$evidenceModule = Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineEvidence.psm1'
$metricsModule = Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineMetrics.psm1'
$scenarioModule = Join-Path $repoRoot 'tools\mvp\RenderExtractPerformanceScenario.psm1'
Import-Module (Join-Path $repoRoot 'tools\maintenance\WindowsPathResolver.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module $scenarioModule -Force -DisableNameChecking -ErrorAction Stop
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\performance-machine-manifest.ps1')
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
Import-Module (Join-Path $repoRoot 'tools\maintenance\WindowsPathResolver.psm1') -Force -DisableNameChecking -ErrorAction Stop

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
    $scenarioPlans = @(Get-RenderExtractBaselineRunPlan `
            -RepeatCount ([Math]::Max(3, $FrameDurationsUs.Count)) `
            -WarmupPresentedFrameCount 60 `
            -MeasuredPresentedFrameCount 300)
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
                    scenario_id = $scenario.scenario_id
                    scenario_version = $scenario.scenario_version
                    scenario_binding_id = $scenario.scenario_binding_id
                    product = $scenario.product
                    attempt = $attempt
                    invocation_id = $invocationId
                    runtime_profile = $scenario.runtime_profile
                    measurement_window = $scenario.measurement_window
                    repeat_count = $scenario.repeat_count
                    warmup_presented_frame_count = $scenario.warmup_presented_frame_count
                    measured_presented_frame_count = $scenario.measured_presented_frame_count
                    target_presented_frame_count = $scenario.target_presented_frame_count
                    cache_contract = $scenario.cache_contract
                    required_metrics = @($scenario.required_metrics)
                    budget_contract = $scenario.budget_contract
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
                        build_set_id = '3' * 64
                        build_set_manifest_sha256 = '4' * 64
                        executable_sha256 = if ($scenario.product -eq 'runtime') { 'C' * 64 } else { 'E' * 64 }
                        library_sha256 = if ($scenario.product -eq 'runtime') { 'D' * 64 } else { 'F' * 64 }
                        asset_manifest_sha256 = if ($scenario.product -eq 'runtime') { '1' * 64 } else { '2' * 64 }
                        asset_file_count = 628
                        asset_bytes = 4465771
                    }
                }) | Out-Null
        }
    }
    $machineObservations = [ordered]@{}
    foreach ($category in @(
            'cpu', 'gpu', 'memory', 'bios', 'os', 'display_modes', 'power_policy',
            'thermal_frequency', 'background_load', 'virtualization'
        )) {
        $machineObservations[$category] = [ordered]@{
            status = 'captured'
            data = @([ordered]@{ fixture = $category })
        }
    }
    $machineManifestPath = Join-Path $Directory 'machine-manifest.json'
    $machineManifest = New-ZirconPerformanceMachineManifest -Observations $machineObservations
    $machineManifest.captured_utc = '2026-08-26T00:00:00.0000000Z'
    [IO.File]::WriteAllText(
        $machineManifestPath,
        ($machineManifest | ConvertTo-Json -Depth 8),
        [Text.UTF8Encoding]::new($false)
    )
    $machineSnapshot = Read-RenderExtractJsonEvidence `
        -Path $machineManifestPath `
        -Label 'Fixture machine manifest'
    $summary = [ordered]@{
        schema_version = 5
        generated_at_utc = '2026-08-11T00:00:00.0000000+00:00'
        source_fingerprint = ('A' * 64)
        profiling_input_manifest_sha256 = ('B' * 64)
        build_set_id = ('3' * 64)
        build_set_manifest_sha256 = ('4' * 64)
        invocation_id = $invocationId
        machine_manifest = [ordered]@{
            path = $machineManifestPath
            sha256 = $machineSnapshot.sha256
        }
        project = [ordered]@{
            runtime_argument = '.'
            physical_identity = 'E:\fixture-project'
            scale_project = $null
        }
        runs = @($runs)
    }
    $summaryPath = Join-Path $Directory 'render-extract-baseline.json'
    [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 10), [Text.UTF8Encoding]::new($false))
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

function Add-MeshCommandPreparationSamples {
    param(
        [Parameter(Mandatory)][string]$TimelinePath,
        [string[]]$OmitSpanNames = @(),
        [string[]]$OmitCounterNames = @()
    )

    $timeline = Get-Content -LiteralPath $TimelinePath -Raw | ConvertFrom-Json
    $primaryFrames = @($timeline.frames | Where-Object {
            $_.stream -eq 'app' -and $_.name -eq 'runtime_redraw'
        })
    $sampleFrameIndex = [Math]::Min(60, $primaryFrames.Count - 1)
    $sampleStartUs = [Int64]$primaryFrames[$sampleFrameIndex].start_us
    $spanNames = @(
        'extract_cached_pre_mesh_draw',
        'pre_mesh_materialize',
        'prepare_cached_dispatch',
        'normalize_source_order',
        'parallel_admission',
        'owner_transaction',
        'worker_projection_wait',
        'ordered_merge',
        'seal_phase_buffers',
        'finalize_old_path',
        'indirect_plan',
        'replay_record'
    )
    for ($index = 0; $index -lt $spanNames.Count; $index++) {
        if ($OmitSpanNames -contains $spanNames[$index]) {
            continue
        }
        $timeline.spans += [pscustomobject][ordered]@{
            id = 100 + $index
            parent_id = $null
            frame_index = $sampleFrameIndex
            stream = 'render'
            category = 'mesh_commands'
            name = $spanNames[$index]
            path = "runtime/mesh_commands:$($spanNames[$index])"
            start_us = $sampleStartUs + 50 + $index
            duration_us = 1
            depth = 0
        }
    }

    $counterValues = [ordered]@{
        'mesh_commands.batch_count' = 1000
        'mesh_commands.worker_count' = 8
        'mesh_commands.parallel_enabled' = 1
        'mesh_commands.dispatch_reason_code' = 0
        'mesh_commands.cache_hit_count' = 1000
        'mesh_commands.cache_miss_count' = 0
        'mesh_commands.command_rebuild_count' = 0
        'mesh_commands.command_count' = 3000
        'mesh_commands.command_arena_grow_count' = 14
        'mesh_commands.command_arena_peak_capacity' = 4096
        'mesh_commands.phase_bucket_grow_count' = 31
        'mesh_commands.phase_bucket_capacity' = 6144
        'mesh_commands.command_build_count' = 3000
        'mesh_commands.partition_move_count' = 3000
        'mesh_commands.merge_move_count' = 1500
        'mesh_commands.finalize_count' = 2
        'mesh_commands.sort_count' = 30
        'mesh_commands.partition_command_visit_count' = 3000
        'mesh_commands.merge_command_visit_count' = 1500
        'mesh_commands.sort_command_visit_count' = 7500
        'mesh_commands.active_bucket_count' = 6
        'mesh_commands.cache_hit_payload_arc_clone_count' = 1000
        'mesh_commands.command_generation_reuse_count' = 0
        'mesh_commands.view_generation_reuse_count' = 0
        'mesh_commands.bucket.depth_prepass_length' = 500
        'mesh_commands.bucket.shadow_length' = 500
        'mesh_commands.bucket.opaque_length' = 1000
        'mesh_commands.bucket.alpha_mask_length' = 250
        'mesh_commands.bucket.advanced_pbr_opaque_length' = 0
        'mesh_commands.bucket.transmission_length' = 0
        'mesh_commands.bucket.transparent_length' = 500
        'mesh_commands.bucket.half_resolution_transparent_length' = 0
        'mesh_commands.bucket.velocity_length' = 125
        'mesh_commands.bucket.taa_reactive_mask_length' = 125
    }
    $counterIndex = 0
    foreach ($entry in $counterValues.GetEnumerator()) {
        if ($OmitCounterNames -notcontains $entry.Key) {
            $timeline.counters += [pscustomobject][ordered]@{
                stream = 'render'
                name = $entry.Key
                value = $entry.Value
                timestamp_us = $sampleStartUs + 70 + $counterIndex
                frame_index = $sampleFrameIndex
            }
        }
        $counterIndex++
    }
    [IO.File]::WriteAllText($TimelinePath, ($timeline | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
}

