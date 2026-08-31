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
}
