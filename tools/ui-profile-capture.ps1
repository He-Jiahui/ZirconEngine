param(
    [string]$Scenario = "manual",
    [string[]]$ScenarioList = @(),
    [switch]$AllUiScenarios,
    [string]$OutputRoot = "target/zircon-profiles",
    [switch]$SkipBuild,
    [switch]$UseTracy,
    [switch]$UseWpr,
    [int]$AutoCloseSeconds = 0,
    [switch]$AutoInteract,
    [switch]$RequireScenarioEvidence,
    [int]$MaxFrames = 2048,
    [int]$MaxSpans = 65536,
    [int]$MaxCounters = 65536
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$OutputPath = Join-Path $RepoRoot $OutputRoot

function Resolve-ProfilingTargetDir {
    if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        $cargoTarget = $env:CARGO_TARGET_DIR
        if (-not [System.IO.Path]::IsPathRooted($cargoTarget)) {
            $cargoTarget = Join-Path $RepoRoot $cargoTarget
        }
        return Join-Path $cargoTarget "profiling"
    }
    return Join-Path $RepoRoot "target\profiling"
}

$TargetDir = Resolve-ProfilingTargetDir
$EditorExe = Join-Path $TargetDir "zircon_editor.exe"
$RuntimeDll = Join-Path $TargetDir "zircon_runtime.dll"
$TracyProfiler = Join-Path $RepoRoot "dev\tracy\tracy-profiler.exe"

function Resolve-CaptureScenarios {
    if ($AllUiScenarios) {
        return @(
            "startup",
            "idle_hover",
            "click",
            "drag",
            "drawer_resize",
            "asset_refresh",
            "viewport_image"
        )
    }
    if ($ScenarioList.Count -gt 0) {
        return $ScenarioList
    }
    return @($Scenario)
}

function Get-ScenarioInstruction {
    param([string]$Name)
    switch ($Name) {
        "startup" { return "Launch, wait until the first editor frame is stable, then close the editor." }
        "idle_hover" { return "Move the pointer slowly across toolbar, hierarchy rows, inspector fields, and tabs for several seconds, then close." }
        "click" { return "Click toolbar buttons, hierarchy rows, tabs, and inspector controls, then close." }
        "drag" { return "Drag selection or draggable editor controls where available, then close." }
        "drawer_resize" { return "Drag side or bottom pane splitters repeatedly, then close." }
        "asset_refresh" { return "Trigger an asset refresh or reopen the project/asset pane, then close." }
        "viewport_image" { return "Let the scene/game viewport update for several seconds, orbit if useful, then close." }
        default { return "Perform the target interaction, then close the editor to export the report." }
    }
}

function Show-ProfileSummary {
    param([string]$ProfileDir)
    $summary = Join-Path $ProfileDir "summary.md"
    if (-not (Test-Path $summary)) {
        return
    }
    Write-Host ""
    Write-Host "Summary excerpt:"
    Get-Content -Path $summary |
        Select-String -Pattern "First Fix Candidates|UI Alerts|^- " -Context 0,0 |
        Select-Object -First 24 |
        ForEach-Object { Write-Host $_.Line }
}

function Show-UiScenarioEvidence {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $hotspots = Join-Path $ProfileDir "ui_hotspots.json"
    if (-not (Test-Path $hotspots)) {
        Write-Warning "UI hotspot evidence was not exported for scenario '$ScenarioName'."
        return $false
    }

    $report = Get-Content -Path $hotspots -Raw | ConvertFrom-Json
    $scenario = $report.scenarios |
        Where-Object { $_.scenario -eq $ScenarioName } |
        Select-Object -First 1
    if ($null -eq $scenario) {
        Write-Warning "UI hotspot report does not contain scenario '$ScenarioName'."
        return $false
    }

    Write-Host ""
    Write-Host "UI scenario evidence:"
    Write-Host ("- frames={0} dirty_paint_only={1} redraw_region={2} redraw_full_frame={3}" -f `
            $scenario.frame_count,
            $scenario.dirty_paint_only_count,
            $scenario.redraw_region_count,
            $scenario.redraw_full_frame_count)
    Write-Host ("- gpu_draw_calls={0} gpu_visible_commands={1} gpu_visible_draw_items={2} gpu_batch_layers={3} gpu_batch_dependencies={4}" -f `
            $scenario.gpu_draw_calls,
            $scenario.gpu_visible_commands,
            $scenario.gpu_visible_draw_items,
            $scenario.gpu_batch_layers,
            $scenario.gpu_batch_dependencies)
    Write-Host ("- gpu_upload_bytes={0}" -f $scenario.gpu_upload_bytes)
    Write-Host ("- alerts={0}" -f @($report.alerts).Count)

    $redrawCount = [int64]$scenario.redraw_region_count + [int64]$scenario.redraw_full_frame_count
    $hasGpuBatch = [int64]$scenario.gpu_draw_calls -gt 0 -and
        [int64]$scenario.gpu_visible_draw_items -gt [int64]$scenario.gpu_draw_calls
    $hasGpuUpload = [int64]$scenario.gpu_upload_bytes -gt 0
    $hasFrameSamples = [int64]$scenario.frame_count -gt 0

    $evidenceOk = switch ($ScenarioName) {
        "startup" { $hasGpuBatch; break }
        "idle_hover" { $redrawCount -gt 0 -and $hasGpuBatch; break }
        "click" { $redrawCount -gt 0 -and $hasGpuBatch; break }
        "drag" { $redrawCount -gt 0 -and $hasGpuBatch; break }
        "drawer_resize" { $redrawCount -gt 0 -and $hasGpuBatch; break }
        "viewport_image" {
            [int64]$scenario.dirty_paint_only_count -gt 0 -and
                [int64]$scenario.redraw_region_count -gt 0 -and
                $hasGpuUpload -and
                $hasGpuBatch
            break
        }
        default { $hasFrameSamples -or $hasGpuBatch; break }
    }

    if (-not $evidenceOk) {
        Write-Warning "Scenario '$ScenarioName' did not produce enough UI/GPU evidence for automated acceptance."
    }
    elseif ($ScenarioName -eq "idle_hover" -and $redrawCount -eq 0) {
        Write-Warning "Scenario 'idle_hover' recorded pointer frames but no hover redraw; treat it as event-path evidence, not GPU patch evidence."
    }

    return $evidenceOk
}

function Wait-EditorMainWindow {
    param(
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutSeconds
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) {
            return $false
        }
        $Process.Refresh()
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) {
            return $true
        }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Initialize-CaptureInputApi {
    if ("ZirconProfileCaptureNative" -as [type]) {
        return
    }

    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
public struct ZirconProfileCaptureRect
{
    public int Left;
    public int Top;
    public int Right;
    public int Bottom;
}

[StructLayout(LayoutKind.Sequential)]
public struct ZirconProfileCapturePoint
{
    public int X;
    public int Y;
}

public static class ZirconProfileCaptureNative
{
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out ZirconProfileCaptureRect rect);

    [DllImport("user32.dll")]
    public static extern bool GetClientRect(IntPtr hWnd, out ZirconProfileCaptureRect rect);

    [DllImport("user32.dll")]
    public static extern bool ClientToScreen(IntPtr hWnd, ref ZirconProfileCapturePoint point);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);

    [DllImport("user32.dll")]
    public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extraInfo);
}
"@
}

function Get-EditorWindowRect {
    param([System.Diagnostics.Process]$Process)

    $Process.Refresh()
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) {
        return $null
    }

    Initialize-CaptureInputApi
    $client = New-Object ZirconProfileCaptureRect
    $origin = New-Object ZirconProfileCapturePoint
    if ([ZirconProfileCaptureNative]::GetClientRect($Process.MainWindowHandle, [ref]$client) -and
        [ZirconProfileCaptureNative]::ClientToScreen($Process.MainWindowHandle, [ref]$origin)) {
        $rect = New-Object ZirconProfileCaptureRect
        $rect.Left = $origin.X
        $rect.Top = $origin.Y
        $rect.Right = $origin.X + $client.Right
        $rect.Bottom = $origin.Y + $client.Bottom
        if ($rect.Right -gt $rect.Left -and $rect.Bottom -gt $rect.Top) {
            return $rect
        }
    }

    $windowRect = New-Object ZirconProfileCaptureRect
    if (-not [ZirconProfileCaptureNative]::GetWindowRect($Process.MainWindowHandle, [ref]$windowRect)) {
        return $null
    }
    if ($windowRect.Right -le $windowRect.Left -or $windowRect.Bottom -le $windowRect.Top) {
        return $null
    }
    return $windowRect
}

function Get-CapturePoint {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [double]$XRatio,
        [double]$YRatio
    )

    $width = [Math]::Max(1, $Rect.Right - $Rect.Left)
    $height = [Math]::Max(1, $Rect.Bottom - $Rect.Top)
    [pscustomobject]@{
        X = [int]($Rect.Left + [Math]::Round($width * $XRatio))
        Y = [int]($Rect.Top + [Math]::Round($height * $YRatio))
    }
}

function Move-CaptureCursor {
    param(
        [pscustomobject]$Point,
        [int]$DelayMs = 120
    )

    [ZirconProfileCaptureNative]::SetCursorPos($Point.X, $Point.Y) | Out-Null
    Start-Sleep -Milliseconds $DelayMs
}

function Click-CapturePoint {
    param([pscustomobject]$Point)

    Move-CaptureCursor -Point $Point -DelayMs 120
    [ZirconProfileCaptureNative]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 80
    [ZirconProfileCaptureNative]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 160
}

function Drag-CapturePath {
    param([pscustomobject[]]$Points)

    if ($Points.Count -eq 0) {
        return
    }
    Move-CaptureCursor -Point $Points[0] -DelayMs 120
    if ($Points.Count -eq 1) {
        return
    }
    [ZirconProfileCaptureNative]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 120
    foreach ($point in $Points[1..($Points.Count - 1)]) {
        Move-CaptureCursor -Point $point -DelayMs 120
    }
    [ZirconProfileCaptureNative]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 160
}

function Invoke-AutoScenarioInteraction {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$ScenarioName
    )

    if (-not $AutoInteract) {
        return
    }

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    if ($normalizedScenario -in @("manual", "startup", "")) {
        return
    }

    $rect = Get-EditorWindowRect -Process $Process
    if ($null -eq $rect) {
        Write-Warning "Auto interaction skipped because the editor window rectangle was unavailable."
        return
    }

    [ZirconProfileCaptureNative]::SetForegroundWindow($Process.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 300

    switch ($normalizedScenario) {
        "idle_hover" {
            Click-CapturePoint -Point (Get-CapturePoint -Rect $rect -XRatio 0.02 -YRatio 0.18)
            Start-Sleep -Milliseconds 250
            $points = @(
                (Get-CapturePoint -Rect $rect -XRatio 0.04 -YRatio 0.04),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.14),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.22),
                (Get-CapturePoint -Rect $rect -XRatio 0.12 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.34 -YRatio 0.20),
                (Get-CapturePoint -Rect $rect -XRatio 0.82 -YRatio 0.55),
                (Get-CapturePoint -Rect $rect -XRatio 0.38 -YRatio 0.72)
            )
            foreach ($point in $points) {
                Move-CaptureCursor -Point $point -DelayMs 180
            }
        }
        "click" {
            foreach ($point in @(
                (Get-CapturePoint -Rect $rect -XRatio 0.12 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.30 -YRatio 0.16),
                (Get-CapturePoint -Rect $rect -XRatio 0.52 -YRatio 0.18)
            )) {
                Click-CapturePoint -Point $point
            }
        }
        "drag" {
            Drag-CapturePath -Points @(
                (Get-CapturePoint -Rect $rect -XRatio 0.44 -YRatio 0.46),
                (Get-CapturePoint -Rect $rect -XRatio 0.50 -YRatio 0.48),
                (Get-CapturePoint -Rect $rect -XRatio 0.56 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.62 -YRatio 0.52)
            )
        }
        "drawer_resize" {
            Drag-CapturePath -Points @(
                (Get-CapturePoint -Rect $rect -XRatio 0.26 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.30 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.34 -YRatio 0.50)
            )
        }
        "viewport_image" {
            Drag-CapturePath -Points @(
                (Get-CapturePoint -Rect $rect -XRatio 0.52 -YRatio 0.44),
                (Get-CapturePoint -Rect $rect -XRatio 0.56 -YRatio 0.46),
                (Get-CapturePoint -Rect $rect -XRatio 0.60 -YRatio 0.48),
                (Get-CapturePoint -Rect $rect -XRatio 0.54 -YRatio 0.50)
            )
        }
        default {
            Move-CaptureCursor -Point (Get-CapturePoint -Rect $rect -XRatio 0.50 -YRatio 0.50) -DelayMs 200
        }
    }
}

function Resolve-EditorCaptureArguments {
    param(
        [string]$ScenarioName,
        [string]$SessionId
    )

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    if ($normalizedScenario -notin @("idle_hover", "viewport_image")) {
        return @()
    }

    $projectLocation = Join-Path $RepoRoot (Join-Path "target\zircon-profile-projects" $SessionId)
    New-Item -ItemType Directory -Force -Path $projectLocation | Out-Null
    return @(
        "--create-project",
        "--project-name",
        "ProfileCaptureProject",
        "--location",
        $projectLocation,
        "--template",
        "renderable-empty"
    )
}

function Invoke-EditorCapture {
    param(
        [string]$ScenarioName,
        [string]$SessionId
    )

    $editorArguments = Resolve-EditorCaptureArguments -ScenarioName $ScenarioName -SessionId $SessionId

    if ($AutoCloseSeconds -le 0) {
        & $EditorExe @editorArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Editor exited with code $LASTEXITCODE"
        }
        return
    }

    $startProcessArgs = @{
        FilePath = $EditorExe
        WorkingDirectory = $RepoRoot
        PassThru = $true
    }
    if ($editorArguments.Count -gt 0) {
        $startProcessArgs.ArgumentList = $editorArguments
    }
    if (-not [string]::IsNullOrWhiteSpace($SessionId)) {
        $profileDir = Join-Path $OutputPath $SessionId
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $startProcessArgs.RedirectStandardOutput = Join-Path $profileDir "editor.stdout.log"
        $startProcessArgs.RedirectStandardError = Join-Path $profileDir "editor.stderr.log"
    }
    $process = Start-Process @startProcessArgs
    $windowTimeout = [Math]::Max(30, $AutoCloseSeconds)
    if (-not (Wait-EditorMainWindow -Process $process -TimeoutSeconds $windowTimeout)) {
        if ($process.HasExited) {
            if ($process.ExitCode -ne 0) {
                throw "Editor exited with code $($process.ExitCode)"
            }
            return
        }
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        throw "Editor did not create a main window before the auto-close timeout."
    }

    Write-Host "Auto-close timer started after the editor main window became available."
    if ($AutoInteract) {
        Start-Sleep -Milliseconds 900
    }
    Invoke-AutoScenarioInteraction -Process $process -ScenarioName $ScenarioName
    if ($process.WaitForExit($AutoCloseSeconds * 1000)) {
        if ($process.ExitCode -ne 0) {
            throw "Editor exited with code $($process.ExitCode)"
        }
        return
    }

    Write-Host "Auto-close requested after $AutoCloseSeconds second(s)."
    $process.Refresh()
    $closed = $false
    try {
        $closed = $process.CloseMainWindow()
    }
    catch {
        $closed = $false
    }

    if ($closed -and $process.WaitForExit(15000)) {
        if ($process.ExitCode -ne 0) {
            throw "Editor exited with code $($process.ExitCode)"
        }
        return
    }

    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    throw "Editor did not exit after the auto-close request; the process was stopped before profiling could export normally."
}

$features = "target-editor-host profiling profiling-chrome"
if ($UseTracy) {
    $features = "$features profiling-tracy"
}

if (-not $SkipBuild) {
    Push-Location $RepoRoot
    try {
        cargo build -p zircon_runtime --lib --profile profiling --features $features --locked
        cargo build -p zircon_app --bin zircon_editor --profile profiling --features $features --locked
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path $EditorExe)) {
    throw "Missing profiling editor executable: $EditorExe"
}
if (-not (Test-Path $RuntimeDll)) {
    throw "Missing profiling runtime library: $RuntimeDll"
}

New-Item -ItemType Directory -Force -Path $OutputPath | Out-Null

$previous = @{
    ZIRCON_PROFILE_CAPTURE = $env:ZIRCON_PROFILE_CAPTURE
    ZIRCON_PROFILE_SESSION = $env:ZIRCON_PROFILE_SESSION
    ZIRCON_PROFILE_OUTPUT_ROOT = $env:ZIRCON_PROFILE_OUTPUT_ROOT
    ZIRCON_PROFILE_MAX_FRAMES = $env:ZIRCON_PROFILE_MAX_FRAMES
    ZIRCON_PROFILE_MAX_SPANS = $env:ZIRCON_PROFILE_MAX_SPANS
    ZIRCON_PROFILE_MAX_COUNTERS = $env:ZIRCON_PROFILE_MAX_COUNTERS
    ZIRCON_RUNTIME_LIBRARY = $env:ZIRCON_RUNTIME_LIBRARY
}

if ($UseTracy -and (Test-Path $TracyProfiler)) {
    Start-Process -FilePath $TracyProfiler | Out-Null
}

try {
    foreach ($scenarioName in (Resolve-CaptureScenarios)) {
        $SessionId = "$(Get-Date -Format 'yyyyMMdd-HHmmss')-$scenarioName"
        $ProfileDir = Join-Path $OutputPath $SessionId
        $wprStarted = $false

        $env:ZIRCON_PROFILE_CAPTURE = "1"
        $env:ZIRCON_PROFILE_SESSION = $SessionId
        $env:ZIRCON_PROFILE_OUTPUT_ROOT = $OutputPath
        $env:ZIRCON_PROFILE_MAX_FRAMES = "$MaxFrames"
        $env:ZIRCON_PROFILE_MAX_SPANS = "$MaxSpans"
        $env:ZIRCON_PROFILE_MAX_COUNTERS = "$MaxCounters"
        $env:ZIRCON_RUNTIME_LIBRARY = $RuntimeDll

        if ($UseWpr) {
            $wpr = Get-Command wpr.exe -ErrorAction SilentlyContinue
            if ($wpr) {
                & $wpr.Source -start CPU -filemode
                $wprStarted = $true
            }
            else {
                Write-Warning "wpr.exe was not found; continuing without ETL capture."
            }
        }

        try {
            Write-Host ""
            Write-Host "Profiling session: $SessionId"
            Write-Host "Scenario: $scenarioName"
            Write-Host (Get-ScenarioInstruction $scenarioName)
            Invoke-EditorCapture -ScenarioName $scenarioName -SessionId $SessionId
        }
        finally {
            if ($wprStarted) {
                New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
                wpr.exe -stop (Join-Path $ProfileDir "system.etl") | Out-Null
            }
        }

        if (Test-Path $ProfileDir) {
            Write-Host "Profile report: $ProfileDir"
            Write-Host "Open timeline.perfetto.json in Perfetto/Chrome trace, and ui_hotspots.json for UI slow-path alerts."
            Show-ProfileSummary $ProfileDir
            $scenarioEvidenceOk = Show-UiScenarioEvidence -ProfileDir $ProfileDir -ScenarioName $scenarioName
            if ($RequireScenarioEvidence -and -not $scenarioEvidenceOk) {
                throw "Scenario '$scenarioName' did not meet the requested evidence gate."
            }
        }
        else {
            Write-Warning "Profile directory was not created. Check whether the editor exited normally and profiling features were enabled."
        }
    }
}
finally {
    foreach ($key in $previous.Keys) {
        if ($null -eq $previous[$key]) {
            Remove-Item "Env:\$key" -ErrorAction SilentlyContinue
        }
        else {
            Set-Item "Env:\$key" $previous[$key]
        }
    }
}
