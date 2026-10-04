[CmdletBinding()]
param(
    [ValidateSet('render-only', 'export-apply')]
    [string]$CaptureMode = 'render-only',
    [string]$ArtifactRoot,
    [string]$CacheSource,
    [switch]$BuildOnly,
    [switch]$Offline
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $ArtifactRoot) {
    $ArtifactRoot = Join-Path 'E:\cargo-targets\zircon-local\penpot' ('workbench-' + [guid]::NewGuid().ToString('N'))
}
$arguments = @{ ArtifactRoot = $ArtifactRoot; BuildOnly = $true }
if ($CacheSource) { $arguments.CacheSource = $CacheSource }
if ($Offline) { $arguments.Offline = $true }
& (Join-Path $PSScriptRoot 'run-validation.ps1') @arguments
$validation = Get-Content -LiteralPath (Join-Path $ArtifactRoot 'validation.json') -Raw | ConvertFrom-Json
if ($validation.status -ne 'passed') { throw 'Build validation did not pass.' }

$manifest = [ordered]@{
    schema = 'dev.zircon.penpot.managed-plugin-validation'
    version = 2
    status = 'built'
    captureMode = $CaptureMode
    artifactRoot = $ArtifactRoot
    bundlePath = $env:ZUI_PLUGIN_BUNDLE_PATH
    bundleSha256 = (Get-FileHash -LiteralPath $env:ZUI_PLUGIN_BUNDLE_PATH).Hash.ToLowerInvariant()
    distributionBundlePath = Join-Path $env:ZUI_PLUGIN_DIST_ROOT 'assets\plugin.js'
    captureSummaryPath = $null
    persistenceVerified = $false
}
try {
    if (-not $BuildOnly) {
        $env:ZIRCON_PENPOT_ARTIFACT_ROOT = $ArtifactRoot
        $workspace = $validation.workspace
        $captureArguments = @(
            (Join-Path $workspace 'node_modules\tsx\dist\cli.mjs'),
            (Join-Path $workspace 'apps\zircon-zui-plugin\tools\penpot-workbench-export-apply-contract.ts')
        )
        if ($CaptureMode -eq 'render-only') { $captureArguments += '--render-only' }
        $log = Join-Path $ArtifactRoot 'capture-command.log'
        Push-Location $workspace
        try { & node @captureArguments *> $log; $captureCode = $LASTEXITCODE }
        finally { Pop-Location }
        Get-Content -LiteralPath $log
        if ($captureCode -ne 0) { throw "Capture failed with exit code $captureCode. See $log" }
        $summaryName = if ($CaptureMode -eq 'render-only') { 'penpot-full-workbench-render-diagnostic.json' } else { 'penpot-full-workbench-export-apply-contract.json' }
        $summaryPath = Join-Path $ArtifactRoot ('penpot-workbench-export-apply\' + $summaryName)
        $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
        if ($summary.backend -ne 'repository-mock' -or $summary.persistenceVerified -ne $false) {
            throw 'Capture did not preserve the mock backend and unverified persistence boundary.'
        }
        if ($CaptureMode -eq 'render-only' -and $summary.status -ne 'pending') {
            throw 'Render-only evidence must remain pending.'
        }
        if (-not (Test-Path -LiteralPath $summary.hostScreenshotPath -PathType Leaf)) {
            throw 'Capture screenshot is missing.'
        }
        $manifest.status = 'captured'
        $manifest.captureSummaryPath = $summaryPath
    }
}
catch {
    $manifest.status = 'failed'
    $manifest.error = $_.Exception.Message
    throw
}
finally {
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $ArtifactRoot 'managed-run-manifest.json') -Encoding utf8
    Write-Host "Artifacts retained for review: $ArtifactRoot"
}
