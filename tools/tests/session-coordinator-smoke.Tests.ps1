[CmdletBinding()]
param(
    [switch]$KernelOnly,
    [switch]$LeaseAndPatch,
    [switch]$CargoAndCleanup,
    [switch]$FinalizeInTempRepo,
    [switch]$LegacyRollout
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$sourceRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")).Path
$python = (Get-Command python -ErrorAction Stop).Source

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) {
        throw $Message
    }
}

function Invoke-PythonCoordinator {
    param([string]$RepoRoot, [string[]]$CommandArguments)
    $oldPythonPath = $env:PYTHONPATH
    try {
        $env:PYTHONPATH = $sourceRoot
        $output = & $python -m tools.session_coordinator --repo-root $RepoRoot --json @CommandArguments
        return [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = ($output -join "`n") }
    }
    finally {
        $env:PYTHONPATH = $oldPythonPath
    }
}

function Test-Kernel {
    $testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("zircon-coordinator-smoke-" + [guid]::NewGuid().ToString("N"))
    $repo = Join-Path $testRoot "repo"
    New-Item -ItemType Directory -Path $repo -Force | Out-Null
    $process = $null
    $oldMaintenanceToken = $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN
    try {
        & git -C $repo init -q
        & git -C $repo config user.email "coordinator-smoke@example.invalid"
        & git -C $repo config user.name "Coordinator Smoke"
        & git -C $repo config core.autocrlf false
        & git -C $repo branch -M main
        Set-Content -LiteralPath (Join-Path $repo "README.md") -Value "baseline" -Encoding utf8
        & git -C $repo add README.md
        & git -C $repo commit -q -m "test: baseline"

        $oldPythonPath = $env:PYTHONPATH
        $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN = [guid]::NewGuid().ToString("N")
        $env:PYTHONPATH = $sourceRoot
        $process = Start-Process -FilePath $python `
            -ArgumentList @("-m", "tools.session_coordinator", "--repo-root", $repo, "serve") `
            -WorkingDirectory $repo -WindowStyle Hidden -PassThru
        $env:PYTHONPATH = $oldPythonPath

        $healthy = $false
        for ($attempt = 0; $attempt -lt 50; $attempt++) {
            Start-Sleep -Milliseconds 100
            $status = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("status")
            if ($status.ExitCode -eq 0) {
                $healthy = $true
                break
            }
        }
        Assert-True $healthy "Coordinator did not become healthy."

        $registered = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "register", "--session-id", "smoke-session"
        )
        Assert-True ($registered.ExitCode -eq 0) "Session registration failed: $($registered.Output)"

        $active = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "set-status", "active", "--session-id", "smoke-session"
        )
        Assert-True ($active.ExitCode -eq 0) "Session activation failed: $($active.Output)"

        $invalid = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "set-status", "archived", "--session-id", "smoke-session"
        )
        Assert-True ($invalid.ExitCode -eq 2) "Invalid transition did not fail with exit code 2."

        $stopped = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("stop")
        Assert-True ($stopped.ExitCode -eq 0) "Coordinator stop failed: $($stopped.Output)"
        $process.WaitForExit(5000) | Out-Null
        Assert-True $process.HasExited "Coordinator process remained alive after stop."
        Write-Host "PASS: coordinator kernel smoke"
    }
    finally {
        $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN = $oldMaintenanceToken
        if ($null -ne $process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $testRoot) {
            Remove-Item -LiteralPath $testRoot -Recurse -Force
        }
    }
}

function Test-LeaseAndPatch {
    $testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("zircon-coordinator-patch-" + [guid]::NewGuid().ToString("N"))
    $repo = Join-Path $testRoot "repo"
    New-Item -ItemType Directory -Path $repo -Force | Out-Null
    $process = $null
    try {
        & git -C $repo init -q
        & git -C $repo config user.email "coordinator-smoke@example.invalid"
        & git -C $repo config user.name "Coordinator Smoke"
        & git -C $repo config core.autocrlf false
        & git -C $repo branch -M main
        [System.IO.File]::WriteAllText(
            (Join-Path $repo "README.md"),
            "baseline`n",
            [System.Text.UTF8Encoding]::new($false)
        )
        & git -C $repo add README.md
        & git -C $repo commit -q -m "test: baseline"

        $oldPythonPath = $env:PYTHONPATH
        $env:PYTHONPATH = $sourceRoot
        $process = Start-Process -FilePath $python `
            -ArgumentList @("-m", "tools.session_coordinator", "--repo-root", $repo, "serve") `
            -WorkingDirectory $repo -WindowStyle Hidden -PassThru
        $env:PYTHONPATH = $oldPythonPath

        for ($attempt = 0; $attempt -lt 50; $attempt++) {
            Start-Sleep -Milliseconds 100
            $status = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("status")
            if ($status.ExitCode -eq 0) { break }
        }
        Assert-True ($status.ExitCode -eq 0) "Coordinator did not become healthy."

        foreach ($sessionId in @("session-a", "session-b")) {
            $registered = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
                "session", "register", "--session-id", $sessionId
            )
            Assert-True ($registered.ExitCode -eq 0) "Registration failed for $sessionId."
            $activated = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
                "session", "set-status", "active", "--session-id", $sessionId
            )
            Assert-True ($activated.ExitCode -eq 0) "Activation failed for $sessionId."
        }

        $baseline = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("baseline", "init")
        Assert-True ($baseline.ExitCode -eq 0) "Baseline initialization failed."
        $claim = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "lease", "claim", "README.md", "--session-id", "session-a"
        )
        Assert-True ($claim.ExitCode -eq 0) "Lease claim failed."

        $patchPath = Join-Path $testRoot "change.patch"
        $patchContent = @"
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-baseline
+patched
"@
        $patchContent += "`n"
        [System.IO.File]::WriteAllText(
            $patchPath,
            $patchContent,
            [System.Text.UTF8Encoding]::new($false)
        )

        $queued = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "patch", "enqueue", "--file", $patchPath, "--target", "README.md", "--session-id", "session-b"
        )
        Assert-True ($queued.ExitCode -eq 0) "Patch enqueue failed: $($queued.Output)"
        Assert-True ($queued.Output -match '"status": "queued"') "Patch was not queued behind the active lease."

        $released = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "lease", "release", "README.md", "--session-id", "session-a"
        )
        Assert-True ($released.ExitCode -eq 0) "Lease release/process failed: $($released.Output)"
        Assert-True ($released.Output -match '"status": "applied"') "Queued patch did not apply after release: $($released.Output)"
        Assert-True ((Get-Content -Raw -LiteralPath (Join-Path $repo "README.md")).Trim() -eq "patched") "Patch result was not written."

        $stopped = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("stop")
        Assert-True ($stopped.ExitCode -eq 0) "Coordinator stop failed."
        $process.WaitForExit(5000) | Out-Null
        Assert-True $process.HasExited "Coordinator process remained alive after patch smoke."
        Write-Host "PASS: coordinator lease and delayed patch smoke"
    }
    finally {
        if ($null -ne $process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $testRoot) {
            Remove-Item -LiteralPath $testRoot -Recurse -Force
        }
    }
}

function Test-CargoAndCleanup {
    $oldPythonPath = $env:PYTHONPATH
    try {
        $env:PYTHONPATH = $sourceRoot
        & $python -u -m unittest `
            tools.session_coordinator.tests.test_cargo_jobs `
            tools.session_coordinator.tests.test_cleanup
        Assert-True ($LASTEXITCODE -eq 0) "Cargo lane or cleanup unit smoke failed."
    }
    finally {
        $env:PYTHONPATH = $oldPythonPath
    }

    $taskInstaller = Join-Path $sourceRoot "tools\install-session-coordinator-task.ps1"
    $taskPlan = & $taskInstaller -Action Install -RepoRoot $sourceRoot -DryRun
    Assert-True ($LASTEXITCODE -eq 0) "Scheduled-task dry-run failed."
    Assert-True (($taskPlan -join "`n") -match "ONLOGON") "Daemon at-logon task was not planned."
    Assert-True (($taskPlan -join "`n") -match "retire external scheduler") "External maintenance scheduler was not retired."
    $cutoverPlan = & $taskInstaller -Action Cutover -RepoRoot $sourceRoot -DryRun `
        -LegacyTaskName "LegacyZirconCleanup"
    Assert-True ($LASTEXITCODE -eq 0) "Scheduled-task cutover dry-run failed."
    Assert-True (($cutoverPlan -join "`n") -match "two consecutive plan-only maintenance") "Cutover health gate was not planned."
    Assert-True (($cutoverPlan -join "`n") -match "preparing record before startup mutation") "Cutover journal was not planned before startup mutation."
    Assert-True (($cutoverPlan -join "`n") -match "LegacyZirconCleanup.*DISABLE") "Legacy task disable was not delayed until cutover."
    Assert-True (($cutoverPlan -join "`n") -match "retire external scheduler") "Scheduled-task cutover did not retire the external scheduler."
    $startupPlan = & $taskInstaller -Action Cutover -RepoRoot $sourceRoot -DryRun `
        -Backend UserStartup -LegacyTaskName "LegacyZirconCleanup"
    Assert-True ($LASTEXITCODE -eq 0) "Current-user startup cutover dry-run failed."
    Assert-True (($startupPlan -join "`n") -match "HKCU Run") "User startup backend was not planned."
    Assert-True (($startupPlan -join "`n") -match "two consecutive plan-only maintenance") "User startup cutover omitted health gates."
    Assert-True (($startupPlan -join "`n") -match "retire external scheduler") "User startup cutover did not retire the external scheduler."

    $tokens = $null
    $parseErrors = $null
    $installerAst = [Management.Automation.Language.Parser]::ParseFile(
        $taskInstaller,
        [ref]$tokens,
        [ref]$parseErrors
    )
    Assert-True ($parseErrors.Count -eq 0) "Task installer could not be parsed for path-scope tests."
    $matcherAst = $installerAst.Find({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -eq "Test-LegacyCleanupActionForRepo"
    }, $true)
    Assert-True ($null -ne $matcherAst) "Legacy cleanup path matcher was not found."
    Invoke-Expression $matcherAst.Extent.Text
    $resolvedRepoRoot = [IO.Path]::GetFullPath($sourceRoot).TrimEnd('\', '/')
    $cleanup = Join-Path $resolvedRepoRoot "tools\cleanup-stale-targets.ps1"
    $exactLegacyAction = "cmd.exe /c cd /d `"$resolvedRepoRoot`" && powershell -File tools\cleanup-stale-targets.ps1"
    $backupLegacyAction = "cmd.exe /c cd /d `"$resolvedRepoRoot-backup`" && powershell -File tools\cleanup-stale-targets.ps1"
    Assert-True (Test-LegacyCleanupActionForRepo -ActionText $exactLegacyAction) "Exact repository cleanup action was not matched."
    Assert-True (-not (Test-LegacyCleanupActionForRepo -ActionText $backupLegacyAction)) "Repository-prefix collision matched a foreign cleanup task."

    $restoreAst = $installerAst.Find({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -eq "Restore-ExternalMaintenanceTaskIfRequired"
    }, $true)
    Assert-True ($null -ne $restoreAst) "Maintenance rollback helper was not found."
    Invoke-Expression $restoreAst.Extent.Text
    $maintenanceTask = "TestMaintenanceTask"
    $script:restoredMaintenance = $false
    function Get-ScheduledTaskOrNull { return [pscustomobject]@{ Settings = [pscustomobject]@{ Enabled = $false } } }
    function Invoke-TaskCommand { $script:restoredMaintenance = $true }
    Restore-ExternalMaintenanceTaskIfRequired -Record @{ maintenance_task_was_enabled = $true }
    Assert-True $script:restoredMaintenance "Hashtable cutover record did not restore external maintenance."

    $compatibilityAst = $installerAst.Find({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -eq "Assert-CompatibleActiveCutover"
    }, $true)
    Assert-True ($null -ne $compatibilityAst) "Cutover compatibility guard was not found."
    Invoke-Expression $compatibilityAst.Extent.Text
    $preparingRejected = $false
    try {
        Assert-CompatibleActiveCutover -Record ([pscustomobject]@{ status = "preparing" })
    }
    catch {
        $preparingRejected = $true
    }
    Assert-True $preparingRejected "Interrupted preparing cutover was allowed to overwrite its rollback journal."
    Write-Host "PASS: managed Cargo lanes and cleanup smoke"
}

function Test-FinalizeInTempRepo {
    $testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("zircon-coordinator-finalize-" + [guid]::NewGuid().ToString("N"))
    $repo = Join-Path $testRoot "repo"
    New-Item -ItemType Directory -Path $repo -Force | Out-Null
    $process = $null
    try {
        & git -C $repo init -q
        & git -C $repo config user.email "coordinator-smoke@example.invalid"
        & git -C $repo config user.name "Coordinator Smoke"
        & git -C $repo config core.autocrlf false
        & git -C $repo branch -M main
        [System.IO.File]::WriteAllText(
            (Join-Path $repo "README.md"),
            "baseline`n",
            [System.Text.UTF8Encoding]::new($false)
        )
        & git -C $repo add README.md
        & git -C $repo commit -q -m "test: baseline"
        $before = (& git -C $repo rev-parse HEAD).Trim()

        $oldPythonPath = $env:PYTHONPATH
        $env:PYTHONPATH = $sourceRoot
        $process = Start-Process -FilePath $python `
            -ArgumentList @("-m", "tools.session_coordinator", "--repo-root", $repo, "serve") `
            -WorkingDirectory $repo -WindowStyle Hidden -PassThru
        $env:PYTHONPATH = $oldPythonPath

        for ($attempt = 0; $attempt -lt 50; $attempt++) {
            Start-Sleep -Milliseconds 100
            $status = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("status")
            if ($status.ExitCode -eq 0) { break }
        }
        Assert-True ($status.ExitCode -eq 0) "Coordinator did not become healthy."
        $registered = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "register", "--session-id", "session-a"
        )
        Assert-True ($registered.ExitCode -eq 0) "Finalize Session registration failed."
        $activated = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "set-status", "active", "--session-id", "session-a"
        )
        Assert-True ($activated.ExitCode -eq 0) "Finalize Session activation failed."
        $baseline = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("baseline", "init")
        Assert-True ($baseline.ExitCode -eq 0) "Finalize baseline initialization failed."

        $paths = @("src/feature.py", "docs/feature.md", "tests/test_feature.py", "tools/check.ps1")
        foreach ($path in $paths) {
            $absolute = Join-Path $repo $path
            New-Item -ItemType Directory -Path (Split-Path -Parent $absolute) -Force | Out-Null
            [System.IO.File]::WriteAllText($absolute, "$path`n", [System.Text.UTF8Encoding]::new($false))
        }
        $attributed = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "baseline", "attribute", $paths[0], $paths[1], $paths[2], $paths[3], "--session-id", "session-a"
        )
        Assert-True ($attributed.ExitCode -eq 0) "Finalize attribution failed: $($attributed.Output)"
        $completed = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "session", "set-status", "completed", "--session-id", "session-a"
        )
        Assert-True ($completed.ExitCode -eq 0) "Finalize Session completion failed."
        Assert-True ((& git -C $repo rev-parse HEAD).Trim() -eq $before) "Completion created an implicit commit."

        $finalized = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "finalize", "--commit", "--session-id", "session-a",
            "--message", "feat(test): finalize owned files",
            "--path", $paths[0], "--path", $paths[1], "--path", $paths[2], "--path", $paths[3]
        )
        Assert-True ($finalized.ExitCode -eq 0) "Explicit finalize failed: $($finalized.Output)"
        $after = (& git -C $repo rev-parse HEAD).Trim()
        Assert-True ($after -ne $before) "Explicit finalize did not create a commit."
        $committed = @(& git -C $repo show --pretty= --name-only HEAD | Where-Object { $_ })
        $committedText = (($committed | Sort-Object) -join "`n")
        $expectedText = (($paths | Sort-Object) -join "`n")
        Assert-True ($committedText -eq $expectedText) "Finalize commit scope mismatch."
        $subject = (& git -C $repo log -1 --format=%s).Trim()
        Assert-True ($subject -eq "feat(test): finalize owned files") "Finalize commit message mismatch."
        Assert-True ($subject -notmatch '\[zircon-session:') "Finalize introduced a forbidden Session tag."

        $stopped = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("stop")
        Assert-True ($stopped.ExitCode -eq 0) "Coordinator stop failed."
        $process.WaitForExit(5000) | Out-Null
        Assert-True $process.HasExited "Coordinator process remained alive after finalize smoke."
        Write-Host "PASS: explicit finalize temporary-repository smoke"
    }
    finally {
        if ($null -ne $process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $testRoot) {
            Remove-Item -LiteralPath $testRoot -Recurse -Force
        }
    }
}

function Test-LegacyRollout {
    $testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("zircon-coordinator-legacy-" + [guid]::NewGuid().ToString("N"))
    $repo = Join-Path $testRoot "repo"
    New-Item -ItemType Directory -Path $repo -Force | Out-Null
    $process = $null
    $oldMaintenanceToken = $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN
    try {
        & git -C $repo init -q
        & git -C $repo config user.email "coordinator-smoke@example.invalid"
        & git -C $repo config user.name "Coordinator Smoke"
        & git -C $repo config core.autocrlf false
        & git -C $repo branch -M main
        [System.IO.File]::WriteAllText(
            (Join-Path $repo "README.md"), "baseline`n", [System.Text.UTF8Encoding]::new($false)
        )
        & git -C $repo add README.md
        & git -C $repo commit -q -m "test: baseline"
        $sessionRoot = Join-Path $repo ".codex\sessions"
        New-Item -ItemType Directory -Path $sessionRoot -Force | Out-Null
        $legacyNote = Join-Path $sessionRoot "legacy-old.md"
        [System.IO.File]::WriteAllText(
            $legacyNote,
            "---`nsession: legacy-old`nstatus: blocked`n---`n`n# Legacy`n",
            [System.Text.UTF8Encoding]::new($false)
        )
        [System.IO.File]::SetLastWriteTimeUtc($legacyNote, [DateTime]::UtcNow.AddDays(-2))

        $oldPythonPath = $env:PYTHONPATH
        $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN = [guid]::NewGuid().ToString("N")
        $env:PYTHONPATH = $sourceRoot
        $process = Start-Process -FilePath $python `
            -ArgumentList @("-m", "tools.session_coordinator", "--repo-root", $repo, "serve") `
            -WorkingDirectory $repo -WindowStyle Hidden -PassThru
        $env:PYTHONPATH = $oldPythonPath
        for ($attempt = 0; $attempt -lt 100; $attempt++) {
            Start-Sleep -Milliseconds 100
            $status = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("status")
            if ($status.ExitCode -eq 0) { break }
        }
        Assert-True ($status.ExitCode -eq 0) "Legacy rollout service did not become healthy."
        $baseline = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("baseline", "init")
        Assert-True ($baseline.ExitCode -eq 0) "Legacy rollout baseline init failed."

        $reportOne = Join-Path $testRoot "report-one.json"
        $reportTwo = Join-Path $testRoot "report-two.json"
        $first = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "legacy", "report", "--report", $reportOne
        )
        $second = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "legacy", "report", "--report", $reportTwo
        )
        Assert-True ($first.ExitCode -eq 0 -and $second.ExitCode -eq 0) "Legacy report failed."
        Assert-True ((Get-FileHash $reportOne).Hash -eq (Get-FileHash $reportTwo).Hash) "Legacy report was not repeatable."

        $imported = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "legacy", "import", "--apply"
        )
        Assert-True ($imported.ExitCode -eq 0) "Legacy import failed: $($imported.Output)"
        $archived = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "legacy", "archive", "--apply"
        )
        Assert-True ($archived.ExitCode -eq 0) "Legacy archive failed: $($archived.Output)"
        Assert-True (Test-Path (Join-Path $sessionRoot "archive\legacy-old.md")) "Legacy note was not hash-preservingly archived."

        foreach ($tick in 1..2) {
            $maintenance = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
                "maintenance", "tick"
            )
            Assert-True ($maintenance.ExitCode -eq 0) "Maintenance tick $tick failed: $($maintenance.Output)"
        }
        $auditPath = Join-Path $testRoot "audit.json"
        $audit = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @(
            "audit", "all", "--report", $auditPath
        )
        Assert-True ($audit.ExitCode -eq 0) "Audit all failed: $($audit.Output)"
        $auditJson = Get-Content -Raw -LiteralPath $auditPath | ConvertFrom-Json
        Assert-True ($auditJson.audit.maintenance_tick_count -eq 2) "Audit did not observe two maintenance ticks."
        Assert-True ($auditJson.audit.invalid_session_statuses.Count -eq 0) "Audit found a non-enum Session status."

        $stopped = Invoke-PythonCoordinator -RepoRoot $repo -CommandArguments @("stop")
        Assert-True ($stopped.ExitCode -eq 0) "Legacy rollout coordinator stop failed."
        $process.WaitForExit(5000) | Out-Null
        Assert-True $process.HasExited "Legacy rollout process remained alive."
        Write-Host "PASS: legacy migration and rollout smoke"
    }
    finally {
        $env:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN = $oldMaintenanceToken
        if ($null -ne $process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $testRoot) {
            Remove-Item -LiteralPath $testRoot -Recurse -Force
        }
    }
}

$runAll = -not $KernelOnly -and -not $LeaseAndPatch -and -not $CargoAndCleanup -and `
    -not $FinalizeInTempRepo -and -not $LegacyRollout

if ($KernelOnly -or $runAll) {
    Test-Kernel
}

if ($LeaseAndPatch -or $runAll) {
    Test-LeaseAndPatch
}

if ($CargoAndCleanup -or $runAll) {
    Test-CargoAndCleanup
}

if ($FinalizeInTempRepo -or $runAll) {
    Test-FinalizeInTempRepo
}

if ($LegacyRollout -or $runAll) {
    Test-LegacyRollout
}
