[CmdletBinding()]
param(
    [switch]$ReadOnlyConsole,
    [switch]$ControlledActions,
    [switch]$TrayLifecycle
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$selected = @($ReadOnlyConsole, $ControlledActions, $TrayLifecycle).Where({ $_ }).Count
if ($selected -ne 1) {
    throw 'Select exactly one acceptance gate: -ReadOnlyConsole, -ControlledActions, or -TrayLifecycle.'
}

Push-Location $repoRoot
try {
    if ($TrayLifecycle) {
        if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
            throw 'Tray lifecycle smoke requires a coordinator-managed CARGO_TARGET_DIR.'
        }
        $tray = Join-Path $env:CARGO_TARGET_DIR 'debug\zircon-session-tray.exe'
        if (-not (Test-Path -LiteralPath $tray -PathType Leaf)) {
            throw "Built tray executable is missing: $tray"
        }
        $fixture = Join-Path ([System.IO.Path]::GetTempPath()) ("zircon-tray-smoke-{0}" -f [guid]::NewGuid().ToString('N'))
        $fixtureState = Join-Path $fixture '.codex\state\session-coordinator'
        $daemon = $null
        $trayProcess = $null
        try {
            New-Item -ItemType Directory -Path (Join-Path $fixture 'tools') -Force | Out-Null
            Set-Content -LiteralPath (Join-Path $fixture 'README.md') -Value 'tray smoke fixture' -Encoding UTF8
            Set-Content -LiteralPath (Join-Path $fixture 'tools\zircon-session.ps1') -Value '# smoke marker' -Encoding UTF8
            git -C $fixture init -q -b main
            git -C $fixture config user.email zircon-smoke@example.invalid
            git -C $fixture config user.name ZirconSmoke
            git -C $fixture add README.md tools/zircon-session.ps1
            git -C $fixture commit -q -m 'chore: initialize tray smoke fixture'
            $daemonOut = Join-Path $fixture 'daemon.out.log'
            $daemonErr = Join-Path $fixture 'daemon.err.log'
            $daemon = Start-Process -FilePath python -ArgumentList @(
                '-m', 'tools.session_coordinator', '--repo-root', $fixture,
                '--state-root', $fixtureState, 'serve'
            ) -WorkingDirectory $repoRoot -RedirectStandardOutput $daemonOut -RedirectStandardError $daemonErr -PassThru -WindowStyle Hidden
            $runtime = Join-Path $fixtureState 'runtime.json'
            $deadline = [DateTime]::UtcNow.AddSeconds(20)
            while (-not (Test-Path -LiteralPath $runtime) -and [DateTime]::UtcNow -lt $deadline) {
                Start-Sleep -Milliseconds 100
            }
            if (-not (Test-Path -LiteralPath $runtime)) {
                throw "Fixture coordinator did not start: $(Get-Content -LiteralPath $daemonErr -Raw -ErrorAction SilentlyContinue)"
            }
            $descriptor = Get-Content -LiteralPath $runtime -Raw | ConvertFrom-Json
            if ($descriptor.descriptor_version -ne 2) {
                throw 'Fixture coordinator did not publish runtime descriptor v2.'
            }
            $trayProcess = Start-Process -FilePath $tray -ArgumentList @('--repo-root', $fixture) -PassThru -WindowStyle Hidden
            Start-Sleep -Seconds 3
            if ($trayProcess.HasExited) {
                throw 'Tray exited before completing authenticated observation.'
            }
            $daemonPid = [int]$descriptor.pid
            Stop-Process -Id $trayProcess.Id -Force
            $trayProcess.WaitForExit(5000) | Out-Null
            if (-not (Get-Process -Id $daemonPid -ErrorAction SilentlyContinue)) {
                throw 'Exiting the tray stopped the coordinator daemon.'
            }
            python -m tools.session_coordinator --repo-root $fixture --state-root $fixtureState stop
            if ($LASTEXITCODE -ne 0) {
                throw 'Controlled fixture stop failed.'
            }
            $daemon.WaitForExit(10000) | Out-Null
            if (-not $daemon.HasExited) {
                throw 'Fixture coordinator did not stop after the controlled action.'
            }
            $automatic = Start-Process -FilePath python -ArgumentList @(
                '-m', 'tools.session_coordinator', '--repo-root', $fixture,
                '--state-root', $fixtureState, 'serve', '--automatic-start'
            ) -WorkingDirectory $repoRoot -PassThru -WindowStyle Hidden
            $automatic.WaitForExit(10000) | Out-Null
            if (-not $automatic.HasExited) {
                Stop-Process -Id $automatic.Id -Force
                throw 'Explicit stop did not suppress automatic restart.'
            }
            Write-Host 'tray lifecycle smoke passed'
        }
        finally {
            if ($trayProcess -and -not $trayProcess.HasExited) {
                Stop-Process -Id $trayProcess.Id -Force -ErrorAction SilentlyContinue
            }
            if ($daemon -and -not $daemon.HasExited) {
                Stop-Process -Id $daemon.Id -Force -ErrorAction SilentlyContinue
            }
            if (Test-Path -LiteralPath $fixture) {
                $resolvedFixture = (Resolve-Path -LiteralPath $fixture).Path
                $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\')
                if (-not $resolvedFixture.StartsWith($tempRoot + '\', [System.StringComparison]::OrdinalIgnoreCase) -or
                    -not (Split-Path -Leaf $resolvedFixture).StartsWith('zircon-tray-smoke-', [System.StringComparison]::Ordinal)) {
                    throw "Refusing to remove unexpected tray fixture path: $resolvedFixture"
                }
                Remove-Item -LiteralPath $resolvedFixture -Recurse -Force
            }
        }
    }
    else {
        $gate = if ($ControlledActions) { '--controlled-actions' } else { '--read-only-console' }
        python -m tools.tests.workflow_control_center_smoke --repo-root $repoRoot $gate
        if ($LASTEXITCODE -ne 0) {
            throw "Workflow control-center smoke failed with exit code $LASTEXITCODE"
        }
    }
}
finally {
    Pop-Location
}
