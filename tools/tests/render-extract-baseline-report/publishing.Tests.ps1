. (Join-Path $PSScriptRoot 'support.ps1')

Describe 'Render-extract baseline publication' {
    It 'requires three source-bound attempts before publishing a scenario percentile' {
        $directory = Join-Path $TestDrive ("baseline-report-incomplete-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000) `
                -ProcessDurationsMs @(10, 20)
            $failure = $null
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            try {
                Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null
            }
            catch {
                $failure = $_
            }

            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message | Should Match 'at least 3'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'requires a nonempty invocation-scoped PNG for every product run' {
        $directory = Join-Path $TestDrive ("baseline-report-missing-png-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            Remove-Item -LiteralPath $summary.runs[0].frame_capture_png -Force
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'frame_capture_png'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects an unexpected runtime profile for a required baseline scenario' {
        $directory = Join-Path $TestDrive ("baseline-report-profile-mismatch-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs[0].runtime_profile = 'runtime'
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'runtime_profile'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'requires every planned baseline scenario before publishing a report' {
        $directory = Join-Path $TestDrive ("baseline-report-missing-scenario-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs = @($summary.runs | Where-Object { $_.logical_id -eq 'pipelined-steady' })
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'required scenario'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a run whose executable input hash differs from the capture summary' {
        $directory = Join-Path $TestDrive ("baseline-report-input-drift-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs[0].profiling_input.executable_sha256 = 'E' * 64
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'profiling input'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a run whose BuildSet identity differs from the capture summary' {
        $directory = Join-Path $TestDrive ("baseline-report-build-set-drift-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs[0].profiling_input.build_set_id = '9' * 64
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 10), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'BuildSet identity'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a run whose frozen asset input differs from its product capture session' {
        $directory = Join-Path $TestDrive ("baseline-report-asset-input-drift-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs[1].profiling_input.asset_manifest_sha256 = '9' * 64
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'profiling input'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'refuses to overwrite an existing report artifact' {
        $directory = Join-Path $TestDrive ("baseline-report-existing-output-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $reportPath = Join-Path $directory 'render-extract-baseline-report.json'
            [IO.File]::WriteAllText($reportPath, 'foreign-report', [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }
            $failure = $null

            try {
                Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null
            }
            catch {
                $failure = $_
            }

            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message | Should Match 'Refusing to overwrite existing render-extract report'
            [IO.File]::ReadAllText($reportPath) | Should Be 'foreign-report'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a timeline whose session identity does not match the summary run' {
        $directory = Join-Path $TestDrive ("baseline-report-session-mismatch-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $timelinePath = Join-Path $summary.runs[0].profile_directory 'timeline.zrtrace.json'
            $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
            $timeline.session_id = 'unrelated-session'
            [IO.File]::WriteAllText($timelinePath, ($timeline | ConvertTo-Json -Depth 6), [Text.UTF8Encoding]::new($false))
            $failure = $null
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            try {
                Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null
            }
            catch {
                $failure = $_
            }

            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message | Should Match 'does not match baseline run'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a summary that mixes runs from separate capture invocations' {
        $directory = Join-Path $TestDrive ("baseline-report-mixed-invocation-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.runs[1].invocation_id = ('B' * 32)
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }
            $failure = $null

            try {
                Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null
            }
            catch {
                $failure = $_
            }

            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message | Should Match 'does not match summary invocation_id'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'records an optional session-scoped WPR trace without claiming its CPU metrics were parsed' {
        $directory = Join-Path $TestDrive ("baseline-report-wpr-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $tracesDirectory = Join-Path (Join-Path $directory 'traces') $summary.runs[0].invocation_id
            [IO.Directory]::CreateDirectory($tracesDirectory) | Out-Null
            foreach ($run in $summary.runs) {
                $tracePath = Join-Path $tracesDirectory ("$($run.logical_id)-$($run.attempt).etl")
                [IO.File]::WriteAllBytes($tracePath, [byte[]](1, 2, 3, $run.attempt))
                $run.system_trace_etl = $tracePath
            }
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath

            $report.raw_evidence.system_trace_artifacts.Count | Should Be 12
            $report.raw_evidence.system_trace_artifacts[0].kind | Should Be 'system_trace_etl'
            $report.raw_evidence.system_trace_artifacts[0].process_id | Should Be 1001
            $report.raw_evidence.system_trace_artifacts[0].sha256 | Should Match '^[0-9A-F]{64}$'
            $report.measurement_coverage.cpu_scheduling.status | Should Be 'not_measured'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'publishes only receipt-bound product Heap allocation stacks as measured evidence' {
        $directory = Join-Path $TestDrive ("baseline-report-heap-wpr-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $tracesDirectory = Join-Path (Join-Path $directory 'traces') $summary.runs[0].invocation_id
            [IO.Directory]::CreateDirectory($tracesDirectory) | Out-Null
            foreach ($run in $summary.runs) {
                $sessionId = "$($run.logical_id)-$($run.attempt)"
                $tracePath = Join-Path $tracesDirectory "$sessionId.heap.etl"
                $analysisPath = Join-Path $tracesDirectory "$sessionId.heap-product-allocation-stacks.txt"
                $receiptPath = Join-Path $tracesDirectory "$sessionId.heap-wpr-capture.json"
                [IO.File]::WriteAllBytes($tracePath, [byte[]](1, 2, 3, $run.attempt))
                "zircon_runtime.exe!mesh_command::prepare $($run.attempt)" |
                    Set-Content -LiteralPath $analysisPath -Encoding UTF8
                $traceHash = (Get-FileHash -LiteralPath $tracePath -Algorithm SHA256).Hash
                $analysisHash = (Get-FileHash -LiteralPath $analysisPath -Algorithm SHA256).Hash
                $traceStarted = ([DateTimeOffset]$run.started_at_utc).ToUniversalTime()
                $processStarted = $traceStarted.AddMilliseconds(10)
                $processEnded = $traceStarted.AddMilliseconds(20)
                $receipt = [ordered]@{
                    schema_version = 1
                    evidence_kind = 'windows_product_heap_allocation_stacks'
                    profile = 'Heap'
                    started_at_utc = $traceStarted.ToString('o')
                    trace = [ordered]@{ path = $tracePath; bytes = [IO.FileInfo]::new($tracePath).Length; sha256 = $traceHash }
                    analysis = [ordered]@{ path = $analysisPath; bytes = [IO.FileInfo]::new($analysisPath).Length; sha256 = $analysisHash }
                    attribution = [ordered]@{
                        scope = 'product_process'
                        process_id = [Int64]$run.process_id
                        process_started_at_utc = $processStarted.ToString('o')
                        process_ended_at_utc = $processEnded.ToString('o')
                        process_lifetime_range_applied = $true
                        trace_range = [ordered]@{
                            format = 'utc_wall_clock'
                            start_at_utc = $processStarted.ToString('o')
                            end_at_utc = $processEnded.ToString('o')
                            xperf_start = $processStarted.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", [Globalization.CultureInfo]::InvariantCulture)
                            xperf_end = $processEnded.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", [Globalization.CultureInfo]::InvariantCulture)
                        }
                    }
                }
                [IO.File]::WriteAllText(
                    $receiptPath,
                    ($receipt | ConvertTo-Json -Depth 6),
                    [Text.UTF8Encoding]::new($false)
                )
                $run | Add-Member -NotePropertyName system_trace_profile -NotePropertyValue 'heap'
                $run.system_trace_etl = $tracePath
                $run | Add-Member -NotePropertyName system_trace_analysis -NotePropertyValue $analysisPath
                $run | Add-Member -NotePropertyName system_trace_receipt -NotePropertyValue $receiptPath
            }
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath

            $report.raw_evidence.system_trace_artifacts.Count | Should Be 36
            @($report.raw_evidence.system_trace_artifacts |
                    Where-Object { $_.kind -eq 'system_trace_heap_analysis' }).Count | Should Be 12
            $report.measurement_coverage.heap_allocations.status | Should Be 'measured'
            $report.measurement_coverage.cpu_sampling.status | Should Be 'not_measured'
            $report.measurement_coverage.cpu_timeline.status | Should Be 'instrumented_not_baseline'
            $report.raw_evidence.system_trace_artifacts[1].process_id | Should Be 1001
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }
}
