$script:ModulePath = Join-Path $PSScriptRoot '..\mvp\RenderExtractWprCapture.psm1'
Import-Module $script:ModulePath -Force -ErrorAction Stop

function New-RenderExtractWprTestRoot {
    $root = "E:\ZirconBuilds\render02-wpr-pester-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($root) | Out-Null
    return $root
}

Describe 'Render-extract WPR capture' {
    It 'selects one explicit CPU or Heap profile' {
        (Resolve-RenderExtractWprProfile -UseWpr) | Should Be 'CPU'
        (Resolve-RenderExtractWprProfile -UseWprHeap) | Should Be 'Heap'
        (Resolve-RenderExtractWprProfile) | Should BeNullOrEmpty
        { Resolve-RenderExtractWprProfile -UseWpr -UseWprHeap } |
            Should Throw 'mutually exclusive'
    }

    It 'starts file-mode Heap tracing in the caller-owned temporary directory' {
        $root = New-RenderExtractWprTestRoot
        try {
            Mock Get-Command {
                [pscustomobject]@{ Source = 'D:\tools\wpr.exe' }
            } -ModuleName RenderExtractWprCapture
            Mock Invoke-RenderExtractWprNativeTool { 0 } -ModuleName RenderExtractWprCapture

            $capture = Start-RenderExtractWprCapture -TemporaryDirectory $root -Profile 'Heap'

            $capture.profile | Should Be 'Heap'
            $capture.wpr_path | Should Be 'D:\tools\wpr.exe'
            Assert-MockCalled Invoke-RenderExtractWprNativeTool -Times 1 -Exactly `
                -ModuleName RenderExtractWprCapture -ParameterFilter {
                    $FilePath -eq 'D:\tools\wpr.exe' -and
                    $Arguments.Count -eq 5 -and
                    $Arguments[0] -eq '-start' -and
                    $Arguments[1] -eq 'Heap' -and
                    $Arguments[2] -eq '-filemode' -and
                    $Arguments[3] -eq '-recordtempto' -and
                    $Arguments[4] -eq $root
                }
        }
        finally {
            if ([IO.Directory]::Exists($root)) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It 'exports PID and lifetime filtered Heap allocation stacks with hashes' {
        $root = New-RenderExtractWprTestRoot
        try {
            $tracePath = Join-Path $root 'run.heap.etl'
            $analysisPath = Join-Path $root 'run.heap-product-allocation-stacks.txt'
            $receiptPath = Join-Path $root 'run.heap-wpr-capture.json'
            Mock Get-Command {
                [pscustomobject]@{ Source = 'D:\tools\xperf.exe' }
            } -ModuleName RenderExtractWprCapture
            Mock Invoke-RenderExtractWprNativeTool {
                param([string]$FilePath, [string[]]$Arguments)
                if ($FilePath.EndsWith('wpr.exe')) {
                    [IO.File]::WriteAllBytes($Arguments[1], [byte[]](1, 2, 3, 4))
                }
                else {
                    $outputIndex = [Array]::IndexOf($Arguments, '-o')
                    'zircon_runtime.exe!mesh_command::prepare 4096' |
                        Set-Content -LiteralPath $Arguments[$outputIndex + 1] -Encoding UTF8
                }
                return 0
            } -ModuleName RenderExtractWprCapture
            $capture = [pscustomobject]@{
                schema_version = 1
                profile = 'Heap'
                temporary_directory = $root
                wpr_path = 'D:\tools\wpr.exe'
                started_at_utc = '2026-09-01T00:00:00.0000000Z'
            }

            $receipt = Stop-RenderExtractWprCapture `
                -Capture $capture `
                -TracePath $tracePath `
                -AnalysisPath $analysisPath `
                -ReceiptPath $receiptPath `
                -ProcessId 4242 `
                -ProcessStartedAtUtc '2026-09-01T00:00:10.0000000Z' `
                -ProcessEndedAtUtc '2026-09-01T00:00:20.0000000Z'

            $receipt.profile | Should Be 'Heap'
            $receipt.attribution.process_id | Should Be 4242
            $receipt.attribution.process_lifetime_range_applied | Should Be $true
            $receipt.trace.sha256 | Should Match '^[0-9A-F]{64}$'
            $receipt.analysis.sha256 | Should Match '^[0-9A-F]{64}$'
            Test-Path -LiteralPath $receiptPath -PathType Leaf | Should Be $true
            Assert-MockCalled Invoke-RenderExtractWprNativeTool -Times 1 -Exactly `
                -ModuleName RenderExtractWprCapture -ParameterFilter {
                    $FilePath -eq 'D:\tools\xperf.exe' -and
                    $Arguments -contains 'heap' -and
                    $Arguments -contains '-pid' -and
                    $Arguments -contains '4242' -and
                    $Arguments -contains '-range' -and
                    $Arguments -contains '2026/09/01:00:00:10.0000000+UTC' -and
                    $Arguments -contains '2026/09/01:00:00:20.0000000+UTC' -and
                    $Arguments -contains '-stacks' -and
                    $Arguments -contains 'st'
                }
        }
        finally {
            if ([IO.Directory]::Exists($root)) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It 'keeps sampled CPU analysis separate from Heap allocation analysis' {
        $root = New-RenderExtractWprTestRoot
        try {
            $tracePath = Join-Path $root 'run.etl'
            $analysisPath = Join-Path $root 'run.cpu-product-sampled-stacks.txt'
            $receiptPath = Join-Path $root 'run.cpu-wpr-capture.json'
            Mock Get-Command {
                [pscustomobject]@{ Source = 'D:\tools\xperf.exe' }
            } -ModuleName RenderExtractWprCapture
            Mock Invoke-RenderExtractWprNativeTool {
                param([string]$FilePath, [string[]]$Arguments)
                if ($FilePath.EndsWith('wpr.exe')) {
                    [IO.File]::WriteAllBytes($Arguments[1], [byte[]](1, 2, 3, 4))
                }
                else {
                    $outputIndex = [Array]::IndexOf($Arguments, '-o')
                    'zircon_runtime.exe!mesh_command::prepare 32' |
                        Set-Content -LiteralPath $Arguments[$outputIndex + 1] -Encoding UTF8
                }
                return 0
            } -ModuleName RenderExtractWprCapture
            $capture = [pscustomobject]@{
                schema_version = 1
                profile = 'CPU'
                temporary_directory = $root
                wpr_path = 'D:\tools\wpr.exe'
                started_at_utc = '2026-09-01T00:00:00.0000000Z'
            }

            Stop-RenderExtractWprCapture `
                -Capture $capture `
                -TracePath $tracePath `
                -AnalysisPath $analysisPath `
                -ReceiptPath $receiptPath `
                -ProcessId 4242 `
                -ProcessStartedAtUtc '2026-09-01T00:00:10.0000000Z' `
                -ProcessEndedAtUtc '2026-09-01T00:00:20.0000000Z' | Out-Null

            Assert-MockCalled Invoke-RenderExtractWprNativeTool -Times 1 -Exactly `
                -ModuleName RenderExtractWprCapture -ParameterFilter {
                    $FilePath -eq 'D:\tools\xperf.exe' -and
                    $Arguments -contains 'stack' -and
                    $Arguments -contains 'PROFILE' -and
                    $Arguments -notcontains 'heap'
                }
        }
        finally {
            if ([IO.Directory]::Exists($root)) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }
}
