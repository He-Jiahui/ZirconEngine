$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $repoRoot 'tools\mvp\RenderExtractSystemTraceEvidence.psm1') `
    -Force -DisableNameChecking -ErrorAction Stop

function New-RenderExtractSystemTraceFixture {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][Int64]$RunProcessId,
        [Parameter(Mandatory)][Int64]$ReceiptProcessId
    )

    $invocationId = 'A' * 32
    $logicalId = 'pipelined-first-frame'
    $attempt = 1
    $tracesRoot = Join-Path $Root 'traces'
    $invocationRoot = Join-Path $tracesRoot $invocationId
    [IO.Directory]::CreateDirectory($invocationRoot) | Out-Null
    $sessionId = "$logicalId-$attempt"
    $tracePath = Join-Path $invocationRoot "$sessionId.heap.etl"
    $analysisPath = Join-Path $invocationRoot "$sessionId.heap-product-allocation-stacks.txt"
    $receiptPath = Join-Path $invocationRoot "$sessionId.heap-wpr-capture.json"
    [IO.File]::WriteAllBytes($tracePath, [byte[]](1, 2, 3, 4))
    [IO.File]::WriteAllText($analysisPath, 'allocation stack', [Text.UTF8Encoding]::new($false))
    $receipt = [ordered]@{
        schema_version = 1
        evidence_kind = 'windows_product_heap_allocation_stacks'
        profile = 'Heap'
        started_at_utc = '2026-09-01T00:00:00.0000000Z'
        trace = [ordered]@{
            path = $tracePath
            bytes = [IO.FileInfo]::new($tracePath).Length
            sha256 = (Get-FileHash -LiteralPath $tracePath -Algorithm SHA256).Hash
        }
        analysis = [ordered]@{
            path = $analysisPath
            bytes = [IO.FileInfo]::new($analysisPath).Length
            sha256 = (Get-FileHash -LiteralPath $analysisPath -Algorithm SHA256).Hash
        }
        attribution = [ordered]@{
            scope = 'product_process'
            process_id = $ReceiptProcessId
            process_started_at_utc = '2026-09-01T00:00:10.0000000Z'
            process_ended_at_utc = '2026-09-01T00:00:20.0000000Z'
            process_lifetime_range_applied = $true
            trace_range = [ordered]@{
                format = 'utc_wall_clock'
                start_at_utc = '2026-09-01T00:00:10.0000000Z'
                end_at_utc = '2026-09-01T00:00:20.0000000Z'
                xperf_start = '2026/09/01:00:00:10.0000000+UTC'
                xperf_end = '2026/09/01:00:00:20.0000000+UTC'
            }
        }
    }
    [IO.File]::WriteAllText(
        $receiptPath,
        ($receipt | ConvertTo-Json -Depth 6),
        [Text.UTF8Encoding]::new($false)
    )
    return [pscustomobject]@{
        run = [pscustomobject]@{
            started_at_utc = '2026-09-01T00:00:09.0000000Z'
            ended_at_utc = '2026-09-01T00:00:21.0000000Z'
            system_trace_profile = 'heap'
            system_trace_etl = $tracePath
            system_trace_analysis = $analysisPath
            system_trace_receipt = $receiptPath
        }
        logical_id = $logicalId
        attempt = $attempt
        process_id = $RunProcessId
        invocation_id = $invocationId
        traces_root = $tracesRoot
    }
}

Describe 'Render-extract system trace evidence' {
    It 'accepts only a fully bound product Heap receipt' {
        $root = "E:\ZirconBuilds\render02-trace-evidence-$([guid]::NewGuid().ToString('N'))"
        try {
            $fixture = New-RenderExtractSystemTraceFixture `
                -Root $root `
                -RunProcessId 4242 `
                -ReceiptProcessId 4242

            $result = Get-RenderExtractSystemTraceEvidenceForRun `
                -Run $fixture.run `
                -LogicalId $fixture.logical_id `
                -Attempt $fixture.attempt `
                -ProcessId $fixture.process_id `
                -InvocationId $fixture.invocation_id `
                -TracesRoot $fixture.traces_root

            $result.status | Should Be 'measured'
            $result.profile | Should Be 'heap'
            $result.artifacts.Count | Should Be 3
            (Get-RenderExtractSystemTraceMeasurementCoverage `
                    -Records @($result) `
                    -Profile 'heap').status | Should Be 'measured'
        }
        finally {
            if ([IO.Directory]::Exists($root)) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It 'rejects a receipt attributed to a different process' {
        $root = "E:\ZirconBuilds\render02-trace-evidence-$([guid]::NewGuid().ToString('N'))"
        try {
            $fixture = New-RenderExtractSystemTraceFixture `
                -Root $root `
                -RunProcessId 4242 `
                -ReceiptProcessId 5252

            {
                Get-RenderExtractSystemTraceEvidenceForRun `
                    -Run $fixture.run `
                    -LogicalId $fixture.logical_id `
                    -Attempt $fixture.attempt `
                    -ProcessId $fixture.process_id `
                    -InvocationId $fixture.invocation_id `
                    -TracesRoot $fixture.traces_root
            } | Should Throw 'does not bind its profile, process lifetime, ETL, and analysis artifacts'
        }
        finally {
            if ([IO.Directory]::Exists($root)) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }
}
