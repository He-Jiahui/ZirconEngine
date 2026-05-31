param(
    [string]$Scenario = "manual",
    [string[]]$ScenarioList = @(),
    [switch]$AllUiScenarios,
    [string]$OutputRoot = "target/zircon-profiles",
    [switch]$SkipBuild,
    [switch]$CaptureSoftbufferScreenshot,
    [switch]$UseTracy,
    [switch]$UseWpr,
    [int]$AutoCloseSeconds = 0,
    [switch]$AutoInteract,
    [switch]$RequireScenarioEvidence,
    [int]$MaxFrames = 2048,
    [int]$MaxSpans = 65536,
    [int]$MaxCounters = 65536,
    [double]$ScreenshotDiffMaxDifferentSampleRatio = 0.25,
    [double]$ScreenshotDiffMaxAverageChannelDelta = 10.0
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
$script:LastInteractionEvidence = $null

function Expand-CaptureScenarioNames {
    param([string[]]$Names)

    $expanded = @()
    foreach ($name in $Names) {
        foreach ($part in ($name -split ",")) {
            $trimmed = $part.Trim()
            if (-not [string]::IsNullOrWhiteSpace($trimmed)) {
                $expanded += $trimmed
            }
        }
    }
    return $expanded
}

function Resolve-CaptureScenarios {
    if ($AllUiScenarios) {
        return @(
            "startup",
            "material_lab_startup",
            "material_lab_hover",
            "material_lab_click",
            "idle_hover",
            "click",
            "drag",
            "drawer_resize",
            "asset_refresh",
            "viewport_image"
        )
    }
    if ($ScenarioList.Count -gt 0) {
        return Expand-CaptureScenarioNames -Names $ScenarioList
    }
    return Expand-CaptureScenarioNames -Names @($Scenario)
}

function Get-ScenarioInstruction {
    param([string]$Name)
    switch ($Name) {
        "startup" { return "Launch, wait until the first editor frame is stable, then close the editor." }
        "material_lab_startup" { return "Launch the Material Component Lab, wait until the first frame is stable, then close the editor." }
        "material_lab_hover" { return "Launch the Material Component Lab, move the pointer across prototype controls, then close." }
        "material_lab_click" { return "Launch the Material Component Lab, click representative prototype controls, then close." }
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

    $evidenceScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    $hotspots = Join-Path $ProfileDir "ui_hotspots.json"
    if (-not (Test-Path $hotspots)) {
        Write-Warning "UI hotspot evidence was not exported for scenario '$ScenarioName'."
        return $false
    }

    $report = Get-Content -Path $hotspots -Raw | ConvertFrom-Json
    $scenario = $report.scenarios |
        Where-Object { $_.scenario -eq $evidenceScenario } |
        Select-Object -First 1
    if ($null -eq $scenario) {
        Write-Warning "UI hotspot report does not contain scenario '$evidenceScenario' for requested scenario '$ScenarioName'."
        return $false
    }

    Write-Host ""
    Write-Host "UI scenario evidence ($evidenceScenario):"
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
    Write-Host ("- software_fallback_present_count={0}" -f $scenario.software_fallback_present_count)
    $scenarioAlerts = @($report.alerts | Where-Object { $_.scenario -eq $evidenceScenario })
    $blockingScenarioAlerts = @($scenarioAlerts | Where-Object {
            -not ($evidenceScenario -eq "drawer_resize" -and $_.rule -eq "resize_triggered_slow_path_rebuild")
        })
    Write-Host ("- alerts={0} blocking_alerts={1}" -f $scenarioAlerts.Count, $blockingScenarioAlerts.Count)
    if ($scenarioAlerts.Count -gt $blockingScenarioAlerts.Count) {
        Write-Host "- non_blocking_alerts=resize_triggered_slow_path_rebuild"
    }

    $redrawCount = [int64]$scenario.redraw_region_count + [int64]$scenario.redraw_full_frame_count
    $hasGpuBatch = [int64]$scenario.gpu_draw_calls -gt 0 -and
        [int64]$scenario.gpu_visible_draw_items -gt [int64]$scenario.gpu_draw_calls
    $hasGpuUpload = [int64]$scenario.gpu_upload_bytes -gt 0
    $hasFrameSamples = [int64]$scenario.frame_count -gt 0
    $hasNoSoftwareFallback = [int64]$scenario.software_fallback_present_count -eq 0
    $hasNoAlerts = $blockingScenarioAlerts.Count -eq 0

    $evidenceOk = switch ($evidenceScenario) {
        "startup" { $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "idle_hover" { $hasFrameSamples -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "click" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "drag" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "drawer_resize" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "viewport_image" {
            [int64]$scenario.dirty_paint_only_count -gt 0 -and
                [int64]$scenario.redraw_region_count -gt 0 -and
                $hasGpuUpload -and
                $hasGpuBatch -and
                $hasNoSoftwareFallback -and
                $hasNoAlerts
            break
        }
        "asset_refresh" { $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        default { ($hasFrameSamples -or $hasGpuBatch) -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
    }

    if (-not $evidenceOk) {
        Write-Warning "Scenario '$ScenarioName' did not produce enough UI/GPU evidence for automated acceptance."
    }
    elseif ($evidenceScenario -eq "idle_hover" -and $redrawCount -eq 0) {
        Write-Warning "Scenario '$ScenarioName' recorded pointer frames but no hover redraw; treat it as event-path evidence, not GPU patch evidence."
    }

    return $evidenceOk
}

function Resolve-InteractionScenarioName {
    param([string]$ScenarioName)
    switch ($ScenarioName.Trim().ToLowerInvariant()) {
        "material_lab_startup" { return "startup" }
        "material_lab_hover" { return "idle_hover" }
        "material_lab_click" { return "click" }
        default { return $ScenarioName.Trim().ToLowerInvariant() }
    }
}

function Test-EnvTruthy {
    param([string]$Name)
    $value = [Environment]::GetEnvironmentVariable($Name)
    return $value -in @("1", "true", "TRUE", "yes", "YES", "on", "ON")
}

function Assert-EditorProcessExitSucceeded {
    param([System.Diagnostics.Process]$Process)

    $Process.Refresh()
    if ($Process.HasExited -and $null -ne $Process.ExitCode -and $Process.ExitCode -ne 0) {
        throw "Editor exited with code $($Process.ExitCode)"
    }
}

function Export-UiBatchMetrics {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $hotspots = Join-Path $ProfileDir "ui_hotspots.json"
    if (-not (Test-Path $hotspots)) {
        return
    }
    $report = Get-Content -Path $hotspots -Raw | ConvertFrom-Json
    $metrics = @()
    foreach ($scenario in @($report.scenarios)) {
        $visibleItems = [double]$scenario.gpu_visible_draw_items
        $drawCalls = [double]$scenario.gpu_draw_calls
        $layers = [double]$scenario.gpu_batch_layers
        $dependencies = [double]$scenario.gpu_batch_dependencies
        $maxDependencies = if ($visibleItems -gt 1) { $visibleItems * ($visibleItems - 1.0) / 2.0 } else { 0.0 }
        $metrics += [pscustomobject]@{
            scenario = $scenario.scenario
            gpu_draw_calls = [int64]$scenario.gpu_draw_calls
            gpu_visible_draw_items = [int64]$scenario.gpu_visible_draw_items
            gpu_batch_layers = [int64]$scenario.gpu_batch_layers
            gpu_batch_dependencies = [int64]$scenario.gpu_batch_dependencies
            batch_success_rate = if ($visibleItems -gt 0.0) { 1.0 - ($drawCalls / $visibleItems) } else { 0.0 }
            draw_reduction_ratio = if ($drawCalls -gt 0.0) { $visibleItems / $drawCalls } else { 0.0 }
            dependency_density = if ($maxDependencies -gt 0.0) { $dependencies / $maxDependencies } else { 0.0 }
            layer_density = if ($layers -gt 0.0) { $visibleItems / $layers } else { 0.0 }
            ideal_case = "solid(0/1) + text(0/1) + image_resource_key_count per independent layer"
            worst_case = "all items overlap or cannot share material, draw calls approach visible draw items"
        }
    }

    $artifact = [pscustomobject]@{
        schema_version = 1
        source = "ui_hotspots.json"
        formulas = [pscustomobject]@{
            batch_success_rate = "1 - gpu_draw_calls / gpu_visible_draw_items"
            draw_reduction_ratio = "gpu_visible_draw_items / gpu_draw_calls"
            dependency_density = "gpu_batch_dependencies / (n * (n - 1) / 2)"
            layer_density = "gpu_visible_draw_items / gpu_batch_layers"
        }
        batching_model = [pscustomobject]@{
            partial_order = "stable z/index order is required only for clipped rectangles that intersect"
            list_batching = "disjoint list rows form a background layer and a text layer, so rows can batch by material instead of by item"
            ideal_case = "one solid draw plus one text draw plus one image draw per distinct resource key per independent layer"
            worst_case = "all items overlap, or every image has a distinct resource key, so draw calls approach visible draw items"
            clip_and_mask_policy = "rectangular command/surface/damage clips are CPU-trimmed; non-rectangular masks/stencil are not part of this path and must become a future explicit batch key or fallback"
        }
        scenarios = $metrics
    }
    $artifactPath = Join-Path $ProfileDir "ui_batch_metrics.json"
    $artifact | ConvertTo-Json -Depth 8 | Set-Content -Path $artifactPath -Encoding UTF8

    $evidenceScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    $current = $metrics | Where-Object { $_.scenario -eq $evidenceScenario } | Select-Object -First 1
    if ($null -ne $current) {
        Write-Host ("- batch_success_rate={0:N3} draw_reduction_ratio={1:N3} dependency_density={2:N3} layer_density={3:N3}" -f `
                $current.batch_success_rate,
                $current.draw_reduction_ratio,
                $current.dependency_density,
                $current.layer_density)
    }
}

function Export-UiHitConsistency {
    param([string]$ProfileDir)

    $geometryPath = Join-Path $ProfileDir "ui_profile_geometry.json"
    if (-not (Test-Path $geometryPath)) {
        return
    }
    $geometry = Get-Content -Path $geometryPath -Raw | ConvertFrom-Json
    $frames = @{}
    foreach ($frame in @($geometry.clickable_frames)) {
        if ($null -ne $frame.id) {
            $frames[(Get-HitConsistencyFrameKey -Id $frame.id -Kind $frame.kind -Surface $frame.surface)] = $frame
        }
    }
    $samples = @()
    $passed = 0
    $failed = 0
    foreach ($sample in @($geometry.hit_samples)) {
        $frameEntry = $frames[(Get-HitConsistencyFrameKey -Id $sample.id -Kind $sample.kind -Surface $sample.surface)]
        $actualHit = $false
        if ($null -ne $frameEntry) {
            $frame = $frameEntry.frame
            $actualHit = [double]$sample.point.x -ge [double]$frame.x -and
                [double]$sample.point.x -lt ([double]$frame.x + [double]$frame.width) -and
                [double]$sample.point.y -ge [double]$frame.y -and
                [double]$sample.point.y -lt ([double]$frame.y + [double]$frame.height)
        }
        $routeHit = if ($null -ne $sample.route_hit) { [bool]$sample.route_hit } else { $actualHit }
        $ok = $actualHit -eq [bool]$sample.expected_hit -and $routeHit -eq [bool]$sample.expected_hit
        if ($ok) { $passed++ } else { $failed++ }
        $samples += [pscustomobject]@{
            id = $sample.id
            kind = $sample.kind
            surface = $sample.surface
            sample = $sample.sample
            expected_hit = [bool]$sample.expected_hit
            frame_contains_point = $actualHit
            route_hit = $routeHit
            passed = $ok
            point = $sample.point
        }
    }
    $artifact = [pscustomobject]@{
        schema_version = 1
        source = "ui_profile_geometry.json"
        method = "rendered_frame_bounds_and_shared_hit_route_samples"
        sample_count = $samples.Count
        passed = $passed
        failed = $failed
        samples = $samples
    }
    $artifact | ConvertTo-Json -Depth 8 | Set-Content -Path (Join-Path $ProfileDir "ui_hit_consistency.json") -Encoding UTF8
    Write-Host ("- hit_consistency_samples={0} failed={1}" -f $samples.Count, $failed)
}

function Get-HitConsistencyFrameKey {
    param(
        [string]$Id,
        [string]$Kind,
        [string]$Surface
    )

    return "$Kind|$Surface|$Id"
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

function Initialize-DrawingApi {
    if ("System.Drawing.Bitmap" -as [type]) {
        return
    }
    Add-Type -AssemblyName System.Drawing
}

function Save-EditorClientScreenshot {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$ProfileDir
    )

    $rect = Get-EditorWindowRect -Process $Process
    if ($null -eq $rect) {
        return
    }
    $width = [Math]::Max(1, $rect.Right - $rect.Left)
    $height = [Math]::Max(1, $rect.Bottom - $rect.Top)
    Initialize-DrawingApi
    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    $name = if (Test-EnvTruthy "ZIRCON_PROFILE_FORCE_SOFTBUFFER") { "screenshot_softbuffer.png" } else { "screenshot_gpu.png" }
    $path = Join-Path $ProfileDir $name
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Wait-EditorClientSize {
    param(
        [System.Diagnostics.Process]$Process,
        [int]$MinWidth = 64,
        [int]$MinHeight = 64,
        [int]$TimeoutSeconds = 8
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) {
            return $null
        }
        $rect = Get-EditorWindowRect -Process $Process
        if ($null -ne $rect) {
            $width = $rect.Right - $rect.Left
            $height = $rect.Bottom - $rect.Top
            if ($width -ge $MinWidth -and $height -ge $MinHeight) {
                return $rect
            }
        }
        Start-Sleep -Milliseconds 100
    }
    return $null
}

function Get-ScreenshotDimensions {
    param([string]$Path)

    Initialize-DrawingApi
    $bitmap = New-Object System.Drawing.Bitmap $Path
    try {
        return [pscustomobject]@{
            width = $bitmap.Width
            height = $bitmap.Height
        }
    }
    finally {
        $bitmap.Dispose()
    }
}

function Wait-ProfileReferenceScreenshot {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$ProfileDir,
        [int]$TimeoutSeconds = 8
    )

    $path = Join-Path $ProfileDir "screenshot_reference.png"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) {
            return $false
        }
        if (Test-Path $path) {
            try {
                $item = Get-Item -Path $path
                $ageMs = ((Get-Date).ToUniversalTime() - $item.LastWriteTimeUtc).TotalMilliseconds
                $dimensions = Get-ScreenshotDimensions -Path $path
                $rect = Wait-EditorClientSize -Process $Process -MinWidth $dimensions.width -MinHeight $dimensions.height -TimeoutSeconds 1
                if ($ageMs -ge 250 -and $dimensions.width -ge 64 -and $dimensions.height -ge 64 -and $null -ne $rect) {
                    return $true
                }
            }
            catch {
                Start-Sleep -Milliseconds 100
            }
        }
        Start-Sleep -Milliseconds 100
    }
    Write-Warning "Timed out waiting for a stable reference screenshot before live screenshot capture."
    return $false
}

function Export-ScreenshotDiff {
    param([string]$ProfileDir)

    $referencePath = Join-Path $ProfileDir "screenshot_reference.png"
    $gpuPath = Join-Path $ProfileDir "screenshot_gpu.png"
    $softbufferPath = Join-Path $ProfileDir "screenshot_softbuffer.png"
    $liveCandidates = @(
        $gpuPath,
        $softbufferPath
    ) | Where-Object { Test-Path $_ }
    if (-not (Test-Path $referencePath) -or $liveCandidates.Count -eq 0) {
        return
    }
    Initialize-DrawingApi
    $entries = @()
    foreach ($livePath in $liveCandidates) {
        $entries += Measure-ScreenshotDelta -ReferencePath $referencePath -LivePath $livePath -Comparison "reference_vs_live"
    }
    if ((Test-Path $gpuPath) -and (Test-Path $softbufferPath)) {
        $entries += Measure-ScreenshotDelta -ReferencePath $gpuPath -LivePath $softbufferPath -Comparison "gpu_vs_softbuffer"
    }
    $artifact = [pscustomobject]@{
        schema_version = 1
        method = "sampled_rgba_channel_delta"
        thresholds = [pscustomobject]@{
            max_different_sample_ratio = $ScreenshotDiffMaxDifferentSampleRatio
            max_average_channel_delta = $ScreenshotDiffMaxAverageChannelDelta
        }
        entries = $entries
    }
    $artifact | ConvertTo-Json -Depth 6 | Set-Content -Path (Join-Path $ProfileDir "screenshot_diff.json") -Encoding UTF8
    foreach ($entry in $entries) {
        Write-Host ("- screenshot_diff {0} {1}->{2}: differing_sample_ratio={3:N4} max_delta={4}" -f $entry.comparison, $entry.reference_file, $entry.live_file, $entry.differing_sample_ratio, $entry.max_channel_delta)
    }
}

function Measure-ScreenshotDelta {
    param(
        [string]$ReferencePath,
        [string]$LivePath,
        [string]$Comparison
    )

    $reference = New-Object System.Drawing.Bitmap $ReferencePath
    $live = New-Object System.Drawing.Bitmap $LivePath
    try {
        $width = [Math]::Min($reference.Width, $live.Width)
        $height = [Math]::Min($reference.Height, $live.Height)
        $step = 2
        $sampleCount = 0
        $different = 0
        $maxDelta = 0
        $totalDelta = 0
        for ($y = 0; $y -lt $height; $y += $step) {
            for ($x = 0; $x -lt $width; $x += $step) {
                $a = $reference.GetPixel($x, $y)
                $b = $live.GetPixel($x, $y)
                $delta = [Math]::Max([Math]::Abs([int]$a.R - [int]$b.R), [Math]::Abs([int]$a.G - [int]$b.G))
                $delta = [Math]::Max($delta, [Math]::Abs([int]$a.B - [int]$b.B))
                $delta = [Math]::Max($delta, [Math]::Abs([int]$a.A - [int]$b.A))
                if ($delta -gt 0) { $different++ }
                if ($delta -gt $maxDelta) { $maxDelta = $delta }
                $totalDelta += $delta
                $sampleCount++
            }
        }
        return [pscustomobject]@{
            comparison = $Comparison
            live_file = [System.IO.Path]::GetFileName($LivePath)
            reference_file = [System.IO.Path]::GetFileName($ReferencePath)
            width = $width
            height = $height
            sample_step = $step
            sampled_pixels = $sampleCount
            differing_samples = $different
            differing_sample_ratio = if ($sampleCount -gt 0) { $different / $sampleCount } else { 0.0 }
            max_channel_delta = $maxDelta
            average_channel_delta = if ($sampleCount -gt 0) { $totalDelta / $sampleCount } else { 0.0 }
        }
    }
    finally {
        $reference.Dispose()
        $live.Dispose()
    }
}

function Test-UiBatchMetricsGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $artifactPath = Join-Path $ProfileDir "ui_batch_metrics.json"
    if (-not (Test-Path $artifactPath)) {
        Write-Warning "UI batch metrics were not exported for scenario '$ScenarioName'."
        return $false
    }
    $artifact = Get-Content -Path $artifactPath -Raw | ConvertFrom-Json
    $evidenceScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    $current = @($artifact.scenarios) | Where-Object { $_.scenario -eq $evidenceScenario } | Select-Object -First 1
    if ($null -eq $current) {
        Write-Warning "UI batch metrics do not contain scenario '$evidenceScenario' for requested scenario '$ScenarioName'."
        return $false
    }
    if ($evidenceScenario -eq "idle_hover" -and [int64]$current.gpu_visible_draw_items -eq 0) {
        Write-Warning "Scenario '$ScenarioName' has no hover redraw batch; batch gate is treated as event-path-only evidence."
        return $true
    }
    if ([int64]$current.gpu_visible_draw_items -le [int64]$current.gpu_draw_calls) {
        if ([double]$current.dependency_density -ge 0.99) {
            Write-Host "Scenario '$ScenarioName' is dependency-bound for evidence scenario '$evidenceScenario'; draw-call reduction is not expected for this patch."
            return $true
        }
        Write-Warning "Scenario '$ScenarioName' did not reduce GPU draw calls below visible draw items for evidence scenario '$evidenceScenario'."
        return $false
    }
    return $true
}

function Test-UiHitConsistencyGate {
    param([string]$ProfileDir)

    $artifactPath = Join-Path $ProfileDir "ui_hit_consistency.json"
    if (-not (Test-Path $artifactPath)) {
        Write-Warning "UI hit consistency artifact was not exported."
        return $false
    }
    $artifact = Get-Content -Path $artifactPath -Raw | ConvertFrom-Json
    if ([int64]$artifact.failed -gt 0) {
        Write-Warning "UI hit consistency failed for $($artifact.failed) sample(s)."
        return $false
    }
    if ([int64]$artifact.sample_count -le 0) {
        Write-Warning "UI hit consistency artifact contains no samples."
        return $false
    }
    return $true
}

function Test-ScreenshotDiffGate {
    param([string]$ProfileDir)

    if (-not $CaptureSoftbufferScreenshot) {
        return $true
    }
    $required = @(
        (Join-Path $ProfileDir "screenshot_reference.png"),
        (Join-Path $ProfileDir "screenshot_gpu.png"),
        (Join-Path $ProfileDir "screenshot_softbuffer.png"),
        (Join-Path $ProfileDir "screenshot_diff.json")
    )
    foreach ($path in $required) {
        if (-not (Test-Path $path)) {
            Write-Warning "Screenshot parity artifact is missing: $path"
            return $false
        }
    }
    $artifact = Get-Content -Path (Join-Path $ProfileDir "screenshot_diff.json") -Raw | ConvertFrom-Json
    $files = @($artifact.entries | ForEach-Object { $_.live_file })
    if ("screenshot_gpu.png" -notin $files -or "screenshot_softbuffer.png" -notin $files) {
        Write-Warning "Screenshot diff did not compare both GPU and softbuffer screenshots."
        return $false
    }
    $direct = @($artifact.entries) | Where-Object { $_.comparison -eq "gpu_vs_softbuffer" } | Select-Object -First 1
    if ($null -eq $direct) {
        Write-Warning "Screenshot diff did not include a direct GPU-vs-softbuffer comparison."
        return $false
    }
    if ([double]$direct.differing_sample_ratio -gt $ScreenshotDiffMaxDifferentSampleRatio) {
        Write-Warning ("GPU-vs-softbuffer screenshot differing sample ratio {0:N4} exceeded threshold {1:N4}." -f `
                [double]$direct.differing_sample_ratio,
                $ScreenshotDiffMaxDifferentSampleRatio)
        return $false
    }
    if ([double]$direct.average_channel_delta -gt $ScreenshotDiffMaxAverageChannelDelta) {
        Write-Warning ("GPU-vs-softbuffer screenshot average channel delta {0:N4} exceeded threshold {1:N4}." -f `
                [double]$direct.average_channel_delta,
                $ScreenshotDiffMaxAverageChannelDelta)
        return $false
    }
    return $true
}

function Test-AssetRefreshCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName -ne "asset_refresh") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Asset refresh gate could not find timeline.zrtrace.json."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $changeCounters = @($snapshot.counters) | Where-Object {
        $_.name -in @(
            "ui.asset_refresh.asset_change_count",
            "ui.asset_refresh.editor_change_count",
            "ui.asset_refresh.resource_change_count"
        )
    }
    $changeCount = 0
    foreach ($counter in $changeCounters) {
        $changeCount += [int64][Math]::Max(0, [Math]::Round([double]$counter.value))
    }
    Write-Host ("- asset_refresh_change_count={0}" -f $changeCount)
    if ($changeCount -le 0) {
        Write-Warning "Scenario 'asset_refresh' did not record any asset/editor/resource change counter."
        return $false
    }
    return $true
}

function Test-UiInteractionEvidenceGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $interactionScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    if ($interactionScenario -ne "drawer_resize") {
        return $true
    }
    $artifactPath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $artifactPath)) {
        Write-Warning "Drawer resize interaction evidence was not exported."
        return $false
    }
    $artifact = Get-Content -Path $artifactPath -Raw | ConvertFrom-Json
    if ($null -eq $artifact.interaction -or -not ([bool]$artifact.interaction.used_geometry)) {
        Write-Warning "Drawer resize did not use geometry-derived splitter coordinates."
        return $false
    }
    if (-not ([bool]$artifact.resize_changed_layout)) {
        Write-Warning "Drawer resize drag did not change retained-host layout geometry."
        return $false
    }
    return $true
}

function Export-SoftbufferRunManifest {
    param(
        [string]$ProfileDir,
        [string]$SoftbufferSessionId
    )

    if ([string]::IsNullOrWhiteSpace($SoftbufferSessionId)) {
        return
    }
    $artifact = [pscustomobject]@{
        schema_version = 1
        session_id = $SoftbufferSessionId
        forced_by_env = "ZIRCON_PROFILE_FORCE_SOFTBUFFER=1"
        capture_profile_enabled = $false
        screenshot_file = "screenshot_softbuffer.png"
        log_stdout = "editor.softbuffer.stdout.log"
        log_stderr = "editor.softbuffer.stderr.log"
    }
    $artifact | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $ProfileDir "softbuffer_run.json") -Encoding UTF8
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

function Get-CapturePointFromFrame {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Frame,
        [double]$XRatio = 0.5,
        [double]$YRatio = 0.5
    )

    [pscustomobject]@{
        X = [int]($Rect.Left + [Math]::Round([double]$Frame.x + ([double]$Frame.width * $XRatio)))
        Y = [int]($Rect.Top + [Math]::Round([double]$Frame.y + ([double]$Frame.height * $YRatio)))
    }
}

function Wait-ProfileGeometry {
    param(
        [string]$ProfileDir,
        [int]$TimeoutSeconds = 6
    )

    $path = Join-Path $ProfileDir "ui_profile_geometry.json"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path $path) {
            try {
                return Get-Content -Path $path -Raw | ConvertFrom-Json
            }
            catch {
                Start-Sleep -Milliseconds 100
            }
        }
        Start-Sleep -Milliseconds 100
    }
    return $null
}

function Wait-ProfileGeometrySnapshot {
    param(
        [string]$ProfileDir,
        [object]$AfterWriteTimeUtc = $null,
        [int]$TimeoutSeconds = 3
    )

    $path = Join-Path $ProfileDir "ui_profile_geometry.json"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path $path) {
            $item = Get-Item -Path $path
            $fresh = $null -eq $AfterWriteTimeUtc -or $item.LastWriteTimeUtc -gt [datetime]$AfterWriteTimeUtc
            if ($fresh) {
                try {
                    return [pscustomobject]@{
                        geometry = Get-Content -Path $path -Raw | ConvertFrom-Json
                        write_time_utc = $item.LastWriteTimeUtc
                        fresh = $true
                    }
                }
                catch {
                    Start-Sleep -Milliseconds 100
                }
            }
        }
        Start-Sleep -Milliseconds 100
    }
    if (Test-Path $path) {
        $item = Get-Item -Path $path
        try {
            return [pscustomobject]@{
                geometry = Get-Content -Path $path -Raw | ConvertFrom-Json
                write_time_utc = $item.LastWriteTimeUtc
                fresh = $false
            }
        }
        catch {
            return $null
        }
    }
    return $null
}

function Get-ProfileLayoutDeltas {
    param(
        [object]$BeforeGeometry,
        [object]$AfterGeometry
    )

    if ($null -eq $BeforeGeometry -or $null -eq $AfterGeometry) {
        return $null
    }
    return [pscustomobject]@{
        left_width = [double]$AfterGeometry.layout.left_region.width - [double]$BeforeGeometry.layout.left_region.width
        right_width = [double]$AfterGeometry.layout.right_region.width - [double]$BeforeGeometry.layout.right_region.width
        bottom_height = [double]$AfterGeometry.layout.bottom_region.height - [double]$BeforeGeometry.layout.bottom_region.height
        document_width = [double]$AfterGeometry.layout.document_region.width - [double]$BeforeGeometry.layout.document_region.width
        document_height = [double]$AfterGeometry.layout.document_region.height - [double]$BeforeGeometry.layout.document_region.height
    }
}

function Export-UiInteractionEvidence {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName,
        [object]$BeforeGeometry,
        [object]$BeforeWriteTimeUtc,
        [object]$Interaction
    )

    $interactionScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    if ($interactionScenario -notin @("drag", "drawer_resize", "click", "idle_hover", "asset_refresh", "viewport_image")) {
        return
    }

    $afterSnapshot = Wait-ProfileGeometrySnapshot -ProfileDir $ProfileDir -AfterWriteTimeUtc $BeforeWriteTimeUtc
    $afterGeometry = if ($null -ne $afterSnapshot) { $afterSnapshot.geometry } else { $null }
    $layoutDeltas = Get-ProfileLayoutDeltas -BeforeGeometry $BeforeGeometry -AfterGeometry $afterGeometry
    $resizeChanged = $false
    if ($null -ne $layoutDeltas) {
        $resizeChanged = [Math]::Abs([double]$layoutDeltas.left_width) -ge 1.0 -or
            [Math]::Abs([double]$layoutDeltas.right_width) -ge 1.0 -or
            [Math]::Abs([double]$layoutDeltas.bottom_height) -ge 1.0 -or
            [Math]::Abs([double]$layoutDeltas.document_width) -ge 1.0 -or
            [Math]::Abs([double]$layoutDeltas.document_height) -ge 1.0
    }
    if (-not $resizeChanged -and
        $interactionScenario -eq "drawer_resize" -and
        $null -ne $Interaction -and
        [bool]$Interaction.used_geometry -and
        $null -ne $afterGeometry) {
        $afterSplitter = @($afterGeometry.resize_splitters) |
            Where-Object { $_.id -eq $Interaction.target_id } |
            Select-Object -First 1
        if ($null -ne $afterSplitter -and $null -ne $afterSplitter.frame -and $null -ne $Interaction.target_frame) {
            $resizeChanged = [Math]::Abs([double]$afterSplitter.frame.x - [double]$Interaction.target_frame.x) -ge 1.0 -or
                [Math]::Abs([double]$afterSplitter.frame.y - [double]$Interaction.target_frame.y) -ge 1.0 -or
                [Math]::Abs([double]$afterSplitter.frame.width - [double]$Interaction.target_frame.width) -ge 1.0 -or
                [Math]::Abs([double]$afterSplitter.frame.height - [double]$Interaction.target_frame.height) -ge 1.0
        }
    }

    $artifact = [pscustomobject]@{
        schema_version = 1
        scenario = $interactionScenario
        requested_scenario = $ScenarioName
        geometry_available_before_interaction = $null -ne $BeforeGeometry
        geometry_refreshed_after_interaction = $null -ne $afterSnapshot -and ([bool]$afterSnapshot.fresh)
        interaction = $Interaction
        layout_deltas = $layoutDeltas
        resize_changed_layout = $resizeChanged
    }
    $artifact | ConvertTo-Json -Depth 8 | Set-Content -Path (Join-Path $ProfileDir "ui_interaction_evidence.json") -Encoding UTF8
    if ($interactionScenario -eq "drawer_resize") {
        Write-Host ("- drawer_resize_changed_layout={0} geometry_refreshed={1}" -f $resizeChanged, $artifact.geometry_refreshed_after_interaction)
    }
}

function Select-FirstVisibleFrame {
    param([object[]]$Frames)
    foreach ($entry in @($Frames)) {
        if ($null -ne $entry.frame -and [double]$entry.frame.width -gt 0 -and [double]$entry.frame.height -gt 0) {
            return $entry
        }
    }
    return $null
}

function Invoke-LiveGeometryDragInteraction {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry
    )

    $tab = (@($Geometry.document_tabs) | Where-Object { $_.active } | Select-Object -First 1)
    if ($null -eq $tab) {
        $tab = Select-FirstVisibleFrame -Frames @($Geometry.document_tabs)
    }
    if ($null -eq $tab) {
        $tab = Select-FirstVisibleFrame -Frames @($Geometry.drawer_tabs)
    }
    if ($null -eq $tab) {
        return $false
    }
    $start = Get-CapturePointFromFrame -Rect $Rect -Frame $tab.frame
    $targetFrame = $Geometry.layout.document_region
    $mid = Get-CapturePointFromFrame -Rect $Rect -Frame $targetFrame -XRatio 0.38 -YRatio 0.18
    $end = Get-CapturePointFromFrame -Rect $Rect -Frame $targetFrame -XRatio 0.62 -YRatio 0.28
    $script:LastInteractionEvidence = [pscustomobject]@{
        scenario = "drag"
        used_geometry = $true
        target_id = $tab.id
        target_kind = $tab.kind
        target_surface = $tab.surface
        start = $start
        mid = $mid
        end = $end
        target_frame = $tab.frame
    }
    Drag-CapturePath -Points @($start, $mid, $end)
    return $true
}

function Invoke-LiveGeometryResizeInteraction {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry
    )

    $splitter = Select-FirstVisibleFrame -Frames @($Geometry.resize_splitters)
    if ($null -eq $splitter) {
        return $false
    }
    $start = Get-CapturePointFromFrame -Rect $Rect -Frame $splitter.frame
    $directionX = 0
    $directionY = 0
    if ($splitter.id -like "*bottom*") {
        $directionY = -80
    }
    elseif ($splitter.id -like "*right*") {
        $directionX = -80
    }
    else {
        $directionX = 80
    }
    $mid = [pscustomobject]@{ X = $start.X + [int]($directionX * 0.6); Y = $start.Y + [int]($directionY * 0.6) }
    $end = [pscustomobject]@{ X = $start.X + $directionX; Y = $start.Y + $directionY }
    $script:LastInteractionEvidence = [pscustomobject]@{
        scenario = "drawer_resize"
        used_geometry = $true
        target_id = $splitter.id
        target_kind = $splitter.kind
        target_surface = $splitter.surface
        start = $start
        mid = $mid
        end = $end
        delta_x = $directionX
        delta_y = $directionY
        target_frame = $splitter.frame
    }
    Drag-CapturePath -Points @($start, $mid, $end)
    return $true
}

function Invoke-LiveGeometryClickInteraction {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry,
        [switch]$TemplateControlsOnly
    )

    $frames = if ($TemplateControlsOnly) {
        @($Geometry.template_controls)
    }
    else {
        @($Geometry.activity_rail_buttons) + @($Geometry.document_tabs) + @($Geometry.viewport_toolbar_controls) + @($Geometry.template_controls)
    }
    $clicked = 0
    foreach ($entry in $frames) {
        if ($clicked -ge 3) { break }
        if ($null -eq $entry.frame -or [double]$entry.frame.width -le 0 -or [double]$entry.frame.height -le 0) {
            continue
        }
        Click-CapturePoint -Point (Get-CapturePointFromFrame -Rect $Rect -Frame $entry.frame)
        $clicked++
    }
    return $clicked -gt 0
}

function Invoke-LiveGeometryHoverInteraction {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry
    )

    $frames = @($Geometry.activity_rail_buttons) + @($Geometry.document_tabs) + @($Geometry.template_controls) + @($Geometry.viewport_toolbar_controls)
    $points = @()
    foreach ($entry in $frames) {
        if ($points.Count -ge 8) { break }
        if ($null -ne $entry.frame -and [double]$entry.frame.width -gt 0 -and [double]$entry.frame.height -gt 0) {
            $points += Get-CapturePointFromFrame -Rect $Rect -Frame $entry.frame
        }
    }
    if ($points.Count -eq 0) {
        return $false
    }
    foreach ($point in $points) {
        Move-CaptureCursor -Point $point -DelayMs 180
    }
    return $true
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
        [string]$ScenarioName,
        [string]$ProfileDir,
        [string]$SessionId
    )

    if (-not $AutoInteract) {
        return
    }

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    $interactionScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    if ($interactionScenario -in @("manual", "startup", "")) {
        return
    }

    $rect = Get-EditorWindowRect -Process $Process
    if ($null -eq $rect) {
        Write-Warning "Auto interaction skipped because the editor window rectangle was unavailable."
        return
    }

    [ZirconProfileCaptureNative]::SetForegroundWindow($Process.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 300

    $script:LastInteractionEvidence = $null
    $geometryPath = Join-Path $ProfileDir "ui_profile_geometry.json"
    $geometry = Wait-ProfileGeometry -ProfileDir $ProfileDir
    $geometryWriteTimeUtc = if (Test-Path $geometryPath) { (Get-Item -Path $geometryPath).LastWriteTimeUtc } else { $null }

    if ($interactionScenario -eq "asset_refresh") {
        Invoke-AssetRefreshChange -SessionId $SessionId
    }

    switch ($interactionScenario) {
        "idle_hover" {
            if ($null -ne $geometry -and (Invoke-LiveGeometryHoverInteraction -Rect $rect -Geometry $geometry)) {
                break
            }
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
            $templateControlsOnly = $normalizedScenario -eq "material_lab_click"
            if ($null -ne $geometry -and (Invoke-LiveGeometryClickInteraction -Rect $rect -Geometry $geometry -TemplateControlsOnly:$templateControlsOnly)) {
                break
            }
            foreach ($point in @(
                (Get-CapturePoint -Rect $rect -XRatio 0.12 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.30 -YRatio 0.16),
                (Get-CapturePoint -Rect $rect -XRatio 0.52 -YRatio 0.18)
            )) {
                Click-CapturePoint -Point $point
            }
        }
        "drag" {
            if ($null -ne $geometry -and (Invoke-LiveGeometryDragInteraction -Rect $rect -Geometry $geometry)) {
                break
            }
            $script:LastInteractionEvidence = [pscustomobject]@{
                scenario = "drag"
                used_geometry = $false
                fallback = "ratio_path"
            }
            Drag-CapturePath -Points @(
                (Get-CapturePoint -Rect $rect -XRatio 0.44 -YRatio 0.46),
                (Get-CapturePoint -Rect $rect -XRatio 0.50 -YRatio 0.48),
                (Get-CapturePoint -Rect $rect -XRatio 0.56 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.62 -YRatio 0.52)
            )
        }
        "drawer_resize" {
            if ($null -ne $geometry -and (Invoke-LiveGeometryResizeInteraction -Rect $rect -Geometry $geometry)) {
                break
            }
            $script:LastInteractionEvidence = [pscustomobject]@{
                scenario = "drawer_resize"
                used_geometry = $false
                fallback = "ratio_path"
            }
            Drag-CapturePath -Points @(
                (Get-CapturePoint -Rect $rect -XRatio 0.26 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.30 -YRatio 0.50),
                (Get-CapturePoint -Rect $rect -XRatio 0.34 -YRatio 0.50)
            )
        }
        "asset_refresh" {
            foreach ($point in @(
                (Get-CapturePoint -Rect $rect -XRatio 0.06 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.18 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.10 -YRatio 0.32)
            )) {
                Move-CaptureCursor -Point $point -DelayMs 260
            }
            Click-CapturePoint -Point (Get-CapturePoint -Rect $rect -XRatio 0.10 -YRatio 0.18)
            Start-Sleep -Milliseconds 800
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
    Export-UiInteractionEvidence -ProfileDir $ProfileDir -ScenarioName $ScenarioName -BeforeGeometry $geometry -BeforeWriteTimeUtc $geometryWriteTimeUtc -Interaction $script:LastInteractionEvidence
}

function Resolve-ProfileProjectRoot {
    param([string]$SessionId)
    return Join-Path (Join-Path $RepoRoot (Join-Path "target\zircon-profile-projects" $SessionId)) "ProfileCaptureProject"
}

function Invoke-AssetRefreshChange {
    param([string]$SessionId)

    if ([string]::IsNullOrWhiteSpace($SessionId)) {
        return
    }
    $projectRoot = Resolve-ProfileProjectRoot -SessionId $SessionId
    $assetsRoot = Join-Path $projectRoot "assets"
    $deadline = (Get-Date).AddSeconds(6)
    while ((Get-Date) -lt $deadline -and -not (Test-Path $assetsRoot)) {
        Start-Sleep -Milliseconds 150
    }
    if (-not (Test-Path $assetsRoot)) {
        Write-Warning "Asset refresh interaction could not find project assets root: $assetsRoot"
        return
    }
    $materialPath = Join-Path (Join-Path $assetsRoot "materials") "default.zmaterial"
    if (Test-Path $materialPath) {
        Add-Content -Path $materialPath -Value ("`n# profile capture asset refresh {0}" -f (Get-Date -Format o)) -Encoding UTF8
    }
    else {
        $touchPath = Join-Path $assetsRoot "profile_capture_touch.txt"
        Set-Content -Path $touchPath -Value "profile capture $(Get-Date -Format o)" -Encoding UTF8
    }
    Start-Sleep -Milliseconds 1800
}

function Resolve-EditorCaptureArguments {
    param(
        [string]$ScenarioName,
        [string]$SessionId
    )

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    if ($normalizedScenario -in @("material_lab_startup", "material_lab_hover", "material_lab_click")) {
        return @(
            "--builtin-view",
            "editor.material_component_lab"
        )
    }
    if ($normalizedScenario -notin @("idle_hover", "viewport_image", "drag", "drawer_resize", "asset_refresh")) {
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

function Test-ScenarioUsesProfileProject {
    param([string]$ScenarioName)
    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    return $normalizedScenario -in @("idle_hover", "viewport_image", "drag", "drawer_resize", "asset_refresh")
}

function Invoke-EditorCapture {
    param(
        [string]$ScenarioName,
        [string]$SessionId,
        [switch]$OpenExistingProject,
        [string]$LogStem = "editor"
    )

    if ($OpenExistingProject) {
        $projectRoot = Resolve-ProfileProjectRoot -SessionId $SessionId
        $editorArguments = if (Test-Path $projectRoot) { @("--project", $projectRoot) } else { @() }
    }
    else {
        $editorArguments = Resolve-EditorCaptureArguments -ScenarioName $ScenarioName -SessionId $SessionId
    }
    $profileDir = if ([string]::IsNullOrWhiteSpace($SessionId)) { $null } else { Join-Path $OutputPath $SessionId }

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
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $startProcessArgs.RedirectStandardOutput = Join-Path $profileDir "$LogStem.stdout.log"
        $startProcessArgs.RedirectStandardError = Join-Path $profileDir "$LogStem.stderr.log"
    }
    $process = Start-Process @startProcessArgs
    $windowTimeout = [Math]::Max(30, $AutoCloseSeconds)
    if (-not (Wait-EditorMainWindow -Process $process -TimeoutSeconds $windowTimeout)) {
        if ($process.HasExited) {
            Assert-EditorProcessExitSucceeded -Process $process
            return
        }
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        throw "Editor did not create a main window before the auto-close timeout."
    }

    Write-Host "Auto-close timer started after the editor main window became available."
    if ($AutoInteract) {
        Start-Sleep -Milliseconds 900
    }
    Invoke-AutoScenarioInteraction -Process $process -ScenarioName $ScenarioName -ProfileDir $profileDir -SessionId $SessionId
    if (-not [string]::IsNullOrWhiteSpace($profileDir)) {
        if (Test-EnvTruthy "ZIRCON_PROFILE_FORCE_SOFTBUFFER") {
            Wait-EditorClientSize -Process $process | Out-Null
        }
        else {
            Wait-ProfileReferenceScreenshot -Process $process -ProfileDir $profileDir | Out-Null
        }
        Start-Sleep -Milliseconds 250
        Save-EditorClientScreenshot -Process $process -ProfileDir $profileDir
    }
    if ($process.WaitForExit($AutoCloseSeconds * 1000)) {
        Assert-EditorProcessExitSucceeded -Process $process
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
        Assert-EditorProcessExitSucceeded -Process $process
        return
    }

    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    throw "Editor did not exit after the auto-close request; the process was stopped before profiling could export normally."
}

function Invoke-SoftbufferScreenshotCapture {
    param(
        [string]$ScenarioName,
        [string]$SessionId
    )

    $softbufferSessionId = "$SessionId-softbuffer"
    if (-not $CaptureSoftbufferScreenshot) {
        return $null
    }
    if ($AutoCloseSeconds -le 0) {
        Write-Warning "Softbuffer screenshot capture requires -AutoCloseSeconds; skipping fallback screenshot."
        return $null
    }

    $previousCapture = $env:ZIRCON_PROFILE_CAPTURE
    $previousSession = $env:ZIRCON_PROFILE_SESSION
    $previousForceSoftbuffer = $env:ZIRCON_PROFILE_FORCE_SOFTBUFFER
    $profileDir = Join-Path $OutputPath $SessionId
    $interactionEvidencePath = Join-Path $profileDir "ui_interaction_evidence.json"
    $primaryInteractionEvidence = if (Test-Path $interactionEvidencePath) {
        Get-Content -Path $interactionEvidencePath -Raw
    }
    else {
        $null
    }
    $capturedSessionId = $null
    try {
        $env:ZIRCON_PROFILE_CAPTURE = "0"
        $env:ZIRCON_PROFILE_SESSION = $softbufferSessionId
        $env:ZIRCON_PROFILE_FORCE_SOFTBUFFER = "1"
        if (Test-ScenarioUsesProfileProject -ScenarioName $ScenarioName) {
            Invoke-EditorCapture -ScenarioName $ScenarioName -SessionId $SessionId -OpenExistingProject -LogStem "editor.softbuffer"
        }
        else {
            Invoke-EditorCapture -ScenarioName $ScenarioName -SessionId $SessionId -LogStem "editor.softbuffer"
        }
        $capturedSessionId = $softbufferSessionId
    }
    finally {
        if ($null -ne $primaryInteractionEvidence) {
            Set-Content -Path $interactionEvidencePath -Value $primaryInteractionEvidence -Encoding UTF8
        }
        if ($null -eq $previousCapture) {
            Remove-Item "Env:\ZIRCON_PROFILE_CAPTURE" -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_PROFILE_CAPTURE = $previousCapture
        }
        if ($null -eq $previousSession) {
            Remove-Item "Env:\ZIRCON_PROFILE_SESSION" -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_PROFILE_SESSION = $previousSession
        }
        if ($null -eq $previousForceSoftbuffer) {
            Remove-Item "Env:\ZIRCON_PROFILE_FORCE_SOFTBUFFER" -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_PROFILE_FORCE_SOFTBUFFER = $previousForceSoftbuffer
        }
    }
    return $capturedSessionId
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
    $dependencyRuntimeDll = Join-Path (Join-Path $TargetDir "deps") "zircon_runtime.dll"
    if (Test-Path $dependencyRuntimeDll) {
        $RuntimeDll = $dependencyRuntimeDll
    }
    else {
        throw "Missing profiling runtime library: $RuntimeDll"
    }
}

New-Item -ItemType Directory -Force -Path $OutputPath | Out-Null

$previous = @{
    ZIRCON_PROFILE_CAPTURE = $env:ZIRCON_PROFILE_CAPTURE
    ZIRCON_PROFILE_SESSION = $env:ZIRCON_PROFILE_SESSION
    ZIRCON_PROFILE_OUTPUT_ROOT = $env:ZIRCON_PROFILE_OUTPUT_ROOT
    ZIRCON_PROFILE_MAX_FRAMES = $env:ZIRCON_PROFILE_MAX_FRAMES
    ZIRCON_PROFILE_MAX_SPANS = $env:ZIRCON_PROFILE_MAX_SPANS
    ZIRCON_PROFILE_MAX_COUNTERS = $env:ZIRCON_PROFILE_MAX_COUNTERS
    ZIRCON_PROFILE_CAPTURE_SCREENSHOTS = $env:ZIRCON_PROFILE_CAPTURE_SCREENSHOTS
    ZIRCON_PROFILE_FORCE_SOFTBUFFER = $env:ZIRCON_PROFILE_FORCE_SOFTBUFFER
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
        $SoftbufferSessionId = $null

        $env:ZIRCON_PROFILE_CAPTURE = "1"
        $env:ZIRCON_PROFILE_SESSION = $SessionId
        $env:ZIRCON_PROFILE_OUTPUT_ROOT = $OutputPath
        $env:ZIRCON_PROFILE_MAX_FRAMES = "$MaxFrames"
        $env:ZIRCON_PROFILE_MAX_SPANS = "$MaxSpans"
        $env:ZIRCON_PROFILE_MAX_COUNTERS = "$MaxCounters"
        $env:ZIRCON_PROFILE_CAPTURE_SCREENSHOTS = "1"
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
            $SoftbufferSessionId = Invoke-SoftbufferScreenshotCapture -ScenarioName $scenarioName -SessionId $SessionId
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
            Export-UiBatchMetrics -ProfileDir $ProfileDir -ScenarioName $scenarioName
            Export-UiHitConsistency -ProfileDir $ProfileDir
            Export-ScreenshotDiff -ProfileDir $ProfileDir
            $batchMetricsOk = Test-UiBatchMetricsGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $hitConsistencyOk = Test-UiHitConsistencyGate -ProfileDir $ProfileDir
            $screenshotDiffOk = Test-ScreenshotDiffGate -ProfileDir $ProfileDir
            $assetRefreshOk = Test-AssetRefreshCounterGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $interactionEvidenceOk = Test-UiInteractionEvidenceGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            Export-SoftbufferRunManifest -ProfileDir $ProfileDir -SoftbufferSessionId $SoftbufferSessionId
            if ($RequireScenarioEvidence -and -not ($scenarioEvidenceOk -and $batchMetricsOk -and $hitConsistencyOk -and $screenshotDiffOk -and $assetRefreshOk -and $interactionEvidenceOk)) {
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
