[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BundleDirectory,

    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9A-Fa-f]{64}$')]
    [string]$ExpectedEditorSha256,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9A-Fa-f]{64}$')]
    [string]$ExpectedRuntimeSha256,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9A-Fa-f]{64}$')]
    [string]$ExpectedSourceSha256,

    [string]$ProfileSessionId = 'editor-ui-visual-acceptance',
    [switch]$SkipVisualOracle
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $repoRoot 'tools\profile-capture-paths.ps1')
. (Join-Path $repoRoot 'tools\profile-capture-manifest.ps1')
. (Join-Path $repoRoot 'tools\editor-ui-visual-source-binding.ps1')
. (Join-Path $repoRoot 'tools\editor-ui-visual-interactions.ps1')

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
public struct ZirconEditorVisualCaptureRect
{
    public int Left;
    public int Top;
    public int Right;
    public int Bottom;
}

[StructLayout(LayoutKind.Sequential)]
public struct ZirconEditorVisualCapturePoint
{
    public int X;
    public int Y;
}

public static class ZirconEditorVisualCaptureNative
{
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr window, out ZirconEditorVisualCaptureRect rect);

    [DllImport("user32.dll")]
    public static extern bool GetClientRect(IntPtr window, out ZirconEditorVisualCaptureRect rect);

    [DllImport("user32.dll")]
    public static extern bool ClientToScreen(IntPtr window, ref ZirconEditorVisualCapturePoint point);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr window);

    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(
        IntPtr window,
        IntPtr insertAfter,
        int x,
        int y,
        int width,
        int height,
        uint flags);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);

    [DllImport("user32.dll")]
    public static extern uint GetDpiForWindow(IntPtr window);

    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int index);

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);

    [DllImport("dwmapi.dll")]
    public static extern int DwmFlush();
}
"@

function Get-ZirconEditorVisualClientGeometry {
    param([Parameter(Mandatory = $true)][IntPtr]$Window)

    $client = New-Object ZirconEditorVisualCaptureRect
    $windowRect = New-Object ZirconEditorVisualCaptureRect
    $origin = New-Object ZirconEditorVisualCapturePoint
    if (-not [ZirconEditorVisualCaptureNative]::GetClientRect($Window, [ref]$client) -or
        -not [ZirconEditorVisualCaptureNative]::GetWindowRect($Window, [ref]$windowRect) -or
        -not [ZirconEditorVisualCaptureNative]::ClientToScreen($Window, [ref]$origin)) {
        throw 'Could not query the editor client geometry.'
    }

    [pscustomobject]@{
        ClientLeft = $origin.X
        ClientTop = $origin.Y
        ClientWidth = $client.Right - $client.Left
        ClientHeight = $client.Bottom - $client.Top
        WindowLeft = $windowRect.Left
        WindowTop = $windowRect.Top
        WindowWidth = $windowRect.Right - $windowRect.Left
        WindowHeight = $windowRect.Bottom - $windowRect.Top
    }
}

function Set-ZirconEditorVisualClientExtent {
    param(
        [Parameter(Mandatory = $true)][IntPtr]$Window,
        [Parameter(Mandatory = $true)][int]$Width,
        [Parameter(Mandatory = $true)][int]$Height
    )

    for ($attempt = 0; $attempt -lt 8; $attempt += 1) {
        $geometry = Get-ZirconEditorVisualClientGeometry -Window $Window
        if ($geometry.ClientWidth -eq $Width -and $geometry.ClientHeight -eq $Height) {
            return $geometry
        }
        $outerWidth = $geometry.WindowWidth + ($Width - $geometry.ClientWidth)
        $outerHeight = $geometry.WindowHeight + ($Height - $geometry.ClientHeight)
        if (-not [ZirconEditorVisualCaptureNative]::SetWindowPos(
                $Window,
                [IntPtr]::Zero,
                20,
                20,
                $outerWidth,
                $outerHeight,
                0x0004)) {
            throw "Could not resize the editor client to ${Width}x${Height}."
        }
        Start-Sleep -Milliseconds 180
    }

    $geometry = Get-ZirconEditorVisualClientGeometry -Window $Window
    if ($geometry.ClientWidth -ne $Width -or $geometry.ClientHeight -ne $Height) {
        throw "Editor client extent is $($geometry.ClientWidth)x$($geometry.ClientHeight), expected ${Width}x${Height}."
    }
    return $geometry
}

function Save-ZirconEditorVisualClientScreenshot {
    param(
        [Parameter(Mandatory = $true)][IntPtr]$Window,
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int]$Width,
        [Parameter(Mandatory = $true)][int]$Height,
        [switch]$PreservePointerPosition
    )

    $geometry = Set-ZirconEditorVisualClientExtent -Window $Window -Width $Width -Height $Height
    [ZirconEditorVisualCaptureNative]::SetForegroundWindow($Window) | Out-Null
    if (-not $PreservePointerPosition) {
        [ZirconEditorVisualCaptureNative]::SetCursorPos(0, 0) | Out-Null
    }
    [ZirconEditorVisualCaptureNative]::DwmFlush() | Out-Null
    Start-Sleep -Milliseconds 750

    $geometry = Get-ZirconEditorVisualClientGeometry -Window $Window
    if ($geometry.ClientWidth -ne $Width -or $geometry.ClientHeight -ne $Height) {
        throw "Editor client changed before capture: $($geometry.ClientWidth)x$($geometry.ClientHeight)."
    }

    $bitmap = [System.Drawing.Bitmap]::new($Width, $Height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $virtualLeft = [ZirconEditorVisualCaptureNative]::GetSystemMetrics(76)
        $virtualTop = [ZirconEditorVisualCaptureNative]::GetSystemMetrics(77)
        $virtualWidth = [ZirconEditorVisualCaptureNative]::GetSystemMetrics(78)
        $virtualHeight = [ZirconEditorVisualCaptureNative]::GetSystemMetrics(79)
        if ($virtualWidth -le 0 -or $virtualHeight -le 0) {
            throw 'Could not resolve the Windows virtual-screen extent.'
        }

        for ($destinationY = 0; $destinationY -lt $Height; $destinationY += $virtualHeight) {
            for ($destinationX = 0; $destinationX -lt $Width; $destinationX += $virtualWidth) {
                $geometry = Get-ZirconEditorVisualClientGeometry -Window $Window
                $desiredClientLeft = $virtualLeft - $destinationX
                $desiredClientTop = $virtualTop - $destinationY
                $windowLeft = $geometry.WindowLeft + ($desiredClientLeft - $geometry.ClientLeft)
                $windowTop = $geometry.WindowTop + ($desiredClientTop - $geometry.ClientTop)
                if (-not [ZirconEditorVisualCaptureNative]::SetWindowPos(
                        $Window,
                        [IntPtr](-1),
                        $windowLeft,
                        $windowTop,
                        $geometry.WindowWidth,
                        $geometry.WindowHeight,
                        0x0040)) {
                    throw 'Could not position the editor window for tiled capture.'
                }
                [ZirconEditorVisualCaptureNative]::SetForegroundWindow($Window) | Out-Null
                [ZirconEditorVisualCaptureNative]::DwmFlush() | Out-Null
                Start-Sleep -Milliseconds 250

                $geometry = Get-ZirconEditorVisualClientGeometry -Window $Window
                if ($geometry.ClientLeft -ne $desiredClientLeft -or
                    $geometry.ClientTop -ne $desiredClientTop) {
                    throw "Editor client stopped at ($($geometry.ClientLeft),$($geometry.ClientTop)); " +
                        "expected ($desiredClientLeft,$desiredClientTop) for tiled capture."
                }
                $sourceLeft = [Math]::Max($geometry.ClientLeft, $virtualLeft)
                $sourceTop = [Math]::Max($geometry.ClientTop, $virtualTop)
                $sourceRight = [Math]::Min(
                    $geometry.ClientLeft + $Width,
                    $virtualLeft + $virtualWidth)
                $sourceBottom = [Math]::Min(
                    $geometry.ClientTop + $Height,
                    $virtualTop + $virtualHeight)
                $tileWidth = $sourceRight - $sourceLeft
                $tileHeight = $sourceBottom - $sourceTop
                $bitmapX = $sourceLeft - $geometry.ClientLeft
                $bitmapY = $sourceTop - $geometry.ClientTop
                if ($tileWidth -le 0 -or $tileHeight -le 0) {
                    throw 'Editor capture tile does not intersect the Windows virtual screen.'
                }
                $graphics.CopyFromScreen(
                    $sourceLeft,
                    $sourceTop,
                    $bitmapX,
                    $bitmapY,
                    [System.Drawing.Size]::new($tileWidth, $tileHeight),
                    [System.Drawing.CopyPixelOperation]::SourceCopy)
            }
        }
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)

        $colors = [System.Collections.Generic.HashSet[int]]::new()
        $minimumLuma = 255
        $maximumLuma = 0
        for ($y = 0; $y -lt $Height; $y += 8) {
            for ($x = 0; $x -lt $Width; $x += 8) {
                $pixel = $bitmap.GetPixel($x, $y)
                $colors.Add($pixel.ToArgb()) | Out-Null
                $luma = [int][Math]::Round(
                    (0.2126 * $pixel.R) + (0.7152 * $pixel.G) + (0.0722 * $pixel.B))
                $minimumLuma = [Math]::Min($minimumLuma, $luma)
                $maximumLuma = [Math]::Max($maximumLuma, $luma)
            }
        }
        if ($colors.Count -lt 32 -or ($maximumLuma - $minimumLuma) -lt 20) {
            throw "Captured image is blank or low-information: colors=$($colors.Count), luma=${minimumLuma}..${maximumLuma}."
        }
        [pscustomobject]@{
            path = $Path
            width = $Width
            height = $Height
            sha256 = Get-ZirconProfileFileSha256 -Path $Path
            sampled_colors = $colors.Count
            sampled_luma_minimum = $minimumLuma
            sampled_luma_maximum = $maximumLuma
        }
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Wait-ZirconEditorVisualProfileGeometry {
    param(
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)][string]$Path,
        [int]$TimeoutSeconds = 30
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
        }
        if ($Process.HasExited) {
            throw "Editor exited before publishing presenter evidence (exit $($Process.ExitCode))."
        }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Editor did not publish presenter evidence within ${TimeoutSeconds}s: $Path"
}

function Start-ZirconEditorVisualProcess {
    param(
        [Parameter(Mandatory = $true)][string]$Editor,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string]$StandardOutputPath,
        [Parameter(Mandatory = $true)][string]$StandardErrorPath,
        [Parameter(Mandatory = $true)][hashtable]$Environment
    )

    $previousEnvironment = @{}
    foreach ($name in $Environment.Keys) {
        $previousEnvironment[$name] = [Environment]::GetEnvironmentVariable(
            $name,
            [EnvironmentVariableTarget]::Process)
        [Environment]::SetEnvironmentVariable(
            $name,
            [string]$Environment[$name],
            [EnvironmentVariableTarget]::Process)
    }
    try {
        return Start-Process `
            -FilePath $Editor `
            -WorkingDirectory $WorkingDirectory `
            -PassThru `
            -RedirectStandardOutput $StandardOutputPath `
            -RedirectStandardError $StandardErrorPath
    }
    finally {
        foreach ($name in $Environment.Keys) {
            [Environment]::SetEnvironmentVariable(
                $name,
                $previousEnvironment[$name],
                [EnvironmentVariableTarget]::Process)
        }
    }
}

$editor = Join-Path $BundleDirectory 'zircon_editor.exe'
$runtime = Join-Path $BundleDirectory 'zircon_runtime.dll'
if (-not (Test-Path -LiteralPath $editor -PathType Leaf)) {
    throw "Editor executable does not exist: $editor"
}
if (-not (Test-Path -LiteralPath $runtime -PathType Leaf)) {
    throw "Runtime library does not exist: $runtime"
}
$editorFingerprint = Get-ZirconProfileRequiredFileFingerprint `
    -Path $editor `
    -Description 'editor binary fingerprint'
$runtimeFingerprint = Get-ZirconProfileRequiredFileFingerprint `
    -Path $runtime `
    -Description 'Runtime binary fingerprint'
if ($editorFingerprint.sha256 -ne $ExpectedEditorSha256.ToLowerInvariant()) {
    throw "Editor binary does not match the managed build receipt: $editor"
}
if ($runtimeFingerprint.sha256 -ne $ExpectedRuntimeSha256.ToLowerInvariant()) {
    throw "Runtime binary does not match the managed build receipt: $runtime"
}

$sourceBinding = Get-ZirconEditorVisualSourceBinding -RepositoryRoot $repoRoot
if ($sourceBinding.source_sha256 -ne $ExpectedSourceSha256.ToLowerInvariant()) {
    throw 'Current editor UI source differs from the source fingerprint captured before the managed build.'
}
$bundleAssetBinding = Get-ZirconEditorVisualBundleAssetBinding `
    -BundleDirectory $BundleDirectory `
    -SourceBinding $sourceBinding
$newestSourceWriteUtc = @(
    $sourceBinding.critical_source_files |
        ForEach-Object { [datetime]$_.last_write_utc } |
        Sort-Object -Descending |
        Select-Object -First 1
)[0]
foreach ($binaryFingerprint in @($editorFingerprint, $runtimeFingerprint)) {
    if ([datetime]$binaryFingerprint.last_write_utc -lt $newestSourceWriteUtc) {
        throw "Managed build artifact predates current editor UI source: $($binaryFingerprint.path)"
    }
}
[System.IO.Directory]::CreateDirectory($OutputDirectory) | Out-Null
$captureResults = foreach ($extent in @(
        @{ Width = 640; Height = 520 },
        @{ Width = 900; Height = 620 },
        @{ Width = 1672; Height = 941 }
    )) {
    $captureSessionId = '{0}-{1}x{2}' -f $ProfileSessionId, $extent.Width, $extent.Height
    $profileDirectory = Join-Path $OutputDirectory (
        ConvertTo-ZirconProfileSessionBasename -SessionId $captureSessionId)
    [System.IO.Directory]::CreateDirectory($profileDirectory) | Out-Null
    $profileGeometryPath = Join-Path $profileDirectory 'ui_profile_geometry.json'
    Remove-Item -LiteralPath $profileGeometryPath -Force -ErrorAction SilentlyContinue
    $stdoutPath = Join-Path $profileDirectory 'editor.stdout.log'
    $stderrPath = Join-Path $profileDirectory 'editor.stderr.log'

    $process = Start-ZirconEditorVisualProcess `
        -Editor $editor `
        -WorkingDirectory $BundleDirectory `
        -StandardOutputPath $stdoutPath `
        -StandardErrorPath $stderrPath `
        -Environment @{
            ZIRCON_PROFILE_CAPTURE = '1'
            ZIRCON_PROFILE_CAPTURE_SCREENSHOTS = '1'
            ZIRCON_PROFILE_OUTPUT_ROOT = $OutputDirectory
            ZIRCON_PROFILE_SESSION = $captureSessionId
            ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS = '0'
            ZIRCON_PROFILE_INITIAL_CLIENT_WIDTH = [string]$extent.Width
            ZIRCON_PROFILE_INITIAL_CLIENT_HEIGHT = [string]$extent.Height
        }
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(90)
        do {
            if ($process.HasExited) {
                throw "Editor exited before opening a window (exit $($process.ExitCode))."
            }
            $process.Refresh()
            if ($process.MainWindowHandle -ne [IntPtr]::Zero -and
                -not [string]::IsNullOrWhiteSpace($process.MainWindowTitle)) {
                $geometry = Get-ZirconEditorVisualClientGeometry -Window $process.MainWindowHandle
                if ($geometry.ClientWidth -ge 64 -and $geometry.ClientHeight -ge 64) {
                    break
                }
            }
            Start-Sleep -Milliseconds 250
        } while ([DateTime]::UtcNow -lt $deadline)
        if ($process.MainWindowHandle -eq [IntPtr]::Zero -or
            [string]::IsNullOrWhiteSpace($process.MainWindowTitle)) {
            throw 'Editor did not create a titled render window within 90 seconds.'
        }

        $profileGeometry = Wait-ZirconEditorVisualProfileGeometry `
            -Process $process `
            -Path $profileGeometryPath
        if ($profileGeometry.presenter_backend -ne 'gpu') {
            throw "Visual acceptance requires presenter_backend=gpu, got '$($profileGeometry.presenter_backend)'."
        }
        if ($profileGeometry.window_client_size.width -ne $extent.Width -or
            $profileGeometry.window_client_size.height -ne $extent.Height) {
            throw "GPU-presented profile extent is $($profileGeometry.window_client_size.width)x$($profileGeometry.window_client_size.height), expected $($extent.Width)x$($extent.Height)."
        }

        $windowDpi = [ZirconEditorVisualCaptureNative]::GetDpiForWindow($process.MainWindowHandle)
        if ($windowDpi -eq 0) {
            throw 'Could not query the editor window DPI.'
        }
        $path = Join-Path $OutputDirectory ("editor-{0}x{1}.png" -f $extent.Width, $extent.Height)
        $capture = Save-ZirconEditorVisualClientScreenshot `
            -Window $process.MainWindowHandle `
            -Path $path `
            -Width $extent.Width `
            -Height $extent.Height
        $mainMenuInteraction = $null
        $moduleDetailsTooltipInteraction = $null
        $moduleDetailsInteraction = $null
        if ($extent.Width -eq 900 -and $extent.Height -eq 620) {
            $menuPointer = Invoke-ZirconEditorVisualControlClick `
                -Window $process.MainWindowHandle `
                -ProfileGeometry $profileGeometry `
                -ControlId 'WorkbenchToolbarMenu'
            $menuPath = Join-Path $OutputDirectory 'editor-900x620-main-menu.png'
            $menuCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $menuPath `
                -Width $extent.Width `
                -Height $extent.Height
            $menuRegionLeft = [Math]::Max(0, [int][Math]::Floor([double]$menuPointer.frame.x) - 8)
            $menuRegionTop = [Math]::Max(
                0,
                [int][Math]::Floor(
                    [double]$menuPointer.frame.y + [double]$menuPointer.frame.height))
            $menuRegionRight = [Math]::Min($extent.Width, $menuRegionLeft + 280)
            $menuRegionBottom = [Math]::Min($extent.Height, $menuRegionTop + 220)
            $menuDifference = Measure-ZirconEditorVisualRegionDifference `
                -BeforePath $path `
                -AfterPath $menuPath `
                -RegionLeft $menuRegionLeft `
                -RegionTop $menuRegionTop `
                -RegionRight $menuRegionRight `
                -RegionBottom $menuRegionBottom `
                -Stride 2
            if ($menuDifference.different_pixels -lt 1000 -or
                $menuDifference.different_pixel_ratio -lt 0.15) {
                throw "Main menu interaction did not materially change its anchored popup region: pixels=$($menuDifference.different_pixels) ratio=$($menuDifference.different_pixel_ratio)."
            }

            $menuDismissPointer = Invoke-ZirconEditorVisualControlClick `
                -Window $process.MainWindowHandle `
                -ProfileGeometry $profileGeometry `
                -ControlId 'WorkbenchToolbarMenu'
            $menuDismissedPath = Join-Path $OutputDirectory 'editor-900x620-main-menu-dismissed.png'
            $menuDismissedCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $menuDismissedPath `
                -Width $extent.Width `
                -Height $extent.Height
            $menuDismissedDifference = Measure-ZirconEditorVisualRegionDifference `
                -BeforePath $path `
                -AfterPath $menuDismissedPath `
                -RegionLeft $menuRegionLeft `
                -RegionTop $menuRegionTop `
                -RegionRight $menuRegionRight `
                -RegionBottom $menuRegionBottom `
                -Stride 2
            if ($menuDismissedDifference.different_pixels -ge 1000 -or
                $menuDismissedDifference.different_pixel_ratio -ge 0.08) {
                throw "Main menu did not dismiss back to the default workspace region: pixels=$($menuDismissedDifference.different_pixels) ratio=$($menuDismissedDifference.different_pixel_ratio)."
            }
            $mainMenuInteraction = [pscustomobject]@{
                state = 'opened_then_closed'
                trigger = $menuPointer
                dismiss_trigger = $menuDismissPointer
                source_geometry_scope = 'pre_interaction_trigger_only'
                screenshot = $menuCapture
                visual_difference = $menuDifference
                dismissed_screenshot = $menuDismissedCapture
                dismissed_visual_difference = $menuDismissedDifference
            }

            $tooltipHover = Invoke-ZirconEditorVisualControlHover `
                -Window $process.MainWindowHandle `
                -ProfileGeometry $profileGeometry `
                -ControlId 'WorkbenchModuleDetailsDrawerToggle' `
                -WaitMilliseconds 350
            $tooltipPath = Join-Path $OutputDirectory 'editor-900x620-module-details-tooltip.png'
            $tooltipCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $tooltipPath `
                -Width $extent.Width `
                -Height $extent.Height `
                -PreservePointerPosition
            $tooltipRegionLeft = [Math]::Max(
                0,
                [int][Math]::Floor([double]$tooltipHover.frame.x) - 220)
            $tooltipRegionTop = [Math]::Max(
                0,
                [int][Math]::Floor(
                    [double]$tooltipHover.frame.y + [double]$tooltipHover.frame.height))
            $tooltipRegionRight = [Math]::Min(
                $extent.Width,
                [int][Math]::Ceiling(
                    [double]$tooltipHover.frame.x + [double]$tooltipHover.frame.width) + 8)
            $tooltipRegionBottom = [Math]::Min($extent.Height, $tooltipRegionTop + 120)
            $tooltipDifference = Measure-ZirconEditorVisualRegionDifference `
                -BeforePath $path `
                -AfterPath $tooltipPath `
                -RegionLeft $tooltipRegionLeft `
                -RegionTop $tooltipRegionTop `
                -RegionRight $tooltipRegionRight `
                -RegionBottom $tooltipRegionBottom `
                -Stride 2
            if ($tooltipDifference.different_pixels -lt 200 -or
                $tooltipDifference.different_pixel_ratio -lt 0.02) {
                throw "Module Details tooltip did not become visible below its source-bound trigger: pixels=$($tooltipDifference.different_pixels) ratio=$($tooltipDifference.different_pixel_ratio)."
            }

            $tooltipDismissPointer = Invoke-ZirconEditorVisualPointerMove `
                -Window $process.MainWindowHandle `
                -X 0 `
                -Y 0 `
                -WaitMilliseconds 200
            $tooltipDismissedPath = Join-Path $OutputDirectory 'editor-900x620-module-details-tooltip-dismissed.png'
            $tooltipDismissedCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $tooltipDismissedPath `
                -Width $extent.Width `
                -Height $extent.Height
            $tooltipDismissedDifference = Measure-ZirconEditorVisualRegionDifference `
                -BeforePath $path `
                -AfterPath $tooltipDismissedPath `
                -RegionLeft $tooltipRegionLeft `
                -RegionTop $tooltipRegionTop `
                -RegionRight $tooltipRegionRight `
                -RegionBottom $tooltipRegionBottom `
                -Stride 2
            if ($tooltipDismissedDifference.different_pixels -ge 200 -or
                $tooltipDismissedDifference.different_pixel_ratio -ge 0.02) {
                throw "Module Details tooltip did not dismiss after the pointer left its trigger: pixels=$($tooltipDismissedDifference.different_pixels) ratio=$($tooltipDismissedDifference.different_pixel_ratio)."
            }
            $moduleDetailsTooltipInteraction = [pscustomobject]@{
                state = 'visible_then_dismissed'
                trigger = $tooltipHover
                dismiss_trigger = $tooltipDismissPointer
                source_geometry_scope = 'pre_interaction_trigger_only'
                screenshot = $tooltipCapture
                visual_difference = $tooltipDifference
                dismissed_screenshot = $tooltipDismissedCapture
                dismissed_visual_difference = $tooltipDismissedDifference
            }

            $pointer = Invoke-ZirconEditorVisualControlClick `
                -Window $process.MainWindowHandle `
                -ProfileGeometry $profileGeometry `
                -ControlId 'WorkbenchModuleDetailsDrawerToggle'
            $detailsPath = Join-Path $OutputDirectory 'editor-900x620-module-details.png'
            $detailsCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $detailsPath `
                -Width $extent.Width `
                -Height $extent.Height
            $centerBandTop = [int][Math]::Floor([double]$profileGeometry.layout.center_band.y)
            $difference = Measure-ZirconEditorVisualRegionDifference `
                -BeforePath $path `
                -AfterPath $detailsPath `
                -RegionLeft ($extent.Width - 360) `
                -RegionTop $centerBandTop `
                -Stride 2
            if ($difference.different_pixels -lt 1000 -or
                $difference.different_pixel_ratio -lt 0.20) {
                throw "Module Details interaction did not materially change the right workspace region: pixels=$($difference.different_pixels) ratio=$($difference.different_pixel_ratio)."
            }
            $moduleDetailsInteraction = [pscustomobject]@{
                state = 'open'
                trigger = $pointer
                source_geometry_scope = 'pre_interaction_trigger_only'
                screenshot = $detailsCapture
                visual_difference = $difference
            }
        }
        [pscustomobject]@{
            presenter_backend = $profileGeometry.presenter_backend
            window_title = $process.MainWindowTitle
            window_dpi = $windowDpi
            window_scale_factor = $windowDpi / 96.0
            profile_geometry_path = $profileGeometryPath
            profile_geometry_sha256 = Get-ZirconProfileFileSha256 -Path $profileGeometryPath
            profile_surface_width = $profileGeometry.window_client_size.width
            profile_surface_height = $profileGeometry.window_client_size.height
            stdout_path = $stdoutPath
            stderr_path = $stderrPath
            screenshot = $capture
            main_menu_interaction = $mainMenuInteraction
            module_details_tooltip_interaction = $moduleDetailsTooltipInteraction
            module_details_interaction = $moduleDetailsInteraction
        }
    }
    finally {
        if (-not $process.HasExited) {
            [ZirconEditorVisualCaptureNative]::PostMessage(
                $process.MainWindowHandle,
                0x0010,
                [IntPtr]::Zero,
                [IntPtr]::Zero) | Out-Null
            if (-not $process.WaitForExit(10000)) {
                $process.Kill()
                $process.WaitForExit()
            }
        }
        $process.Dispose()
    }
}

$manifest = [pscustomobject]@{
    schema_version = 2
    repository = [pscustomobject]@{
        root = $repoRoot
        source_sha256 = $sourceBinding.source_sha256
        git = $sourceBinding.git
        critical_source_files = $sourceBinding.critical_source_files
    }
    binaries = [pscustomobject]@{
        editor = [pscustomobject]@{
            path = $editorFingerprint.path
            expected_sha256 = $ExpectedEditorSha256.ToLowerInvariant()
            actual_sha256 = $editorFingerprint.sha256
            byte_length = $editorFingerprint.byte_length
            last_write_utc = $editorFingerprint.last_write_utc
        }
        runtime = [pscustomobject]@{
            path = $runtimeFingerprint.path
            expected_sha256 = $ExpectedRuntimeSha256.ToLowerInvariant()
            actual_sha256 = $runtimeFingerprint.sha256
            byte_length = $runtimeFingerprint.byte_length
            last_write_utc = $runtimeFingerprint.last_write_utc
        }
    }
    assets = $bundleAssetBinding
    captures = @($captureResults)
}
$manifestPath = Join-Path $OutputDirectory 'capture-manifest.json'
$manifestJson = $manifest | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText(
    $manifestPath,
    $manifestJson + [Environment]::NewLine,
    [System.Text.UTF8Encoding]::new($false))

if (-not $SkipVisualOracle) {
    $oracle = Join-Path $repoRoot 'tools\zircon_editor_ui_visual_oracle.py'
    & python $oracle `
        --capture-manifest $manifestPath `
        --output-directory (Join-Path $OutputDirectory 'visual-oracle')
    if ($LASTEXITCODE -ne 0) {
        throw "Editor UI visual oracle failed with exit code $LASTEXITCODE."
    }
}

$manifestJson
