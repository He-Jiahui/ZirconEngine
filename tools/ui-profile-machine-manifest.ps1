Set-StrictMode -Version Latest

$script:ZirconPerformanceMachineManifestScript =
    Join-Path $PSScriptRoot "performance-machine-manifest.ps1"
if (-not (Test-Path -LiteralPath $script:ZirconPerformanceMachineManifestScript)) {
    throw "UI profile machine manifest requires: $script:ZirconPerformanceMachineManifestScript"
}
. $script:ZirconPerformanceMachineManifestScript

function Export-ZirconUiProfileMachineManifest {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProfileDir,
        [System.Collections.IDictionary]$Observations
    )

    $manifest = if ($PSBoundParameters.ContainsKey("Observations")) {
        New-ZirconPerformanceMachineManifest -Observations $Observations
    }
    else {
        New-ZirconPerformanceMachineManifest
    }
    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    $path = Join-Path $ProfileDir "machine_manifest.json"
    $manifest | ConvertTo-Json -Depth 12 |
        Set-Content -LiteralPath $path -Encoding UTF8
    return $path
}
