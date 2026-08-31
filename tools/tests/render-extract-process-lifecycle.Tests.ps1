$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$jobModule = Join-Path $repoRoot 'tools\mvp\RenderExtractProcessJob.psm1'

Import-Module $jobModule -Force -ErrorAction Stop

Describe 'Render-extract process lifecycle' {
    It 'configures active-process, job-memory, and CPU hard-cap limits before launch' {
        $job = New-RenderExtractBaselineProcessJob `
            -MaximumActiveProcessCount 2 `
            -MaximumJobMemoryBytes 134217728 `
            -MaximumCpuRatePerTenThousand 7500
        try {
            $limits = Get-RenderExtractBaselineProcessJobLimits -Job $job

            $limits.ActiveProcessLimit | Should Be 2
            $limits.JobMemoryLimitBytes | Should Be 134217728
            $limits.CpuRatePerTenThousand | Should Be 7500
            (($limits.CpuRateControlFlags -band 0x00000005) -eq 0x00000005) | Should Be $true
        }
        finally {
            $job.Dispose()
        }
    }

    It 'starts job-bound output capture before explicitly resuming the product process' {
        $job = New-RenderExtractBaselineProcessJob
        $assigned = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "(for /L %i in (1,1,512) do @echo lifecycle-line-%i) & @echo lifecycle-tail-marker"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $assigned = Start-RenderExtractBaselineSuspendedProcess -Job $job -StartInfo $startInfo
            $assigned.Process.HasExited | Should Be $false

            $stdoutPath = Join-Path $TestDrive 'stdout.log'
            $stdoutTailPath = Join-Path $TestDrive 'stdout.tail.log'
            $stdoutCapture = Start-RenderExtractBaselineBoundedOutputCapture `
                -Reader $assigned.StandardOutput `
                -OutputPath $stdoutPath `
                -MaximumRetainedBytes 1024 `
                -TailOutputPath $stdoutTailPath `
                -MaximumTailBytes 1024
            Test-Path -LiteralPath $stdoutPath -PathType Leaf | Should Be $true
            Test-Path -LiteralPath $stdoutTailPath -PathType Leaf | Should Be $true
            Resume-RenderExtractBaselineProcess -Process $assigned

            $assigned.Process.WaitForExit()
            (Wait-RenderExtractBaselineProcessJobEmpty -Job $job -SessionId 'lifecycle-fixture' -TimeoutMilliseconds 5000) | Should Be $true
            $capture = $stdoutCapture.GetAwaiter().GetResult()
            $capture.TotalBytes | Should BeGreaterThan 1024
            ($capture.RetainedBytes -le 1024) | Should Be $true
            ($capture.TailRetainedBytes -le 1024) | Should Be $true
            [IO.File]::ReadAllText($stdoutPath) | Should Match 'lifecycle-line-1'
            [IO.File]::ReadAllText($stdoutTailPath) | Should Match 'lifecycle-tail-marker'
        }
        finally {
            if ($null -ne $assigned) {
                $assigned.Dispose()
            }
            $job.Dispose()
        }
    }
}
