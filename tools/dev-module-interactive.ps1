[CmdletBinding()]
param(
    [ValidateSet("check", "build", "test", "run")]
    [string]$Action = "run",
    [switch]$Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runner = Join-Path $scriptDir "dev-fast-build.ps1"

if (-not (Test-Path $runner)) {
    throw "Missing script: $runner"
}

$targets = @(
    [pscustomobject]@{ Id = "runtime"; Label = "Runtime (client)"; Profile = "client"; TargetMode = "client_runtime"; BaseFeatures = @("core-min", "plugin-graphics-base", "plugin-ui", "target-client") },
    [pscustomobject]@{ Id = "editor"; Label = "Editor host"; Profile = "editor"; TargetMode = "editor_host"; BaseFeatures = @("core-min", "plugin-graphics-base", "plugin-ui", "target-editor-host") }
)

$moduleOptions = @(
    [pscustomobject]@{ Key = "physics"; Feature = "plugin-physics"; Manifest = "physics" },
    [pscustomobject]@{ Key = "sound"; Feature = "plugin-sound"; Manifest = "sound" },
    [pscustomobject]@{ Key = "animation"; Feature = "plugin-animation"; Manifest = "animation" },
    [pscustomobject]@{ Key = "net"; Feature = "plugin-net"; Manifest = "net" },
    [pscustomobject]@{ Key = "navigation"; Feature = "plugin-navigation"; Manifest = "navigation" },
    [pscustomobject]@{ Key = "particles"; Feature = "plugin-particles"; Manifest = "particles" },
    [pscustomobject]@{ Key = "texture"; Feature = "plugin-texture"; Manifest = "texture" },
    [pscustomobject]@{ Key = "vg"; Feature = "plugin-vg"; Manifest = "vg" },
    [pscustomobject]@{ Key = "gi"; Feature = "plugin-gi"; Manifest = "gi" }
)

Write-Host "Select target:"
for ($i = 0; $i -lt $targets.Count; $i++) {
    Write-Host ("  {0}) {1}" -f ($i + 1), $targets[$i].Label)
}
$targetIndexRaw = Read-Host "Input number (default 1)"
$targetIndex = 0
if (-not [string]::IsNullOrWhiteSpace($targetIndexRaw)) {
    $targetIndex = [Math]::Max(0, [Math]::Min($targets.Count - 1, ([int]$targetIndexRaw - 1)))
}
$selectedTarget = $targets[$targetIndex]

Write-Host ""
Write-Host "Optional modules (comma separated, e.g. physics,sound,vg):"
Write-Host ("  {0}" -f (($moduleOptions | ForEach-Object { $_.Key }) -join ", "))
$moduleRaw = Read-Host "Modules"

$selectedKeys = @()
if (-not [string]::IsNullOrWhiteSpace($moduleRaw)) {
    $selectedKeys = $moduleRaw.Split(",") | ForEach-Object { $_.Trim().ToLowerInvariant() } | Where-Object { $_ }
}

$selectedModules = @()
foreach ($key in $selectedKeys) {
    $match = $moduleOptions | Where-Object { $_.Key -eq $key } | Select-Object -First 1
    if ($match) { $selectedModules += $match }
}

$featureSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
foreach ($f in $selectedTarget.BaseFeatures) { [void]$featureSet.Add($f) }
foreach ($m in $selectedModules) { [void]$featureSet.Add($m.Feature) }

$manifestParts = [System.Collections.Generic.List[string]]::new()
$manifestParts.Add("graphics") | Out-Null
$manifestParts.Add("ui") | Out-Null
foreach ($m in $selectedModules) { $manifestParts.Add($m.Manifest) | Out-Null }
$manifest = ($manifestParts | Select-Object -Unique) -join ","

$features = ($featureSet | Sort-Object) -join ","

$env:ZIRCON_TARGET_MODE = $selectedTarget.TargetMode
$env:ZIRCON_PLUGIN_MANIFEST = $manifest

Write-Host ""
Write-Host ("TargetMode: {0}" -f $env:ZIRCON_TARGET_MODE)
Write-Host ("PluginManifest: {0}" -f $env:ZIRCON_PLUGIN_MANIFEST)
Write-Host ("Features: {0}" -f $features)

$args = @(
    "-Profile", $selectedTarget.Profile,
    "-Action", $Action,
    "-FeatureOverride", $features
)
if ($Release) {
    $args += "-Release"
}
if ($Action -in @("check", "build", "test")) {
    $args += @("-Package", "zircon_app")
}

& $runner @args
