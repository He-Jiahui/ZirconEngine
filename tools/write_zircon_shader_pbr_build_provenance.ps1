[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ViewerExe,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9a-fA-F]{32}$')]
    [string]$ValidationTicketId,
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9a-fA-F]{32}$')]
    [string]$ArtifactReceiptId
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
. (Join-Path $PSScriptRoot "profile-capture-manifest.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-contract.ps1")

function Get-ZirconShaderPbrBuildRequiredFingerprint {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Description
    )

    return Get-ZirconProfileRequiredFileFingerprint -Path $Path -Description $Description
}

function Write-ZirconShaderPbrBuildProvenance {
    $viewer = Get-ZirconShaderPbrBuildRequiredFingerprint `
        -Path $ViewerExe `
        -Description "Shader PBR viewer binary"
    $resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
    if ($resolvedOutput.StartsWith("C:\", [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Managed shader PBR build provenance must not be written beneath C:."
    }
    if ($resolvedOutput -eq [System.IO.Path]::GetFullPath($viewer.path)) {
        throw "Managed shader PBR build provenance must not overwrite the viewer binary."
    }
    $sourceManifest = [ordered]@{}
    foreach ($relativePath in Get-ZirconShaderPbrProfileCriticalSourcePaths) {
        $source = Get-ZirconShaderPbrBuildRequiredFingerprint `
            -Path (Join-Path $RepoRoot $relativePath) `
            -Description "critical Shader06 source file '$relativePath'"
        $sourceManifest[$relativePath] = $source.sha256
    }
    $sourceFiles = @($sourceManifest.GetEnumerator() | ForEach-Object {
        [pscustomobject]@{
            relative_path = [string]$_.Key
            sha256 = [string]$_.Value
        }
    })
    $validationTicket = Get-ZirconShaderPbrCoordinatorValidationTicket `
        -RepoRoot $RepoRoot `
        -ValidationTicketId $ValidationTicketId
    $sourceValidationTicket = Assert-ZirconShaderPbrCoordinatorValidationTicket `
        -Ticket $validationTicket `
        -SourceFiles $sourceFiles `
        -Description "Shader PBR viewer source validation ticket"
    $artifactReceipt = Get-ZirconShaderPbrCoordinatorArtifactReceipt `
        -RepoRoot $RepoRoot `
        -ArtifactReceiptId $ArtifactReceiptId
    $managedArtifactReceipt = Assert-ZirconShaderPbrCoordinatorArtifactReceipt `
        -Receipt $artifactReceipt `
        -ViewerFingerprint $viewer `
        -ValidationTicketId $sourceValidationTicket.validation_ticket_id `
        -SourceManifestHash $sourceValidationTicket.source_manifest_hash `
        -Description "Shader PBR viewer managed artifact receipt"
    $parent = Split-Path -Parent $resolvedOutput
    if (-not [string]::IsNullOrWhiteSpace($parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    $provenance = [pscustomobject]@{
        schema_version = 2
        provenance_kind = "zircon_managed_viewer_artifact_provenance"
        generated_utc = (Get-Date).ToUniversalTime().ToString("o")
        binary = [pscustomobject]@{
            path = [string]$viewer.path
            sha256 = [string]$viewer.sha256
            byte_length = [int64]$viewer.byte_length
        }
        repository = [pscustomobject]@{
            root = $RepoRoot
            source_manifest = $sourceManifest
        }
        source_validation_ticket = $sourceValidationTicket
        artifact_receipt = $managedArtifactReceipt
    }
    $provenance | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding UTF8
    Write-Output $resolvedOutput
}

if ($MyInvocation.InvocationName -ne ".") {
    Write-ZirconShaderPbrBuildProvenance
}
