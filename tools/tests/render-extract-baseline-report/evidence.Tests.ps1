. (Join-Path $PSScriptRoot 'support.ps1')

Describe 'Render-extract baseline evidence' {
    It 'parses JSON evidence from the same bytes it hashes' {
        $path = Join-Path $TestDrive 'single-read-evidence.json'
        $originalBytes = [Text.UTF8Encoding]::new($false).GetBytes('{"session_id":"original"}')
        [IO.File]::WriteAllBytes($path, $originalBytes)
        $hasher = [Security.Cryptography.SHA256]::Create()
        try {
            $expectedHash = ([BitConverter]::ToString($hasher.ComputeHash($originalBytes))).Replace('-', '')
        }
        finally {
            $hasher.Dispose()
        }

        $snapshot = Read-RenderExtractJsonEvidence -Path $path -Label 'single-read fixture'
        [IO.File]::WriteAllText($path, '{"session_id":"replacement"}', [Text.UTF8Encoding]::new($false))

        $snapshot.sha256 | Should Be $expectedHash
        $snapshot.json.session_id | Should Be 'original'
    }

    It 'hashes file evidence through one fixed-size uppercase buffer' {
        $path = Join-Path $TestDrive 'artifact-evidence.bin'
        [byte[]]$bytes = @(0, 15, 16, 127, 128, 240, 255)
        [IO.File]::WriteAllBytes($path, $bytes)
        $hasher = [Security.Cryptography.SHA256]::Create()
        try {
            $expectedHash = ([BitConverter]::ToString($hasher.ComputeHash($bytes))).Replace('-', '')
        }
        finally {
            $hasher.Dispose()
        }

        $evidence = Get-RenderExtractFileEvidence `
            -Path $path `
            -Kind 'artifact' `
            -LogicalId 'fixed-buffer-fixture' `
            -Attempt 1
        $evidenceSource = Get-Content -Raw $evidenceModule

        $evidence.sha256 | Should Be $expectedHash
        $evidenceSource | Should Match '\[char\[\]\]::new\(\$HashBytes.Length \* 2\)'
        $evidenceSource | Should Not Match 'Get-FileHash'
        $evidenceSource | Should Not Match "ToString\('X2'\)"
    }

    It 'uses the JSON snapshot reader for both summary and timeline evidence' {
        $reportSource = Get-Content -LiteralPath $reporter -Raw

        $reportSource | Should Match '\$summarySnapshot = Read-RenderExtractJsonEvidence'
        $reportSource | Should Match '\$timelineSnapshot = Read-RenderExtractJsonEvidence'
        $reportSource | Should Not Match '\[IO\.File\]::ReadAllText\(\$timelinePath\)'
    }

    It 'requires a finite nonnegative monotonic process duration' {
        { Get-RenderExtractProcessElapsedMilliseconds -Run ([pscustomobject]@{}) } |
            Should Throw 'process_elapsed_ms'
        { Get-RenderExtractProcessElapsedMilliseconds -Run ([pscustomobject]@{ process_elapsed_ms = -1 }) } |
            Should Throw 'finite nonnegative'
        { Get-RenderExtractProcessElapsedMilliseconds -Run ([pscustomobject]@{ process_elapsed_ms = [double]::NaN }) } |
            Should Throw 'finite nonnegative'
        foreach ($coercedZero in @($false, '', '   ')) {
            { Get-RenderExtractProcessElapsedMilliseconds -Run ([pscustomobject]@{ process_elapsed_ms = $coercedZero }) } |
                Should Throw 'JSON number'
        }
    }

    It 'accepts a canonical .NET integer product process identity' {
        $processId = Get-RenderExtractProcessId -Run ([pscustomobject]@{ process_id = [Int32]1001 })

        $processId | Should Be 1001
    }

    It 'rejects a baseline run without its product process identity' {
        $directory = Join-Path $TestDrive ("baseline-report-missing-pid-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            [void]$summary.runs[0].PSObject.Properties.Remove('process_id')
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'process_id'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects pre-warmup summaries before they can publish a steady percentile' {
        $directory = Join-Path $TestDrive ("baseline-report-schema-three-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.schema_version = 3
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw 'schema_version must be 5'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects fractional presented-frame window counts' {
        $directory = Join-Path $TestDrive ("baseline-report-fractional-window-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $steadyRun = @($summary.runs | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
            $steadyRun.warmup_presented_frame_count = 59.5
            [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 7), [Text.UTF8Encoding]::new($false))
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            { Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath | Out-Null } |
                Should Throw "scenario run 'warmup_presented_frame_count' must be an integer in 0..1000000"
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'preserves generated scale-project identity without absolute resource paths' {
        $directory = Join-Path $TestDrive ("baseline-report-scale-project-" + [guid]::NewGuid().ToString('N'))
        try {
            $summaryPath = New-RenderExtractBaselineFixture `
                -Directory $directory `
                -FrameDurationsUs @(1000, 2000, 3000) `
                -ProcessDurationsMs @(10, 20, 30)
            $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
            $summary.project.scale_project = [pscustomobject][ordered]@{
                primitive_count = 1000
                scene_virtual_path = 'res://scenes/main.scene.toml'
            }
            [IO.File]::WriteAllText(
                $summaryPath,
                ($summary | ConvertTo-Json -Depth 8),
                [Text.UTF8Encoding]::new($false)
            )
            Mock Assert-RenderExtractBaselineEvidenceDirectory {
                param($Path)
                Resolve-ZirconWindowsPath -Path $Path
            }

            $report = Write-RenderExtractBaselineReport -BaselineSummaryPath $summaryPath
            $markdown = [IO.File]::ReadAllText((Join-Path $directory 'render-extract-baseline-report.md'))

            $report.project.scale_project.primitive_count | Should Be 1000
            $report.project.scale_project.scene_virtual_path | Should Be 'res://scenes/main.scene.toml'
            $markdown | Should Match 'Primitive count: `1000`'
            $markdown | Should Match 'Scene virtual path: `res://scenes/main.scene.toml`'
            $markdown | Should Not Match '[A-Z]:\\.*main\.scene\.toml'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }

    It 'rejects a report summary outside the plan-owned perf evidence root' {
        $directory = Join-Path $TestDrive ("baseline-report-outside-root-" + [guid]::NewGuid().ToString('N'))
        try {
            [IO.Directory]::CreateDirectory($directory) | Out-Null
            $summaryPath = Join-Path $directory 'render-extract-baseline.json'
            [IO.File]::WriteAllText($summaryPath, '{}', [Text.UTF8Encoding]::new($false))
            $failure = $null

            try {
                & $assertEvidenceDirectoryContract -Path $directory | Out-Null
            }
            catch {
                $failure = $_
            }

            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message | Should Match 'approved.*storage roots'
        }
        finally {
            if ([IO.Directory]::Exists($directory)) {
                Remove-Item -LiteralPath $directory -Recurse -Force
            }
        }
    }
}

