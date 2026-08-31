$script:ZirconShaderPbrProfileContractRoot = Split-Path -Parent $PSScriptRoot
$script:ZirconShaderPbrProfileSourceClosurePath = Join-Path `
    $PSScriptRoot `
    "shader-pbr-profile-source-closure.ps1"
if (-not (Test-Path -LiteralPath $script:ZirconShaderPbrProfileSourceClosurePath -PathType Leaf)) {
    throw "Shader PBR profile contract cannot find the viewer source closure helper: $script:ZirconShaderPbrProfileSourceClosurePath"
}
. $script:ZirconShaderPbrProfileSourceClosurePath

function Get-ZirconShaderPbrProfileToolPaths {
    return @(
        "tools/performance-machine-manifest.ps1",
        "tools/profile-capture-manifest.ps1",
        "tools/shader-pbr-profile-contract.ps1",
        "tools/shader-pbr-profile-evidence-identity.ps1",
        "tools/shader-pbr-profile-publication.ps1",
        "tools/shader-pbr-profile-runtime-evidence.ps1",
        "tools/shader-pbr-profile-toolchain.ps1",
        "tools/write_zircon_shader_pbr_build_provenance.ps1",
        "tools/zircon_pbr_visual_oracle.py",
        "tools/zircon_profile_shader_pbr_viewer.ps1",
        "tools/zircon_shader_pbr_evidence_identity.py",
        "tools/zircon_shader_pbr_profile_tool_identity.py",
        "tools/zircon_summarize_shader_pbr_profile.py",
        "tools/zircon_validate_shader_pbr_gpu_timing_evidence.py",
        "tools/zircon_validate_shader_pbr_renderdoc_replay.py",
        "tools/zircon_validate_shader_pbr_viewer_evidence.py"
    )
}

function Get-ZirconShaderPbrProfileCriticalSourcePaths {
    param([string]$RepoRoot = $script:ZirconShaderPbrProfileContractRoot)

    $viewerSources = @(Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $RepoRoot)
    $crossModuleSources = @(
        "zircon_runtime/src/asset/artifact/mod.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/error.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/manifest.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/mod.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/payload_stamp.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/wire.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging/bundle_recovery.rs",
        "zircon_runtime/src/asset/assets/material/material_asset.rs",
        "zircon_runtime/src/asset/importer/mod.rs",
        "zircon_runtime/src/asset/importer/environment_ibl.rs",
        "zircon_runtime/src/asset/importer/environment_ibl/restore.rs",
        "zircon_runtime/src/asset/importer/environment_ibl/source_cubemap_texture.rs",
        "zircon_runtime/src/asset/importer/environment_ibl/source_identity.rs",
        "zircon_runtime/src/asset/importer/environment_ibl/source_staging/error.rs",
        "zircon_runtime/src/asset/importer/environment_ibl/warm_cache.rs",
        "zircon_runtime/src/asset/importer/image_decode.rs",
        "zircon_runtime/src/asset/importer/image_decode/source_format_identity.rs",
        "zircon_runtime/src/asset/importer/image_decode/source_metadata.rs",
        "zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs",
        "zircon_runtime/src/core/framework/render/backend_types.rs",
        "zircon_runtime/src/core/framework/render/frame_profile.rs",
        "zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_recipe.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/mod.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/ibl_bake_key.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/mode.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/settings.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/mod.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/bake_key.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/constants.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/params.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/resolved_sun.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/sun_resolution.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/mod.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/equality.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/identity.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/provenance.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/state.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/upload.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/source_cubemap_environment/upload_key.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/projection.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/rebuild.rs",
        "zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs",
        "zircon_runtime/src/core/framework/render/material/standard_material.rs",
        "zircon_runtime/src/core/framework/render/material/texture_transform.rs",
        "zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
        "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/mesh_submission.rs",
        "zircon_runtime/src/core/framework/render/environment/realtime_ibl_status.rs",
        "zircon_runtime/src/core/framework/render/environment/runtime_snapshot.rs",
        "zircon_runtime/src/core/framework/render/environment/mod.rs",
        "zircon_runtime/src/core/framework/render/framework.rs",
        "zircon_runtime/src/core/framework/render/framework_error.rs",
        "zircon_runtime/src/core/framework/render/mod.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/mod.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/query_environment_runtime_snapshot/mod.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/query_environment_runtime_snapshot/query_environment_runtime_snapshot.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_cpu_timing.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/realtime_ibl_capture.wgsl",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources/execution_resource_cache.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs",
        "zircon_runtime/src/graphics/scene/gpu_scene/bindless_material_payload.rs",
        "zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/material_pipeline_publication.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl",
        "zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs",
        "zircon_runtime/src/graphics/shader/includes/zr_normal.wgsl",
        "zircon_runtime/src/graphics/shader/includes/zr_pbr_common.wgsl",
        "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras_core.wgsl",
        "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl",
        "zircon_runtime/src/graphics/shader/template/assemble.rs",
        "zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs",
        "zircon_runtime/src/graphics/shader/template/material_surface.rs",
        "zircon_runtime/src/graphics/shader/template/module_registry.rs",
        "zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs",
        "zircon_runtime/src/graphics/shader/variant_cache/disk.rs",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_generic_api.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_procedural_sky.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl",
        "zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
        "zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_manifest.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs"
    )
    return @($viewerSources + $crossModuleSources | Sort-Object -Unique)
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
    $propertyByName = [System.Collections.Generic.Dictionary[string, object]]::new(
        [System.StringComparer]::Ordinal
    )
    $propertyNames = [System.Collections.Generic.List[string]]::new()
    foreach ($property in $properties) {
        $relativePath = [string]$property.Name
        if ($propertyByName.ContainsKey($relativePath)) {
            throw "$Description source manifest has a duplicate path: $relativePath"
        }
        $propertyByName[$relativePath] = $property
        $propertyNames.Add($relativePath)
    }
    $propertyNames.Sort([System.StringComparer]::Ordinal)
    foreach ($relativePath in $propertyNames) {
        $property = $propertyByName[$relativePath]
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
