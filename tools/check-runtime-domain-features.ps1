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
$oldTargetDir = $env:CARGO_TARGET_DIR
$failedFeatures = [System.Collections.Generic.List[string]]::new()

Push-Location $repoRoot
try {
    if (-not [string]::IsNullOrWhiteSpace($TargetDir)) {
        $resolvedTargetDir = [System.IO.Path]::GetFullPath($TargetDir)
        New-Item -ItemType Directory -Force -Path $resolvedTargetDir | Out-Null
        $env:CARGO_TARGET_DIR = $resolvedTargetDir
    }

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
    if ($null -eq $oldTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $oldTargetDir
    }
}

if ($failedFeatures.Count -gt 0) {
    Write-Error "Runtime domain feature checks failed: $($failedFeatures -join ', ')."
    exit 1
}

Write-Host "Runtime domain feature checks passed: $($selectedFeatures -join ', ')." -ForegroundColor Green
