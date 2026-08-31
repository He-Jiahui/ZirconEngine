$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpProcessLivenessProbe.psm1'
$registryModulePath = Join-Path $repoRoot 'tools\mvp\MvpScenarioRegistry.psm1'
$registryPath = Join-Path $repoRoot 'tools\mvp\mvp-scenario-registry.json'
$stageScript = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

Import-Module $registryModulePath -Force -ErrorAction Stop
Import-Module $modulePath -Force -ErrorAction Stop

$scenarioRegistry = Read-MvpScenarioRegistry -Path $registryPath
$runtimeScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.runtime-first-frame.v1'
$authoringScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.editor-authoring.v1'

function Assert-MvpLivenessProbeThrows {
    param([Parameter(Mandatory)][scriptblock]$Action)

    $threw = $false
    try {
        & $Action | Out-Null
    }
    catch {
        $threw = $true
    }
    $threw | Should Be $true
}

Describe 'MVP process liveness probe' {
    It 'requires every registered scenario to declare ordered typed progress event identities' {
        foreach ($scenario in @($scenarioRegistry.scenarios)) {
            $eventIds = @($scenario.progress_event_ids)
            $eventIds.Count | Should BeGreaterThan 0
            @($eventIds | Select-Object -Unique).Count | Should Be $eventIds.Count
            $eventIds | ForEach-Object {
                $_ | Should Match '^mvp\.[a-z0-9.-]+\.v1$'
            }
        }
        $moduleSource = Get-Content -LiteralPath $modulePath -Raw
        $stageSource = Get-Content -LiteralPath $stageScript -Raw
        $moduleSource | Should Not Match 'progress_name\s*='
        $stageSource | Should Match '-ScenarioRegistration \$ScenarioRegistration'
        $stageSource | Should Match '-ScenarioRegistration \$createScenario'
    }

    It 'publishes startup, first-frame, and teardown milestones once in semantic order' {
        $diagnosticRoot = Join-Path $TestDrive 'runtime-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192

        [IO.File]::WriteAllText($logPath, "runtime_first_frame_presented`n")
        $first = @(Read-MvpProcessLivenessProgress -State $state)
        [IO.File]::AppendAllText($logPath, "runtime_process_teardown_complete`n")
        $second = @(Read-MvpProcessLivenessProgress -State $state)
        $third = @(Read-MvpProcessLivenessProgress -State $state)

        $first.Count | Should Be 2
        $first[0] | Should Be 'mvp.runtime.startup-ready.v1'
        $first[1] | Should Be 'mvp.runtime.first-frame-presented.v1'
        $second.Count | Should Be 1
        $second[0] | Should Be 'mvp.runtime.teardown-complete.v1'
        $third.Count | Should Be 0
    }

    It 'retains marker overlap across append boundaries' {
        $diagnosticRoot = Join-Path $TestDrive 'split-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192

        [IO.File]::WriteAllText($logPath, 'runtime_first_frame_pre')
        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 0
        [IO.File]::AppendAllText($logPath, "sented`n")
        $progress = @(Read-MvpProcessLivenessProgress -State $state)

        $progress.Count | Should Be 2
        $progress[0] | Should Be 'mvp.runtime.startup-ready.v1'
        $progress[1] | Should Be 'mvp.runtime.first-frame-presented.v1'
    }

    It 'publishes typed project-save and authoring milestones' {
        $diagnosticRoot = Join-Path $TestDrive 'authoring-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'editor.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $authoringScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        [IO.File]::WriteAllText(
            $logPath,
            "editor_project_save result=started project=fixture`n" +
                "editor_project_save result=completed project=fixture`n" +
                "editor_authoring_trace result=completed event=save_project`n"
        )

        $progress = @(Read-MvpProcessLivenessProgress -State $state)

        $progress.Count | Should Be 3
        $progress[0] | Should Be 'mvp.editor.automation.project-save-started.v1'
        $progress[1] | Should Be 'mvp.editor.automation.project-save-completed.v1'
        $progress[2] | Should Be 'mvp.editor.automation.completed.v1'
    }

    It 'rejects diagnostic inventories and bytes above caller-owned budgets' {
        $fileCountRoot = Join-Path $TestDrive 'file-count-diagnostics'
        [IO.Directory]::CreateDirectory($fileCountRoot) | Out-Null
        1..3 | ForEach-Object {
            [IO.File]::WriteAllText((Join-Path $fileCountRoot "$_.log"), 'fixture')
        }
        $fileCountState = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $fileCountRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 2 `
            -MaximumBytesPerFile 64 `
            -MaximumTotalBytesPerPoll 128
        Assert-MvpLivenessProbeThrows { Read-MvpProcessLivenessProgress -State $fileCountState }

        $fileBytesRoot = Join-Path $TestDrive 'file-bytes-diagnostics'
        [IO.Directory]::CreateDirectory($fileBytesRoot) | Out-Null
        [IO.File]::WriteAllText((Join-Path $fileBytesRoot 'oversized.log'), ('x' * 65))
        $fileBytesState = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $fileBytesRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 2 `
            -MaximumBytesPerFile 64 `
            -MaximumTotalBytesPerPoll 128
        Assert-MvpLivenessProbeThrows { Read-MvpProcessLivenessProgress -State $fileBytesState }

        $aggregateRoot = Join-Path $TestDrive 'aggregate-diagnostics'
        [IO.Directory]::CreateDirectory($aggregateRoot) | Out-Null
        [IO.File]::WriteAllText((Join-Path $aggregateRoot 'one.log'), ('x' * 40))
        [IO.File]::WriteAllText((Join-Path $aggregateRoot 'two.log'), ('y' * 40))
        $aggregateState = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $aggregateRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 2 `
            -MaximumBytesPerFile 64 `
            -MaximumTotalBytesPerPoll 64
        Assert-MvpLivenessProbeThrows { Read-MvpProcessLivenessProgress -State $aggregateState }
    }

    It 'uses typed directory stacks without sorting diagnostic files on every poll' {
        $source = Get-Content -LiteralPath $modulePath -Raw

        $source | Should Match '\[Collections\.Generic\.Stack\[IO\.DirectoryInfo\]\]::new\(\)'
        $source | Should Match '\[Collections\.Generic\.Stack\[int\]\]::new\(\)'
        $source | Should Not Match '\[Collections\.Generic\.Stack\[object\]\]::new\(\)'
        $source | Should Not Match 'Sort-Object FullName'
    }

    It 'allocates one read buffer per probe state and reuses it across polls' {
        $diagnosticRoot = Join-Path $TestDrive 'reused-buffer-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        $buffer = $state.read_buffer

        [IO.File]::WriteAllText($logPath, "runtime_first_frame_presented`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        [IO.File]::AppendAllText($logPath, "runtime_process_teardown_complete`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        $source = Get-Content -LiteralPath $modulePath -Raw

        $buffer.Length | Should Be 8192
        [Object]::ReferenceEquals($buffer, $state.read_buffer) | Should Be $true
        $source | Should Match 'read_buffer = \[byte\[\]\]::new\(\$script:MvpLivenessReadBufferBytes\)'
        $source | Should Not Match 'New-Object byte\[\] \$script:MvpLivenessReadBufferBytes'
    }

    It 'reuses typed snapshot and marker scratch collections across polls' {
        $diagnosticRoot = Join-Path $TestDrive 'reused-scratch-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        $activePaths = $state.active_paths_scratch
        $detectedMarkers = $state.detected_markers_scratch
        $snapshotPaths = $state.snapshot_paths_scratch

        [IO.File]::WriteAllText($logPath, "runtime_first_frame_presented`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        [IO.File]::AppendAllText($logPath, "runtime_process_teardown_complete`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        $source = Get-Content -LiteralPath $modulePath -Raw

        [Object]::ReferenceEquals($activePaths, $state.active_paths_scratch) | Should Be $true
        [Object]::ReferenceEquals($detectedMarkers, $state.detected_markers_scratch) | Should Be $true
        [Object]::ReferenceEquals($snapshotPaths, $state.snapshot_paths_scratch) | Should Be $true
        $source | Should Match 'snapshot_offsets_scratch = \[Collections\.Generic\.List\[Int64\]\]::new\(\)'
        $source | Should Match 'snapshot_bytes_scratch = \[Collections\.Generic\.List\[Int64\]\]::new\(\)'
        $source | Should Match 'snapshot_carry_scratch = \[Collections\.Generic\.List\[string\]\]::new\(\)'
        $source | Should Not Match '\$snapshots\.Add\(\[pscustomobject\]'
    }

    It 'skips empty progress materialization and reuses the typed progress scratch' {
        $diagnosticRoot = Join-Path $TestDrive 'reused-progress-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        $progressScratch = $state.progress_scratch

        [IO.File]::WriteAllText($logPath, "no semantic marker`n")
        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 0
        [IO.File]::AppendAllText($logPath, "runtime_first_frame_presented`n")
        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 2
        $source = Get-Content -LiteralPath $modulePath -Raw
        $readFunction = [regex]::Match(
            $source,
            '(?s)function Read-MvpProcessLivenessProgress \{.*?(?=\r?\nExport-ModuleMember)')

        [Object]::ReferenceEquals($progressScratch, $state.progress_scratch) | Should Be $true
        $source | Should Match 'progress_scratch = \[Collections\.Generic\.List\[string\]\]::new\(\)'
        $readFunction.Value | Should Match '\$detectedMarkers\.Count -eq 0'
        $readFunction.Value | Should Not Match 'return \[string\[\]\]::new\(0\)'
        $readFunction.Value.IndexOf('detectedMarkers.Count', [StringComparison]::Ordinal) |
            Should BeLessThan $readFunction.Value.IndexOf('progress = $State.progress_scratch', [StringComparison]::Ordinal)
    }

    It 'reuses the diagnostic inventory stacks and file list across polls' {
        $diagnosticRoot = Join-Path $TestDrive 'reused-inventory-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        $directories = $state.pending_directories_scratch
        $depths = $state.pending_depths_scratch
        $files = $state.diagnostic_files_scratch

        [IO.File]::WriteAllText($logPath, "runtime_first_frame_presented`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        [IO.File]::AppendAllText($logPath, "runtime_process_teardown_complete`n")
        $null = @(Read-MvpProcessLivenessProgress -State $state)
        $source = Get-Content -LiteralPath $modulePath -Raw

        [Object]::ReferenceEquals($directories, $state.pending_directories_scratch) | Should Be $true
        [Object]::ReferenceEquals($depths, $state.pending_depths_scratch) | Should Be $true
        [Object]::ReferenceEquals($files, $state.diagnostic_files_scratch) | Should Be $true
        $source | Should Match 'diagnostic_files_scratch = \[Collections\.Generic\.List\[IO\.FileInfo\]\]::new\(\)'
        $source | Should Match 'Write-Output -NoEnumerate \$files'
        $source | Should Not Match 'return @\(\$files\.ToArray\(\)\)'
    }

    It 'reuses a stale-path list instead of materializing the tracked key set each poll' {
        $diagnosticRoot = Join-Path $TestDrive 'reused-stale-path-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $retainedLog = Join-Path $diagnosticRoot 'retained.log'
        $removedLog = Join-Path $diagnosticRoot 'removed.log'
        [IO.File]::WriteAllText($retainedLog, "no semantic marker`n")
        [IO.File]::WriteAllText($removedLog, "no semantic marker`n")
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 4 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        $stalePaths = $state.stale_paths_scratch

        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 0
        [IO.File]::Delete($removedLog)
        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 0
        $source = Get-Content -LiteralPath $modulePath -Raw

        $state.file_offsets.ContainsKey($retainedLog) | Should Be $true
        $state.file_offsets.ContainsKey($removedLog) | Should Be $false
        [Object]::ReferenceEquals($stalePaths, $state.stale_paths_scratch) | Should Be $true
        $source | Should Match 'stale_paths_scratch = \[Collections\.Generic\.List\[string\]\]::new\(\)'
        $source | Should Not Match 'foreach \(\$path in @\(\$State\.file_offsets\.Keys\)\)'
    }

    It 'short-circuits inventory work after every registered progress event is emitted' {
        $diagnosticRoot = Join-Path $TestDrive 'completed-progress-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 1 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        [IO.File]::WriteAllText(
            $logPath,
            "runtime_first_frame_presented`nruntime_process_teardown_complete`n")

        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 3
        [IO.File]::WriteAllText((Join-Path $diagnosticRoot 'over-budget.log'), 'must not be scanned')
        $afterCompletion = @(Read-MvpProcessLivenessProgress -State $state)
        $source = Get-Content -LiteralPath $modulePath -Raw
        $readFunction = [regex]::Match(
            $source,
            '(?s)function Read-MvpProcessLivenessProgress \{.*?(?=\r?\nExport-ModuleMember)')

        $afterCompletion.Count | Should Be 0
        $readFunction.Success | Should Be $true
        $readFunction.Value | Should Match '\$State\.emitted_progress\.Count -eq \$State\.milestones\.Count'
        $readFunction.Value.IndexOf('emitted_progress.Count', [StringComparison]::Ordinal) |
            Should BeLessThan $readFunction.Value.IndexOf('Get-MvpProcessLivenessDiagnosticFiles', [StringComparison]::Ordinal)
    }

    It 'skips marker searches for progress events emitted by an earlier poll' {
        $diagnosticRoot = Join-Path $TestDrive 'remaining-marker-diagnostics'
        [IO.Directory]::CreateDirectory($diagnosticRoot) | Out-Null
        $logPath = Join-Path $diagnosticRoot 'runtime.log'
        $state = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $runtimeScenario `
            -MaximumFileCount 2 `
            -MaximumBytesPerFile 4096 `
            -MaximumTotalBytesPerPoll 8192
        [IO.File]::WriteAllText($logPath, "runtime_first_frame_presented`n")
        @(Read-MvpProcessLivenessProgress -State $state).Count | Should Be 2
        [IO.File]::AppendAllText($logPath, "runtime_process_teardown_complete`n")

        $remaining = @(Read-MvpProcessLivenessProgress -State $state)
        $source = Get-Content -LiteralPath $modulePath -Raw

        $remaining.Count | Should Be 1
        $remaining[0] | Should Be 'mvp.runtime.teardown-complete.v1'
        $source | Should Match '\$State\.emitted_progress\.Contains\(\$milestone\.progress_event_id\)'
        $source | Should Match '(?s)emitted_progress\.Contains\(\$milestone\.progress_event_id\).*?continue.*?candidate\.IndexOf'
    }
}
