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

    [string]$ProjectRoot,
    [string]$ProfileSessionId = 'editor-ui-visual-acceptance',
    [ValidateSet('100', '150')]
    [string]$DpiProfile = '150',
    [switch]$SkipInteractions,
    [switch]$PreservePointerPosition,
    [switch]$SkipVisualOracle
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$BundleDirectory = [System.IO.Path]::GetFullPath($BundleDirectory)
$OutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\profile-capture-paths.ps1')
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\profile-capture-manifest.ps1')
. (Join-Path $repoRoot 'tools\analysis\visual\editor-ui-visual-source-binding.ps1')
. (Join-Path $repoRoot 'tools\analysis\visual\editor-ui-visual-interactions.ps1')
. (Join-Path $repoRoot 'tools\analysis\visual\editor-ui-visual-matrix.ps1')
$captureCases = @(Get-ZirconEditorVisualCaptureCases -DpiProfile $DpiProfile)

function ConvertTo-ZirconEditorVisualCommandLineArgument {
    param([Parameter(Mandatory = $true)][string]$Value)

    $builder = [System.Text.StringBuilder]::new()
    [void]$builder.Append('"')
    $backslashCount = 0
    foreach ($character in $Value.ToCharArray()) {
        if ($character -eq '\') {
            $backslashCount += 1
            continue
        }
        if ($character -eq '"') {
            [void]$builder.Append('\', (2 * $backslashCount) + 1)
            [void]$builder.Append('"')
        }
        else {
            [void]$builder.Append('\', $backslashCount)
            [void]$builder.Append($character)
        }
        $backslashCount = 0
    }
    [void]$builder.Append('\', 2 * $backslashCount)
    [void]$builder.Append('"')
    return $builder.ToString()
}

function Get-ZirconEditorVisualProjectSnapshot {
    param([Parameter(Mandatory = $true)][string]$Root)

    $fullRoot = [System.IO.Path]::GetFullPath($Root)
    if (-not [System.IO.Directory]::Exists($fullRoot)) {
        throw "Editor visual project root does not exist as a directory: $fullRoot"
    }
    $resolvedRoot = (Resolve-Path -LiteralPath $fullRoot -ErrorAction Stop).Path.TrimEnd('\')
    $rootPrefix = $resolvedRoot + [System.IO.Path]::DirectorySeparatorChar
    $projectManifestPath = Join-Path $resolvedRoot 'zircon-project.toml'
    if (-not [System.IO.File]::Exists($projectManifestPath)) {
        throw "Editor visual project is missing zircon-project.toml: $resolvedRoot"
    }
    $projectManifestPath = (Resolve-Path -LiteralPath $projectManifestPath -ErrorAction Stop).Path
    if (-not $projectManifestPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw 'Editor visual project manifest resolves outside the requested project root.'
    }

    $manifestText = [System.IO.File]::ReadAllText($projectManifestPath)
    $defaultSceneMatches = [regex]::Matches(
        $manifestText,
        '(?m)^\s*default_scene\s*=\s*"([^"\\]+)"\s*(?:#.*)?$')
    if ($defaultSceneMatches.Count -ne 1) {
        throw "Editor visual project must declare exactly one supported default_scene URI: $projectManifestPath"
    }
    $defaultSceneUri = $defaultSceneMatches[0].Groups[1].Value
    if (-not $defaultSceneUri.StartsWith('res://', [System.StringComparison]::Ordinal) -or
        $defaultSceneUri.Contains('#') -or $defaultSceneUri.Contains('?')) {
        throw "Editor visual project default_scene must be a project resource URI without a fragment: $defaultSceneUri"
    }

    $assetRootMatches = [regex]::Matches(
        $manifestText,
        '(?ms)^\s*asset_roots\s*=\s*\[(.*?)\]\s*(?:#.*)?$')
    if ($assetRootMatches.Count -ne 1) {
        throw "Editor visual project must declare exactly one asset_roots array: $projectManifestPath"
    }
    $assetRootArray = $assetRootMatches[0].Groups[1].Value
    $assetRootStringMatches = [regex]::Matches($assetRootArray, '"([^"\\]+)"')
    $unparsedAssetRootText = [regex]::Replace($assetRootArray, '"([^"\\]+)"', '')
    if ($assetRootStringMatches.Count -eq 0 -or
        -not [string]::IsNullOrWhiteSpace([regex]::Replace($unparsedAssetRootText, '[,\s]', ''))) {
        throw "Editor visual project asset_roots must contain plain relative paths: $projectManifestPath"
    }

    $assetRoots = [System.Collections.Generic.List[object]]::new()
    foreach ($assetRootMatch in $assetRootStringMatches) {
        $relativeAssetRoot = $assetRootMatch.Groups[1].Value.Replace('/', '\')
        if ([System.IO.Path]::IsPathRooted($relativeAssetRoot) -or
            @($relativeAssetRoot.Split('\') | Where-Object { $_ -in @('', '.', '..') }).Count -gt 0) {
            throw "Editor visual project asset root is not a contained relative path: $relativeAssetRoot"
        }
        $assetRootPath = [System.IO.Path]::GetFullPath((Join-Path $resolvedRoot $relativeAssetRoot))
        if (-not $assetRootPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
            -not [System.IO.Directory]::Exists($assetRootPath)) {
            throw "Editor visual project asset root is missing or outside the project: $relativeAssetRoot"
        }
        $assetRootItem = Get-Item -LiteralPath $assetRootPath -Force
        if (($assetRootItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Editor visual project asset root cannot be a reparse point: $assetRootPath"
        }
        $resolvedAssetRoot = (Resolve-Path -LiteralPath $assetRootPath -ErrorAction Stop).Path.TrimEnd('\')
        if (-not $resolvedAssetRoot.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Editor visual project asset root resolves outside the project: $relativeAssetRoot"
        }
        $assetRoots.Add([pscustomobject]@{
                relative_path = $relativeAssetRoot.Replace('\', '/')
                path = $resolvedAssetRoot
            })
    }

    $sceneRelativePath = $defaultSceneUri.Substring('res://'.Length).Replace('/', '\')
    if ([string]::IsNullOrWhiteSpace($sceneRelativePath) -or
        [System.IO.Path]::IsPathRooted($sceneRelativePath) -or
        @($sceneRelativePath.Split('\') | Where-Object { $_ -in @('', '.', '..') }).Count -gt 0) {
        throw "Editor visual project default_scene URI does not resolve to a contained asset path: $defaultSceneUri"
    }
    $sceneCandidates = @(
        foreach ($assetRoot in $assetRoots) {
            $candidate = Join-Path $assetRoot.path $sceneRelativePath
            if ([System.IO.File]::Exists($candidate)) {
                (Resolve-Path -LiteralPath $candidate -ErrorAction Stop).Path
            }
        }
    )
    if ($sceneCandidates.Count -ne 1) {
        throw "Editor visual project default_scene URI must resolve to exactly one source file: $defaultSceneUri"
    }
    $defaultScenePath = $sceneCandidates[0]
    if (-not $defaultScenePath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw 'Editor visual project default_scene resolves outside the requested project root.'
    }

    $sourceFiles = [System.Collections.Generic.List[object]]::new()
    $manifestFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $projectManifestPath `
        -Description 'visual capture project manifest'
    $sourceFiles.Add([pscustomobject]@{
            relative_path = 'zircon-project.toml'
            path = $manifestFingerprint.path
            sha256 = $manifestFingerprint.sha256
            byte_length = $manifestFingerprint.byte_length
            last_write_utc = $manifestFingerprint.last_write_utc
        })
    foreach ($assetRoot in $assetRoots) {
        $assetItems = @(Get-ChildItem -LiteralPath $assetRoot.path -Recurse -Force -ErrorAction Stop)
        foreach ($assetItem in $assetItems) {
            if (($assetItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Editor visual project asset tree contains a reparse point: $($assetItem.FullName)"
            }
        }
        foreach ($assetFile in $assetItems | Where-Object { -not $_.PSIsContainer }) {
            $resolvedAssetFile = (Resolve-Path -LiteralPath $assetFile.FullName -ErrorAction Stop).Path
            if (-not $resolvedAssetFile.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "Editor visual project source file resolves outside the project root: $($assetFile.FullName)"
            }
            $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
                -Path $resolvedAssetFile `
                -Description 'visual capture project asset source'
            $sourceFiles.Add([pscustomobject]@{
                    relative_path = $resolvedAssetFile.Substring($resolvedRoot.Length + 1).Replace('\', '/')
                    path = $fingerprint.path
                    sha256 = $fingerprint.sha256
                    byte_length = $fingerprint.byte_length
                    last_write_utc = $fingerprint.last_write_utc
                })
        }
    }
    $sourceFiles = @($sourceFiles | Sort-Object relative_path -Unique)
    $canonicalSource = [System.Text.StringBuilder]::new()
    foreach ($sourceFile in $sourceFiles) {
        [void]$canonicalSource.Append($sourceFile.relative_path).Append("`0").
            Append($sourceFile.sha256).Append("`0").
            Append($sourceFile.byte_length).Append("`n")
    }
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $sourceSha256 = -join ($hasher.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($canonicalSource.ToString())) |
            ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }
    $sceneFingerprint = $sourceFiles | Where-Object {
        $_.path.Equals($defaultScenePath, [System.StringComparison]::OrdinalIgnoreCase)
    } | Select-Object -First 1
    if ($null -eq $sceneFingerprint) {
        throw 'Editor visual project source manifest did not include the declared default scene.'
    }

    return [pscustomobject]@{
        project_root = $resolvedRoot
        project_manifest_path = $manifestFingerprint.path
        project_manifest_sha256 = $manifestFingerprint.sha256
        project_manifest_byte_length = $manifestFingerprint.byte_length
        default_scene_uri = $defaultSceneUri
        default_scene_path = $sceneFingerprint.path
        default_scene_sha256 = $sceneFingerprint.sha256
        default_scene_byte_length = $sceneFingerprint.byte_length
        asset_roots = @($assetRoots | ForEach-Object { $_.relative_path })
        source_file_count = $sourceFiles.Count
        source_total_byte_length = [int64](($sourceFiles | Measure-Object -Property byte_length -Sum).Sum)
        source_sha256 = $sourceSha256
        source_files = @($sourceFiles)
    }
}

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
    public static extern uint GetDpiForSystem();

    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr dpiContext);

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
        [string[]]$ArgumentList = @(),
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
        $startArguments = @{
            FilePath = $Editor
            WorkingDirectory = $WorkingDirectory
            PassThru = $true
            RedirectStandardOutput = $StandardOutputPath
            RedirectStandardError = $StandardErrorPath
        }
        if ($ArgumentList.Count -gt 0) {
            $startArguments.ArgumentList = $ArgumentList
        }
        return Start-Process @startArguments
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
$editor = (Resolve-Path -LiteralPath $editor -ErrorAction Stop).Path
$runtime = (Resolve-Path -LiteralPath $runtime -ErrorAction Stop).Path
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
$runtimeManifestPath = "$runtime.manifest.json"
if (-not [System.IO.File]::Exists($runtimeManifestPath)) {
    throw "Runtime BuildSet sidecar manifest does not exist: $runtimeManifestPath"
}
$runtimeManifestPath = (Resolve-Path -LiteralPath $runtimeManifestPath -ErrorAction Stop).Path
$runtimeManifestFingerprint = Get-ZirconProfileRequiredFileFingerprint `
    -Path $runtimeManifestPath `
    -Description 'runtime BuildSet sidecar manifest fingerprint'
try {
    $runtimeManifest = Get-Content -LiteralPath $runtimeManifestPath -Raw -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
}
catch {
    throw "Runtime BuildSet sidecar manifest is not valid JSON: $runtimeManifestPath ($($_.Exception.Message))"
}
if ([string]::IsNullOrWhiteSpace([string]$runtimeManifest.build_set_id) -or
    $null -eq $runtimeManifest.artifact -or
    [string]$runtimeManifest.artifact.file_name -ne [System.IO.Path]::GetFileName($runtime) -or
    [string]$runtimeManifest.artifact.sha256 -ne $runtimeFingerprint.sha256) {
    throw "Runtime BuildSet sidecar does not declare the exact selected Runtime library: $runtimeManifestPath"
}
$matchingEditorHostArtifacts = @(
    $runtimeManifest.host_artifacts | Where-Object {
        [string]$_.file_name -eq [System.IO.Path]::GetFileName($editor) -and
        [string]$_.sha256 -eq $editorFingerprint.sha256
    }
)
if ($matchingEditorHostArtifacts.Count -ne 1) {
    throw "Runtime BuildSet sidecar does not declare the exact selected Editor host binary: $runtimeManifestPath"
}
$runtimeBuildSetBinding = [pscustomobject]@{
    path = $runtimeManifestFingerprint.path
    sha256 = $runtimeManifestFingerprint.sha256
    byte_length = $runtimeManifestFingerprint.byte_length
    declared_build_set_id = [string]$runtimeManifest.build_set_id
    declaration_source = 'runtime_sidecar_manifest'
    app_preflight_authenticated = $false
}

$sourceBinding = Get-ZirconEditorVisualSourceBinding -RepositoryRoot $repoRoot
if ($sourceBinding.source_sha256 -ne $ExpectedSourceSha256.ToLowerInvariant()) {
    throw 'Current editor UI source differs from the source fingerprint captured before the managed build.'
}
$bundleAssetBinding = Get-ZirconEditorVisualBundleAssetBinding `
    -BundleDirectory $BundleDirectory `
    -SourceBinding $sourceBinding
$bundleAssetRoot = Join-Path $BundleDirectory 'assets'
if (-not [System.IO.Directory]::Exists($bundleAssetRoot)) {
    throw "Product bundle asset root does not exist: $bundleAssetRoot"
}
$bundleAssetRoot = (Resolve-Path -LiteralPath $bundleAssetRoot -ErrorAction Stop).Path
$newestSourceWriteUtc = @(
    $sourceBinding.critical_source_files |
        Where-Object { $_.relative_path -match '(?:^|/)(?:Cargo\.toml|Cargo\.lock)$|\.rs$' } |
        ForEach-Object { [datetime]$_.last_write_utc } |
        Sort-Object -Descending |
        Select-Object -First 1
)[0]
if ($null -eq $newestSourceWriteUtc) {
    throw 'Source binding did not contain any compiled Rust or Cargo source timestamps.'
}
foreach ($binaryFingerprint in @($editorFingerprint, $runtimeFingerprint)) {
    if ([datetime]$binaryFingerprint.last_write_utc -lt $newestSourceWriteUtc) {
        throw "Managed build artifact predates current compiled Rust/Cargo source: $($binaryFingerprint.path)"
    }
}
$projectBindingBefore = $null
if (-not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $projectBindingBefore = Get-ZirconEditorVisualProjectSnapshot -Root $ProjectRoot
}
[System.IO.Directory]::CreateDirectory($OutputDirectory) | Out-Null
$previousDpiAwarenessContext =
    [ZirconEditorVisualCaptureNative]::SetThreadDpiAwarenessContext([IntPtr](-4))
if ($previousDpiAwarenessContext -eq [IntPtr]::Zero) {
    $win32Error = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "Could not enable per-monitor-v2 capture coordinates (Win32 error $win32Error)."
}
try {
    $systemDpi = [ZirconEditorVisualCaptureNative]::GetDpiForSystem()
    if ($systemDpi -eq 0) {
        throw 'Could not query the Windows system DPI.'
    }
    $captureResults = foreach ($extent in $captureCases) {
    $captureSessionId = '{0}-{1}' -f $ProfileSessionId, $extent.CaseId
    $profileDirectory = Join-Path $OutputDirectory (
        ConvertTo-ZirconProfileSessionBasename -SessionId $captureSessionId)
    [System.IO.Directory]::CreateDirectory($profileDirectory) | Out-Null
    $profileGeometryPath = Join-Path $profileDirectory 'ui_profile_geometry.json'
    Remove-Item -LiteralPath $profileGeometryPath -Force -ErrorAction SilentlyContinue
    $stdoutPath = Join-Path $profileDirectory 'editor.stdout.log'
    $stderrPath = Join-Path $profileDirectory 'editor.stderr.log'
    $editorArguments = @()
    if ($null -ne $projectBindingBefore) {
        $editorArguments = @(
            '--project'
            (ConvertTo-ZirconEditorVisualCommandLineArgument -Value $projectBindingBefore.project_root)
        )
    }

    $process = Start-ZirconEditorVisualProcess `
        -Editor $editor `
        -WorkingDirectory $BundleDirectory `
        -StandardOutputPath $stdoutPath `
        -StandardErrorPath $stderrPath `
        -ArgumentList $editorArguments `
        -Environment @{
            ZIRCON_RUNTIME_LIBRARY = $runtime
            ZIRCON_ASSET_ROOT = $bundleAssetRoot
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
        $winitScaleFactorProperty = $profileGeometry.PSObject.Properties['winit_scale_factor']
        $logicalClientSizeProperty = $profileGeometry.PSObject.Properties['window_client_logical_size']
        if ($null -eq $winitScaleFactorProperty -or $null -eq $logicalClientSizeProperty) {
            throw 'Profile geometry must report Winit scale factor and logical client extent separately from Windows DPI and physical pixels.'
        }
        $winitScaleFactor = [double]$winitScaleFactorProperty.Value
        if ([double]::IsNaN($winitScaleFactor) -or [double]::IsInfinity($winitScaleFactor) -or
            [Math]::Abs($winitScaleFactor - [double]$extent.ExpectedWinitScaleFactor) -gt 0.0001) {
            throw "Capture case '$($extent.CaseId)' requires Winit scale $($extent.ExpectedWinitScaleFactor), got $winitScaleFactor."
        }
        $logicalClientSize = $logicalClientSizeProperty.Value
        if ($null -eq $logicalClientSize) {
            throw 'Profile geometry must report a logical client extent.'
        }
        if ($logicalClientSize.width -ne $extent.LogicalWidth -or
            $logicalClientSize.height -ne $extent.LogicalHeight) {
            throw "Winit profile logical client extent is $($logicalClientSize.width)x$($logicalClientSize.height), expected $($extent.LogicalWidth)x$($extent.LogicalHeight)."
        }
        if ($profileGeometry.window_client_size.width -ne $extent.Width -or
            $profileGeometry.window_client_size.height -ne $extent.Height) {
            throw "GPU-presented physical profile extent is $($profileGeometry.window_client_size.width)x$($profileGeometry.window_client_size.height), expected $($extent.Width)x$($extent.Height)."
        }

        $windowDpiBeforeCapture = [ZirconEditorVisualCaptureNative]::GetDpiForWindow($process.MainWindowHandle)
        if ($windowDpiBeforeCapture -eq 0) {
            throw 'Could not query the editor window DPI.'
        }
        if ($null -ne $extent.ExpectedWindowDpi -and
            $windowDpiBeforeCapture -ne $extent.ExpectedWindowDpi) {
            throw "Capture case '$($extent.CaseId)' requires a real $($extent.ExpectedWindowDpi)-DPI Windows editor window before capture; GetDpiForWindow returned $windowDpiBeforeCapture. Use a real display at the required scale and rerun with the matching -DpiProfile."
        }
        $windowGeometry = Get-ZirconEditorVisualClientGeometry -Window $process.MainWindowHandle
        if ($windowGeometry.ClientWidth -ne $extent.Width -or
            $windowGeometry.ClientHeight -ne $extent.Height) {
            throw "Windows physical client extent is $($windowGeometry.ClientWidth)x$($windowGeometry.ClientHeight), expected $($extent.Width)x$($extent.Height)."
        }
        $path = Join-Path $OutputDirectory ("editor-{0}.png" -f $extent.CaseId)
        $preservePointerForCapture = $PreservePointerPosition -or $SkipInteractions
        $capture = Save-ZirconEditorVisualClientScreenshot `
            -Window $process.MainWindowHandle `
            -Path $path `
            -Width $extent.Width `
            -Height $extent.Height `
            -PreservePointerPosition:$preservePointerForCapture
        $windowDpi = [ZirconEditorVisualCaptureNative]::GetDpiForWindow($process.MainWindowHandle)
        if ($windowDpi -eq 0) {
            throw 'Could not query the editor window DPI after positioning it for capture.'
        }
        if ($null -ne $extent.ExpectedWindowDpi -and
            $windowDpi -ne $extent.ExpectedWindowDpi) {
            throw "Capture case '$($extent.CaseId)' requires a real $($extent.ExpectedWindowDpi)-DPI Windows editor window at capture time; GetDpiForWindow returned $windowDpi after positioning. Keep the editor on a display with the requested DPI profile and rerun."
        }
        $mainMenuInteraction = $null
        $moduleDetailsTooltipInteraction = $null
        $moduleDetailsInteraction = $null
        if ($extent.CaseId -eq '900x620' -and -not $SkipInteractions) {
            $menuPointer = Invoke-ZirconEditorVisualControlClick `
                -Window $process.MainWindowHandle `
                -ProfileGeometry $profileGeometry `
                -ControlId 'WorkbenchToolbarMenu'
            $menuPath = Join-Path $OutputDirectory 'editor-900x620-main-menu.png'
            $menuCapture = Save-ZirconEditorVisualClientScreenshot `
                -Window $process.MainWindowHandle `
                -Path $menuPath `
                -Width $extent.Width `
                -Height $extent.Height `
                -PreservePointerPosition:$preservePointerForCapture
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
                -Height $extent.Height `
                -PreservePointerPosition:$preservePointerForCapture
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
                -PreservePointerPosition:$preservePointerForCapture
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
                -Height $extent.Height `
                -PreservePointerPosition:$preservePointerForCapture
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
                -Height $extent.Height `
                -PreservePointerPosition:$preservePointerForCapture
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
        $windowDpiAfterInteractions =
            [ZirconEditorVisualCaptureNative]::GetDpiForWindow($process.MainWindowHandle)
        if ($windowDpiAfterInteractions -eq 0) {
            throw 'Could not query the editor window DPI after visual interactions.'
        }
        if ($null -ne $extent.ExpectedWindowDpi -and
            $windowDpiAfterInteractions -ne $extent.ExpectedWindowDpi) {
            throw "Capture case '$($extent.CaseId)' left the required $($extent.ExpectedWindowDpi)-DPI display during capture; GetDpiForWindow returned $windowDpiAfterInteractions. Keep the editor on a display with the requested DPI profile and rerun."
        }
        [pscustomobject]@{
            case_id = $extent.CaseId
            requested_client_extent_pixels = [pscustomobject]@{
                width = $extent.Width
                height = $extent.Height
            }
            requested_client_extent_logical = [pscustomobject]@{
                width = $extent.LogicalWidth
                height = $extent.LogicalHeight
            }
            presenter_backend = $profileGeometry.presenter_backend
            window_title = $process.MainWindowTitle
            windows_system_dpi = $systemDpi
            window_dpi = $windowDpi
            windows_window_dpi_before_capture = $windowDpiBeforeCapture
            windows_window_dpi = $windowDpi
            windows_window_dpi_after_interactions = $windowDpiAfterInteractions
            expected_windows_window_dpi = $extent.ExpectedWindowDpi
            winit_scale_factor = $winitScaleFactor
            expected_winit_scale_factor = $extent.ExpectedWinitScaleFactor
            capture_dpi_awareness_context = 'per_monitor_v2'
            profile_geometry_path = $profileGeometryPath
            profile_geometry_sha256 = Get-ZirconProfileFileSha256 -Path $profileGeometryPath
            profile_surface_width = $profileGeometry.window_client_size.width
            profile_surface_height = $profileGeometry.window_client_size.height
            profile_logical_client_width = $logicalClientSize.width
            profile_logical_client_height = $logicalClientSize.height
            input_project_root = if ($null -ne $projectBindingBefore) { $projectBindingBefore.project_root } else { $null }
            input_project_default_scene_uri = if ($null -ne $projectBindingBefore) { $projectBindingBefore.default_scene_uri } else { $null }
            runtime_library_path = $runtime
            asset_root = $bundleAssetRoot
            declared_runtime_build_set_id = $runtimeBuildSetBinding.declared_build_set_id
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
}
finally {
    [ZirconEditorVisualCaptureNative]::SetThreadDpiAwarenessContext(
        $previousDpiAwarenessContext) | Out-Null
}

$projectProvenance = $null
if ($null -ne $projectBindingBefore) {
    $projectBindingAfter = Get-ZirconEditorVisualProjectSnapshot -Root $projectBindingBefore.project_root
    $projectUnchanged = $projectBindingBefore.source_sha256 -ceq $projectBindingAfter.source_sha256
    $projectProvenance = [pscustomobject]@{
        project_root = $projectBindingBefore.project_root
        project_manifest_path = $projectBindingBefore.project_manifest_path
        project_manifest_sha256_before = $projectBindingBefore.project_manifest_sha256
        project_manifest_sha256_after = $projectBindingAfter.project_manifest_sha256
        default_scene_uri = $projectBindingBefore.default_scene_uri
        default_scene_path = $projectBindingBefore.default_scene_path
        default_scene_sha256_before = $projectBindingBefore.default_scene_sha256
        default_scene_sha256_after = $projectBindingAfter.default_scene_sha256
        asset_roots = $projectBindingBefore.asset_roots
        source_file_count = $projectBindingBefore.source_file_count
        source_total_byte_length = $projectBindingBefore.source_total_byte_length
        source_sha256_before = $projectBindingBefore.source_sha256
        source_sha256_after = $projectBindingAfter.source_sha256
        unchanged_before_after = $projectUnchanged
        source_files = $projectBindingBefore.source_files
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
            build_set_sidecar = $runtimeBuildSetBinding
        }
    }
    assets = $bundleAssetBinding
    input_project = $projectProvenance
    launch_environment = [pscustomobject]@{
        zircon_runtime_library = $runtime
        zircon_asset_root = $bundleAssetRoot
        tmp = [Environment]::GetEnvironmentVariable('TMP', [EnvironmentVariableTarget]::Process)
        temp = [Environment]::GetEnvironmentVariable('TEMP', [EnvironmentVariableTarget]::Process)
        project_argument = if ($null -ne $projectBindingBefore) { $projectBindingBefore.project_root } else { $null }
        project_default_scene_uri = if ($null -ne $projectBindingBefore) { $projectBindingBefore.default_scene_uri } else { $null }
        interaction_capture = if ($SkipInteractions) { 'skipped' } else { 'legacy-interactions-enabled' }
        pointer_position = if ($preservePointerForCapture -or $PreservePointerPosition) { 'preserved' } else { 'moved-to-origin-for-capture' }
    }
    captures = @($captureResults)
}
$manifestPath = Join-Path $OutputDirectory 'capture-manifest.json'
$manifestJson = $manifest | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText(
    $manifestPath,
    $manifestJson + [Environment]::NewLine,
    [System.Text.UTF8Encoding]::new($false))

if ($null -ne $projectProvenance -and -not $projectProvenance.unchanged_before_after) {
    throw "Editor visual capture project source changed during capture: $($projectProvenance.project_root)"
}

if (-not $SkipVisualOracle) {
    $oracle = Join-Path $repoRoot 'tools\analysis\visual\zircon_editor_ui_visual_oracle.py'
    $oracleRoot = Join-Path $OutputDirectory 'visual-oracle'
    foreach ($oracleCase in @(
            @{
                CaseId = ('dpi{0}' -f $DpiProfile)
                Captures = @($manifest.captures)
                ExpectedExtents = @($captureCases | ForEach-Object { '{0}x{1}' -f $_.Width, $_.Height })
            }
        )) {
        $caseOutput = Join-Path $oracleRoot $oracleCase.CaseId
        [System.IO.Directory]::CreateDirectory($caseOutput) | Out-Null
        $caseManifest = [pscustomobject]@{
            schema_version = $manifest.schema_version
            repository = $manifest.repository
            binaries = $manifest.binaries
            assets = $manifest.assets
            captures = @($oracleCase.Captures)
        }
        $caseManifestPath = Join-Path $caseOutput 'capture-manifest.json'
        [System.IO.File]::WriteAllText(
            $caseManifestPath,
            ($caseManifest | ConvertTo-Json -Depth 8) + [Environment]::NewLine,
            [System.Text.UTF8Encoding]::new($false))
        $oracleArguments = @(
            $oracle,
            '--capture-manifest',
            $caseManifestPath,
            '--output-directory',
            (Join-Path $caseOutput 'analysis')
        )
        foreach ($expectedExtent in $oracleCase.ExpectedExtents) {
            $oracleArguments += @('--expected-extent', $expectedExtent)
        }
        & python @oracleArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Editor UI visual oracle failed for '$($oracleCase.CaseId)' with exit code $LASTEXITCODE."
        }
    }
}

$manifestJson
