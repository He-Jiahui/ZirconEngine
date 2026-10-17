$ErrorActionPreference = 'Stop'

$root = 'E:\cargo-targets\zircon-local'

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

    $current = 'E:\'
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

$drive = [System.IO.DriveInfo]::new('E:\')
if (-not $drive.IsReady -or $drive.DriveType -ne [System.IO.DriveType]::Fixed) {
    throw 'The required E: output volume is not a ready fixed drive.'
}
$substitutions = & subst.exe 2>$null
if ($LASTEXITCODE -ne 0 -or ($substitutions -match '(?im)^E:\\:\s*=>')) {
    throw 'The E: output root is unavailable or aliases another path through SUBST.'
}

$runRoot = Get-NormalPath $env:ZR_CI_OUTPUT_ROOT
$runPrefix = Join-Path $root ("ci\runs\" + $env:GITHUB_RUN_ID + '-' + $env:GITHUB_RUN_ATTEMPT)
$lane = Split-Path $runRoot -Leaf
if ((Split-Path $runRoot -Parent) -ne $runPrefix -or $lane -notmatch '^[a-z0-9-]+-[0-9]+$') {
    throw 'CI output must identify this run, attempt and unique job/matrix lane.'
}
Assert-NoReparseAncestors $runRoot
if (Test-Path -LiteralPath $runRoot) {
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
if ($target -ne (Join-Path $runRoot 'target')) { throw 'CARGO_TARGET_DIR escaped its isolated run root' }
Assert-NoReparseAncestors $target
if ($env:MIRI_SYSROOT) {
    if ((Get-NormalPath $env:MIRI_SYSROOT) -ne (Join-Path $runRoot 'sysroot')) {
        throw 'MIRI_SYSROOT escaped its isolated run root'
    }
    Assert-NoReparseAncestors $env:MIRI_SYSROOT
}

$directories = @($fixedRoots.CARGO_HOME, $fixedRoots.RUSTUP_HOME, $fixedRoots.TEMP, $target)
if ($env:MIRI_SYSROOT) { $directories += $env:MIRI_SYSROOT }
foreach ($directory in $directories) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    Assert-NoReparseAncestors $directory
}

Write-Output "Validated E: CI output roots; target=$target"
