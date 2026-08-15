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
$pathResolver = Join-Path $PSScriptRoot "WindowsPathResolver.psm1"
if (-not (Test-Path -LiteralPath $pathResolver -PathType Leaf)) {
    throw "Windows path resolver was not found: $pathResolver"
}
Import-Module $pathResolver -Force -DisableNameChecking -ErrorAction Stop

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

function Resolve-AllowedCargoTargetPath {
    param([string]$Path)

    $targetResolution = Resolve-ZirconWindowsPath -Path $Path
    $candidate = $targetResolution.OperationalPath.TrimEnd('\', '/')
    $allowedRoots = @(
        'D:\cargo-targets',
        'E:\cargo-targets',
        'F:\cargo-targets'
    )
    foreach ($rootPath in $allowedRoots) {
        $root = (Resolve-ZirconWindowsPath -Path $rootPath).OperationalPath.TrimEnd('\', '/')
        if ($candidate.StartsWith($root + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
            return $targetResolution
        }
    }

    throw "Cargo build output must physically resolve below D:\cargo-targets, E:\cargo-targets, or F:\cargo-targets: $($targetResolution.DisplayPath)"
}

function Assert-ChildPath {
    param(
        [Parameter(Mandatory)]
        [object]$ParentResolution,
        [Parameter(Mandatory)]
        [object]$ChildResolution,
        [Parameter(Mandatory)]
        [string]$Label
    )

    $parent = $ParentResolution.OperationalPath.TrimEnd('\', '/')
    $child = $ChildResolution.OperationalPath.TrimEnd('\', '/')
    if (-not $child.StartsWith($parent + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label must physically resolve below $($ParentResolution.DisplayPath): $($ChildResolution.DisplayPath)"
    }
}

function Push-ManagedFastBuildEnvironment {
    param(
        [Parameter(Mandatory)]
        [object]$SharedTargetResolution,
        [Parameter(Mandatory)]
        [object]$TargetResolution
    )

    Assert-ChildPath -ParentResolution $SharedTargetResolution -ChildResolution $TargetResolution -Label 'Profile target'
    $temporaryResolution = Resolve-ZirconWindowsPath -Path (Join-Path $TargetResolution.DisplayPath 'temporary')
    $cargoHomeResolution = Resolve-ZirconWindowsPath -Path (Join-Path $SharedTargetResolution.DisplayPath 'cargo-home')
    $sccacheResolution = Resolve-ZirconWindowsPath -Path (Join-Path $SharedTargetResolution.DisplayPath 'sccache')
    Assert-ChildPath -ParentResolution $TargetResolution -ChildResolution $temporaryResolution -Label 'Temporary directory'
    Assert-ChildPath -ParentResolution $SharedTargetResolution -ChildResolution $cargoHomeResolution -Label 'Cargo home'
    Assert-ChildPath -ParentResolution $SharedTargetResolution -ChildResolution $sccacheResolution -Label 'sccache directory'

    foreach ($resolution in @($TargetResolution, $temporaryResolution, $cargoHomeResolution, $sccacheResolution)) {
        [System.IO.Directory]::CreateDirectory($resolution.OperationalPath) | Out-Null
    }

    $names = @('CARGO_TARGET_DIR', 'CARGO_HOME', 'SCCACHE_DIR', 'TEMP', 'TMP', 'TMPDIR')
    $previousValues = @{}
    foreach ($name in $names) {
        $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    $previousPath = [Environment]::GetEnvironmentVariable('PATH', 'Process')

    try {
        [Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', $TargetResolution.DisplayPath, 'Process')
        [Environment]::SetEnvironmentVariable('CARGO_HOME', $cargoHomeResolution.DisplayPath, 'Process')
        [Environment]::SetEnvironmentVariable('SCCACHE_DIR', $sccacheResolution.DisplayPath, 'Process')
        foreach ($name in @('TEMP', 'TMP', 'TMPDIR')) {
            [Environment]::SetEnvironmentVariable($name, $temporaryResolution.DisplayPath, 'Process')
        }
        [Environment]::SetEnvironmentVariable('PATH', (Join-Path $cargoHomeResolution.DisplayPath 'bin') + [IO.Path]::PathSeparator + $previousPath, 'Process')
    }
    catch {
        foreach ($name in $names) {
            [Environment]::SetEnvironmentVariable($name, $previousValues[$name], 'Process')
        }
        [Environment]::SetEnvironmentVariable('PATH', $previousPath, 'Process')
        throw
    }

    return [pscustomobject]@{
        PreviousValues = $previousValues
        PreviousPath   = $previousPath
        Temporary      = $temporaryResolution.DisplayPath
        CargoHome      = $cargoHomeResolution.DisplayPath
        Sccache        = $sccacheResolution.DisplayPath
    }
}

function Pop-ManagedFastBuildEnvironment {
    param([Parameter(Mandatory)][object]$Lease)

    foreach ($name in $Lease.PreviousValues.Keys) {
        [Environment]::SetEnvironmentVariable($name, $Lease.PreviousValues[$name], 'Process')
    }
    [Environment]::SetEnvironmentVariable('PATH', $Lease.PreviousPath, 'Process')
}

$repoRoot = Resolve-RepoRoot -Start $PSScriptRoot
$buildEnvironmentLease = $null
$previousRustcWrapper = [Environment]::GetEnvironmentVariable('RUSTC_WRAPPER', 'Process')
Push-Location $repoRoot
try {
    if ($Release -and $CargoProfile -ne "debug") {
        throw "-Release cannot be combined with -CargoProfile $CargoProfile."
    }

    if ([string]::IsNullOrWhiteSpace($SharedTargetRoot)) {
        $drive = [System.IO.Path]::GetPathRoot($repoRoot).TrimEnd('\')
        $SharedTargetRoot = Join-Path $drive "cargo-targets\zircon-shared"
    }
    $sharedTargetResolution = Resolve-AllowedCargoTargetPath -Path $SharedTargetRoot
    $SharedTargetRoot = $sharedTargetResolution.DisplayPath

    $feature = if ([string]::IsNullOrWhiteSpace($FeatureOverride)) {
        Resolve-FeatureSet -RepoRoot $repoRoot -Mode $Profile
    } else {
        $FeatureOverride
    }
    $targetResolution = Resolve-AllowedCargoTargetPath -Path (Join-Path $SharedTargetRoot $Profile)
    $buildEnvironmentLease = Push-ManagedFastBuildEnvironment -SharedTargetResolution $sharedTargetResolution -TargetResolution $targetResolution
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
    Write-Host "CARGO_HOME: $env:CARGO_HOME"
    Write-Host "SCCACHE_DIR: $env:SCCACHE_DIR"
    Write-Host "TEMP: $env:TEMP"
    if ($env:RUSTC_WRAPPER) {
        Write-Host "RUSTC_WRAPPER: $env:RUSTC_WRAPPER"
    }
    Write-Host ("cargo " + ($args -join " ")) -ForegroundColor DarkGray

    cargo @args
}
finally {
    [Environment]::SetEnvironmentVariable('RUSTC_WRAPPER', $previousRustcWrapper, 'Process')
    if ($null -ne $buildEnvironmentLease) {
        Pop-ManagedFastBuildEnvironment -Lease $buildEnvironmentLease
    }
    Pop-Location
}
