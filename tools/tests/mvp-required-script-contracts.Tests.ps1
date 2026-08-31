$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$requiredScriptCases = @(
    @{
        CaseId = 'mvp-staging'
        ScriptRelativePath = 'tools\tests\mvp-staging.Tests.ps1'
        TimeoutMinutes = 15
    },
    @{
        CaseId = 'mvp-acceptance'
        ScriptRelativePath = 'tools\tests\mvp-acceptance.Tests.ps1'
        TimeoutMinutes = 10
    },
    @{
        CaseId = 'mvp-workflow'
        ScriptRelativePath = 'tools\tests\mvp_editor_windows_workflow.Tests.ps1'
        TimeoutMinutes = 2
    }
)

Describe 'MVP required script contracts' {
    BeforeAll {
        $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
    }

    It 'passes <CaseId> within its bounded runtime' -TestCases $requiredScriptCases {
        param(
            [string]$CaseId,
            [string]$ScriptRelativePath,
            [int]$TimeoutMinutes
        )

        $artifactRoot = if ([string]::IsNullOrWhiteSpace($env:MVP_EVIDENCE_ROOT)) {
            $TestDrive
        }
        else {
            Join-Path $env:MVP_EVIDENCE_ROOT 'required-script-contracts'
        }
        [IO.Directory]::CreateDirectory($artifactRoot) | Out-Null
        $stdoutPath = Join-Path $artifactRoot ($CaseId + '.stdout.log')
        $stderrPath = Join-Path $artifactRoot ($CaseId + '.stderr.log')
        $scriptPath = Join-Path $repoRoot $ScriptRelativePath

        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = (Get-Command powershell.exe -ErrorAction Stop).Source
        $startInfo.WorkingDirectory = $repoRoot
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        foreach ($argument in @(
                '-NoProfile',
                '-NonInteractive',
                '-ExecutionPolicy',
                'Bypass',
                '-File',
                $scriptPath)) {
            $startInfo.ArgumentList.Add($argument)
        }

        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $startInfo
        $stdoutStream = [IO.File]::Open(
            $stdoutPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::Read)
        $stderrStream = [IO.File]::Open(
            $stderrPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::Read)
        try {
            if (-not $process.Start()) {
                throw "Could not start required script contract '$CaseId'."
            }
            $stdoutTask = $process.StandardOutput.BaseStream.CopyToAsync($stdoutStream)
            $stderrTask = $process.StandardError.BaseStream.CopyToAsync($stderrStream)
            if (-not $process.WaitForExit($TimeoutMinutes * 60 * 1000)) {
                try {
                    $process.Kill($true)
                }
                catch {
                    if (-not $process.HasExited) {
                        $process.Kill()
                    }
                }
                $process.WaitForExit()
                [Threading.Tasks.Task]::WaitAll(@($stdoutTask, $stderrTask))
                throw [TimeoutException]::new("Required script contract '$CaseId' exceeded its $TimeoutMinutes-minute runner budget.")
            }
            [Threading.Tasks.Task]::WaitAll(@($stdoutTask, $stderrTask))
            $stdoutStream.Flush($true)
            $stderrStream.Flush($true)
            if ($process.ExitCode -ne 0) {
                throw "Required script contract '$CaseId' exited with code $($process.ExitCode); inspect '$stdoutPath' and '$stderrPath'."
            }
        }
        finally {
            $stdoutStream.Dispose()
            $stderrStream.Dispose()
            $process.Dispose()
        }

        Write-Host "MVP_REQUIRED_SCRIPT_CASE case=$CaseId stdout=$stdoutPath stderr=$stderrPath"
    }
}
