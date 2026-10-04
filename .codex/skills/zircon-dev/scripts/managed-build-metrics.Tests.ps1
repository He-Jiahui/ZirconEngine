. (Join-Path $PSScriptRoot 'managed-build-metrics.ps1')

Describe 'Managed validation metrics' {
    It 'records admission separately and sums check, link and test stages' {
        $target = Join-Path $TestDrive 'pool'
        $metrics = New-Item -ItemType Directory -Path (Join-Path $target '.zircon-compile/metrics') -Force
        $paths = @()
        foreach ($name in @('check', 'test')) {
            $path = Join-Path $metrics.FullName ($name + '.json')
            @{ timings = @{ checkSeconds = 2; compileLinkSeconds = 3; testExecutionSeconds = 5 }; exitCode = 0 } |
                ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $path
            $paths += $path
        }
        Write-ManagedValidationMetrics -ResolvedTarget ([pscustomobject]@{ DryRun = $false; TargetDir = $target; JobId = 'sample' }) `
            -CompileWorkspace ([pscustomobject]@{ metrics = @{ sourceSyncSeconds = 7; inputManifestHash = 'sealed' } }) `
            -MetricPaths $paths -QueueSeconds 11 -ExitCode 1
        $record = Get-Content (Join-Path $metrics.FullName 'validation-sample.json') -Raw | ConvertFrom-Json
        $record.exitCode | Should Be 1
        $record.timings.queueSeconds | Should Be 11
        $record.timings.sourceSyncSeconds | Should Be 7
        $record.timings.checkSeconds | Should Be 4
        $record.timings.compileLinkSeconds | Should Be 6
        $record.timings.testExecutionSeconds | Should Be 10
        $record.stages.Count | Should Be 2
    }

    It 'keeps the complete receipt in the log after its file is reclaimed' {
        $target = Join-Path $TestDrive 'reclaimed-pool'
        $metrics = New-Item -ItemType Directory -Path (Join-Path $target '.zircon-compile/metrics') -Force
        $stage = Join-Path $metrics.FullName 'test.json'
        @{
            timings = @{ checkSeconds = 2; compileLinkSeconds = 3; testExecutionSeconds = 5 }
            cacheDelta = @{ stats = @{ cache_hits = @{ counts = @{ Rust = 7 } } } }
            disk = @{ observedDriveGrowthPeakBytes = 4096 }
            exitCode = 0
        } | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $stage
        $script:metricLog = $null
        Mock Write-Host { param($Object) $script:metricLog = [string]$Object }

        Write-ManagedValidationMetrics -ResolvedTarget ([pscustomobject]@{ DryRun = $false; TargetDir = $target; JobId = 'reclaimed' }) `
            -CompileWorkspace ([pscustomobject]@{ metrics = @{ sourceSyncSeconds = 7; inputManifestHash = 'sealed' } }) `
            -MetricPaths @($stage) -QueueSeconds 11 -ExitCode 0
        $envelope = $script:metricLog.Substring('[managed-validation-metrics] '.Length) | ConvertFrom-Json
        $persisted = Get-Content -LiteralPath $envelope.receiptPath -Raw | ConvertFrom-Json
        ($envelope.receipt | ConvertTo-Json -Depth 32 -Compress) | Should Be ($persisted | ConvertTo-Json -Depth 32 -Compress)
        Remove-Item -LiteralPath $envelope.receiptPath -Force

        $envelope.receipt.jobId | Should Be 'reclaimed'
        $envelope.receipt.exitCode | Should Be 0
        $envelope.receipt.source.inputManifestHash | Should Be 'sealed'
        $envelope.receipt.timings.compileLinkSeconds | Should Be 3
        $envelope.receipt.stages[0].cacheDelta.stats.cache_hits.counts.Rust | Should Be 7
        $envelope.receipt.stages[0].disk.observedDriveGrowthPeakBytes | Should Be 4096
    }
}
