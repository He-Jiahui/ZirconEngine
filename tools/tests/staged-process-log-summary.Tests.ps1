$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'

Import-Module $supervisorModule -Force -ErrorAction Stop

Describe 'Staged process bounded log summary' {
    It 'reads only the tail window of a large log' {
        $logPath = Join-Path $TestDrive 'stderr.log'
        [IO.File]::WriteAllText(
            $logPath,
            (('discard-' * 512) + 'fallback-tail-marker'),
            [Text.UTF8Encoding]::new($false))

        $summary = Get-MvpSupervisedBoundedTailText -Path $logPath -MaximumCharacters 32

        $summary | Should Match 'fallback-tail-marker'
        ($summary.Length -le 32) | Should Be $true
    }

    It 'reports an unavailable artifact without reading a missing path' {
        (Get-MvpSupervisedBoundedTailText -Path (Join-Path $TestDrive 'missing.log')) | Should Be '<unavailable>'
    }

    It 'aggregates diagnostic markers within explicit per-file and total byte budgets' {
        $firstPath = Join-Path $TestDrive 'first.log'
        $secondPath = Join-Path $TestDrive 'second.log'
        [IO.File]::WriteAllText($firstPath, 'first-diagnostic-marker', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($secondPath, 'second-diagnostic-marker', [Text.UTF8Encoding]::new($false))

        $diagnostics = Get-MvpSupervisedBoundedDiagnosticText `
            -Paths @($firstPath, $secondPath) `
            -MaximumBytesPerFile 64 `
            -MaximumTotalBytes 64

        $diagnostics | Should Match 'first-diagnostic-marker'
        $diagnostics | Should Match 'second-diagnostic-marker'
    }

    It 'rejects a diagnostic file that exceeds its byte budget' {
        $oversizedPath = Join-Path $TestDrive 'oversized.log'
        [IO.File]::WriteAllText($oversizedPath, ('x' * 65), [Text.UTF8Encoding]::new($false))
        [IO.FileInfo]::new($oversizedPath).Length | Should Be 65

        $rejected = $false
        try {
            $null = Get-MvpSupervisedBoundedDiagnosticText `
                -Paths @($oversizedPath) `
                -MaximumBytesPerFile 64 `
                -MaximumTotalBytes 128
        }
        catch {
            $rejected = $true
        }
        $rejected | Should Be $true
    }
}
