[CmdletBinding()]
param(
    [switch]$KernelOnly,
    [switch]$LeaseAndPatch,
    [switch]$CargoAndCleanup,
    [switch]$FinalizeInTempRepo
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

if ($KernelOnly -or (-not $LeaseAndPatch -and -not $CargoAndCleanup -and -not $FinalizeInTempRepo)) {
    Test-Kernel
}

if ($LeaseAndPatch) {
    Test-LeaseAndPatch
}

if ($CargoAndCleanup) {
    throw "CargoAndCleanup belongs to milestone M4 and is not available during M1-M2."
}

if ($FinalizeInTempRepo) {
    throw "FinalizeInTempRepo belongs to milestone M5 and is not available during M1-M2."
}
