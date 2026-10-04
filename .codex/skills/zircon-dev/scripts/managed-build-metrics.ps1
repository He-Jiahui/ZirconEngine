function Write-ManagedValidationMetrics {
    param([object]$ResolvedTarget, [object]$CompileWorkspace, [string[]]$MetricPaths,
          [double]$QueueSeconds, [int]$ExitCode)

    if ($ResolvedTarget.DryRun -or -not $CompileWorkspace -or -not $MetricPaths) { return }
    $stages = @($MetricPaths | ForEach-Object { Get-Content -LiteralPath $_ -Raw | ConvertFrom-Json })
    $timings = [ordered]@{ queueSeconds = $QueueSeconds; sourceSyncSeconds = $CompileWorkspace.metrics.sourceSyncSeconds }
    foreach ($name in @('checkSeconds', 'compileLinkSeconds', 'testExecutionSeconds')) {
        $timings[$name] = ($stages | ForEach-Object { $_.timings.$name } | Measure-Object -Sum).Sum
    }
    $record = [ordered]@{
        schemaVersion = 1
        jobId = $ResolvedTarget.JobId
        exitCode = $ExitCode
        targetDirectory = $ResolvedTarget.TargetDir
        source = $CompileWorkspace.metrics
        timings = $timings
        stages = $stages
    }
    $path = Join-Path $ResolvedTarget.TargetDir ('.zircon-compile/metrics/validation-' + $ResolvedTarget.JobId + '.json')
    [System.IO.File]::WriteAllText($path, ($record | ConvertTo-Json -Depth 32), [System.Text.UTF8Encoding]::new($false))
    Write-Host ('[managed-validation-metrics] ' + ([ordered]@{
        receiptPath = $path; jobId = $record.jobId; exitCode = $ExitCode; timings = $timings; receipt = $record
    } | ConvertTo-Json -Depth 32 -Compress))
}
