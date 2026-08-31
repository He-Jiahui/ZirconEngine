$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stageScript = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'
$processJobModule = Join-Path $repoRoot 'tools\mvp\RenderExtractProcessJob.psm1'

Describe 'MVP Stage Job process-tree authority' {
    It 'does not scan the machine process table or invoke taskkill in production staging' {
        $stageSource = Get-Content -LiteralPath $stageScript -Raw

        $stageSource | Should Not Match 'Get-CimInstance\s+Win32_Process'
        $stageSource | Should Not Match 'taskkill\.exe'
        $stageSource | Should Not Match 'function\s+Get-MvpStagedProcesses'
        $stageSource | Should Not Match 'function\s+Stop-MvpStagedProcesses'
    }

    It 'uses the directory rename probe without a second process-table audit' {
        $stageSource = Get-Content -LiteralPath $stageScript -Raw
        $releaseProbe = [regex]::Match(
            $stageSource,
            'function Test-MvpStagingDirectoryReleased \{(?<body>[\s\S]*?)\r?\n\}\r?\n\r?\nfunction Invoke-MvpProductStagingCore',
            [Text.RegularExpressions.RegexOptions]::CultureInvariant)

        $releaseProbe.Success | Should Be $true
        $releaseProbe.Groups['body'].Value | Should Match 'Move-ZirconWindowsPath -Source \$StageDirectory -Destination \$probe'
        $releaseProbe.Groups['body'].Value | Should Not Match 'Assert-MvpStagingProcessesReleased'
    }

    It 'assigns every process to its Job while suspended before resume' {
        $supervisorSource = Get-Content -LiteralPath $supervisorModule -Raw
        $jobSource = Get-Content -LiteralPath $processJobModule -Raw

        $supervisorSource | Should Match 'Start-RenderExtractBaselineSuspendedProcess -Job \$processJob -StartInfo \$StartInfo'
        $supervisorSource | Should Match 'Resume-RenderExtractBaselineProcess -Process \$assignedProcess'
        $jobSource | Should Match 'AssignProcessToJobObject'
        $jobSource | Should Match 'CreateSuspended\s+=\s+0x00000004'
        $jobSource | Should Match 'JobObjectLimitKillOnJobClose\s+=\s+0x00002000'
    }

    It 'requires Job-empty evidence before supervised completion returns' {
        $supervisorSource = Get-Content -LiteralPath $supervisorModule -Raw

        $supervisorSource | Should Match 'Wait-RenderExtractBaselineProcessJobEmpty'
        $supervisorSource | Should Match 'Test-RenderExtractBaselineProcessJobEmpty'
        $supervisorSource | Should Match 'job retained a descendant after its root product exited'
        $supervisorSource | Should Match '-JobEmpty \$jobEmpty'
    }
}
