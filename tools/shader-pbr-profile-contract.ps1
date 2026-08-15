function Get-ZirconShaderPbrProfileCriticalSourcePaths {
    return @(
        "zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs",
        "zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs",
        "zircon_app/src/bin/zircon_shader_pbr_viewer/frame_io.rs",
        "zircon_app/src/bin/zircon_shader_pbr_viewer/gpu_timing_evidence.rs",
        "zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs",
        "zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs",
        "zircon_runtime/src/asset/importer/environment_ibl.rs",
        "zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_recipe.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/projection.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/rebuild.rs",
        "zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs",
        "zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_generic_api.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_manifest.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs"
    )
}

function ConvertTo-ZirconShaderPbrValidationSourceManifest {
    param(
        [Parameter(Mandatory = $true)]$SourceManifest,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $properties = if ($SourceManifest -is [System.Collections.IDictionary]) {
        @($SourceManifest.GetEnumerator() | ForEach-Object {
            [pscustomobject]@{ Name = [string]$_.Key; Value = $_.Value }
        })
    }
    else {
        @($SourceManifest.PSObject.Properties | ForEach-Object {
            [pscustomobject]@{ Name = [string]$_.Name; Value = $_.Value }
        })
    }
    if ($properties.Count -eq 0) {
        throw "$Description source manifest must not be empty."
    }

    $normalized = [ordered]@{}
    foreach ($property in $properties | Sort-Object -Property Name) {
        $relativePath = [string]$property.Name
        if ($relativePath -notmatch '^[A-Za-z0-9._/-]+$' -or
            $relativePath.Contains("\\") -or
            $relativePath.Split("/") | Where-Object { $_ -in @("", ".", "..") }) {
            throw "$Description source manifest has an unsafe path: $relativePath"
        }
        if ($normalized.Contains($relativePath)) {
            throw "$Description source manifest has a duplicate path: $relativePath"
        }
        if ($null -eq $property.Value) {
            $normalized[$relativePath] = $null
            continue
        }
        $hash = ([string]$property.Value).ToLowerInvariant()
        if ($hash -notmatch '^[0-9a-f]{64}$') {
            throw "$Description source manifest has an invalid SHA-256 for $relativePath."
        }
        $normalized[$relativePath] = $hash
    }
    return $normalized
}

function Get-ZirconShaderPbrValidationSourceManifestHash {
    param(
        [Parameter(Mandatory = $true)]$SourceManifest,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $normalized = ConvertTo-ZirconShaderPbrValidationSourceManifest `
        -SourceManifest $SourceManifest `
        -Description $Description
    # The coordinator hashes compact, sorted JSON for validation ticket manifests.
    $canonicalJson = $normalized | ConvertTo-Json -Compress
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($canonicalJson)
        return (-join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString("x2") }))
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-ZirconShaderPbrCoordinatorValidationTicket {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{32}$')][string]$ValidationTicketId
    )

    $sessionTool = Join-Path $RepoRoot "tools\zircon-session.ps1"
    if (-not (Test-Path -LiteralPath $sessionTool -PathType Leaf)) {
        throw "Shader PBR build provenance requires the coordinator command wrapper: $sessionTool"
    }
    $output = @(& $sessionTool -RepoRoot $RepoRoot -Json validation status --ticket-id $ValidationTicketId)
    if ($LASTEXITCODE -ne 0) {
        throw "Shader PBR build provenance could not read coordinator validation ticket $ValidationTicketId."
    }
    try {
        $response = ($output -join "`n") | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Shader PBR build provenance received malformed coordinator validation ticket data for $ValidationTicketId."
    }
    if ($null -eq $response.ticket) {
        throw "Shader PBR build provenance coordinator response is missing ticket $ValidationTicketId."
    }
    $requestedTicketId = $ValidationTicketId.ToLowerInvariant()
    $returnedTicketId = ([string]$response.ticket.ticket_id).ToLowerInvariant()
    if ($returnedTicketId -ne $requestedTicketId) {
        throw "Shader PBR build provenance coordinator returned a ticket different from requested validation ticket $ValidationTicketId."
    }
    return $response.ticket
}

function Get-ZirconShaderPbrCoordinatorArtifactReceipt {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{32}$')][string]$ArtifactReceiptId
    )

    $sessionTool = Join-Path $RepoRoot "tools\zircon-session.ps1"
    if (-not (Test-Path -LiteralPath $sessionTool -PathType Leaf)) {
        throw "Shader PBR build provenance requires the coordinator command wrapper: $sessionTool"
    }
    $output = @(& $sessionTool -RepoRoot $RepoRoot -Json validation artifact-receipt-status -receipt-id $ArtifactReceiptId)
    if ($LASTEXITCODE -ne 0) {
        throw "Shader PBR build provenance could not read coordinator artifact receipt $ArtifactReceiptId."
    }
    try {
        $response = ($output -join "`n") | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Shader PBR build provenance received malformed coordinator artifact receipt data for $ArtifactReceiptId."
    }
    if ($null -eq $response.artifactReceipt) {
        throw "Shader PBR build provenance coordinator response is missing artifact receipt $ArtifactReceiptId."
    }
    $requestedReceiptId = $ArtifactReceiptId.ToLowerInvariant()
    $returnedReceiptId = ([string]$response.artifactReceipt.receiptId).ToLowerInvariant()
    if ($returnedReceiptId -ne $requestedReceiptId) {
        throw "Shader PBR build provenance coordinator returned an artifact receipt different from requested receipt $ArtifactReceiptId."
    }
    return $response.artifactReceipt
}

function Assert-ZirconShaderPbrCoordinatorArtifactReceipt {
    param(
        [Parameter(Mandatory = $true)]$Receipt,
        [Parameter(Mandatory = $true)]$ViewerFingerprint,
        [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{32}$')][string]$ValidationTicketId,
        [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')][string]$SourceManifestHash,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $receiptId = ([string]$Receipt.receiptId).ToLowerInvariant()
    if ($receiptId -notmatch '^[0-9a-f]{32}$') {
        throw "$Description is missing a coordinator artifact receipt id."
    }
    if ([string]$Receipt.status -ne "passed") {
        throw "$Description requires a coordinator artifact receipt in terminal passed state."
    }
    if ([string]$Receipt.artifactKind -ne "shader-pbr-viewer") {
        throw "$Description has an unexpected artifact kind."
    }
    foreach ($identity in @(
        @{ Name = "job"; Value = [string]$Receipt.jobId },
        @{ Name = "run"; Value = [string]$Receipt.runId }
    )) {
        if ($identity.Value -notmatch '^[0-9a-f]{32}$') {
            throw "$Description is missing its managed $($identity.Name) identity."
        }
    }
    $returnedTicketId = ([string]$Receipt.validationTicketId).ToLowerInvariant()
    if ($returnedTicketId -ne $ValidationTicketId.ToLowerInvariant()) {
        throw "$Description does not match its coordinator validation ticket."
    }
    $receiptSourceManifest = ([string]$Receipt.sourceManifestHash).ToLowerInvariant()
    if ($receiptSourceManifest -ne $SourceManifestHash.ToLowerInvariant()) {
        throw "$Description source manifest does not match its validation ticket."
    }
    $inputManifest = ([string]$Receipt.inputManifestHash).ToLowerInvariant()
    if ($inputManifest -notmatch '^[0-9a-f]{64}$') {
        throw "$Description is missing the immutable validation-copy input manifest."
    }
    $commandHash = ([string]$Receipt.commandSha256).ToLowerInvariant()
    if ($commandHash -notmatch '^[0-9a-f]{64}$') {
        throw "$Description is missing its producing command identity."
    }
    $command = @($Receipt.command | ForEach-Object { [string]$_ })
    if ($command.Count -lt 7 -or
        [System.IO.Path]::GetFileName($command[0]) -notmatch '^cargo(?:\.exe)?$' -or
        $command -notcontains "build" -or
        $command -notcontains "zircon_app" -or
        $command -notcontains "zircon_shader_pbr_viewer" -or
        $command -notcontains "--locked") {
        throw "$Description does not identify the allow-listed managed Cargo viewer build."
    }
    $targetRelativePath = ([string]$Receipt.targetRelativePath).Replace("\", "/")
    if ($targetRelativePath -notmatch '^(debug|release|profiling)/zircon_shader_pbr_viewer(?:\.exe)?$') {
        throw "$Description has an invalid managed target-relative artifact path."
    }
    $receiptArtifactPath = [System.IO.Path]::GetFullPath([string]$Receipt.artifactPath)
    $viewerPath = [System.IO.Path]::GetFullPath([string]$ViewerFingerprint.path)
    if (-not $receiptArtifactPath.Equals($viewerPath, [System.StringComparison]::OrdinalIgnoreCase) -or
        ([string]$Receipt.sha256).ToLowerInvariant() -ne ([string]$ViewerFingerprint.sha256).ToLowerInvariant() -or
        [int64]$Receipt.byteLength -ne [int64]$ViewerFingerprint.byte_length) {
        throw "$Description artifact fingerprint does not bind the requested viewer binary."
    }
    if (-not [string]::IsNullOrWhiteSpace([string]$Receipt.errorCode)) {
        throw "$Description terminal receipt contains a rejection error."
    }
    return [pscustomobject]@{
        artifact_receipt_id = $receiptId
        status = "passed"
        artifact_kind = "shader-pbr-viewer"
        job_id = ([string]$Receipt.jobId).ToLowerInvariant()
        run_id = ([string]$Receipt.runId).ToLowerInvariant()
        validation_ticket_id = $returnedTicketId
        input_manifest_hash = $inputManifest
        source_manifest_hash = $receiptSourceManifest
        target_relative_path = $targetRelativePath
        artifact_path = $receiptArtifactPath
        sha256 = ([string]$Receipt.sha256).ToLowerInvariant()
        byte_length = [int64]$Receipt.byteLength
        command_sha256 = $commandHash
        command = $command
    }
}

function Assert-ZirconShaderPbrCoordinatorValidationTicket {
    param(
        [Parameter(Mandatory = $true)]$Ticket,
        [Parameter(Mandatory = $true)]$SourceFiles,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $ticketId = ([string]$Ticket.ticket_id).ToLowerInvariant()
    if ($ticketId -notmatch '^[0-9a-f]{32}$') {
        throw "$Description is missing a coordinator validation ticket id."
    }
    if ([string]$Ticket.status -ne "passed") {
        throw "$Description requires a coordinator validation ticket in terminal passed state."
    }
    $sourceManifest = ConvertTo-ZirconShaderPbrValidationSourceManifest `
        -SourceManifest $Ticket.source_manifest `
        -Description $Description
    $sourceManifestHash = Get-ZirconShaderPbrValidationSourceManifestHash `
        -SourceManifest $sourceManifest `
        -Description $Description
    if ([string]$Ticket.source_manifest_hash -ne $sourceManifestHash) {
        throw "$Description coordinator source manifest hash does not match its ticket manifest."
    }
    foreach ($source in @($SourceFiles)) {
        $relativePath = [string]$source.relative_path
        if (-not $sourceManifest.Contains($relativePath) -or
            [string]$sourceManifest[$relativePath] -ne [string]$source.sha256) {
            throw "$Description coordinator validation ticket does not bind current source $relativePath."
        }
    }
    return [pscustomobject]@{
        validation_ticket_id = $ticketId
        status = "passed"
        source_manifest_hash = $sourceManifestHash
        source_manifest = [pscustomobject]$sourceManifest
    }
}
