[CmdletBinding()]
param(
    [ValidateSet("client", "server", "editor")]
    [string]$Profile = "client",
    [ValidateSet("check", "build", "test", "run")]
    [string]$Action = "check",
    [string]$Package = "zircon_app",
    [switch]$Release,
    [switch]$NoLocked,
    [switch]$InstallSccache,
    [string]$SharedTargetRoot = "",
    [string]$FeatureOverride = "",
    [string[]]$ExtraCargoArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
    param([string]$Start)
    $cursor = [System.IO.Path]::GetFullPath($Start)
    while ($true) {
        if (Test-Path (Join-Path $cursor "Cargo.toml")) {
            return $cursor
        }
        $parent = Split-Path $cursor -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $cursor) {
            throw "Cannot locate repository root from $Start."
        }
        $cursor = $parent
    }
}

function Ensure-Sccache {
    param([switch]$AutoInstall)
    $exists = Get-Command sccache -ErrorAction SilentlyContinue
    if ($exists) {
        return $true
    }
    if (-not $AutoInstall) {
        Write-Host "[Hint] sccache not found. Use -InstallSccache to install it." -ForegroundColor Yellow
        return $false
    }
    Write-Host "Installing sccache ..." -ForegroundColor Cyan
    cargo install sccache
    return $true
}

function Resolve-FeatureSet {
    param([string]$Mode)
    switch ($Mode) {
        "client" { return "target-client" }
        "server" { return "target-server" }
        "editor" { return "target-editor-host" }
        default { throw "Unknown profile: $Mode" }
    }
}

function Resolve-RunBin {
    param([string]$Mode)
    switch ($Mode) {
        "client" { return "zircon_runtime" }
        "editor" { return "zircon_editor" }
        default { return $null }
    }
}

$repoRoot = Resolve-RepoRoot -Start $PSScriptRoot
Push-Location $repoRoot
try {
    if ([string]::IsNullOrWhiteSpace($SharedTargetRoot)) {
        $drive = [System.IO.Path]::GetPathRoot($repoRoot).TrimEnd('\')
        $SharedTargetRoot = Join-Path $drive "cargo-targets\zircon-shared"
    }

    $feature = if ([string]::IsNullOrWhiteSpace($FeatureOverride)) {
        Resolve-FeatureSet -Mode $Profile
    } else {
        $FeatureOverride
    }
    $targetDir = Join-Path $SharedTargetRoot $Profile
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null

    $env:CARGO_TARGET_DIR = $targetDir
    if (Ensure-Sccache -AutoInstall:$InstallSccache) {
        $env:RUSTC_WRAPPER = "sccache"
    }

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add($Action) | Out-Null
    $args.Add("-p") | Out-Null
    $args.Add($Package) | Out-Null
    $args.Add("--no-default-features") | Out-Null
    $args.Add("--features") | Out-Null
    $args.Add($feature) | Out-Null
    if (-not $NoLocked) { $args.Add("--locked") | Out-Null }
    if ($Release) { $args.Add("--release") | Out-Null }

    if ($Action -eq "run") {
        $bin = Resolve-RunBin -Mode $Profile
        if ($null -eq $bin) {
            throw "Server profile has no runnable bin. Use check/build/test."
        }
        $args.Add("--bin") | Out-Null
        $args.Add($bin) | Out-Null
    }

    if ($ExtraCargoArgs) {
        foreach ($item in $ExtraCargoArgs) { $args.Add($item) | Out-Null }
    }

    Write-Host "RepoRoot: $repoRoot"
    Write-Host "Profile: $Profile -> feature: $feature"
    Write-Host "Action: $Action, Package: $Package"
    Write-Host "CARGO_TARGET_DIR: $env:CARGO_TARGET_DIR"
    if ($env:RUSTC_WRAPPER) {
        Write-Host "RUSTC_WRAPPER: $env:RUSTC_WRAPPER"
    }
    Write-Host ("cargo " + ($args -join " ")) -ForegroundColor DarkGray

    cargo @args
}
finally {
    Pop-Location
}
