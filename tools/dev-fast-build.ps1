[CmdletBinding()]
param(
    [ValidateSet("minimal", "client2d", "client3d", "editor", "dev", "server")]
    [string]$Profile = "client3d",
    [ValidateSet("check", "build", "test", "run")]
    [string]$Action = "check",
    [string]$Package = "zircon_app",
    [switch]$Release,
    [switch]$NoLocked,
    [switch]$InstallSccache,
    [string]$SharedTargetRoot = "",
    [string]$FeatureOverride = "",
    [ValidateSet("debug", "release", "profiling")]
    [string]$CargoProfile = "debug",
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
    param(
        [string]$RepoRoot,
        [string]$Mode
    )

    $presetTool = Join-Path $RepoRoot "tools\runtime-profile-feature-presets.py"
    $feature = & python $presetTool feature $Mode
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($feature)) {
        throw "Cannot resolve canonical Cargo feature preset for runtime profile $Mode."
    }
    return $feature.Trim()
}

function Resolve-RunBin {
    param([string]$Mode)
    switch ($Mode) {
        "client2d" { return "zircon_runtime" }
        "client3d" { return "zircon_runtime" }
        "editor" { return "zircon_editor" }
        "dev" { return "zircon_editor" }
        default { return $null }
    }
}

function Assert-AllowedCargoTargetPath {
    param([string]$Path)

    $candidate = [System.IO.Path]::GetFullPath($Path).TrimEnd('\', '/')
    $allowedRoots = @(
        [System.IO.Path]::GetFullPath('D:\cargo-targets').TrimEnd('\', '/'),
        [System.IO.Path]::GetFullPath('E:\cargo-targets').TrimEnd('\', '/'),
        [System.IO.Path]::GetFullPath('F:\cargo-targets').TrimEnd('\', '/')
    )
    foreach ($root in $allowedRoots) {
        if ($candidate.StartsWith($root + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
            return
        }
    }

    throw "Cargo build output must be below D:\cargo-targets, E:\cargo-targets, or F:\cargo-targets: $candidate"
}

$repoRoot = Resolve-RepoRoot -Start $PSScriptRoot
Push-Location $repoRoot
try {
    if ($Release -and $CargoProfile -ne "debug") {
        throw "-Release cannot be combined with -CargoProfile $CargoProfile."
    }

    if ([string]::IsNullOrWhiteSpace($SharedTargetRoot)) {
        $drive = [System.IO.Path]::GetPathRoot($repoRoot).TrimEnd('\')
        $SharedTargetRoot = Join-Path $drive "cargo-targets\zircon-shared"
    }
    Assert-AllowedCargoTargetPath -Path $SharedTargetRoot

    $feature = if ([string]::IsNullOrWhiteSpace($FeatureOverride)) {
        Resolve-FeatureSet -RepoRoot $repoRoot -Mode $Profile
    } else {
        $FeatureOverride
    }
    $targetDir = Join-Path $SharedTargetRoot $Profile
    Assert-AllowedCargoTargetPath -Path $targetDir
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
    if ($Release -or $CargoProfile -eq "release") {
        $args.Add("--release") | Out-Null
    } elseif ($CargoProfile -eq "profiling") {
        $args.Add("--profile") | Out-Null
        $args.Add("profiling") | Out-Null
    }

    if ($Action -eq "run") {
        $bin = Resolve-RunBin -Mode $Profile
        if ($null -eq $bin) {
            throw "Runtime profile $Profile has no runnable bin. Use check/build/test."
        }
        $args.Add("--bin") | Out-Null
        $args.Add($bin) | Out-Null
    }

    if ($ExtraCargoArgs) {
        foreach ($item in $ExtraCargoArgs) { $args.Add($item) | Out-Null }
    }

    Write-Host "RepoRoot: $repoRoot"
    Write-Host "Profile: $Profile -> feature: $feature"
    Write-Host "CargoProfile: $CargoProfile"
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
