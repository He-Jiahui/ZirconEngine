[CmdletBinding()]
param(
    [string[]]$Feature = @(),
    [string]$Toolchain = "nightly",
    [string]$TargetDir = "",
    [ValidateRange(1, 64)]
    [int]$Jobs = 1,
    [switch]$Offline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$pathResolver = Join-Path $PSScriptRoot "WindowsPathResolver.psm1"
if (-not (Test-Path -LiteralPath $pathResolver -PathType Leaf)) {
    throw "Windows path resolver was not found: $pathResolver"
}
Import-Module $pathResolver -Force -DisableNameChecking -ErrorAction Stop

function Resolve-ManagedCargoTargetPath {
    param([Parameter(Mandatory)][string]$Path)

    $targetResolution = Resolve-ZirconWindowsPath -Path $Path
    $candidate = $targetResolution.OperationalPath.TrimEnd('\', '/')
    foreach ($allowedRootPath in @('D:\cargo-targets', 'E:\cargo-targets', 'F:\cargo-targets')) {
        $allowedRoot = (Resolve-ZirconWindowsPath -Path $allowedRootPath).OperationalPath.TrimEnd('\', '/')
        if ($candidate.StartsWith($allowedRoot + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
            return $targetResolution
        }
    }

    throw "Cargo build output must physically resolve below D:\cargo-targets, E:\cargo-targets, or F:\cargo-targets: $($targetResolution.DisplayPath)"
}

function Assert-ManagedCargoChildPath {
    param(
        [Parameter(Mandatory)][object]$ParentResolution,
        [Parameter(Mandatory)][object]$ChildResolution,
        [Parameter(Mandatory)][string]$Label
    )

    $parent = $ParentResolution.OperationalPath.TrimEnd('\', '/')
    $child = $ChildResolution.OperationalPath.TrimEnd('\', '/')
    if (-not $child.StartsWith($parent + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label must physically resolve below $($ParentResolution.DisplayPath): $($ChildResolution.DisplayPath)"
    }
}

function Push-ManagedCargoEnvironment {
    param([Parameter(Mandatory)][object]$TargetResolution)

    $cargoHomeResolution = Resolve-ZirconWindowsPath -Path (Join-Path $TargetResolution.DisplayPath 'cargo-home')
    $sccacheResolution = Resolve-ZirconWindowsPath -Path (Join-Path $TargetResolution.DisplayPath 'sccache')
    $temporaryResolution = Resolve-ZirconWindowsPath -Path (Join-Path $TargetResolution.DisplayPath 'temporary')
    foreach ($childResolution in @($cargoHomeResolution, $sccacheResolution, $temporaryResolution)) {
        Assert-ManagedCargoChildPath -ParentResolution $TargetResolution -ChildResolution $childResolution -Label 'Managed Cargo directory'
    }
    foreach ($resolution in @($TargetResolution, $cargoHomeResolution, $sccacheResolution, $temporaryResolution)) {
        [System.IO.Directory]::CreateDirectory($resolution.OperationalPath) | Out-Null
    }

    $names = @('CARGO_TARGET_DIR', 'CARGO_HOME', 'SCCACHE_DIR', 'TEMP', 'TMP', 'TMPDIR')
    $previousValues = @{}
    foreach ($name in $names) {
        $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    try {
        [Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', $TargetResolution.DisplayPath, 'Process')
        [Environment]::SetEnvironmentVariable('CARGO_HOME', $cargoHomeResolution.DisplayPath, 'Process')
        [Environment]::SetEnvironmentVariable('SCCACHE_DIR', $sccacheResolution.DisplayPath, 'Process')
        foreach ($name in @('TEMP', 'TMP', 'TMPDIR')) {
            [Environment]::SetEnvironmentVariable($name, $temporaryResolution.DisplayPath, 'Process')
        }
    }
    catch {
        foreach ($name in $names) {
            [Environment]::SetEnvironmentVariable($name, $previousValues[$name], 'Process')
        }
        throw
    }

    return [pscustomobject]@{ PreviousValues = $previousValues }
}

function Pop-ManagedCargoEnvironment {
    param([Parameter(Mandatory)][object]$Lease)

    foreach ($name in $Lease.PreviousValues.Keys) {
        [Environment]::SetEnvironmentVariable($name, $Lease.PreviousValues[$name], 'Process')
    }
}

$domainFeatures = @(
    "ai-contracts",
    "animation",
    "diagnostic-log",
    "dynamic-api",
    "graphics",
    "navigation",
    "net-contracts",
    "physics-contracts",
    "script",
    "sound-contracts",
    "text",
    "ui"
)

$unknownFeatures = @($Feature | Where-Object { $_ -notin $domainFeatures })
if ($unknownFeatures.Count -gt 0) {
    throw "Unknown runtime domain feature(s): $($unknownFeatures -join ', ')."
}

$selectedFeatures = if ($Feature.Count -eq 0) {
    $domainFeatures
} else {
    $Feature
}

$repoRoot = Split-Path $PSScriptRoot -Parent
if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $repoDrive = [System.IO.Path]::GetPathRoot($repoRoot).TrimEnd('\')
    $TargetDir = Join-Path $repoDrive "cargo-targets\zircon-runtime-domain-matrix"
}
$targetResolution = Resolve-ManagedCargoTargetPath -Path $TargetDir
$environmentLease = Push-ManagedCargoEnvironment -TargetResolution $targetResolution
$failedFeatures = [System.Collections.Generic.List[string]]::new()

Push-Location $repoRoot
try {
    foreach ($domainFeature in $selectedFeatures) {
        $cargoArgs = [System.Collections.Generic.List[string]]::new()
        foreach ($argument in @(
            "check",
            "-p",
            "zircon_runtime",
            "--lib",
            "--no-default-features",
            "--features",
            "core-min,$domainFeature",
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

        Write-Host "[runtime-domain] checking $domainFeature" -ForegroundColor Cyan
        & rustup run $Toolchain cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) {
            $failedFeatures.Add($domainFeature)
        }
    }
}
finally {
    Pop-Location
    Pop-ManagedCargoEnvironment -Lease $environmentLease
}

if ($failedFeatures.Count -gt 0) {
    Write-Error "Runtime domain feature checks failed: $($failedFeatures -join ', ')."
    exit 1
}

Write-Host "Runtime domain feature checks passed: $($selectedFeatures -join ', ')." -ForegroundColor Green
