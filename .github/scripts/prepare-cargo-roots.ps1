$ErrorActionPreference = 'Stop'

$root = 'E:\cargo-targets\zircon-local'
$fixedRoots = @{
    CARGO_HOME = Join-Path $root 'cache\cargo-home'
    RUSTUP_HOME = Join-Path $root 'cache\rustup'
    TEMP = Join-Path $root 'tmp'
    TMP = Join-Path $root 'tmp'
}

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

foreach ($name in $fixedRoots.Keys) {
    $actual = Get-NormalPath ([Environment]::GetEnvironmentVariable($name))
    $expected = Get-NormalPath $fixedRoots[$name]
    if (-not $actual.Equals($expected, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$name must be exactly $expected; received $actual"
    }
    Assert-NoReparseAncestors $actual
}

$target = Get-NormalPath $env:CARGO_TARGET_DIR
$targetRoots = @(
    (Get-NormalPath (Join-Path $root 'ci')) + '\',
    (Get-NormalPath (Join-Path $root 'wasm\ci')) + '\'
)
if (-not ($targetRoots | Where-Object { $target.StartsWith($_, [System.StringComparison]::OrdinalIgnoreCase) })) {
    throw "CARGO_TARGET_DIR must be below a dedicated CI output root: $target"
}
Assert-NoReparseAncestors $target

if ($env:MIRI_SYSROOT) {
    $miriRoot = Get-NormalPath (Join-Path $root 'ci\miri-scene-ecs') + '\'
    $sysroot = Get-NormalPath $env:MIRI_SYSROOT
    if (-not $sysroot.StartsWith($miriRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "MIRI_SYSROOT must stay under $miriRoot"
    }
    Assert-NoReparseAncestors $sysroot
}

$directories = @($fixedRoots.CARGO_HOME, $fixedRoots.RUSTUP_HOME, $fixedRoots.TEMP, $target)
if ($env:MIRI_SYSROOT) { $directories += $env:MIRI_SYSROOT }
foreach ($directory in $directories) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
}

Write-Output "Validated E: CI output roots; target=$target"
