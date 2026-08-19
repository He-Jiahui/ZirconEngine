param(
    [string]$Scenario = "manual",
    [string[]]$ScenarioList = @(),
    [switch]$AllUiScenarios,
    [string]$OutputRoot = "E:\zircon-profiles",
    [switch]$SkipBuild,
    [switch]$CaptureSoftbufferScreenshot,
    [switch]$UseTracy,
    [switch]$UseWpr,
    [int]$AutoCloseSeconds = 0,
    [switch]$AutoInteract,
    [switch]$RequireScenarioEvidence,
    [ValidateRange(0, 1000000)]
    [int]$AutoPointerMoveCount = 0,
    [ValidateRange(0, 1000)]
    [int]$AutoPointerMoveDelayMs = 2,
    [ValidateRange(0, 1000000)]
    [int]$AutoClickCount = 0,
    [ValidateRange(0, 1000)]
    [int]$AutoClickDelayMs = 4,
    [ValidateRange(0, 1000000)]
    [int]$AutoWheelCount = 0,
    [ValidateRange(0, 1000)]
    [int]$AutoWheelDelayMs = 2,
    [ValidateRange(0, 100000)]
    [int]$HierarchyLogicalNodeCount = 0,
    [ValidateRange(0, 10000)]
    [int]$AssetCatalogItemCount = 0,
    [ValidateRange(2, 240)]
    [int]$AutoResizeStepCount = 24,
    [ValidateRange(1, 1000)]
    [int]$AutoResizeDelayMs = 40,
    [ValidateRange(0, 120)]
    [int]$WithinProcessWarmupPresentCount = 1,
    [ValidateRange(0, 30)]
    [int]$WithinProcessQuiescenceSeconds = 2,
    [ValidateRange(1, 10)]
    [int]$MeasuredRunCount = 3,
    [ValidateRange(0, 30)]
    [int]$RunQuiescenceSeconds = 2,
    [int]$MaxFrames = 2048,
    [int]$MaxSpans = 65536,
    [int]$MaxCounters = 65536,
    [double]$ScreenshotDiffMaxDifferentSampleRatio = 0.25,
    [double]$ScreenshotDiffMaxAverageChannelDelta = 10.0
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
. (Join-Path $PSScriptRoot "profile-capture-paths.ps1")
. (Join-Path $PSScriptRoot "profile-capture-manifest.ps1")
. (Join-Path $PSScriptRoot "ui-profile-scenarios.ps1")
. (Join-Path $PSScriptRoot "ui-profile-native-resize.ps1")
. (Join-Path $PSScriptRoot "ui-profile-latency-evidence.ps1")
. (Join-Path $PSScriptRoot "ui-profile-process-evidence.ps1")
. (Join-Path $PSScriptRoot "ui-profile-scale-fixture.ps1")

$captureScenarios = @(Resolve-ZirconUiProfileCaptureScenarios `
        -Scenario $Scenario `
        -ScenarioList $ScenarioList `
        -AllUiScenarios:$AllUiScenarios)
$OutputPath = Resolve-ZirconProfileOutputRoot -RepoRoot $RepoRoot -Path $OutputRoot
$VerificationScreenshotRoot = Join-Path $RepoRoot "docs\tests\editor\profile-captures"
$ManagedCargoTargetRoots = @(
    "D:\cargo-targets",
    "E:\cargo-targets",
    "F:\cargo-targets",
    "D:\targets",
    "E:\targets",
    "F:\targets",
    "D:\ZirconBuilds",
    "E:\ZirconBuilds",
    "F:\ZirconBuilds"
)

function Resolve-ProfilingTargetDir {
    if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        throw "CARGO_TARGET_DIR must be set to a coordinator-managed Windows cargo target before profile capture."
    }
    if (-not [System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
        throw "CARGO_TARGET_DIR must be an absolute coordinator-managed Windows cargo target."
    }

    $cargoTarget = [System.IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
    $managedRoot = $ManagedCargoTargetRoots |
        ForEach-Object { [System.IO.Path]::GetFullPath($_).TrimEnd('\\') } |
        Where-Object {
            $candidateRoot = $_ + [System.IO.Path]::DirectorySeparatorChar
            $cargoTarget.Equals($_, [System.StringComparison]::OrdinalIgnoreCase) -or
                $cargoTarget.StartsWith($candidateRoot, [System.StringComparison]::OrdinalIgnoreCase)
        } |
        Select-Object -First 1
    if ($null -eq $managedRoot) {
        throw "CARGO_TARGET_DIR must resolve beneath a coordinator-managed Windows cargo target root."
    }

    return Join-Path $cargoTarget "profiling"
}

$TargetDir = Resolve-ProfilingTargetDir
$EditorExe = Join-Path $TargetDir "zircon_editor.exe"
$RuntimeDll = Join-Path $TargetDir "zircon_runtime.dll"
$TracyProfiler = Join-Path $RepoRoot "dev\tracy\tracy-profiler.exe"
$script:LastInteractionEvidence = $null

function Get-ScenarioWithinProcessWarmupPresentCount {
    param([string]$ScenarioName)

    if ($ScenarioName.Trim().ToLowerInvariant() -in @('startup', 'material_lab_startup')) {
        return 0
    }
    return $WithinProcessWarmupPresentCount
}

function Get-ScenarioRequestedWheelOperationCount {
    param([string]$ScenarioName)

    if ($ScenarioName.Trim().ToLowerInvariant() -notin @("hierarchy_scroll", "welcome_recent_scroll")) {
        return 0
    }
    if ($AutoWheelCount -gt 0) {
        return $AutoWheelCount
    }
    return 24
}

function Get-ScenarioInstruction {
    param([string]$Name)
    return Get-ZirconUiProfileCaptureScenarioInstruction -ScenarioId $Name
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

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    $evidenceScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    $requiresMaterialLabPaintOnlyAuthority = $normalizedScenario -eq "material_lab_click"
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
    $hostInvalidationTransactionCount = [int64]$scenario.host_invalidation_transaction_count
    $hostInvalidationTargetCount = [int64]$scenario.host_invalidation_full_target_count +
        [int64]$scenario.host_invalidation_shell_content_target_count +
        [int64]$scenario.host_invalidation_workbench_projection_target_count +
        [int64]$scenario.host_invalidation_view_presentation_target_count +
        [int64]$scenario.host_invalidation_window_metrics_target_count +
        [int64]$scenario.host_invalidation_paint_only_target_count
    $hasConsistentHostInvalidationEvidence =
        $hostInvalidationTargetCount -eq $hostInvalidationTransactionCount
    $paintedPixels = [double]$scenario.painted_pixels
    $presentedSurfacePixels = [double]$scenario.presented_surface_pixels
    $hasConsistentDamageEvidence = $paintedPixels -eq 0.0 -or
        ($presentedSurfacePixels -gt 0.0 -and $paintedPixels -le $presentedSurfacePixels)
    $damageCoveragePercent = if ($presentedSurfacePixels -gt 0.0) {
        $paintedPixels * 100.0 / $presentedSurfacePixels
    }
    else {
        0.0
    }

    Write-Host ""
    Write-Host "UI scenario evidence ($evidenceScenario):"
    Write-Host ("- frames={0} dirty_paint_only={1} redraw_region={2} redraw_full_frame={3}" -f `
            $scenario.frame_count,
            $scenario.dirty_paint_only_count,
            $scenario.redraw_region_count,
            $scenario.redraw_full_frame_count)
    Write-Host ("- host_invalidation_transaction_count={0} host_invalidation_target_count={1} host_invalidation_scope_count={2} host_invalidation_legacy_dirty_transaction_count={3} slow_path={4}" -f `
            $scenario.host_invalidation_transaction_count,
            $hostInvalidationTargetCount,
            $scenario.host_invalidation_scope_count,
            $scenario.host_invalidation_legacy_dirty_transaction_count,
            $scenario.slow_path_rebuild_count)
    Write-Host ("- host_invalidation_full_target_count={0} host_invalidation_shell_content_target_count={1} host_invalidation_workbench_projection_target_count={2} host_invalidation_view_presentation_target_count={3} host_invalidation_window_metrics_target_count={4} host_invalidation_paint_only_target_count={5}" -f `
            $scenario.host_invalidation_full_target_count,
            $scenario.host_invalidation_shell_content_target_count,
            $scenario.host_invalidation_workbench_projection_target_count,
            $scenario.host_invalidation_view_presentation_target_count,
            $scenario.host_invalidation_window_metrics_target_count,
            $scenario.host_invalidation_paint_only_target_count)
    Write-Host ("- painted_pixels={0} presented_surface_pixels={1} damage_coverage_percent={2:N2}" -f `
            $scenario.painted_pixels,
            $scenario.presented_surface_pixels,
            $damageCoveragePercent)
    Write-Host ("- gpu_draw_calls={0} gpu_visible_commands={1} gpu_visible_draw_items={2} gpu_batch_layers={3} gpu_batch_dependencies={4}" -f `
            $scenario.gpu_draw_calls,
            $scenario.gpu_visible_commands,
            $scenario.gpu_visible_draw_items,
            $scenario.gpu_batch_layers,
            $scenario.gpu_batch_dependencies)
    Write-Host ("- gpu_timestamp_supported_present_count={0} gpu_time_sample_count={1} gpu_time_p50_us={2} gpu_time_p95_us={3} gpu_time_max_us={4} gpu_profile_latency_max_frames={5}" -f `
            $scenario.gpu_timestamp_supported_present_count,
            $scenario.gpu_time_sample_count,
            $scenario.gpu_time_p50_us,
            $scenario.gpu_time_p95_us,
            $scenario.gpu_time_max_us,
            $scenario.gpu_profile_latency_max_frames)
    Write-Host ("- gpu_compiled_draw_items={0} batch_plan_builds={1} batch_plan_cache_hits={2}" -f `
            $scenario.gpu_compiled_draw_items,
            $scenario.gpu_batch_plan_build_count,
            $scenario.gpu_batch_plan_cache_hit_count)
    Write-Host ("- gpu_vertex_buffer_creates={0} vertex_upload_bytes={1} retained_cache_copy_bytes={2}" -f `
            $scenario.gpu_vertex_buffer_create_count,
            $scenario.gpu_vertex_upload_bytes,
            $scenario.gpu_retained_cache_copy_bytes)
    Write-Host ("- gpu_upload_bytes={0}" -f $scenario.gpu_upload_bytes)
    Write-Host ("- gpu_image_upload_writes={0} shared_resolves={1} cache_hits={2} invalid_payloads={3}" -f `
            $scenario.gpu_image_upload_write_count,
            $scenario.gpu_image_shared_resolve_count,
            $scenario.gpu_image_prepare_cache_hit_count,
            $scenario.gpu_image_invalid_payload_count)
    Write-Host ("- visual_asset_hits={0} misses={1} svg_tree_hits={2} svg_tree_misses={3}" -f `
            $scenario.visual_asset_cache_hit_count,
            $scenario.visual_asset_cache_miss_count,
            $scenario.svg_tree_cache_memory_hit_count,
            $scenario.svg_tree_cache_miss_count)
    Write-Host ("- visual_reconcile_visits={0} invalidated={1} svg_reconcile_visits={2} invalidated={3}" -f `
            $scenario.visual_asset_reconcile_source_visit_count,
            $scenario.visual_asset_reconciled_invalidation_count,
            $scenario.svg_tree_reconcile_source_visit_count,
            $scenario.svg_tree_reconciled_invalidation_count)
    Write-Host ("- software_fallback_present_count={0}" -f $scenario.software_fallback_present_count)
    $scenarioAlerts = @($report.alerts | Where-Object { $_.scenario -eq $evidenceScenario })
    $blockingScenarioAlerts = @($scenarioAlerts | Where-Object {
            -not ($evidenceScenario -in @("drawer_resize", "window_resize") -and $_.rule -eq "resize_triggered_slow_path_rebuild")
        })
    Write-Host ("- alerts={0} blocking_alerts={1}" -f $scenarioAlerts.Count, $blockingScenarioAlerts.Count)
    if ($scenarioAlerts.Count -gt $blockingScenarioAlerts.Count) {
        Write-Host "- non_blocking_alerts=resize_triggered_slow_path_rebuild"
    }

    $redrawCount = [int64]$scenario.redraw_region_count + [int64]$scenario.redraw_full_frame_count
    $hasGpuBatch = [int64]$scenario.gpu_draw_calls -gt 0 -and
        [int64]$scenario.gpu_visible_draw_items -gt [int64]$scenario.gpu_draw_calls
    $hasGpuUpload = [int64]$scenario.gpu_upload_bytes -gt 0
    $hasGpuTimingEvidence =
        [int64]$scenario.gpu_timestamp_supported_present_count -gt 0 -and
        [int64]$scenario.gpu_time_sample_count -gt 0
    $hasFrameSamples = [int64]$scenario.frame_count -gt 0
    $hasNoSoftwareFallback = [int64]$scenario.software_fallback_present_count -eq 0
    $hasNoAlerts = $blockingScenarioAlerts.Count -eq 0

    $evidenceOk = switch ($evidenceScenario) {
        "startup" { $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "idle_hover" { $hasFrameSamples -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "click" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "drag" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "drawer_resize" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "window_resize" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
        "hierarchy_scroll" { $redrawCount -gt 0 -and $hasGpuBatch -and $hasNoSoftwareFallback -and $hasNoAlerts; break }
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
    $evidenceOk = $evidenceOk -and
        $hasConsistentHostInvalidationEvidence -and
        $hasConsistentDamageEvidence
    $hasMaterialLabPaintOnlyAuthority = -not $requiresMaterialLabPaintOnlyAuthority -or (
        [int64]$scenario.host_invalidation_transaction_count -gt 0 -and
        [int64]$scenario.host_invalidation_paint_only_target_count -gt 0 -and
        [int64]$scenario.host_invalidation_full_target_count -eq 0 -and
        [int64]$scenario.host_invalidation_legacy_dirty_transaction_count -eq 0
    )
    $evidenceOk = $evidenceOk -and $hasMaterialLabPaintOnlyAuthority
    if ($hasGpuBatch) {
        $evidenceOk = $evidenceOk -and $hasGpuTimingEvidence
    }

    if (-not $hasConsistentHostInvalidationEvidence) {
        Write-Warning "Scenario '$ScenarioName' host invalidation target count does not match its transaction count."
    }
    if (-not $hasConsistentDamageEvidence) {
        Write-Warning "Scenario '$ScenarioName' painted-pixel evidence has no valid presented-surface denominator."
    }
    if ($hasGpuBatch -and -not $hasGpuTimingEvidence) {
        Write-Warning "Scenario '$ScenarioName' rendered a GPU batch without a supported GPU timestamp sample."
    }
    if (-not $hasMaterialLabPaintOnlyAuthority) {
        Write-Warning "Scenario '$ScenarioName' did not preserve paint-only host invalidation authority."
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
        "viewport_toolbar_click" { return "click" }
        "hierarchy_scroll" { return "idle_hover" }
        "welcome_recent_scroll" { return "idle_hover" }
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
    public static extern bool SetWindowPos(
        IntPtr hWnd,
        IntPtr hWndInsertAfter,
        int x,
        int y,
        int width,
        int height,
        uint flags);

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

function Export-VerificationScreenshots {
    param(
        [string]$ProfileDir,
        [string]$SessionId
    )

    $screenshots = @(
        "screenshot_reference.png",
        "screenshot_gpu.png",
        "screenshot_softbuffer.png"
    ) |
        ForEach-Object { Join-Path $ProfileDir $_ } |
        Where-Object { Test-Path -LiteralPath $_ }
    if ($screenshots.Count -eq 0) {
        return $null
    }

    $destination = Join-Path $VerificationScreenshotRoot $SessionId
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    foreach ($screenshot in $screenshots) {
        Copy-Item -LiteralPath $screenshot -Destination (Join-Path $destination (Split-Path -Leaf $screenshot)) -Force
    }
    return $destination
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
    $fullInvalidationCount = [double](@($snapshot.counters) |
            Where-Object { $_.name -eq "ui.asset_refresh.visual_asset_full_invalidation_count" } |
            Measure-Object -Property value -Sum).Sum
    $targetedInvalidationCount = [double](@($snapshot.counters) |
            Where-Object { $_.name -in @(
                    "ui.asset_refresh.visual_asset_targeted_invalidation_count",
                    "ui.asset_refresh.svg_tree_targeted_invalidation_count"
                ) } |
            Measure-Object -Property value -Sum).Sum
    $reconcileVisitCount = [double](@($snapshot.counters) |
            Where-Object { $_.name -in @(
                    "ui.asset_refresh.visual_asset_reconcile_source_visit_count",
                    "ui.asset_refresh.svg_tree_reconcile_source_visit_count"
                ) } |
            Measure-Object -Property value -Sum).Sum
    $reconciledInvalidationCount = [double](@($snapshot.counters) |
            Where-Object { $_.name -in @(
                    "ui.asset_refresh.visual_asset_reconciled_invalidation_count",
                    "ui.asset_refresh.svg_tree_reconciled_invalidation_count"
                ) } |
            Measure-Object -Property value -Sum).Sum
    Write-Host ("- asset_refresh_targeted_invalidation={0} reconcile_visits={1} reconciled_invalidation={2} full_invalidation={3}" -f `
            $targetedInvalidationCount, $reconcileVisitCount,
            $reconciledInvalidationCount, $fullInvalidationCount)
    if ($fullInvalidationCount -gt 0) {
        Write-Warning "Scenario 'asset_refresh' cleared all visual asset caches for a non-visual project change."
        return $false
    }
    return $true
}

function Export-UiSurfacePresentOutcomeEvidence {
    param([string]$ProfileDir)

    Export-ZirconUiSurfacePresentOutcomeEvidence -ProfileDir $ProfileDir
}

function Test-UiSurfaceLatencyEvidenceGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $interactionScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    return Test-ZirconUiSurfaceLatencyEvidenceGate `
        -ProfileDir $ProfileDir `
        -ScenarioName $ScenarioName `
        -InteractionScenarioName $interactionScenario `
        -AutoClickCount $AutoClickCount `
        -AutoPointerMoveCount $AutoPointerMoveCount `
        -AutoWheelCount $AutoWheelCount
}

function Test-InteractionProcessEvidence {
    param(
        [object]$Interaction,
        [int64]$OperationCount = 0,
        [double]$MaxCpuMsPerOperation = 0.0
    )

    return Test-ZirconUiInteractionProcessEvidence `
        -Interaction $Interaction `
        -OperationCount $OperationCount `
        -MaxCpuMsPerOperation $MaxCpuMsPerOperation
}

function Test-WindowResizeCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ((Resolve-InteractionScenarioName -ScenarioName $ScenarioName) -ne "window_resize") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Window resize gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_steps) `
            -MaxCpuMsPerOperation 35.0)) {
        Write-Warning "Window resize gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }
    $counterTotals = @{}
    foreach ($name in @(
            "ui.window_resize.command_snapshot_build_count",
            "ui.window_resize.command_snapshot_reuse_count",
            "ui.window_resize.surface_reconfigure_count",
            "ui.window_resize.duplicate_size_suppressed_count",
            "ui.window_resize.duplicate_scale_suppressed_count",
            "ui.window_resize.workbench_model_build_count",
            "ui.window_resize.chrome_snapshot_count",
            "ui.window_resize.gpu_image_vertices",
            "ui.window_resize.gpu_image_prepare_cache_hits",
            "ui.window_resize.gpu_image_prepare_command_visits",
            "ui.window_resize.gpu_image_upload_writes",
            "ui.window_resize.gpu_image_cache_key_allocations",
            "ui.window_resize.gpu_image_cache_admission_rejects",
            "ui.window_resize.gpu_image_invalid_payloads",
            "ui.window_resize.visual_asset_cache_hit_count",
            "ui.window_resize.visual_asset_cache_miss_count",
            "ui.window_resize.svg_tree_cache_memory_hit_count",
            "ui.window_resize.svg_tree_cache_miss_count",
            "ui.window_resize.visual_asset_full_invalidation_count",
            "ui.window_resize.shell_drag_authority_rebuild_count",
            "ui.window_resize.shell_drag_node_insert_count",
            "ui.window_resize.shell_drag_geometry_patch_count",
            "ui.window_resize.shell_drag_node_patch_count",
            "ui.window_resize.shell_drag_dispatcher_rebuild_count",
            "ui.window_resize.shell_drag_route_map_rebuild_count"
        )) {
        $counterTotals[$name] = [double](@($snapshot.counters) |
                Where-Object { $_.name -eq $name } |
                Measure-Object -Property value -Sum).Sum
    }
    $buildCount = $counterTotals["ui.window_resize.command_snapshot_build_count"]
    $reuseCount = $counterTotals["ui.window_resize.command_snapshot_reuse_count"]
    $surfaceCount = $counterTotals["ui.window_resize.surface_reconfigure_count"]
    $duplicateSizeCount = $counterTotals["ui.window_resize.duplicate_size_suppressed_count"]
    $duplicateScaleCount = $counterTotals["ui.window_resize.duplicate_scale_suppressed_count"]
    $modelCount = $counterTotals["ui.window_resize.workbench_model_build_count"]
    $chromeCount = $counterTotals["ui.window_resize.chrome_snapshot_count"]
    $imageVertexCount = $counterTotals["ui.window_resize.gpu_image_vertices"]
    $imagePrepareCacheHitCount = $counterTotals["ui.window_resize.gpu_image_prepare_cache_hits"]
    $imagePrepareCommandVisitCount = $counterTotals["ui.window_resize.gpu_image_prepare_command_visits"]
    $imageUploadCount = $counterTotals["ui.window_resize.gpu_image_upload_writes"]
    $imageAllocationCount = $counterTotals["ui.window_resize.gpu_image_cache_key_allocations"]
    $imageAdmissionRejectCount = $counterTotals["ui.window_resize.gpu_image_cache_admission_rejects"]
    $imageInvalidPayloadCount = $counterTotals["ui.window_resize.gpu_image_invalid_payloads"]
    $visualHitCount = $counterTotals["ui.window_resize.visual_asset_cache_hit_count"]
    $visualMissCount = $counterTotals["ui.window_resize.visual_asset_cache_miss_count"]
    $svgHitCount = $counterTotals["ui.window_resize.svg_tree_cache_memory_hit_count"]
    $svgMissCount = $counterTotals["ui.window_resize.svg_tree_cache_miss_count"]
    $visualFullInvalidationCount = $counterTotals["ui.window_resize.visual_asset_full_invalidation_count"]
    $shellDragAuthorityRebuildCount = $counterTotals["ui.window_resize.shell_drag_authority_rebuild_count"]
    $shellDragNodeInsertCount = $counterTotals["ui.window_resize.shell_drag_node_insert_count"]
    $shellDragGeometryPatchCount = $counterTotals["ui.window_resize.shell_drag_geometry_patch_count"]
    $shellDragNodePatchCount = $counterTotals["ui.window_resize.shell_drag_node_patch_count"]
    $shellDragDispatcherRebuildCount = $counterTotals["ui.window_resize.shell_drag_dispatcher_rebuild_count"]
    $shellDragRouteMapRebuildCount = $counterTotals["ui.window_resize.shell_drag_route_map_rebuild_count"]
    Write-Host ("- resize_snapshot_build={0} reuse={1} surface_reconfigure={2} duplicate_size_suppressed={3} duplicate_scale_suppressed={4} model_build={5} chrome_snapshot={6} image_vertices={7} image_prepare_cache_hits={8} image_prepare_command_visits={9} image_uploads={10} image_allocations={11} visual_hits={12} visual_misses={13} svg_tree_hits={14} svg_tree_misses={15} shell_drag_authority_rebuild={16} shell_drag_node_insert={17} shell_drag_geometry_patch={18} shell_drag_node_patch={19} shell_drag_dispatcher_rebuild={20} shell_drag_route_map_rebuild={21}" -f `
            $buildCount, $reuseCount, $surfaceCount, $duplicateSizeCount, $duplicateScaleCount,
            $modelCount, $chromeCount, $imageVertexCount, $imagePrepareCacheHitCount,
            $imagePrepareCommandVisitCount, $imageUploadCount, $imageAllocationCount,
            $visualHitCount, $visualMissCount, $svgHitCount, $svgMissCount,
            $shellDragAuthorityRebuildCount, $shellDragNodeInsertCount,
            $shellDragGeometryPatchCount, $shellDragNodePatchCount,
            $shellDragDispatcherRebuildCount, $shellDragRouteMapRebuildCount)

    $expectedSteps = [int64]$evidence.interaction.requested_steps
    $completedSteps = [int64]$evidence.interaction.completed_steps
    return $expectedSteps -gt 1 -and
        $completedSteps -eq $expectedSteps -and
        [bool]$evidence.interaction.restored_original_extent -and
        $buildCount -eq 1 -and
        $reuseCount -gt 0 -and
        $surfaceCount -gt 0 -and
        $surfaceCount -le $completedSteps -and
        $modelCount -le 1 -and
        $chromeCount -le 1 -and
        $imageVertexCount -gt 0 -and
        $imagePrepareCacheHitCount -gt 0 -and
        $imageUploadCount -le 1 -and
        $imageAllocationCount -le 1 -and
        $imageAdmissionRejectCount -eq 0 -and
        $imageInvalidPayloadCount -eq 0 -and
        $visualHitCount -gt 0 -and
        $visualMissCount -eq 0 -and
        $svgHitCount -gt 0 -and
        $svgMissCount -eq 0 -and
        $visualFullInvalidationCount -eq 0 -and
        $shellDragAuthorityRebuildCount -eq 0 -and
        $shellDragNodeInsertCount -eq 0 -and
        $shellDragGeometryPatchCount -gt 0 -and
        $shellDragNodePatchCount -ge $shellDragGeometryPatchCount -and
        $shellDragDispatcherRebuildCount -eq 0 -and
        $shellDragRouteMapRebuildCount -eq 0
}

function Test-HierarchyScrollCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "hierarchy_scroll") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Hierarchy scroll gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_wheel_events) `
            -MaxCpuMsPerOperation 0.25)) {
        Write-Warning "Hierarchy scroll gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.hierarchy_scroll_dispatch_count",
            "ui.idle_hover.hierarchy_surface_rebuild_count",
            "ui.idle_hover.hierarchy_row_insert_count",
            "ui.idle_hover.hierarchy_dispatcher_rebuild_count",
            "ui.idle_hover.hierarchy_route_map_rebuild_count"
        )) {
        $counterTotals[$name] = [double](@($snapshot.counters) |
                Where-Object { $_.name -eq $name } |
                Measure-Object -Property value -Sum).Sum
    }

    $dispatchCount = $counterTotals["ui.idle_hover.hierarchy_scroll_dispatch_count"]
    $surfaceRebuildCount = $counterTotals["ui.idle_hover.hierarchy_surface_rebuild_count"]
    $rowInsertCount = $counterTotals["ui.idle_hover.hierarchy_row_insert_count"]
    $dispatcherRebuildCount = $counterTotals["ui.idle_hover.hierarchy_dispatcher_rebuild_count"]
    $routeMapRebuildCount = $counterTotals["ui.idle_hover.hierarchy_route_map_rebuild_count"]
    Write-Host ("- hierarchy_scroll_dispatch={0} surface_rebuild={1} row_insert={2} dispatcher_rebuild={3} route_map_rebuild={4}" -f `
            $dispatchCount, $surfaceRebuildCount, $rowInsertCount,
            $dispatcherRebuildCount, $routeMapRebuildCount)

    $requestedWheelEvents = [int64]$evidence.interaction.requested_wheel_events
    $completedWheelEvents = [int64]$evidence.interaction.completed_wheel_events
    $hasNoRetainedAuthorityRebuildWork =
        $surfaceRebuildCount -eq 0 -and
        $rowInsertCount -eq 0 -and
        $dispatcherRebuildCount -eq 0 -and
        $routeMapRebuildCount -eq 0
    if ($dispatchCount -ne $completedWheelEvents -or
        $requestedWheelEvents -le 0 -or
        $completedWheelEvents -ne $requestedWheelEvents -or
        -not $hasNoRetainedAuthorityRebuildWork) {
        Write-Warning "Hierarchy scroll profiling counters are missing, inconsistent, or rebuilt retained authority."
        return $false
    }
    return $true
}

function Test-WelcomeRecentScrollCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "welcome_recent_scroll") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $evidencePath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $timelinePath) -or -not (Test-Path $evidencePath)) {
        Write-Warning "Welcome recent scroll gate requires timeline and interaction evidence artifacts."
        return $false
    }
    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $evidence = Get-Content -Path $evidencePath -Raw | ConvertFrom-Json
    if (-not (Test-InteractionProcessEvidence `
            -Interaction $evidence.interaction `
            -OperationCount ([int64]$evidence.interaction.completed_wheel_events) `
            -MaxCpuMsPerOperation 0.25)) {
        Write-Warning "Welcome recent scroll gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    $counterTotals = @{}
    foreach ($name in @(
            "ui.idle_hover.welcome_recent_scroll_dispatch_count",
            "ui.idle_hover.welcome_recent_surface_rebuild_count",
            "ui.idle_hover.welcome_recent_authority_rebuild_count",
            "ui.idle_hover.welcome_recent_row_insert_count",
            "ui.idle_hover.welcome_recent_geometry_patch_count",
            "ui.idle_hover.welcome_recent_dispatcher_rebuild_count",
            "ui.idle_hover.welcome_recent_route_map_rebuild_count"
        )) {
        $counterTotals[$name] = [double](@($snapshot.counters) |
                Where-Object { $_.name -eq $name } |
                Measure-Object -Property value -Sum).Sum
    }

    $dispatchCount = $counterTotals["ui.idle_hover.welcome_recent_scroll_dispatch_count"]
    $surfaceRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_surface_rebuild_count"]
    $authorityRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_authority_rebuild_count"]
    $rowInsertCount = $counterTotals["ui.idle_hover.welcome_recent_row_insert_count"]
    $geometryPatchCount = $counterTotals["ui.idle_hover.welcome_recent_geometry_patch_count"]
    $dispatcherRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_dispatcher_rebuild_count"]
    $routeMapRebuildCount = $counterTotals["ui.idle_hover.welcome_recent_route_map_rebuild_count"]
    Write-Host ("- welcome_recent_scroll_dispatch={0} surface_rebuild={1} authority_rebuild={2} row_insert={3} geometry_patch={4} dispatcher_rebuild={5} route_map_rebuild={6}" -f `
            $dispatchCount, $surfaceRebuildCount, $authorityRebuildCount,
            $rowInsertCount, $geometryPatchCount, $dispatcherRebuildCount,
            $routeMapRebuildCount)

    $requestedWheelEvents = [int64]$evidence.interaction.requested_wheel_events
    $completedWheelEvents = [int64]$evidence.interaction.completed_wheel_events
    $hasNoRetainedAuthorityWork =
        $surfaceRebuildCount -eq 0 -and
        $authorityRebuildCount -eq 0 -and
        $rowInsertCount -eq 0 -and
        $geometryPatchCount -eq 0 -and
        $dispatcherRebuildCount -eq 0 -and
        $routeMapRebuildCount -eq 0
    if ($dispatchCount -ne $completedWheelEvents -or
        $requestedWheelEvents -le 0 -or
        $completedWheelEvents -ne $requestedWheelEvents -or
        -not $hasNoRetainedAuthorityWork) {
        Write-Warning "Welcome recent scroll profiling counters are missing, inconsistent, or rebuilt retained authority."
        return $false
    }
    return $true
}

function Test-UiInteractionEvidenceGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    $interactionScenario = Resolve-InteractionScenarioName -ScenarioName $ScenarioName
    $requiresClickStorm = $interactionScenario -eq "click" -and $AutoClickCount -gt 0
    $requiresPointerStorm = $normalizedScenario -in @("idle_hover", "material_lab_hover") -and
        $AutoPointerMoveCount -gt 0
    $requiresWheelStorm = $normalizedScenario -in @("hierarchy_scroll", "welcome_recent_scroll") -and
        $AutoWheelCount -gt 0
    if ($interactionScenario -ne "drawer_resize" -and -not $requiresClickStorm -and -not $requiresPointerStorm -and -not $requiresWheelStorm) {
        return $true
    }
    $artifactPath = Join-Path $ProfileDir "ui_interaction_evidence.json"
    if (-not (Test-Path $artifactPath)) {
        Write-Warning "Required UI interaction evidence was not exported."
        return $false
    }
    $artifact = Get-Content -Path $artifactPath -Raw | ConvertFrom-Json
    $processOperationCount = 0
    $maxCpuMsPerOperation = 0.0
    if ($requiresClickStorm) {
        $processOperationCount = [int64]$artifact.interaction.completed_clicks
        $maxCpuMsPerOperation = 0.5
    }
    elseif ($requiresPointerStorm) {
        $processOperationCount = [int64]$artifact.interaction.completed_moves
        $maxCpuMsPerOperation = 0.25
    }
    elseif ($requiresWheelStorm) {
        $processOperationCount = [int64]$artifact.interaction.completed_wheel_events
        $maxCpuMsPerOperation = 0.25
    }
    if (($requiresClickStorm -or $requiresPointerStorm -or $requiresWheelStorm) -and
        -not (Test-InteractionProcessEvidence `
            -Interaction $artifact.interaction `
            -OperationCount $processOperationCount `
            -MaxCpuMsPerOperation $maxCpuMsPerOperation)) {
        Write-Warning "UI interaction gate requires complete and internally consistent CPU/RSS evidence."
        return $false
    }

    if ($requiresClickStorm) {
        $requestedClicks = [int64]$artifact.interaction.requested_clicks
        $completedClicks = [int64]$artifact.interaction.completed_clicks
        $targets = @($artifact.interaction.targets)
        $invalidTargets = @($targets | Where-Object {
                [string]::IsNullOrWhiteSpace([string]$_.target_id) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_kind) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_surface) -or
                $_.source -ne "ui_profile_geometry.json"
            })
        if ($artifact.interaction.scenario -ne "pointer_click_storm" -or
            $requestedClicks -ne $AutoClickCount -or
            $completedClicks -ne $requestedClicks -or
            $null -eq $artifact.interaction.processor_time_delta_ms -or
            -not ([bool]$artifact.interaction.used_geometry) -or
            $targets.Count -eq 0 -or
            $invalidTargets.Count -gt 0) {
            Write-Warning "Click storm did not complete the source-bound request with geometry target identity and CPU evidence."
            return $false
        }
        if ($normalizedScenario -eq "material_lab_click") {
            $nonTemplateTargets = @($targets | Where-Object {
                    $_.target_kind -ne "template_control" -or
                    -not ([string]$_.target_id).StartsWith("template.", [StringComparison]::Ordinal)
                })
            if ($nonTemplateTargets.Count -gt 0) {
                Write-Warning "Material Lab click storm included a target outside the dispatchable template-control set."
                return $false
            }
        }
        if ($normalizedScenario -eq "viewport_toolbar_click") {
            $nonViewportToolbarTargets = @($targets | Where-Object {
                    $_.target_kind -ne "viewport_toolbar_control"
                })
            if ($nonViewportToolbarTargets.Count -gt 0) {
                Write-Warning "Viewport toolbar click storm included a target outside the published toolbar-control set."
                return $false
            }
        }
        return $true
    }

    if ($requiresPointerStorm) {
        $requestedMoves = [int64]$artifact.interaction.requested_moves
        $completedMoves = [int64]$artifact.interaction.completed_moves
        $targets = @($artifact.interaction.targets)
        $invalidTargets = @($targets | Where-Object {
                [string]::IsNullOrWhiteSpace([string]$_.target_id) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_kind) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_surface) -or
                $_.source -ne "ui_profile_geometry.json"
            })
        if ($artifact.interaction.scenario -ne "pointer_move_storm" -or
            $requestedMoves -ne $AutoPointerMoveCount -or
            $completedMoves -ne $requestedMoves -or
            $null -eq $artifact.interaction.processor_time_delta_ms -or
            -not ([bool]$artifact.interaction.used_geometry) -or
            $targets.Count -eq 0 -or
            $invalidTargets.Count -gt 0) {
            Write-Warning "Pointer storm did not complete the source-bound request with geometry target identity and CPU evidence."
            return $false
        }
        if ($normalizedScenario -eq "material_lab_hover") {
            $nonTemplateTargets = @($targets | Where-Object {
                    $_.target_kind -ne "template_control" -or
                    -not ([string]$_.target_id).StartsWith("template.", [StringComparison]::Ordinal)
                })
            if ($nonTemplateTargets.Count -gt 0) {
                Write-Warning "Material Lab pointer storm included a target outside the dispatchable template-control set."
                return $false
            }
        }
        return $true
    }

    if ($requiresWheelStorm) {
        $requestedWheelEvents = [int64]$artifact.interaction.requested_wheel_events
        $completedWheelEvents = [int64]$artifact.interaction.completed_wheel_events
        $targets = @($artifact.interaction.targets)
        $invalidTargets = @($targets | Where-Object {
                [string]::IsNullOrWhiteSpace([string]$_.target_id) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_kind) -or
                [string]::IsNullOrWhiteSpace([string]$_.target_surface) -or
                $_.source -ne "ui_profile_geometry.json"
            })
        $hasExpectedTarget = if ($normalizedScenario -eq "welcome_recent_scroll") {
            $targets.Count -eq 1 -and
                $targets[0].target_id -eq "welcome.recent.viewport" -and
                $targets[0].target_kind -eq "welcome_recent_viewport" -and
                -not [string]::IsNullOrWhiteSpace([string]$targets[0].target_surface)
        }
        else {
            $targets.Count -eq 1 -and
                $targets[0].target_id -eq "layout.left_region" -and
                $targets[0].target_kind -eq "pane_region" -and
                $targets[0].target_surface -eq "left"
        }
        if ($artifact.interaction.scenario -ne "pointer_wheel_storm" -or
            $requestedWheelEvents -ne $AutoWheelCount -or
            $completedWheelEvents -ne $requestedWheelEvents -or
            $null -eq $artifact.interaction.processor_time_delta_ms -or
            -not ([bool]$artifact.interaction.used_geometry) -or
            -not $hasExpectedTarget -or
            $invalidTargets.Count -gt 0) {
            Write-Warning "Wheel storm did not complete the source-bound request on the expected live pane with CPU evidence."
            return $false
        }
        return $true
    }

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

function Wait-ProfileMeasurementReady {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$ProfileDir,
        [int]$TimeoutSeconds = 10
    )

    if (-not (Test-EnvTruthy "ZIRCON_PROFILE_CAPTURE")) {
        return $true
    }
    $warmupPresentCount = 0
    if (-not [int]::TryParse(
            [string]$env:ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS,
            [ref]$warmupPresentCount) -or $warmupPresentCount -le 0) {
        return $true
    }

    $path = Join-Path $ProfileDir "ui_profile_measurement_ready.json"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) {
            return $false
        }
        if (Test-Path $path) {
            try {
                $ready = Get-Content -Path $path -Raw | ConvertFrom-Json
                if ([int]$ready.schema_version -eq 1 -and
                    [bool]$ready.measurement_ready -and
                    [int]$ready.process_id -eq $Process.Id) {
                    return $true
                }
            }
            catch {
                # The producer writes through an atomic rename; retry malformed external files.
            }
        }
        Start-Sleep -Milliseconds 100
    }
    return $false
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
    if ($interactionScenario -notin @("drag", "drawer_resize", "window_resize", "click", "idle_hover", "hierarchy_scroll", "asset_refresh", "viewport_image")) {
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

function Get-LiveGeometryInteractionTargets {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry,
        [switch]$TemplateControlsOnly,
        [switch]$ViewportToolbarControlsOnly
    )

    $frames = if ($ViewportToolbarControlsOnly) {
        @($Geometry.viewport_toolbar_controls)
    }
    elseif ($TemplateControlsOnly) {
        @($Geometry.template_controls)
    }
    else {
        @($Geometry.activity_rail_buttons) + @($Geometry.document_tabs) + @($Geometry.viewport_toolbar_controls) + @($Geometry.template_controls)
    }
    $targets = @()
    foreach ($entry in $frames) {
        if ($targets.Count -ge 8) { break }
        if ($null -eq $entry.frame -or [double]$entry.frame.width -le 0 -or [double]$entry.frame.height -le 0) {
            continue
        }
        $point = Get-CapturePointFromFrame -Rect $Rect -Frame $entry.frame
        $targets += [pscustomobject]@{
            X = $point.X
            Y = $point.Y
            target_id = [string]$entry.id
            target_kind = [string]$entry.kind
            target_surface = [string]$entry.surface
            source = "ui_profile_geometry.json"
        }
    }
    return $targets
}

function Get-LiveGeometryScrollTargets {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry
    )

    $frame = $Geometry.layout.left_region
    if ($null -eq $frame -or [double]$frame.width -le 0 -or [double]$frame.height -le 0) {
        return @()
    }
    $point = Get-CapturePointFromFrame -Rect $Rect -Frame $frame
    return @([pscustomobject]@{
            X = $point.X
            Y = $point.Y
            target_id = "layout.left_region"
            target_kind = "pane_region"
            target_surface = "left"
            source = "ui_profile_geometry.json"
        })
}

function Get-WelcomeRecentScrollTargets {
    param(
        [ZirconProfileCaptureRect]$Rect,
        [object]$Geometry
    )

    $entry = $Geometry.welcome_recent_frame
    if ($null -eq $entry -or
        $entry.id -ne "welcome.recent.viewport" -or
        $entry.kind -ne "welcome_recent_viewport" -or
        [string]::IsNullOrWhiteSpace([string]$entry.surface) -or
        $null -eq $entry.frame -or
        [double]$entry.frame.width -le 0 -or
        [double]$entry.frame.height -le 0) {
        return @()
    }
    $point = Get-CapturePointFromFrame -Rect $Rect -Frame $entry.frame
    return @([pscustomobject]@{
            X = $point.X
            Y = $point.Y
            target_id = [string]$entry.id
            target_kind = [string]$entry.kind
            target_surface = [string]$entry.surface
            source = "ui_profile_geometry.json"
        })
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

    if (-not (Wait-ProfileMeasurementReady -Process $Process -ProfileDir $ProfileDir)) {
        throw "Editor did not publish the current-process UI measurement epoch before automatic input."
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
    $pointerTargets = @()

    if ($interactionScenario -eq "asset_refresh") {
        Invoke-AssetRefreshChange -SessionId $SessionId
    }

    $interactionKind = if ($normalizedScenario -in @("hierarchy_scroll", "welcome_recent_scroll")) {
        $normalizedScenario
    }
    else {
        $interactionScenario
    }
    switch ($interactionKind) {
        "idle_hover" {
            $templateControlsOnly = $normalizedScenario -eq "material_lab_hover"
            $pointerTargets = if ($null -ne $geometry) {
                @(Get-LiveGeometryInteractionTargets -Rect $rect -Geometry $geometry -TemplateControlsOnly:$templateControlsOnly)
            }
            else {
                @()
            }
            if ($pointerTargets.Count -gt 0) {
                if ($AutoPointerMoveCount -le 0) {
                    foreach ($target in $pointerTargets) {
                        Move-CaptureCursor -Point $target -DelayMs 180
                    }
                }
                break
            }
            $fallbackPoints = @(
                (Get-CapturePoint -Rect $rect -XRatio 0.04 -YRatio 0.04),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.14),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.05 -YRatio 0.22),
                (Get-CapturePoint -Rect $rect -XRatio 0.12 -YRatio 0.18),
                (Get-CapturePoint -Rect $rect -XRatio 0.34 -YRatio 0.20),
                (Get-CapturePoint -Rect $rect -XRatio 0.82 -YRatio 0.55),
                (Get-CapturePoint -Rect $rect -XRatio 0.38 -YRatio 0.72)
            )
            $pointerTargets = for ($index = 0; $index -lt $fallbackPoints.Count; $index++) {
                [pscustomobject]@{
                    X = $fallbackPoints[$index].X
                    Y = $fallbackPoints[$index].Y
                    target_id = "fallback.ratio.$index"
                    target_kind = "fallback"
                    target_surface = "window"
                    source = "ratio_fallback"
                }
            }
            if ($AutoPointerMoveCount -le 0) {
                foreach ($target in $pointerTargets) {
                    Move-CaptureCursor -Point $target -DelayMs 180
                }
            }
        }
        "click" {
            $templateControlsOnly = $normalizedScenario -eq "material_lab_click"
            $viewportToolbarControlsOnly = $normalizedScenario -eq "viewport_toolbar_click"
            $clickTargets = if ($null -ne $geometry) {
                @(Get-LiveGeometryInteractionTargets `
                        -Rect $rect `
                        -Geometry $geometry `
                        -TemplateControlsOnly:$templateControlsOnly `
                        -ViewportToolbarControlsOnly:$viewportToolbarControlsOnly)
            }
            else {
                @()
            }
            if ($clickTargets.Count -eq 0) {
                $fallbackPoints = @(
                    (Get-CapturePoint -Rect $rect -XRatio 0.12 -YRatio 0.18),
                    (Get-CapturePoint -Rect $rect -XRatio 0.30 -YRatio 0.16),
                    (Get-CapturePoint -Rect $rect -XRatio 0.52 -YRatio 0.18)
                )
                $clickTargets = for ($index = 0; $index -lt $fallbackPoints.Count; $index++) {
                    [pscustomobject]@{
                        X = $fallbackPoints[$index].X
                        Y = $fallbackPoints[$index].Y
                        target_id = "fallback.ratio.$index"
                        target_kind = "fallback"
                        target_surface = "window"
                        source = "ratio_fallback"
                    }
                }
            }
            if ($AutoClickCount -gt 0) {
                $script:LastInteractionEvidence = Invoke-PointerClickStorm `
                    -Process $Process `
                    -Targets $clickTargets `
                    -Count $AutoClickCount `
                    -DelayMs $AutoClickDelayMs
                break
            }
            foreach ($target in @($clickTargets | Select-Object -First 3)) {
                Click-CapturePoint -Point $target
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
        "window_resize" {
            $script:LastInteractionEvidence = Invoke-ZirconNativeResizeInteraction `
                -Process $Process `
                -StepCount $AutoResizeStepCount `
                -DelayMs $AutoResizeDelayMs
        }
        "hierarchy_scroll" {
            $scrollTargets = if ($null -ne $geometry) {
                @(Get-LiveGeometryScrollTargets -Rect $rect -Geometry $geometry)
            }
            else {
                @()
            }
            if ($scrollTargets.Count -eq 0) {
                $fallback = Get-CapturePoint -Rect $rect -XRatio 0.14 -YRatio 0.48
                $scrollTargets = @([pscustomobject]@{
                        X = $fallback.X
                        Y = $fallback.Y
                        target_id = "fallback.ratio.left_pane"
                        target_kind = "fallback"
                        target_surface = "left"
                        source = "ratio_fallback"
                    })
            }
            $wheelCount = Get-ScenarioRequestedWheelOperationCount -ScenarioName $ScenarioName
            $script:LastInteractionEvidence = Invoke-PointerWheelStorm `
                -Process $Process `
                -Targets $scrollTargets `
                -Count $wheelCount `
                -DelayMs $AutoWheelDelayMs
        }
        "welcome_recent_scroll" {
            $scrollTargets = if ($null -ne $geometry) {
                @(Get-WelcomeRecentScrollTargets -Rect $rect -Geometry $geometry)
            }
            else {
                @()
            }
            if ($scrollTargets.Count -eq 0) {
                $fallback = Get-CapturePoint -Rect $rect -XRatio 0.50 -YRatio 0.68
                $scrollTargets = @([pscustomobject]@{
                        X = $fallback.X
                        Y = $fallback.Y
                        target_id = "fallback.ratio.welcome_recent"
                        target_kind = "fallback"
                        target_surface = "document"
                        source = "ratio_fallback"
                    })
            }
            $wheelCount = Get-ScenarioRequestedWheelOperationCount -ScenarioName $ScenarioName
            $script:LastInteractionEvidence = Invoke-PointerWheelStorm `
                -Process $Process `
                -Targets $scrollTargets `
                -Count $wheelCount `
                -DelayMs $AutoWheelDelayMs
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
    if ($interactionScenario -eq "idle_hover" -and $AutoPointerMoveCount -gt 0) {
        $script:LastInteractionEvidence = Invoke-PointerMoveStorm `
            -Process $Process `
            -Targets $pointerTargets `
            -Count $AutoPointerMoveCount `
            -DelayMs $AutoPointerMoveDelayMs
    }
    if ($null -ne $script:LastInteractionEvidence) {
        $script:LastInteractionEvidence = Complete-ZirconProcessQuiescenceEvidence `
            -Process $Process `
            -Interaction $script:LastInteractionEvidence `
            -QuiescenceSeconds $WithinProcessQuiescenceSeconds
    }
    Export-UiInteractionEvidence -ProfileDir $ProfileDir -ScenarioName $ScenarioName -BeforeGeometry $geometry -BeforeWriteTimeUtc $geometryWriteTimeUtc -Interaction $script:LastInteractionEvidence
}

function Resolve-ProfileProjectRoot {
    param([string]$SessionId)
    return Join-Path (Join-Path (Join-Path $OutputPath "profile-projects") $SessionId) "ProfileCaptureProject"
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
    $scaleAssetPath = Join-Path $assetsRoot "profile_catalog_asset_000001.json"
    $materialPath = Join-Path (Join-Path $assetsRoot "materials") "default.zmaterial"
    if ($AssetCatalogItemCount -gt 0 -and (Test-Path -LiteralPath $scaleAssetPath)) {
        $encoding = New-Object System.Text.UTF8Encoding($false)
        $payload = [ordered]@{
            profile_asset_index = 1
            profile_refresh_generation = (Get-Date).ToUniversalTime().ToString("o")
        } | ConvertTo-Json -Compress
        [System.IO.File]::WriteAllText($scaleAssetPath, $payload, $encoding)
    }
    elseif (Test-Path $materialPath) {
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
    if ($normalizedScenario -eq "hierarchy_scroll" -and $HierarchyLogicalNodeCount -gt 0) {
        $projectRoot = Resolve-ProfileProjectRoot -SessionId $SessionId
        if (-not (Test-Path -LiteralPath $projectRoot -PathType Container)) {
            throw "Hierarchy scale project was not materialized before Editor launch: $projectRoot"
        }
        return @("--project", $projectRoot)
    }
    if ($normalizedScenario -eq "asset_refresh" -and $AssetCatalogItemCount -gt 0) {
        $projectRoot = Resolve-ProfileProjectRoot -SessionId $SessionId
        if (-not (Test-Path -LiteralPath $projectRoot -PathType Container)) {
            throw "Asset catalog scale project was not materialized before Editor launch: $projectRoot"
        }
        return @("--project", $projectRoot)
    }
    if ($normalizedScenario -in @("material_lab_startup", "material_lab_hover", "material_lab_click")) {
        return @(
            "--builtin-view",
            "editor.material_component_lab"
        )
    }
    if ($normalizedScenario -notin @("idle_hover", "viewport_toolbar_click", "viewport_image", "drag", "drawer_resize", "window_resize", "hierarchy_scroll", "asset_refresh")) {
        return @()
    }

    $projectLocation = Join-Path (Join-Path $OutputPath "profile-projects") $SessionId
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
    return $normalizedScenario -in @("idle_hover", "viewport_toolbar_click", "viewport_image", "drag", "drawer_resize", "window_resize", "hierarchy_scroll", "asset_refresh")
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
        if (Test-EnvTruthy "ZIRCON_PROFILE_CAPTURE") {
            Remove-Item -LiteralPath (Join-Path $profileDir "ui_profile_measurement_ready.json") `
                -Force -ErrorAction SilentlyContinue
        }
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

# Profiling binaries are produced only by the coordinator-managed Windows build.
if (-not $SkipBuild) {
    throw "Profile capture requires a managed profiling build. Run the coordinator-owned Windows validation build first, then rerun with -SkipBuild."
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

if ($HierarchyLogicalNodeCount -gt 0 -and
    @($captureScenarios | Where-Object { $_.Trim().ToLowerInvariant() -ne "hierarchy_scroll" }).Count -gt 0) {
    throw "Hierarchy scale inputs are valid only for the hierarchy_scroll scenario."
}
if ($AssetCatalogItemCount -gt 0 -and
    @($captureScenarios | Where-Object { $_.Trim().ToLowerInvariant() -ne "asset_refresh" }).Count -gt 0) {
    throw "Asset catalog scale inputs are valid only for the asset_refresh scenario."
}
if ($HierarchyLogicalNodeCount -gt 0 -and $AssetCatalogItemCount -gt 0) {
    throw "Only one UI profile scale input can be active for a capture."
}

$previous = @{
    ZIRCON_PROFILE_CAPTURE = $env:ZIRCON_PROFILE_CAPTURE
    ZIRCON_PROFILE_SESSION = $env:ZIRCON_PROFILE_SESSION
    ZIRCON_PROFILE_OUTPUT_ROOT = $env:ZIRCON_PROFILE_OUTPUT_ROOT
    ZIRCON_PROFILE_MAX_FRAMES = $env:ZIRCON_PROFILE_MAX_FRAMES
    ZIRCON_PROFILE_MAX_SPANS = $env:ZIRCON_PROFILE_MAX_SPANS
    ZIRCON_PROFILE_MAX_COUNTERS = $env:ZIRCON_PROFILE_MAX_COUNTERS
    ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS = $env:ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS
    ZIRCON_PROFILE_CAPTURE_SCREENSHOTS = $env:ZIRCON_PROFILE_CAPTURE_SCREENSHOTS
    ZIRCON_PROFILE_FORCE_SOFTBUFFER = $env:ZIRCON_PROFILE_FORCE_SOFTBUFFER
    ZIRCON_RUNTIME_LIBRARY = $env:ZIRCON_RUNTIME_LIBRARY
}

if ($UseTracy -and (Test-Path $TracyProfiler)) {
    Start-Process -FilePath $TracyProfiler | Out-Null
}

try {
    foreach ($scenarioName in $captureScenarios) {
        for ($runIndex = 0; $runIndex -lt $MeasuredRunCount; $runIndex++) {
            $runPhase = 'measured'
            $phaseRunOrdinal = $runIndex + 1
            $withinProcessWarmupPresentCount = Get-ScenarioWithinProcessWarmupPresentCount -ScenarioName $scenarioName
            $runProcessScope = if ($withinProcessWarmupPresentCount -gt 0) {
                'within_process_warm_measure'
            }
            else {
                'fresh_process_startup'
            }
            $SessionId = "$(Get-Date -Format 'yyyyMMdd-HHmmss')-$scenarioName-$runPhase-$('{0:D2}' -f $phaseRunOrdinal)"
            $ProfileDir = Join-Path $OutputPath $SessionId
            $wprStarted = $false
            $SoftbufferSessionId = $null
            $inputFixture = $null
            $requestedWheelOperationCount = Get-ScenarioRequestedWheelOperationCount -ScenarioName $scenarioName

        if ($HierarchyLogicalNodeCount -gt 0) {
            $inputFixture = New-ZirconUiHierarchyScaleFixture `
                -RepoRoot $RepoRoot `
                -ProjectRoot (Resolve-ProfileProjectRoot -SessionId $SessionId) `
                -LogicalNodeCount $HierarchyLogicalNodeCount
        }
        elseif ($AssetCatalogItemCount -gt 0) {
            $inputFixture = New-ZirconUiAssetCatalogScaleFixture `
                -RepoRoot $RepoRoot `
                -ProjectRoot (Resolve-ProfileProjectRoot -SessionId $SessionId) `
                -AssetItemCount $AssetCatalogItemCount
        }

        $env:ZIRCON_PROFILE_CAPTURE = "1"
        $env:ZIRCON_PROFILE_SESSION = $SessionId
        $env:ZIRCON_PROFILE_OUTPUT_ROOT = $OutputPath
        $env:ZIRCON_PROFILE_MAX_FRAMES = "$MaxFrames"
        $env:ZIRCON_PROFILE_MAX_SPANS = "$MaxSpans"
        $env:ZIRCON_PROFILE_MAX_COUNTERS = "$MaxCounters"
        $env:ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS = "$withinProcessWarmupPresentCount"
        $env:ZIRCON_PROFILE_CAPTURE_SCREENSHOTS = "1"
        $env:ZIRCON_RUNTIME_LIBRARY = $RuntimeDll

        $sourceManifest = Export-ZirconProfileCaptureManifest `
            -ProfileDir $ProfileDir `
            -RepoRoot $RepoRoot `
            -OutputRoot $OutputPath `
            -VerificationScreenshotRoot $VerificationScreenshotRoot `
            -TargetDir $TargetDir `
            -SessionId $SessionId `
            -ScenarioName $scenarioName `
            -EditorExe $EditorExe `
            -RuntimeDll $RuntimeDll `
            -InputFixture $inputFixture `
            -CaptureOptions @{
                auto_close_seconds = $AutoCloseSeconds
                auto_interact = $AutoInteract.IsPresent
                auto_pointer_move_count = $AutoPointerMoveCount
                auto_pointer_move_delay_ms = $AutoPointerMoveDelayMs
                auto_click_count = $AutoClickCount
                auto_click_delay_ms = $AutoClickDelayMs
                auto_wheel_count = $AutoWheelCount
                auto_wheel_delay_ms = $AutoWheelDelayMs
                hierarchy_logical_node_count = $HierarchyLogicalNodeCount
                asset_catalog_item_count = $AssetCatalogItemCount
                requested_wheel_operation_count = $requestedWheelOperationCount
                auto_resize_step_count = $AutoResizeStepCount
                auto_resize_delay_ms = $AutoResizeDelayMs
                run_phase = $runPhase
                run_ordinal = $phaseRunOrdinal
                measured_run_count = $MeasuredRunCount
                run_quiescence_seconds = $RunQuiescenceSeconds
                run_process_scope = $runProcessScope
                within_process_warmup = $withinProcessWarmupPresentCount -gt 0
                within_process_warmup_present_count = $withinProcessWarmupPresentCount
                within_process_quiescence_seconds = $WithinProcessQuiescenceSeconds
                capture_softbuffer_screenshot = $CaptureSoftbufferScreenshot.IsPresent
                max_counters = $MaxCounters
                max_frames = $MaxFrames
                max_spans = $MaxSpans
                use_tracy = $UseTracy.IsPresent
                use_wpr = $UseWpr.IsPresent
            }
        Write-Host "Source manifest: $sourceManifest"

        if ($UseWpr -and $runPhase -eq 'measured') {
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
            if ($runPhase -eq 'measured') {
                $SoftbufferSessionId = Invoke-SoftbufferScreenshotCapture -ScenarioName $scenarioName -SessionId $SessionId
            }
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
            Export-UiSurfacePresentOutcomeEvidence -ProfileDir $ProfileDir
            $batchMetricsOk = Test-UiBatchMetricsGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $hitConsistencyOk = Test-UiHitConsistencyGate -ProfileDir $ProfileDir
            $screenshotDiffOk = Test-ScreenshotDiffGate -ProfileDir $ProfileDir
            $assetRefreshOk = Test-AssetRefreshCounterGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $windowResizeOk = Test-WindowResizeCounterGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $hierarchyScrollOk = Test-HierarchyScrollCounterGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $welcomeRecentScrollOk = Test-WelcomeRecentScrollCounterGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $interactionEvidenceOk = Test-UiInteractionEvidenceGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            $surfaceLatencyOk = Test-UiSurfaceLatencyEvidenceGate -ProfileDir $ProfileDir -ScenarioName $scenarioName
            Export-SoftbufferRunManifest -ProfileDir $ProfileDir -SoftbufferSessionId $SoftbufferSessionId
            $verificationScreenshotDir = if ($runPhase -eq 'measured') {
                Export-VerificationScreenshots -ProfileDir $ProfileDir -SessionId $SessionId
            }
            else {
                $null
            }
            if ($null -ne $verificationScreenshotDir) {
                Write-Host "Verification screenshots: $verificationScreenshotDir"
            }
            if ($RequireScenarioEvidence -and $runPhase -eq 'measured' -and -not ($scenarioEvidenceOk -and $batchMetricsOk -and $hitConsistencyOk -and $screenshotDiffOk -and $assetRefreshOk -and $windowResizeOk -and $hierarchyScrollOk -and $welcomeRecentScrollOk -and $interactionEvidenceOk -and $surfaceLatencyOk)) {
                throw "Scenario '$scenarioName' did not meet the requested evidence gate."
            }
        }
        else {
            Write-Warning "Profile directory was not created. Check whether the editor exited normally and profiling features were enabled."
        }
            if ($runIndex -lt ($MeasuredRunCount - 1) -and $RunQuiescenceSeconds -gt 0) {
                Write-Host "Quiescence between measured profile processes: $RunQuiescenceSeconds seconds"
                Start-Sleep -Seconds $RunQuiescenceSeconds
            }
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
