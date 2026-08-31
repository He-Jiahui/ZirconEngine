function ConvertTo-ZirconShaderPbrIdentityFileFingerprint {
    param(
        [Parameter(Mandatory = $true)]$Fingerprint
    )

    return [ordered]@{
        path = [string]$Fingerprint.path
        sha256 = [string]$Fingerprint.sha256
        byte_length = [int64]$Fingerprint.byte_length
    }
}

function New-ZirconShaderPbrReadyFrameEvidenceIdentity {
    param(
        [Parameter(Mandatory = $true)][string]$RunDirectory,
        [Parameter(Mandatory = $true)][string]$ProfileId,
        [Parameter(Mandatory = $true)][ValidateSet("cold", "warm")][string]$CacheMode,
        [Parameter(Mandatory = $true)][ValidateSet("cache_seed", "measured", "renderdoc")][string]$Role,
        [Parameter(Mandatory = $true)][int]$Ordinal,
        [Parameter(Mandatory = $true)][string]$ViewerExe,
        [Parameter(Mandatory = $true)][string]$HdriPath,
        [Parameter(Mandatory = $true)][string]$BuildProvenance
    )

    $identityPath = Join-Path $RunDirectory "evidence_identity.json"
    $viewerFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $ViewerExe `
        -Description "shader PBR viewer identity binary"
    $hdriFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $HdriPath `
        -Description "shader PBR viewer identity HDRI"
    $provenanceFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $BuildProvenance `
        -Description "shader PBR viewer identity build provenance"
    $viewer = ConvertTo-ZirconShaderPbrIdentityFileFingerprint -Fingerprint $viewerFingerprint
    $hdri = ConvertTo-ZirconShaderPbrIdentityFileFingerprint -Fingerprint $hdriFingerprint
    $provenance = ConvertTo-ZirconShaderPbrIdentityFileFingerprint -Fingerprint $provenanceFingerprint
    try {
        $provenanceJson = Get-Content -LiteralPath $provenance.path -Raw | ConvertFrom-Json
    }
    catch {
        throw "Shader PBR evidence identity build provenance is malformed: $($provenance.path)"
    }
    $sourceManifestHash = [string]$provenanceJson.source_validation_ticket.source_manifest_hash
    if ($sourceManifestHash -notmatch '^[0-9a-f]{64}$') {
        throw "Shader PBR evidence identity build provenance has no valid source manifest hash: $($provenance.path)"
    }

    $runId = "{0}-{1}-{2}-{3:D2}" -f $ProfileId, $CacheMode, $Role, $Ordinal
    if ($runId -notmatch '^[a-z][a-z0-9-]{2,159}$') {
        throw "Shader PBR evidence identity run id is invalid: $runId"
    }
    $identity = [ordered]@{
        schema = "zircon_shader_pbr_viewer_evidence_identity_v1"
        run_id = $runId
        validation_policy = "zircon_shader_pbr_viewer_ready_frame_v15"
        source_manifest_sha256 = $sourceManifestHash
        viewer_binary = $viewer
        hdri = $hdri
        build_provenance = $provenance
    }
    $identity | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $identityPath -Encoding UTF8
    return Get-ZirconProfileRequiredFileFingerprint `
        -Path $identityPath `
        -Description "shader PBR Ready-frame evidence identity"
}
