param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot,
    [Parameter(Mandatory = $true)]
    [string]$ProfileRoot,
    [Parameter(Mandatory = $true)]
    [string]$ViewerExe,
    [Parameter(Mandatory = $true)]
    [string]$HdriPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Import-Module Microsoft.PowerShell.Utility -ErrorAction Stop

$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$ProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
$ViewerExe = [System.IO.Path]::GetFullPath($ViewerExe)
$HdriPath = [System.IO.Path]::GetFullPath($HdriPath)
$writer = Join-Path $RepoRoot "tools\write_zircon_shader_pbr_build_provenance.ps1"
$capture = Join-Path $RepoRoot "tools\zircon_profile_shader_pbr_viewer.ps1"
$provenancePath = Join-Path $ProfileRoot "viewer-build-provenance.json"
$toolchainPath = Join-Path $ProfileRoot "capture-toolchain.json"

[ordered]@{
    schema_version = 2
    toolchain_kind = "zircon_shader_pbr_capture_toolchain"
    graphics = [ordered]@{
        wgpu_backend = "dx12"
        evidence_backend = "wgpu(dx12)"
        permitted_backends = @("dx12")
        unsupported_backends = @("vulkan", "gl", "metal")
    }
    renderdoc = $null
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

. $writer `
    -ViewerExe $ViewerExe `
    -OutputPath $provenancePath `
    -ValidationTicketId ("a" * 32) `
    -ArtifactReceiptId ("f" * 32) | Out-Null
. $capture `
    -ViewerExe $ViewerExe `
    -HdriPath $HdriPath `
    -BuildProvenance $provenancePath `
    -CaptureToolchainManifest $toolchainPath | Out-Null

$sourceFiles = @(Get-ZirconShaderPbrProfileCriticalSourcePaths | ForEach-Object {
    $relativePath = $_
    $fingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
        -Path (Join-Path $RepoRoot $relativePath) `
        -Description "temporary integration critical source '$relativePath'"
    [pscustomobject]@{
        relative_path = $relativePath
        sha256 = $fingerprint.sha256
    }
})
$sourceManifest = [ordered]@{}
foreach ($sourceFile in $sourceFiles) {
    $sourceManifest[$sourceFile.relative_path] = $sourceFile.sha256
}
$sourceManifestHash = Get-ZirconShaderPbrValidationSourceManifestHash `
    -SourceManifest $sourceManifest `
    -Description "temporary integration validation ticket"
$viewerFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
    -Path $ViewerExe `
    -Description "temporary integration viewer"
$script:IntegrationValidationTicket = [pscustomobject]@{
    ticket_id = ("a" * 32)
    status = "passed"
    source_manifest = [pscustomobject]$sourceManifest
    source_manifest_hash = $sourceManifestHash
}
$script:IntegrationArtifactReceipt = [pscustomobject]@{
    receiptId = ("f" * 32)
    sessionId = "tooling07-integration"
    jobId = ("c" * 32)
    validationTicketId = ("a" * 32)
    artifactKind = "shader-pbr-viewer"
    status = "passed"
    inputManifestHash = ("d" * 64)
    sourceManifestHash = $sourceManifestHash
    runId = ("e" * 32)
    targetRelativePath = "release/zircon_shader_pbr_viewer.exe"
    artifactPath = $ViewerExe
    sha256 = $viewerFingerprint.sha256
    byteLength = $viewerFingerprint.byte_length
    command = @("cargo", "+1.94.1", "build", "-p", "zircon_app", "--bin", "zircon_shader_pbr_viewer", "--locked", "--release")
    commandSha256 = ("9" * 64)
    errorCode = $null
}

function Get-ZirconShaderPbrCoordinatorValidationTicket {
    param(
        [string]$RepoRoot,
        [string]$ValidationTicketId
    )

    if ($ValidationTicketId -ne ("a" * 32)) {
        throw "Temporary integration received an unexpected validation ticket id."
    }
    return $script:IntegrationValidationTicket
}

function Get-ZirconShaderPbrCoordinatorArtifactReceipt {
    param(
        [string]$RepoRoot,
        [string]$ArtifactReceiptId
    )

    if ($ArtifactReceiptId -ne ("f" * 32)) {
        throw "Temporary integration received an unexpected artifact receipt id."
    }
    return $script:IntegrationArtifactReceipt
}

Write-ZirconShaderPbrBuildProvenance | Out-Null
$captureToolchain = Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
$machineManifest = New-ZirconPerformanceMachineManifest
$manifestPath = Export-ZirconShaderPbrProfileManifest `
    -ProfileRoot $ProfileRoot `
    -ViewerExe $ViewerExe `
    -HdriPath $HdriPath `
    -BuildProvenance $provenancePath `
    -EvidenceRoot $ProfileRoot `
    -Repetitions 5 `
    -FaceSize 64 `
    -PmremFaceSize 64 `
    -CacheModes @("cold", "warm") `
    -CaptureToolchain $captureToolchain `
    -MachineManifest $machineManifest
Write-Output $manifestPath
