[CmdletBinding()]
param(
    [string[]]$Profile = @(),
    [string]$Toolchain = "nightly",
    [string]$TargetDir = "",
    [ValidateRange(1, 64)]
    [int]$Jobs = 1,
    [switch]$Offline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-AllowedCargoTargetPath {
    param([string]$Path)

    $candidate = [System.IO.Path]::GetFullPath($Path).TrimEnd('\', '/')
    foreach ($allowedRoot in @('D:\cargo-targets', 'E:\cargo-targets', 'F:\cargo-targets')) {
        $root = [System.IO.Path]::GetFullPath($allowedRoot).TrimEnd('\', '/')
        if ($candidate.StartsWith($root + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
            return $candidate
        }
    }
    throw "Cargo build output must be below D:\cargo-targets, E:\cargo-targets, or F:\cargo-targets: $candidate"
}

$repoRoot = Split-Path $PSScriptRoot -Parent
$presetTool = Join-Path $PSScriptRoot "runtime-profile-feature-presets.py"
$matrixText = & python $presetTool matrix
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($matrixText)) {
    throw "Cannot load the canonical runtime profile feature matrix."
}
$matrix = $matrixText | ConvertFrom-Json
$rows = @($matrix.include)

if ($Profile.Count -gt 0) {
    $unknownProfiles = @($Profile | Where-Object { $_ -notin $rows.profile })
    if ($unknownProfiles.Count -gt 0) {
        throw "Unknown runtime profile(s): $($unknownProfiles -join ', ')."
    }
    $rows = @($rows | Where-Object { $_.profile -in $Profile })
}

if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $repoDrive = [System.IO.Path]::GetPathRoot($repoRoot).TrimEnd('\')
    $TargetDir = Join-Path $repoDrive "cargo-targets\zircon-runtime-profile-matrix"
}
$resolvedTargetDir = Assert-AllowedCargoTargetPath -Path $TargetDir
New-Item -ItemType Directory -Force -Path $resolvedTargetDir | Out-Null

$oldTargetDir = $env:CARGO_TARGET_DIR
$failedProfiles = [System.Collections.Generic.List[string]]::new()
Push-Location $repoRoot
try {
    $env:CARGO_TARGET_DIR = $resolvedTargetDir
    foreach ($row in $rows) {
        $cargoArgs = [System.Collections.Generic.List[string]]::new()
        foreach ($argument in @(
            "check",
            "-p",
            "zircon_app",
            "--lib",
            "--no-default-features",
            "--features",
            $row.cargo_feature,
            "--locked",
            "--jobs",
            $Jobs.ToString(),
            "--color",
            "never"
        )) {
            $cargoArgs.Add($argument)
        }
        if ($Offline) {
            $cargoArgs.Add("--offline")
        }

        Write-Host "[runtime-profile] checking $($row.profile) -> $($row.cargo_feature)" -ForegroundColor Cyan
        & rustup run $Toolchain cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) {
            $failedProfiles.Add($row.profile)
        }
    }
}
finally {
    Pop-Location
    if ($null -eq $oldTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    }
    else {
        $env:CARGO_TARGET_DIR = $oldTargetDir
    }
}

if ($failedProfiles.Count -gt 0) {
    Write-Error "Runtime profile feature checks failed: $($failedProfiles -join ', ')."
    exit 1
}

Write-Host "Runtime profile feature checks passed: $($rows.profile -join ', ')." -ForegroundColor Green
