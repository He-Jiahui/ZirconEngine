. (Join-Path $PSScriptRoot 'support.ps1')

Describe 'Render-extract baseline metrics' {
    It 'aggregates source-bound raw timeline samples with runtime-compatible percentiles' {
        $directory = Join-Path $TestDrive ("baseline-report-test-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $rawSummary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $reportPath = Join-Path $directory 'render-extract-baseline-report.json'
            $persisted = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json

            $report.schema_version | Should Be 5
            $persisted.source_fingerprint | Should Be ('A' * 64)
            $persisted.build_set_id | Should Be ('3' * 64)
            $persisted.build_set_manifest_sha256 | Should Be ('4' * 64)
            $persisted.aggregation.percentile_method | Should Be 'upper-nearest-index: ceil((n-1) * percentile / 100), zero-based'
            $persisted.scenarios.Count | Should Be 4
            $scenario = @($persisted.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $scenario.attempt_count | Should Be 3
            $scenario.attempt_processes.Count | Should Be 3
            $scenario.attempt_processes[0].attempt | Should Be 1
            $scenario.attempt_processes[0].process_id | Should Be 1101
            $scenario.attempt_processes[2].attempt | Should Be 3
            $scenario.attempt_processes[2].process_id | Should Be 1103
            $scenario.product | Should Be 'runtime'
            $scenario.scenario_id | Should Be 'render-extract.runtime.pipelined-steady'
            $scenario.scenario_version | Should Be 1
            $scenario.scenario_binding_id | Should Match '^[0-9A-F]{64}$'
            $scenario.measurement_window | Should Be 'steady-presented-frames-after-warmup'
            $scenario.warmup_presented_frame_count | Should Be 60
            $scenario.measured_presented_frame_count | Should Be 300
            $scenario.target_presented_frame_count | Should Be 360
            $scenario.process_measurement_scope | Should Be 'full-process-lifetime-including-warmup'
            $editorScenario = @($persisted.scenarios | Where-Object { $_.logical_id -eq 'editor-first-frame' })[0]
            $editorScenario.product | Should Be 'editor'
            $editorScenario.measurement_window | Should Be 'cold-first-presented-frame'
            $persisted.profiling_inputs.runtime.executable_sha256 | Should Be ('C' * 64)
            $persisted.profiling_inputs.runtime.build_set_id | Should Be ('3' * 64)
            $persisted.profiling_inputs.runtime.build_set_manifest_sha256 | Should Be ('4' * 64)
            $persisted.profiling_inputs.editor.executable_sha256 | Should Be ('E' * 64)
            $persisted.profiling_inputs.runtime.asset_manifest_sha256 | Should Be ('1' * 64)
            $persisted.profiling_inputs.editor.asset_manifest_sha256 | Should Be ('2' * 64)
            $persisted.profiling_inputs.runtime.asset_file_count | Should Be 628
            $persisted.profiling_inputs.runtime.asset_bytes | Should Be 4465771
            (Get-RenderExtractProcessElapsedMilliseconds -Run $rawSummary.runs[1]) | Should Be 20
            $scenario.process_elapsed_ms.median | Should Be 20
            $scenario.process_elapsed_ms.p95 | Should Be 30
            $scenario.peak_working_set_bytes.median | Should Be 106954752
            $scenario.peak_working_set_bytes.p95 | Should Be 108003328
            $scenario.total_processor_time_ms.median | Should Be 7
            $scenario.frame_duration_us.median | Should Be 2000
            $scenario.frame_duration_us.p95 | Should Be 3000
            $scenario.frame_duration_us.p99 | Should Be 3000
            $scenario.frame_duration_us.sample_count | Should Be 900
            $scenario.budget_evaluation.status | Should Be 'within_budget'
            $scenario.budget_evaluation.observed | Should Be 3000
            $scenario.budget_evaluation.threshold | Should Be 16670
            $persisted.qualification.status | Should Be 'unqualified'
            (@($persisted.qualification.blocking_reasons) -contains 'product_receipt_not_bound') | Should Be $true
            (@($persisted.qualification.blocking_reasons) -contains 'device_profile_not_bound') | Should Be $true
            $persisted.machine_manifest.all_required_observed | Should Be $true
            $persisted.raw_evidence.machine_manifest.sha256 | Should Match '^[0-9A-F]{64}$'
            $scenario.lock_wait.status | Should Be 'measured'
            $scenario.queue_backpressure.status | Should Be 'measured'
            $scenario.queue_backpressure.spans.Count | Should Be 0
            $scenario.queue_backpressure.counters.Count | Should Be 1
            $scenario.worker_utilization.status | Should Be 'not_emitted'
            $scenario.worker_utilization.counters.Count | Should Be 0
            $persisted.measurement_coverage.worker_utilization.status | Should Be 'not_emitted'
            $scenario.app_cadence.status | Should Be 'measured'
            $scenario.app_cadence.counters.Count | Should Be 4
            $scenario.surface_presentation.status | Should Be 'measured'
            $scenario.surface_presentation.counters.Count | Should Be 4
            $scenario.asset_management.status | Should Be 'measured'
            $scenario.asset_management.counters.Count | Should Be 5
            $scenario.asset_management_page.status | Should Be 'measured'
            $scenario.asset_management_page.counters.Count | Should Be 6
            $persisted.measurement_coverage.app_cadence.status | Should Be 'measured'
            $persisted.measurement_coverage.surface_presentation.status | Should Be 'measured'
            $persisted.measurement_coverage.asset_management.status | Should Be 'measured'
            $persisted.measurement_coverage.asset_management_page.status | Should Be 'measured'
            $persisted.measurement_coverage.gpu_timing.status | Should Be 'not_measured'
            $persisted.measurement_coverage.system_power.status | Should Be 'not_measured'
            $persisted.measurement_coverage.working_set.status | Should Be 'measured'
            $persisted.measurement_coverage.disk_io.status | Should Be 'not_measured'
            $persisted.raw_evidence.summary_sha256 | Should Match '^[0-9A-F]{64}$'
            $persisted.raw_evidence.profile_artifacts.Count | Should Be 48
            $persisted.raw_evidence.frame_capture_artifacts.Count | Should Be 12
            [IO.File]::Exists((Join-Path $directory 'render-extract-baseline-report.md')) | Should Be $true
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'uses the app presented-frame window and excludes warmup and nested frame samples' {
        $frames = [System.Collections.Generic.List[object]]::new()
        for ($frameIndex = 0; $frameIndex -lt 360; $frameIndex++) {
            $frames.Add([pscustomobject]@{
                    stream = 'app'
                    name = 'runtime_redraw'
                    frame_index = $frameIndex
                    start_us = $frameIndex * 100
                    duration_us = 100
                }) | Out-Null
        }
        $frames.Add([pscustomobject]@{
                stream = 'runtime'
                name = 'present_viewport'
                frame_index = 60
                start_us = 6000
                duration_us = 100000
            }) | Out-Null
        $window = Get-RenderExtractSteadyMeasurementWindow `
            -Frames @($frames) `
            -WarmupPresentedFrameCount 60 `
            -MeasuredPresentedFrameCount 300 `
            -Label 'fixture steady window'
        $samples = Select-RenderExtractTimelineWindowSamples `
            -Frames @($frames) `
            -Spans @(
                [pscustomobject]@{ name = 'warmup'; start_us = 10; duration_us = 5 },
                [pscustomobject]@{ name = 'steady'; start_us = 6010; duration_us = 5 }
            ) `
            -Counters @(
                [pscustomobject]@{ name = 'warmup'; timestamp_us = 10 },
                [pscustomobject]@{ name = 'steady'; timestamp_us = 6010 }
            ) `
            -Window $window

        $window.primary_frames.Count | Should Be 300
        $window.start_us | Should Be 6000
        $window.end_us | Should Be 36000
        $samples.frames.Count | Should Be 300
        @($samples.frames | Where-Object { $_.stream -ne 'app' }).Count | Should Be 0
        @($samples.spans | ForEach-Object { $_.name }) | Should Be @('steady')
        @($samples.counters | ForEach-Object { $_.name }) | Should Be @('steady')
    }

    It 'accepts canonical .NET integers for scheduler worker occupancy samples' {
        $counters = @(
            [pscustomobject]@{
                stream = 'runtime'
                name = 'render_framework.scheduler.worker_utilization'
                timestamp_us = [Int64]100
                value = [Int32]0
            },
            [pscustomobject]@{
                stream = 'runtime'
                name = 'render_framework.scheduler.worker_utilization'
                timestamp_us = [Int64]110
                value = [Int32]1
            },
            [pscustomobject]@{
                stream = 'runtime'
                name = 'render_framework.scheduler.worker_utilization'
                timestamp_us = [Int64]150
                value = [Int32]0
            }
        )

        $occupancy = Get-RenderExtractSchedulerWorkerOccupancyAttempt `
            -Counters $counters `
            -Label 'portable integer fixture'

        $occupancy.status | Should Be 'measured'
        $occupancy.occupancy_ratio | Should Be 0.8
    }

    It 'reports time-weighted scheduler worker occupancy only from complete idle-busy-idle samples' {
        $directory = Join-Path $TestDrive ("baseline-report-worker-occupancy-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            foreach ($run in $summary.runs | Where-Object { $_.logical_id -like 'pipelined-*' }) {
                Add-RenderSchedulerWorkerOccupancyCounters `
                    -TimelinePath (Join-Path $run.profile_directory 'timeline.zrtrace.json') `
                    -IdleAtUs 100 `
                    -BusyAtUs 110 `
                    -CompleteAtUs 150
            }
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $scenario = @($report.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $scenario.worker_utilization.status | Should Be 'measured'
            $scenario.worker_utilization.attempts.Count | Should Be 3
            $scenario.worker_utilization.occupancy_ratio.median | Should Be 0.8
            $scenario.worker_utilization.observed_window_us.median | Should Be 50
            $scenario.worker_utilization.busy_duration_us.median | Should Be 40
            $report.measurement_coverage.worker_utilization.source | Should Match 'occupancy'
            $markdown = [IO.File]::ReadAllText((Join-Path $directory 'render-extract-baseline-report.md'))
            $markdown | Should Match 'Worker submission occupancy: measured'
            $markdown | Should Not Match 'Worker utilization:'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects malformed scheduler worker occupancy samples instead of estimating a ratio' {
        $directory = Join-Path $TestDrive ("baseline-report-worker-occupancy-invalid-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $run = @($summary.runs | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $timelinePath = Join-Path $run.profile_directory 'timeline.zrtrace.json'
            $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
            $measurementStartUs = [Int64](@($timeline.frames | Where-Object {
                    $_.stream -eq 'app' -and $_.name -eq 'runtime_redraw'
                })[60].start_us)
            $timeline.counters += @(
                [pscustomobject][ordered]@{
                    stream = 'runtime'
                    name = 'render_framework.scheduler.worker_utilization'
                    value = 0
                    timestamp_us = $measurementStartUs + 100
                    frame_index = $null
                },
                [pscustomobject][ordered]@{
                    stream = 'runtime'
                    name = 'render_framework.scheduler.worker_utilization'
                    value = 1
                    timestamp_us = $measurementStartUs + 110
                    frame_index = $null
                }
            )
            [IO.File]::WriteAllText($timelinePath, ($timeline | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'complete idle-busy-idle'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'marks worker occupancy partial when a pipelined scenario omits an attempt' {
        $directory = Join-Path $TestDrive ("baseline-report-worker-occupancy-partial-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            foreach ($run in $summary.runs | Where-Object {
                    $_.logical_id -eq 'pipelined-steady' -and $_.attempt -lt 3
                }) {
                Add-RenderSchedulerWorkerOccupancyCounters `
                    -TimelinePath (Join-Path $run.profile_directory 'timeline.zrtrace.json') `
                    -IdleAtUs 100 `
                    -BusyAtUs 110 `
                    -CompleteAtUs 150
            }
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $scenario = @($report.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $scenario.worker_utilization.status | Should Be 'partial'
            $report.measurement_coverage.worker_utilization.status | Should Be 'partial'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'keeps different named spans on one path in separate aggregates' {
        $directory = Join-Path $TestDrive ("baseline-report-span-name-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            foreach ($run in $summary.runs) {
                $timelinePath = Join-Path $run.profile_directory 'timeline.zrtrace.json'
                $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
                $primaryFrames = @($timeline.frames | Where-Object {
                        $_.stream -eq 'app' -and $_.name -eq 'runtime_redraw'
                    })
                $sampleFrameIndex = [Math]::Min(60, $primaryFrames.Count - 1)
                $sampleStartUs = [Int64]$primaryFrames[$sampleFrameIndex].start_us
                $timeline.spans += [pscustomobject][ordered]@{
                    id = 99
                    parent_id = $null
                    frame_index = $sampleFrameIndex
                    stream = 'runtime'
                    category = 'render'
                    name = 'submit_finalize'
                    path = 'runtime/render:submit'
                    start_us = $sampleStartUs + 40
                    duration_us = 7
                    depth = 0
                }
                [IO.File]::WriteAllText(
                    $timelinePath,
                    ($timeline | ConvertTo-Json -Depth 8),
                    [Text.UTF8Encoding]::new($false)
                )
            }
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $scenario = @($report.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $same_path_names = @(
                $scenario.top_cpu_spans |
                    Where-Object { $_.path -eq 'runtime/render:submit' } |
                    ForEach-Object { $_.name }
            )

            ($same_path_names -contains 'submit') | Should Be $true
            ($same_path_names -contains 'submit_finalize') | Should Be $true
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'marks incomplete asset-management scan counters as partial coverage' {
        $directory = Join-Path $TestDrive ("baseline-report-partial-asset-counters-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $removedCounterNames = @(
                'resource_management.scan.matching_rows',
                'resource_management.scan.rows_emitted',
                'resource_management.scan.shard_candidate_checks',
                'resource_management.scan.filtered_rows_skipped'
            )
            foreach ($run in $summary.runs) {
                $timelinePath = Join-Path $run.profile_directory 'timeline.zrtrace.json'
                $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
                $timeline.counters = @($timeline.counters | Where-Object { $_.name -notin $removedCounterNames })
                [IO.File]::WriteAllText(
                    $timelinePath,
                    ($timeline | ConvertTo-Json -Depth 8),
                    [Text.UTF8Encoding]::new($false)
                )
            }
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $scenario = @($report.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]

            $scenario.asset_management.status | Should Be 'partial'
            (@($scenario.asset_management.missing_counter_names) -join ',') | Should Be ($removedCounterNames -join ',')
            $report.measurement_coverage.asset_management.status | Should Be 'partial'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'marks incomplete asset-management page counters as partial coverage' {
        $directory = Join-Path $TestDrive ("baseline-report-partial-page-counters-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $removedCounterNames = @(
                'resource_management.page.matching_rows',
                'resource_management.page.candidate_rows',
                'resource_management.page.rows_returned',
                'resource_management.page.shard_candidate_checks',
                'resource_management.page.filtered_rows_skipped'
            )
            foreach ($run in $summary.runs) {
                $timelinePath = Join-Path $run.profile_directory 'timeline.zrtrace.json'
                $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
                $timeline.counters = @($timeline.counters | Where-Object { $_.name -notin $removedCounterNames })
                [IO.File]::WriteAllText(
                    $timelinePath,
                    ($timeline | ConvertTo-Json -Depth 8),
                    [Text.UTF8Encoding]::new($false)
                )
            }
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $scenario = @($report.scenarios | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]

            $scenario.asset_management.status | Should Be 'measured'
            $scenario.asset_management_page.status | Should Be 'partial'
            (@($scenario.asset_management_page.missing_counter_names) -join ',') | Should Be ($removedCounterNames -join ',')
            $report.measurement_coverage.asset_management_page.status | Should Be 'partial'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }
}

