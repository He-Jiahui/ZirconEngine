$script:ZirconShaderPbrProfileContractRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$script:ZirconShaderPbrProfileSourceClosurePath = Join-Path `
    $PSScriptRoot `
    "shader-pbr-profile-source-closure.ps1"
if (-not (Test-Path -LiteralPath $script:ZirconShaderPbrProfileSourceClosurePath -PathType Leaf)) {
    throw "Shader PBR profile contract cannot find the viewer source closure helper: $script:ZirconShaderPbrProfileSourceClosurePath"
}
. $script:ZirconShaderPbrProfileSourceClosurePath

function Get-ZirconShaderPbrProfileToolPaths {
    return @(
        "tools/analysis/profiling/shared/performance-machine-manifest.ps1",
        "tools/analysis/profiling/shared/profile-capture-paths.ps1",
        "tools/analysis/profiling/shared/profile-capture-manifest.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-contract.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-evidence-identity.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-publication.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-runtime-evidence.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-source-closure.ps1",
        "tools/analysis/profiling/shader_pbr/shader-pbr-profile-toolchain.ps1",
        "tools/analysis/profiling/shader_pbr/write_zircon_shader_pbr_build_provenance.ps1",
        "tools/analysis/visual/zircon_pbr_visual_oracle.py",
        "tools/analysis/profiling/shader_pbr/zircon_profile_shader_pbr_viewer.ps1",
        "tools/analysis/profiling/shader_pbr/zircon_shader_pbr_evidence_identity.py",
        "tools/analysis/profiling/shader_pbr/zircon_shader_pbr_profile_tool_identity.py",
        "tools/analysis/profiling/shader_pbr/zircon_summarize_shader_pbr_profile.py",
        "tools/analysis/profiling/shader_pbr/zircon_validate_shader_pbr_gpu_timing_evidence.py",
        "tools/analysis/profiling/shader_pbr/zircon_validate_shader_pbr_renderdoc_replay.py",
        "tools/analysis/profiling/shader_pbr/zircon_validate_shader_pbr_viewer_evidence.py"
    )
}

function Get-ZirconShaderPbrProfileCriticalSourcePaths {
    param([string]$RepoRoot = $script:ZirconShaderPbrProfileContractRoot)

    $viewerSources = @(Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $RepoRoot)
    $crossModuleSources = @(
        "zircon_runtime/crates/zr_rhi_wgpu/src/production/buffer_upload_batch.rs",
        "zircon_editor/src/ui/workbench/state/editor_state_render.rs",
        "zircon_plugins/rendering/features/reflection_probes/editor/src/trigger.rs",
        "zircon_plugins/rendering/features/reflection_probes/runtime/src/consume.rs",
        "zircon_plugins/rendering/features/reflection_probes/runtime/src/execute.rs",
        "zircon_plugins/rendering/features/reflection_probes/runtime/src/request.rs",
        "zircon_runtime/src/asset/artifact/mod.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs",
        "zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/error.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/manifest.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/mod.rs",
        "zircon_runtime/src/asset/ibl_source_cubemap_bundle_manifest/payload_stamp.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_bundle_manifest/wire.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs",
        "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging/bundle_recovery.rs",
        "zircon_runtime/src/asset/assets/scene/lighting.rs",
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
        "zircon_runtime/src/core/framework/render/environment_capture.rs",
        "zircon_runtime/src/core/framework/render/frame_profile.rs",
        "zircon_runtime/src/core/framework/render/frame_extract/post_process.rs",
        "zircon_runtime/src/core/framework/render/frame_extract/world_snapshot_handle.rs",
        "zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs",
        "zircon_runtime/src/core/framework/render/environment/environment_pbr_recipe.rs",
        "zircon_runtime/src/core/framework/render/environment/extract.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs",
        "zircon_runtime/src/core/render/environment/ibl_bake_artifact_resolution.rs",
        "zircon_runtime/src/core/framework/render/environment/ibl_bake_recipe.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/mod.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/ibl_bake_key.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/mode.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/settings.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/mod.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/bake_key.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/constants.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/params.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/resolved_sun.rs",
        "zircon_runtime/src/core/framework/render/skybox/procedural_sky/sun_resolution.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/cubemap/mod.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/cubemap/equality.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/cubemap/identity.rs",
        "zircon_runtime/src/core/framework/render/skybox/cubemap/provenance.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/cubemap/state.rs",
        "zircon_runtime/src/core/framework/render/environment/skybox/cubemap/upload.rs",
        "zircon_runtime/src/core/framework/render/skybox/cubemap/upload_key.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs",
        "zircon_runtime/src/core/framework/render/source_cubemap_upload/artifact.rs",
        "zircon_runtime/src/core/framework/render/source_cubemap_upload/build.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs",
        "zircon_runtime/src/core/framework/render/source_cubemap/projection.rs",
        "zircon_runtime/src/core/framework/render/environment/source_cubemap/rebuild.rs",
        "zircon_runtime/src/core/render/environment/source_irradiance_cubemap.rs",
        "zircon_runtime/src/core/framework/render/material/standard_material.rs",
        "zircon_runtime/src/core/framework/render/material/texture_transform.rs",
        "zircon_runtime/src/core/framework/render/post/color_space.rs",
        "zircon_runtime/src/core/framework/render/post/effects.rs",
        "zircon_runtime/src/core/render/post/effects/color_transform_settings.rs",
        "zircon_runtime/src/core/framework/render/post/effects/style_settings.rs",
        "zircon_runtime/src/core/framework/render/scene_extract/snapshot/aliases.rs",
        "zircon_runtime/src/core/framework/render/snapshot/viewport_packet.rs",
        "zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
        "zircon_runtime/src/core/runtime/diagnostics/product/mesh_queue.rs",
        "zircon_runtime/src/graphics/backend/backend/textures.rs",
        "zircon_runtime/src/graphics/backend/textures/environment_brdf_lut_rg16float.bin",
        "zircon_runtime/src/graphics/backend/backend/textures/payloads.rs",
        "zircon_runtime/src/graphics/backend/backend/textures/resources.rs",
        "zircon_runtime/src/graphics/runtime/framework/extract/update_stats/base_stats.rs",
        "zircon_runtime/src/graphics/runtime/framework/frame_profiler/mesh_submission.rs",
        "zircon_runtime/src/graphics/runtime/framework/environment_capture_scheduler.rs",
        "zircon_runtime/src/graphics/runtime/environment_capture_scheduler/completion.rs",
        "zircon_runtime/src/graphics/environment_capture_scheduler/control_plane.rs",
        "zircon_runtime/src/graphics/runtime/environment_capture_scheduler/report.rs",
        "zircon_runtime/src/core/framework/render/environment/realtime_ibl_status.rs",
        "zircon_runtime/src/core/framework/render/environment/reflection_probe.rs",
        "zircon_runtime/src/core/framework/render/environment/runtime_snapshot.rs",
        "zircon_runtime/src/core/framework/render/environment/mod.rs",
        "zircon_runtime/src/core/framework/render/framework.rs",
        "zircon_runtime/src/core/framework/render/framework_error.rs",
        "zircon_runtime/src/core/framework/render/mod.rs",
        "zircon_runtime/src/graphics/runtime/framework/mod.rs",
        "zircon_runtime/src/graphics/runtime/query_environment_runtime_snapshot/mod.rs",
        "zircon_runtime/src/graphics/query_environment_runtime_snapshot/snapshot.rs",
        "zircon_runtime/src/graphics/runtime/state/environment_capture_residency.rs",
        "zircon_runtime/src/graphics/runtime/state/environment_ibl_hydration_cache.rs",
        "zircon_runtime/src/graphics/runtime/framework/state/state.rs",
        "zircon_runtime/src/graphics/render_framework_trait_binding/wgpu_framework.rs",
        "zircon_runtime/src/graphics/runtime/framework/extract/submission/build.rs",
        "zircon_runtime/src/graphics/submission/environment_ibl_compile_options.rs",
        "zircon_runtime/src/graphics/runtime/extract/frame_submission_context.rs",
        "zircon_runtime/src/graphics/wgpu_render_framework/wgpu_render_framework.rs",
        "zircon_runtime/src/graphics/pipeline/pipeline/resource_descriptors.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_capture_wgpu.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_cpu_timing.rs",
        "zircon_runtime/src/graphics/scene/renderer/shaders/realtime_ibl_capture.wgsl",
        "zircon_runtime/src/graphics/scene/renderer/shaders/skybox_procedural.wgsl",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_gpu_resources.rs",
        "zircon_runtime/src/graphics/execution_resource_cache.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_gpu_timestamps.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_graph_plan.rs",
        "zircon_runtime/src/graphics/scene/tests/realtime_ibl_profile_test_support.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/realtime_ibl_runtime.rs",
        "zircon_runtime/src/graphics/scene/renderer/compiled_graph_cache/cache.rs",
        "zircon_runtime/src/graphics/scene/renderer/compiled_graph_cache/mod.rs",
        "zircon_runtime/src/graphics/scene/renderer/compiled_graph_cache/variant.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_time_slice.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/mod.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/capture_scene_batch.rs",
        "zircon_runtime/src/graphics/scene/environment/realtime_ibl_wgpu_recorder.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/capacity.rs",
        "zircon_runtime/src/graphics/scene/renderer/probe_buffer/gpu_layout.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/mod.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/report.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/resources.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/selection.rs",
        "zircon_runtime/src/graphics/scene/renderer/probe_buffer/slot_allocator.rs",
        "zircon_runtime/src/graphics/scene/renderer/environment/probe_buffer/upload.rs",
        "zircon_runtime/src/graphics/scene/gpu_scene/layout.rs",
        "zircon_runtime/src/graphics/scene/renderer/subsurface_pass/shaders/scatter.wgsl",
        "zircon_runtime/src/graphics/scene/renderer/subsurface_pass/shaders/setup.wgsl",
        "zircon_runtime/src/graphics/scene/deferred/shaders/deferred_lighting.wgsl",
        "zircon_runtime/src/graphics/scene/deferred/shaders/environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/scene/deferred/lighting_pipeline/shader_source.rs",
        "zircon_runtime/src/graphics/scene/renderer/core/core/environment_brdf_lut.rs",
        "zircon_runtime/src/graphics/scene/renderer/core/core/environment_cubemap.rs",
        "zircon_runtime/src/graphics/scene/core/core/environment_cubemap/upload_batch.rs",
        "zircon_runtime/src/graphics/scene/renderer/core/texture_residency.rs",
        "zircon_runtime/src/graphics/scene/renderer/lighting/light_grid_builder.rs",
        "zircon_runtime/src/graphics/scene/renderer/lighting/light_grid_pass.rs",
        "zircon_runtime/src/graphics/scene/renderer/lighting/shaders/zr_light_grid.wgsl",
        "zircon_runtime/src/graphics/scene/gpu_scene/bindless_material_payload.rs",
        "zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/accessors.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/ensure_model.rs",
        "zircon_runtime/src/graphics/scene/resource_streamer/load_model_asset.rs",
        "zircon_runtime/src/graphics/mesh/build/extend_pending_draws_for_mesh_instance.rs",
        "zircon_runtime/src/graphics/mesh_pipeline_cache/material_pipeline_publish.rs",
        "zircon_runtime/src/graphics/mesh/mesh_pipeline/fallback_mesh_shader_source.rs",
        "zircon_runtime/src/graphics/scene/renderer/mesh/shaders/fallback_mesh.wgsl",
        "zircon_runtime/src/graphics/scene/renderer/mesh/shaders/zr_gpu_scene.wgsl",
        "zircon_runtime/src/graphics/scene/overlay/construct/create_sky_pipeline.rs",
        "zircon_runtime/src/graphics/scene/renderer/post/constants/texture_formats.rs",
        "zircon_runtime/src/graphics/post/create_pipeline_bundle/post_process_pipeline.rs",
        "zircon_runtime/src/graphics/scene/renderer/post/execute_output_transfer/mod.rs",
        "zircon_runtime/src/graphics/scene/renderer/post/shaders/output_transfer.wgsl",
        "zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl",
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
        "zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_subsurface.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_procedural_sky.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl",
        "zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl",
        "zircon_runtime/src/graphics/types/viewport_render_frame.rs",
        "zircon_runtime/src/scene/components/scene/lighting.rs",
        "zircon_runtime/src/scene/world/project_io/scene_asset.rs",
        "zircon_runtime/src/scene/world/render.rs",
        "zircon_runtime/src/scene/world/render/lights.rs",
        "zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/resource.rs",
        "zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs",
        "zircon_runtime/src/graphics/scene/resources/resource_streamer/ensure_material.rs",
        "zircon_runtime/src/graphics/scene/renderer/mesh/mesh_pass/commands.rs",
        "zircon_runtime/src/graphics/scene/renderer/mesh/mesh_pipeline_cache/construct.rs",
        "zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
        "zircon_runtime/src/graphics/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
        "zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/shader_source.rs",
        "zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/prewarm_manifest.rs",
        "zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/variant_registry.rs",
        "zircon_runtime/src/graphics/scene/renderer/mesh/prepared_queue/stats.rs",
        "zircon_runtime/src/graphics/scene/renderer/mesh/prepared_queue/stats_bridge.rs",
        "zircon_runtime/src/graphics/scene/renderer/core/renderer/renderer.rs",
        "zircon_runtime/src/graphics/scene/renderer/core/pipeline/pipeline.rs",
        "zircon_runtime/src/graphics/core/write_scene_uniform/write_scene_uniform.rs",
        "zircon_runtime/src/graphics/scene/renderer/overlay/passes/base_scene_pass.rs"
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

    $sessionTool = Join-Path $RepoRoot "tools\dev\zircon-session.ps1"
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

    $sessionTool = Join-Path $RepoRoot "tools\dev\zircon-session.ps1"
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
