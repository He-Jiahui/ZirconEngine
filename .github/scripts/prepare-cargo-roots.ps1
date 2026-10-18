param(
    [ValidateSet('D:\cargo-targets\zircon-local', 'E:\cargo-targets\zircon-local', 'F:\cargo-targets\zircon-local')]
    [string]$StorageRoot = 'E:\cargo-targets\zircon-local',
    [switch]$UseMvpTargetCache,
    [switch]$VerifyOnly
)

$ErrorActionPreference = 'Stop'

$root = $StorageRoot
$driveRoot = $root.Substring(0, 3)

function Get-NormalPath([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value.Contains('..')) {
        throw 'A CI output path is empty or contains parent traversal.'
    }
    return [System.IO.Path]::GetFullPath($Value).TrimEnd('\')
}

function Assert-NoReparseAncestors([string]$Value) {
    $full = Get-NormalPath $Value
    if (-not $full.StartsWith(($root + '\'), [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Output path is outside the required physical root: $full"
    }

    $current = $driveRoot
    foreach ($segment in $full.Substring(3).Split('\')) {
        if ([string]::IsNullOrWhiteSpace($segment)) { continue }
        $current = Join-Path $current $segment
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Output path contains a reparse point: $current"
            }
        }
    }
}

$drive = [System.IO.DriveInfo]::new($driveRoot)
if (-not $drive.IsReady -or $drive.DriveType -ne [System.IO.DriveType]::Fixed) {
    throw "The required $driveRoot output volume is not a ready fixed drive."
}
$substitutions = & subst.exe 2>$null
if ($LASTEXITCODE -ne 0 -or ($substitutions -match ('(?im)^' + [regex]::Escape($driveRoot) + ':\s*=>'))) {
    throw "The $driveRoot output root is unavailable or aliases another path through SUBST."
}

$runRoot = Get-NormalPath $env:ZR_CI_OUTPUT_ROOT
$runPrefix = Join-Path $root ("ci\runs\" + $env:GITHUB_RUN_ID + '-' + $env:GITHUB_RUN_ATTEMPT)
$lane = Split-Path $runRoot -Leaf
if ((Split-Path $runRoot -Parent) -ne $runPrefix -or $lane -notmatch '^[a-z0-9-]+-[0-9]+$') {
    throw 'CI output must identify this run, attempt and unique job/matrix lane.'
}
Assert-NoReparseAncestors $runRoot
if (-not $VerifyOnly -and (Test-Path -LiteralPath $runRoot)) {
    throw "Refusing to reuse a prior CI output tree: $runRoot"
}
$fixedRoots = @{
    CARGO_HOME = Join-Path $runRoot 'cargo-home'
    RUSTUP_HOME = Join-Path $runRoot 'rustup'
    TEMP = Join-Path $runRoot 'tmp'
    TMP = Join-Path $runRoot 'tmp'
}
foreach ($name in $fixedRoots.Keys) {
    $actual = Get-NormalPath ([Environment]::GetEnvironmentVariable($name))
    if ($actual -ne $fixedRoots[$name]) { throw "$name escaped its isolated run root" }
    Assert-NoReparseAncestors $actual
}
$target = Get-NormalPath $env:CARGO_TARGET_DIR
$expectedTarget = Join-Path $runRoot 'target'
if ($UseMvpTargetCache) {
    if ($env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or
        $env:GITHUB_JOB -notin @('compile-gate', 'focused-tests', 'mvp-editor-windows') -or
        $env:ZR_CI_LOCK_HASH -notmatch '^[a-f0-9]{64}$') {
        throw 'MVP target reuse requires a hosted job and exact lockfile hash.'
    }
    $expectedTarget = Join-Path $root "ci\cache\mvp-1.94.1\$env:ZR_CI_LOCK_HASH\$env:GITHUB_JOB\target"
}
if ($target -ne $expectedTarget) { throw 'CARGO_TARGET_DIR escaped its isolated run root or exact MVP cache identity' }
Assert-NoReparseAncestors $target
if ($env:MIRI_SYSROOT) {
    if ((Get-NormalPath $env:MIRI_SYSROOT) -ne (Join-Path $runRoot 'sysroot')) {
        throw 'MIRI_SYSROOT escaped its isolated run root'
    }
    Assert-NoReparseAncestors $env:MIRI_SYSROOT
}

$directories = @($fixedRoots.CARGO_HOME, $fixedRoots.RUSTUP_HOME, $fixedRoots.TEMP, $target)
if ($env:CARGO_BUILD_BUILD_DIR) {
    if ((Get-NormalPath $env:CARGO_BUILD_BUILD_DIR) -ne (Join-Path $runRoot 'build')) {
        throw 'CARGO_BUILD_BUILD_DIR escaped its isolated run root'
    }
    Assert-NoReparseAncestors $env:CARGO_BUILD_BUILD_DIR
    $directories += $env:CARGO_BUILD_BUILD_DIR
}
if ($env:MIRI_SYSROOT) { $directories += $env:MIRI_SYSROOT }
foreach ($directory in $directories) {
    if (-not $VerifyOnly) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }
    if (-not (Test-Path -LiteralPath $directory -PathType Container)) {
        throw "Required CI output directory is missing: $directory"
    }
    Assert-NoReparseAncestors $directory
}
if ($UseMvpTargetCache -and $VerifyOnly) {
    # Restored compiler caches must not introduce a link beneath the checked root.
    $links = @(Get-ChildItem -LiteralPath $target -Recurse -Force -Attributes ReparsePoint)
    if ($links.Count -ne 0) { throw 'Restored MVP target cache contains a reparse point.' }
}

Write-Output "Validated $driveRoot CI output roots; target=$target"
