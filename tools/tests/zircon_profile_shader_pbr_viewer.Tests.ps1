$script:ProfileScript = Join-Path $PSScriptRoot "..\zircon_profile_shader_pbr_viewer.ps1"
$script:ProfileSource = Get-Content -LiteralPath $script:ProfileScript -Raw
$script:RuntimeEvidenceScript = Join-Path $PSScriptRoot "..\shader-pbr-profile-runtime-evidence.ps1"
$script:RuntimeEvidenceSource = Get-Content -LiteralPath $script:RuntimeEvidenceScript -Raw
$script:EvidenceIdentityScript = Join-Path $PSScriptRoot "..\shader-pbr-profile-evidence-identity.ps1"
$script:EvidenceIdentitySource = Get-Content -LiteralPath $script:EvidenceIdentityScript -Raw
. $script:ProfileScript -ViewerExe "E:\ZirconBuilds\fixture\zircon_shader_pbr_viewer.exe" -HdriPath "E:\fixtures\profile.hdr" -BuildProvenance "E:\ZirconBuilds\fixture\viewer-build-provenance.json" -CaptureToolchainManifest "E:\fixtures\capture-toolchain.json"
$script:ProfileCoordinatorTicket = $null
$script:ProfileCoordinatorArtifactReceipt = $null

function Get-ZirconShaderPbrCoordinatorValidationTicket {
    param(
        [string]$RepoRoot,
        [string]$ValidationTicketId
    )

    if ($null -eq $script:ProfileCoordinatorTicket) {
        throw "No test coordinator validation ticket is configured."
    }
    return $script:ProfileCoordinatorTicket
}

function Get-ZirconShaderPbrCoordinatorArtifactReceipt {
    param(
        [string]$RepoRoot,
        [string]$ArtifactReceiptId
    )

    if ($null -eq $script:ProfileCoordinatorArtifactReceipt) {
        throw "No test coordinator artifact receipt is configured."
    }
    return $script:ProfileCoordinatorArtifactReceipt
}

Describe "zircon shader PBR viewer startup profile contract" {
    It "keeps generated evidence beneath the Shader06 evidence root" {
        $repoRoot = "E:\Git\ZirconEngine"
        $expected = Join-Path $repoRoot "docs\tests\runtime\shader\profile-captures"

        Resolve-ZirconShaderPbrProfileEvidenceRoot -RepoRoot $repoRoot -Path $expected |
            Should Be ([System.IO.Path]::GetFullPath($expected))
        $junctionPath = Join-Path $repoRoot ("docs\tests\runtime\shader\profile-captures-junction-test-" + [guid]::NewGuid().ToString("N"))
        $createdJunction = $false
        try {
            New-Item -ItemType Junction -Path $junctionPath -Target "C:\Windows" | Out-Null
            $createdJunction = $true
            $junctionFailure = $null
            try {
                Resolve-ZirconShaderPbrProfileEvidenceRoot -RepoRoot $repoRoot -Path $junctionPath | Out-Null
            }
            catch {
                $junctionFailure = $_
            }
            $junctionFailure | Should Not BeNullOrEmpty
            $junctionFailure.Exception.Message | Should Match "contains a reparse point"
        }
        finally {
            if ($createdJunction -and (Test-Path -LiteralPath $junctionPath)) {
                # Windows PowerShell's filesystem provider can throw while deleting a junction.
                # Directory.Delete removes the generated link itself, never its C:\Windows target.
                [System.IO.Directory]::Delete($junctionPath)
            }
        }
        foreach ($unsafePath in @(
            "C:\zircon-profiles",
            "E:\Git\ZirconEngine\target\shader-profile",
            "E:\Git\ZirconEngine\docs\tests\runtime\shader\..\outside"
        )) {
            $failure = $null
            try {
                Resolve-ZirconShaderPbrProfileEvidenceRoot -RepoRoot $repoRoot -Path $unsafePath | Out-Null
            }
            catch {
                $failure = $_
            }
            $failure | Should Not BeNullOrEmpty
            $failure.Exception.Message |
                Should Be "Shader PBR profile evidence root must resolve beneath E:\Git\ZirconEngine\docs\tests\runtime\shader."
        }
    }

    It "defines five independent cold and warm measurements" {
        $script:ProfileSource | Should Match '\[int\]\$Repetitions = 5'
        $script:ProfileSource | Should Match "new process and new caller-owned IBL cache directory per measured run"
        $script:ProfileSource | Should Match "one unmeasured cache seed, then new processes reusing its caller-owned IBL cache directory"
        $script:ProfileSource | Should Match 'ExpectedStagingStatus "Written"'
        $script:ProfileSource | Should Match 'ExpectedStagingStatus "Reused"'
        $script:ProfileSource | Should Match '-Role "cache_seed"[\s\S]*-Measure:\$false'
        $script:ProfileSource | Should Match "requires exactly the cold and warm cache modes"
    }

    It "publishes a scoped cache contract without claiming a strict cold start" {
        $script:ProfileSource | Should Match 'cache_layers = \[ordered\]@\{'
        $script:ProfileSource | Should Match 'engine_cache = \[ordered\]@\{'
        $script:ProfileSource | Should Match 'shader_cache = \[ordered\]@\{'
        $script:ProfileSource | Should Match 'os_file_cache = \[ordered\]@\{'
        $script:ProfileSource | Should Match 'driver_cache = \[ordered\]@\{'
        $script:ProfileSource | Should Match 'strict_cold_eligible = \$false'
        $script:ProfileSource | Should Match 'comparison_scope = "process_and_caller_owned_engine_cache"'
    }

    It "binds a complete machine and load manifest before profile capture" {
        $script:ProfileSource | Should Match "performance-machine-manifest\.ps1"
        $script:ProfileSource | Should Match "New-ZirconPerformanceMachineManifest"
        $script:ProfileSource | Should Match 'machine_manifest = \$MachineManifest'
        $script:ProfileSource | Should Match '-MachineManifest \$machineManifest'
    }

    It "binds each sampled run to CPU, energy, PNG, and GPU timing evidence" {
        $script:ProfileSource | Should Match "wpr.exe"
        $script:ProfileSource | Should Match '@\("-start", "CPU", "-filemode"\)'
        $script:ProfileSource | Should Match "Energy Meter"
        $script:ProfileSource | Should Match "PathsWithInstances"
        $script:ProfileSource | Should Match "typeperf.exe"
        $script:ProfileSource | Should Match "--gpu-timing-report"
        $script:ProfileSource | Should Match "zircon_validate_shader_pbr_viewer_evidence.py"
        $script:ProfileSource | Should Match "zircon_validate_shader_pbr_gpu_timing_evidence.py"
        $script:ProfileSource | Should Match "zircon_summarize_shader_pbr_profile.py"
        $script:ProfileSource | Should Match "profile_analysis.json"
        $script:ProfileSource | Should Not Match "--require-direct-present"
    }

    It "binds measured runs to Zircon shader timeline evidence" {
        $script:ProfileSource | Should Match 'shader-pbr-profile-runtime-evidence\.ps1'
        foreach ($anchor in @(
            'ZIRCON_PROFILE_CAPTURE',
            'ZIRCON_PROFILE_SESSION',
            'ZIRCON_PROFILE_OUTPUT_ROOT',
            'ZIRCON_PROFILE_MAX_SPANS',
            'ZIRCON_PROFILE_MAX_COUNTERS',
            'Get-ZirconShaderPbrRuntimeProfileEvidence',
            'runtime_profile = $runtimeProfile'
        )) {
            $script:ProfileSource | Should Match ([regex]::Escape($anchor))
        }
        foreach ($anchor in @(
            'timeline.zrtrace.json',
            'hotspots.json',
            'counter_hotspots.json',
            'summary.md'
        )) {
            $script:RuntimeEvidenceSource | Should Match ([regex]::Escape($anchor))
        }
        $script:ProfileSource | Should Match '\$env:ZIRCON_PROFILE_CAPTURE = \$previousProfileCapture'
        $script:ProfileSource | Should Match '\$env:ZIRCON_PROFILE_SESSION = \$previousProfileSession'
        $script:ProfileSource | Should Match '\$env:ZIRCON_PROFILE_OUTPUT_ROOT = \$previousProfileOutputRoot'
    }

    It "accepts complete runtime profile evidence and rejects overwritten samples" {
        $fixtureRoot = Join-Path "E:\Git\ZirconEngine\docs\tests\runtime\shader" `
            ("runtime-profile-helper-" + [guid]::NewGuid().ToString("N"))
        $exportRoot = Join-Path $fixtureRoot "fixture-export"
        New-Item -ItemType Directory -Force -Path $exportRoot | Out-Null
        try {
            $timeline = [ordered]@{
                session_id = "fixture-session"
                output_root = [System.IO.Path]::GetFullPath($fixtureRoot)
                active = $false
                feature_enabled = $true
                spans = @(
                    [ordered]@{
                        stream = "render"
                        category = "shader_pipeline"
                        name = "mesh_source_build"
                    }
                )
                counters = @([ordered]@{ stream = "render"; name = "mesh_shader_source_bytes"; value = 42 })
                recorder_retention = @(
                    [ordered]@{
                        frames = [ordered]@{ overwritten = 0 }
                        spans = [ordered]@{ overwritten = 0 }
                        counters = [ordered]@{ overwritten = 0 }
                    }
                )
            }
            $timeline | ConvertTo-Json -Depth 8 |
                Set-Content -LiteralPath (Join-Path $exportRoot "timeline.zrtrace.json") -Encoding UTF8
            "{}" | Set-Content -LiteralPath (Join-Path $exportRoot "hotspots.json") -Encoding UTF8
            "{}" | Set-Content -LiteralPath (Join-Path $exportRoot "counter_hotspots.json") -Encoding UTF8
            "# fixture" | Set-Content -LiteralPath (Join-Path $exportRoot "summary.md") -Encoding UTF8

            $evidence = Get-ZirconShaderPbrRuntimeProfileEvidence `
                -ProfileRoot $fixtureRoot `
                -SessionId "fixture-session"
            $evidence.span_count | Should Be 1
            $evidence.counter_count | Should Be 1
            $evidence.shader_pipeline_stage_counts.mesh_source_build | Should Be 1

            $timeline["active"] = $true
            $timeline | ConvertTo-Json -Depth 8 |
                Set-Content -LiteralPath (Join-Path $exportRoot "timeline.zrtrace.json") -Encoding UTF8
            $incompleteRejection = $null
            try {
                Get-ZirconShaderPbrRuntimeProfileEvidence `
                    -ProfileRoot $fixtureRoot `
                    -SessionId "fixture-session" | Out-Null
            }
            catch {
                $incompleteRejection = $_.Exception.Message
            }
            $incompleteRejection | Should Match "not a completed enabled capture"
            $timeline["active"] = $false

            $timeline["recorder_retention"] = @(
                [ordered]@{
                    frames = [ordered]@{ overwritten = 0 }
                    spans = [ordered]@{ overwritten = 1 }
                    counters = [ordered]@{ overwritten = 0 }
                }
            )
            $timeline | ConvertTo-Json -Depth 8 |
                Set-Content -LiteralPath (Join-Path $exportRoot "timeline.zrtrace.json") -Encoding UTF8
            $writtenTimeline = Get-Content -LiteralPath (Join-Path $exportRoot "timeline.zrtrace.json") -Raw |
                ConvertFrom-Json
            $writtenTimeline.recorder_retention[0].spans.overwritten | Should Be 1
            $rejection = $null
            try {
                Get-ZirconShaderPbrRuntimeProfileEvidence `
                    -ProfileRoot $fixtureRoot `
                    -SessionId "fixture-session" | Out-Null
            }
            catch {
                $rejection = $_.Exception.Message
            }
            $rejection | Should Match "lost spans samples"
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "matches coordinator ordinal JSON source manifest hashing" {
        $sourceManifest = [ordered]@{
            "project_assets.rs" = ("a" * 64)
            "project_asset_fixture_validation.rs" = ("b" * 64)
        }

        Get-ZirconShaderPbrValidationSourceManifestHash `
            -SourceManifest $sourceManifest `
            -Description "coordinator canonical hash fixture" |
            Should Be "8077d3e44a4cad290a39ff3e24679c9d6f49d8d34b3cbe36effd896d41bd630a"
    }

    It "binds shader attribution owners into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs",
            "zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs",
            "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/material_pipeline_publication.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            "zircon_runtime/src/graphics/shader/template/assemble.rs",
            "zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs",
            "zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs",
            "zircon_runtime/src/graphics/shader/variant_cache/disk.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds the exact profiling tool implementation into the profile manifest" {
        $profileTools = @(Get-ZirconShaderPbrProfileToolPaths)
        $expectedTools = @(
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

        $profileTools.Count | Should Be $expectedTools.Count
        foreach ($relativePath in $expectedTools) {
            ($profileTools -contains $relativePath) | Should Be $true
        }
        $script:ProfileSource | Should Match 'profile_tool_files = @\(\$profileToolFiles\)'
    }

    It "accepts a display oracle only when explicitly supplied beneath the evidence root" {
        $script:ProfileSource | Should Match '\[string\]\$DisplayVisualOracle = ""'
        $script:ProfileSource | Should Match 'Resolve-ZirconShaderPbrProfileEvidenceRoot[\s\S]*-RepoRoot \$RepoRoot[\s\S]*-Path \$DisplayVisualOracle'
        $script:ProfileSource | Should Match '"--display-visual-oracle", \$DisplayVisualOracle'
        $script:ProfileSource | Should Match 'display_visual_oracle = \$DisplayVisualOracleFingerprint'
        $script:ProfileSource | Should Match '-DisplayVisualOracleFingerprint \$displayVisualOracleFingerprint'
        $script:ProfileSource | Should Match 'display_visual_oracle = \$displayVisualOracleFingerprint'
    }

    It "binds warm-cache HDRI loading code into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs",
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
            "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs",
            "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs",
            "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap/projection.rs",
            "zircon_runtime/src/core/framework/render/environment/source_cubemap/rebuild.rs",
            "zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs",
            "zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs",
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
            "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_environment_generic_api.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_environment_only_pbr.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl",
            "zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds realtime IBL scheduling and capture into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_cpu_timing.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources/execution_resource_cache.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds raw procedural-sky consumers and their source assemblers into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/graphics/shader/wgsl/zr_procedural_sky.wgsl",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/realtime_ibl_capture.wgsl",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl",
            "zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs",
            "zircon_runtime/src/graphics/shader/template/module_registry.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds Standard-PBR material surface inputs into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/graphics/shader/template/material_surface.rs",
            "zircon_runtime/src/graphics/shader/includes/zr_normal.wgsl",
            "zircon_runtime/src/graphics/shader/includes/zr_pbr_common.wgsl",
            "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras_core.wgsl",
            "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds Standard-PBR material CPU and GPU payload owners into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/asset/assets/material/material_asset.rs",
            "zircon_runtime/src/core/framework/render/material/standard_material.rs",
            "zircon_runtime/src/core/framework/render/material/texture_transform.rs",
            "zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
            "zircon_runtime/src/graphics/scene/gpu_scene/bindless_material_payload.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds non-default IOR routing and its queue observation owners into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs",
            "zircon_runtime/src/core/framework/render/backend_types.rs",
            "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs",
            "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
            "zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs",
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "binds frame-matched GPU timing submission counters into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_runtime/src/core/framework/render/frame_profile.rs",
            "zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/mesh_submission.rs",
            "zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs"
        )) {
            @($criticalSources | Where-Object { $_ -eq $relativePath }).Count | Should Be 1
        }
    }

    It "records requested and active PMREM layout with IBL phase evidence" {
        foreach ($field in @(
            "requested_source_face_size",
            "requested_pmrem_face_size",
            "active_source_cubemap_face_size",
            "active_source_cubemap_mip_count",
            "active_pmrem_face_size",
            "active_pmrem_mip_count",
            "ibl_staging_source_decode_ns",
            "ibl_staging_cubemap_build_ns",
            "ibl_staging_equirect_projection_ns",
            "ibl_staging_source_mip_build_ns",
            "ibl_staging_pmrem_build_ns",
            "ibl_staging_sh9_build_ns",
            "ibl_staging_irradiance_cube_build_ns",
            "ibl_staging_bundle_write_ns",
            "ibl_staging_parallel_executor_work_items",
            "ibl_staging_equirect_projection_parallel_work_items",
            "ibl_staging_source_mip_build_parallel_work_items",
            "ibl_staging_pmrem_build_parallel_work_items",
            "ibl_staging_irradiance_cube_build_parallel_work_items"
        )) {
            $script:ProfileSource | Should Match ([regex]::Escape($field))
        }
    }

    It "can profile viewer-default and explicit IBL layout policies" {
        $script:ProfileSource | Should Match '\[Nullable\[int\]\]\$FaceSize'
        $script:ProfileSource | Should Match '\[Nullable\[int\]\]\$PmremFaceSize'
        $script:ProfileSource | Should Match 'if \(\$null -ne \$FaceSize\)'
        $script:ProfileSource | Should Match 'if \(\$null -ne \$PmremFaceSize\)'
        { Assert-ZirconShaderPbrOptionalFaceSize -Value $null -Name "-PmremFaceSize" } |
            Should Not Throw
        { Assert-ZirconShaderPbrOptionalFaceSize -Value 128 -Name "-PmremFaceSize" } |
            Should Not Throw
        $invalidSizeFailure = $null
        try {
            Assert-ZirconShaderPbrOptionalFaceSize -Value 96 -Name "-PmremFaceSize"
        }
        catch {
            $invalidSizeFailure = $_
        }
        $invalidSizeFailure | Should Not BeNullOrEmpty
        $invalidSizeFailure.Exception.Message |
            Should Match "must be 64, 128, 256, 512, or 1024"
    }

    It "binds the material fixture to each viewer command and profile artifact" {
        $script:ProfileSource | Should Match '\[ValidateSet\("metal-mirror", "dielectric-ior"\)\]'
        $script:ProfileSource | Should Match 'material_fixture = \$MaterialFixture'
        $script:ProfileSource | Should Match '"--material-fixture", \$MaterialFixture'
        $script:ProfileSource | Should Match 'expected material_fixture=\$MaterialFixture'
        foreach ($field in @(
            "material_fixture",
            "required_material_base_pipeline_kind",
            "required_material_base_pipeline_ready_at_capture",
            "environment_only_base_prewarm_requested"
        )) {
            $script:ProfileSource | Should Match $field
        }
    }

    It "does not claim a driver cache reset or infer watts when the meter is absent" {
        $script:ProfileSource | Should Match "It does not clear DX12 or driver caches"
        $script:ProfileSource | Should Match "diagnostic only because -SkipWpr omits required CPU attribution"
        $script:ProfileSource | Should Match 'status = "unavailable"'
        $script:ProfileSource | Should Match 'unit = "watts"'
        Get-ZirconShaderPbrEnergyMeterCaptureStatus -TerminatedByProfiler $false -TypeperfExitCode 1 -HasRequiredRows $true |
            Should Be "failed"
        Get-ZirconShaderPbrEnergyMeterCaptureStatus -TerminatedByProfiler $true -TypeperfExitCode 1 -HasRequiredRows $true |
            Should Be "captured"
        $script:ProfileSource | Should Not Match "estimated_watts"
        ($script:ProfileSource.IndexOf("`$wprCapture = Start-ZirconShaderPbrWprCapture") -lt
            $script:ProfileSource.IndexOf("`$energyCapture = Start-ZirconShaderPbrEnergyMeterCapture")) |
            Should Be $true
        ($script:ProfileSource.IndexOf("`$energyReport = Stop-ZirconShaderPbrEnergyMeterCapture") -lt
            $script:ProfileSource.IndexOf("`$wprFingerprint = Stop-ZirconShaderPbrWprCapture")) |
            Should Be $true
    }

    It "makes RenderDoc replay optional and source-bound" {
        $script:ProfileSource | Should Match '\[switch\]\$CaptureRenderDoc'
        $script:ProfileSource | Should Match "CaptureToolchainManifest"
        $script:ProfileSource | Should Match "Resolve-ZirconShaderPbrCaptureToolchain"
        $script:ProfileSource | Should Match "CaptureToolchain\.renderdoc\.dll\.path"
        $script:ProfileSource | Should Match "CaptureToolchain\.renderdoc\.command\.path"
        $script:ProfileSource | Should Match "CaptureToolchain\.graphics\.wgpu_backend"
        $script:ProfileSource | Should Match "CaptureToolchain\.graphics\.evidence_backend"
        $script:ProfileSource | Should Not Match 'D:\\Tools\\renderdoc'
        $script:ProfileSource | Should Match "zircon_validate_shader_pbr_renderdoc_replay.py"
        $script:ProfileSource | Should Match '"--renderdoccmd"'
        $script:ProfileSource | Should Match 'renderdoc_replay = \$renderdocReplay'
        $script:ProfileSource | Should Match "Get-ZirconProfileGitMetadata"
        $script:ProfileSource | Should Match '\[string\]\$BuildProvenance'
        $script:ProfileSource | Should Match "zircon_managed_viewer_artifact_provenance"
        $script:ProfileSource | Should Match "does not bind the requested viewer binary"
        $script:ProfileSource | Should Match "FileAttributes]::ReparsePoint"
        $script:ProfileSource | Should Match "WprTimeoutSeconds"
        $script:ProfileSource | Should Match "-cancel"
    }

    It "binds each Ready frame to a source-bound identity manifest" {
        $script:ProfileSource | Should Match "shader-pbr-profile-evidence-identity.ps1"
        $script:ProfileSource | Should Match "New-ZirconShaderPbrReadyFrameEvidenceIdentity"
        $script:ProfileSource | Should Match '"--evidence-identity", \$evidenceIdentity.path'
        $script:ProfileSource | Should Match 'evidence_identity = \$evidenceIdentity'
        $script:ProfileSource | Should Match 'profile_id = \$profileId'
        $script:ProfileSource | Should Match 'source_manifest_sha256 = \$sidecar'
        $script:ProfileSource | Should Match 'viewer_binary_sha256 = \$sidecar'
        $script:ProfileSource | Should Match 'build_provenance_sha256 = \$sidecar'
        $script:EvidenceIdentitySource |
            Should Match 'function ConvertTo-ZirconShaderPbrIdentityFileFingerprint'
        $script:EvidenceIdentitySource |
            Should Match 'byte_length = \[int64\]\$Fingerprint.byte_length'
        $script:EvidenceIdentitySource |
            Should Match 'viewer_binary = \$viewer'
        $script:EvidenceIdentitySource | Should Not Match 'last_write_utc'
    }

    It "serializes only stable content fields into identity file fingerprints" {
        . $script:EvidenceIdentityScript
        $identityFingerprint = ConvertTo-ZirconShaderPbrIdentityFileFingerprint -Fingerprint ([pscustomobject]@{
            path = "E:\profile\viewer.exe"
            sha256 = "a" * 64
            byte_length = [int64]42
            last_write_utc = "2026-08-25T00:00:00.0000000Z"
        })

        ($identityFingerprint.Keys -join ",") | Should Be "path,sha256,byte_length"
        $identityFingerprint.path | Should Be "E:\profile\viewer.exe"
        $identityFingerprint.sha256 | Should Be ("a" * 64)
        [int64]$identityFingerprint.byte_length | Should Be ([int64]42)
    }

    It "requires the coordinator response to match the requested ticket id" {
        $repoRoot = "E:\Git\ZirconEngine"
        $fixtureRoot = Join-Path $repoRoot ("docs\tests\runtime\shader\ticket-id-contract-" + [guid]::NewGuid().ToString("N"))
        $sessionTool = Join-Path $fixtureRoot "tools\zircon-session.ps1"
        try {
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $sessionTool) | Out-Null
            @'
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

'{"ticket":{"ticket_id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","status":"passed"}}'
$global:LASTEXITCODE = 0
'@ | Set-Content -LiteralPath $sessionTool -Encoding UTF8
            $ticketFailure = $null
            try {
                & {
                    . (Join-Path $repoRoot "tools\shader-pbr-profile-contract.ps1")
                    Get-ZirconShaderPbrCoordinatorValidationTicket `
                        -RepoRoot $fixtureRoot `
                        -ValidationTicketId ("a" * 32) | Out-Null
                }
            }
            catch {
                $ticketFailure = $_
            }
            $ticketFailure | Should Not BeNullOrEmpty
            $ticketFailure.Exception.Message |
                Should Match "returned a ticket different from requested validation ticket"
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "requires a terminal managed artifact receipt for the exact viewer binary" {
        $repoRoot = "E:\Git\ZirconEngine"
        $contractRoot = Join-Path $repoRoot ("docs\tests\runtime\shader\build-provenance-contract-" + [guid]::NewGuid().ToString("N"))
        $managedRoot = Join-Path $contractRoot "managed-artifacts\receipt"
        $viewerPath = Join-Path $managedRoot "zircon_shader_pbr_viewer.exe"
        $provenancePath = Join-Path $contractRoot "viewer-build-provenance.json"
        $writer = Join-Path $repoRoot "tools\write_zircon_shader_pbr_build_provenance.ps1"
        try {
            New-Item -ItemType Directory -Force -Path $managedRoot | Out-Null
            [System.IO.File]::WriteAllBytes($viewerPath, [byte[]](1, 2, 3, 4))
            $viewerFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
                -Path $viewerPath `
                -Description "fixture viewer binary"
            $sourceFiles = Get-ZirconShaderPbrProfileCriticalSourcePaths | ForEach-Object {
                $sourceFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
                    -Path (Join-Path $repoRoot $_) `
                    -Description "fixture critical source '$_'"
                [pscustomobject]@{
                    relative_path = $_
                    sha256 = $sourceFingerprint.sha256
                }
            }
            $sourceManifest = [ordered]@{}
            foreach ($sourceFile in $sourceFiles) {
                $sourceManifest[$sourceFile.relative_path] = $sourceFile.sha256
            }
            $script:ProfileCoordinatorTicket = [pscustomobject]@{
                ticket_id = ("a" * 32)
                status = "passed"
                source_manifest = [pscustomobject]$sourceManifest
                source_manifest_hash = Get-ZirconShaderPbrValidationSourceManifestHash `
                    -SourceManifest $sourceManifest `
                    -Description "fixture coordinator validation ticket"
            }
            $originalCoordinatorTicket = $script:ProfileCoordinatorTicket
            $sourceValidationTicket = Assert-ZirconShaderPbrCoordinatorValidationTicket `
                -Ticket $script:ProfileCoordinatorTicket `
                -SourceFiles $sourceFiles `
                -Description "fixture coordinator validation ticket"
            $script:ProfileCoordinatorArtifactReceipt = [pscustomobject]@{
                receiptId = ("f" * 32)
                sessionId = "shader06"
                jobId = ("c" * 32)
                validationTicketId = ("a" * 32)
                artifactKind = "shader-pbr-viewer"
                status = "passed"
                inputManifestHash = ("d" * 64)
                sourceManifestHash = $sourceValidationTicket.source_manifest_hash
                runId = ("e" * 32)
                targetRelativePath = "release/zircon_shader_pbr_viewer.exe"
                artifactPath = $viewerPath
                sha256 = $viewerFingerprint.sha256
                byteLength = $viewerFingerprint.byte_length
                command = @("cargo", "+1.94.1", "build", "-p", "zircon_app", "--bin", "zircon_shader_pbr_viewer", "--locked", "--release")
                commandSha256 = ("9" * 64)
                errorCode = $null
            }
            $originalReceipt = $script:ProfileCoordinatorArtifactReceipt
            . $writer `
                -ViewerExe $viewerPath `
                -OutputPath $provenancePath `
                -ValidationTicketId ("a" * 32) `
                -ArtifactReceiptId ("f" * 32) | Out-Null
            function Get-ZirconShaderPbrCoordinatorValidationTicket {
                param(
                    [string]$RepoRoot,
                    [string]$ValidationTicketId
                )

                return $script:ProfileCoordinatorTicket
            }
            function Get-ZirconShaderPbrCoordinatorArtifactReceipt {
                param(
                    [string]$RepoRoot,
                    [string]$ArtifactReceiptId
                )

                return $script:ProfileCoordinatorArtifactReceipt
            }
            Write-ZirconShaderPbrBuildProvenance | Out-Null
            $provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json

            $writerSource = Get-Content -LiteralPath $writer -Raw
            $writerSource | Should Match '\[string\]\$ValidationTicketId'
            $writerSource | Should Match '\[string\]\$ArtifactReceiptId'
            $writerSource | Should Match 'Get-ZirconShaderPbrCoordinatorValidationTicket'
            $writerSource | Should Match 'Get-ZirconShaderPbrCoordinatorArtifactReceipt'
            $writerSource | Should Not Match 'ManagedValidationRunId'
            $writerSource | Should Not Match 'ManagedSourceManifestHash'
            $writerSource | Should Not Match 'last_write_utc'
            $provenance.schema_version | Should Be 2
            $provenance.provenance_kind | Should Be "zircon_managed_viewer_artifact_provenance"
            $provenance.binary.path | Should Be $viewerPath
            $provenance.artifact_receipt.artifact_receipt_id | Should Be ("f" * 32)
            $provenance.artifact_receipt.job_id | Should Be ("c" * 32)
            $provenance.artifact_receipt.run_id | Should Be ("e" * 32)
            $provenance.source_validation_ticket.validation_ticket_id | Should Be ("a" * 32)
            $provenance.source_validation_ticket.status | Should Be "passed"
            $provenance.source_validation_ticket.source_manifest_hash |
                Should Be $script:ProfileCoordinatorTicket.source_manifest_hash
            @($provenance.repository.source_manifest.PSObject.Properties).Count |
                Should Be @($sourceFiles).Count

            Assert-ZirconShaderPbrBuildProvenance `
                -Path $provenancePath `
                -ViewerFingerprint $viewerFingerprint `
                -SourceFiles $sourceFiles | Should Not BeNullOrEmpty

            $script:ProfileCoordinatorTicket = [pscustomobject]@{
                ticket_id = ("b" * 32)
                status = "passed"
                source_manifest = [pscustomobject]$sourceManifest
                source_manifest_hash = $originalCoordinatorTicket.source_manifest_hash
            }
            $substitutedTicketFailure = $null
            try {
                Assert-ZirconShaderPbrBuildProvenance `
                    -Path $provenancePath `
                    -ViewerFingerprint $viewerFingerprint `
                    -SourceFiles $sourceFiles | Out-Null
            }
            catch {
                $substitutedTicketFailure = $_
            }
            $substitutedTicketFailure | Should Not BeNullOrEmpty
            $substitutedTicketFailure.Exception.Message |
                Should Match "does not match its coordinator ticket id"
            $script:ProfileCoordinatorTicket = $originalCoordinatorTicket

            $copiedViewer = Join-Path $contractRoot "timestamp-advanced-copy.exe"
            Copy-Item -LiteralPath $viewerPath -Destination $copiedViewer
            (Get-Item -LiteralPath $copiedViewer).LastWriteTimeUtc = (Get-Date).ToUniversalTime().AddDays(1)
            $copiedFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
                -Path $copiedViewer `
                -Description "timestamp-advanced copied viewer binary"
            $copiedBinaryFailure = $null
            try {
                Assert-ZirconShaderPbrBuildProvenance `
                    -Path $provenancePath `
                    -ViewerFingerprint $copiedFingerprint `
                    -SourceFiles $sourceFiles | Out-Null
            }
            catch {
                $copiedBinaryFailure = $_
            }
            $copiedBinaryFailure | Should Not BeNullOrEmpty
            $copiedBinaryFailure.Exception.Message |
                Should Match "does not bind the requested viewer binary"

            $script:ProfileCoordinatorArtifactReceipt = $originalReceipt.PSObject.Copy()
            $script:ProfileCoordinatorArtifactReceipt.status = "pending"
            $nonterminalFailure = $null
            try {
                Write-ZirconShaderPbrBuildProvenance | Out-Null
            }
            catch {
                $nonterminalFailure = $_
            }
            $nonterminalFailure | Should Not BeNullOrEmpty
            $nonterminalFailure.Exception.Message | Should Match "terminal passed"

            $script:ProfileCoordinatorArtifactReceipt = $originalReceipt.PSObject.Copy()
            $script:ProfileCoordinatorArtifactReceipt.sourceManifestHash = ("b" * 64)
            $manifestFailure = $null
            try {
                Write-ZirconShaderPbrBuildProvenance | Out-Null
            }
            catch {
                $manifestFailure = $_
            }
            $manifestFailure | Should Not BeNullOrEmpty
            $manifestFailure.Exception.Message | Should Match "source manifest"

            $script:ProfileCoordinatorArtifactReceipt = $originalReceipt.PSObject.Copy()
            $script:ProfileCoordinatorArtifactReceipt.sha256 = ("b" * 64)
            $hashFailure = $null
            try {
                Write-ZirconShaderPbrBuildProvenance | Out-Null
            }
            catch {
                $hashFailure = $_
            }
            $hashFailure | Should Not BeNullOrEmpty
            $hashFailure.Exception.Message | Should Match "fingerprint"
        }
        finally {
            $script:ProfileCoordinatorTicket = $null
            $script:ProfileCoordinatorArtifactReceipt = $null
            if (Test-Path -LiteralPath $contractRoot) {
                Remove-Item -LiteralPath $contractRoot -Recurse -Force
            }
        }
    }

    It "publishes the PBR matrix only after staging summary validation succeeds" {
        $script:ProfileSource | Should Match "shader-pbr-profile-publication\.ps1"
        $stagingIndex = $script:ProfileSource.IndexOf("New-ZirconShaderPbrProfileStagingRoot")
        $summaryIndex = $script:ProfileSource.IndexOf('"profile_summary.json"')
        $analysisValidationIndex = $script:ProfileSource.IndexOf("profile_analysis_validation.log")
        $completionIndex = $script:ProfileSource.IndexOf("Publish-ZirconShaderPbrProfileCompletion")
        $incompleteIndex = $script:ProfileSource.IndexOf("Write-ZirconShaderPbrProfileIncompleteReceipt")

        ($stagingIndex -ge 0) | Should Be $true
        ($summaryIndex -gt $stagingIndex) | Should Be $true
        ($analysisValidationIndex -gt $summaryIndex) | Should Be $true
        ($completionIndex -gt $analysisValidationIndex) | Should Be $true
        ($incompleteIndex -gt $completionIndex) | Should Be $true
    }

    It "owns the staging profile lease through publication and terminal cleanup" {
        $stagingIndex = $script:ProfileSource.IndexOf("New-ZirconShaderPbrProfileStagingRoot")
        $leaseIndex = $script:ProfileSource.IndexOf("New-ZirconShaderPbrProfileRunLease")
        $heartbeatIndex = $script:ProfileSource.IndexOf("Update-ZirconShaderPbrProfileRunLeaseHeartbeat")
        $completionIndex = $script:ProfileSource.IndexOf("Publish-ZirconShaderPbrProfileCompletion")
        $commitIndex = $script:ProfileSource.IndexOf("Complete-ZirconShaderPbrProfileRunLease")
        $failureIndex = $script:ProfileSource.IndexOf("Fail-ZirconShaderPbrProfileRunLease")
        $closeIndex = $script:ProfileSource.IndexOf("Close-ZirconShaderPbrProfileRunLease")

        ($leaseIndex -gt $stagingIndex) | Should Be $true
        ($heartbeatIndex -gt $leaseIndex) | Should Be $true
        ($completionIndex -gt $heartbeatIndex) | Should Be $true
        ($commitIndex -gt $completionIndex) | Should Be $true
        ($failureIndex -gt $commitIndex) | Should Be $true
        ($closeIndex -gt $failureIndex) | Should Be $true
        $script:ProfileSource | Should Match 'Invoke-ZirconShaderPbrProfileStaleRunScavenger'
        $script:ProfileSource | Should Match '\$null -eq \$completionReceiptPath'
    }

    It "writes a profile tool closure accepted by the Python consumer" {
        $fixtureRoot = Join-Path "E:\Git\ZirconEngine\target\codex-temp" `
            ("profile-writer-integration-" + [guid]::NewGuid().ToString("N"))
        $profileRoot = Join-Path $fixtureRoot "profile"
        $viewerPath = Join-Path $fixtureRoot "viewer.exe"
        $hdriPath = Join-Path $fixtureRoot "input.hdr"
        $provenancePath = Join-Path $fixtureRoot "provenance.json"
        $previousPythonBytecode = $env:PYTHONDONTWRITEBYTECODE
        New-Item -ItemType Directory -Force -Path $profileRoot | Out-Null
        Set-Content -LiteralPath $viewerPath -Value "viewer" -Encoding ASCII
        Set-Content -LiteralPath $hdriPath -Value "hdri" -Encoding ASCII
        Set-Content -LiteralPath $provenancePath -Value "{}" -Encoding ASCII
        Mock Get-ZirconProfileGitMetadata {
            [pscustomobject]@{ revision = "fixture"; dirty = $true }
        }
        Mock Assert-ZirconShaderPbrBuildProvenance {
            Get-ZirconShaderPbrProfileFileFingerprint `
                -Path $Path `
                -Description "fixture build provenance"
        }

        try {
            $manifestPath = Export-ZirconShaderPbrProfileManifest `
                -ProfileRoot $profileRoot `
                -ViewerExe $viewerPath `
                -HdriPath $hdriPath `
                -BuildProvenance $provenancePath `
                -EvidenceRoot $profileRoot `
                -Repetitions 5 `
                -FaceSize $null `
                -PmremFaceSize $null `
                -MaterialFixture "metal-mirror" `
                -CacheModes @("cold", "warm") `
                -CaptureToolchain ([pscustomobject]@{
                    manifest = [pscustomobject]@{ path = $provenancePath }
                    graphics = [pscustomobject]@{}
                    renderdoc = $null
                }) `
                -MachineManifest ([pscustomobject]@{
                    schema_version = 1
                    machine_id = "fixture"
                })
            $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
            @($manifest.repository.profile_tool_files).Count | Should Be 16

            $env:PYTHONDONTWRITEBYTECODE = "1"
            $validator = 'import json,sys; from pathlib import Path; from tools.zircon_shader_pbr_profile_tool_identity import validate_profile_tool_files; p=Path(sys.argv[1]); m=json.loads(p.read_text(encoding="utf-8-sig")); validate_profile_tool_files(m["repository"], Path(m["repository"]["root"]).resolve(), p)'
            $acceptOutput = @(& python -c $validator $manifestPath 2>&1)
            $acceptExitCode = $LASTEXITCODE
            $acceptExitCode | Should Be 0
            ($acceptOutput -join "`n") | Should Be ""

            $manifest.repository.profile_tool_files[0].sha256 = "0" * 64
            $tamperedPath = Join-Path $profileRoot "profile_manifest_tampered.json"
            $manifest | ConvertTo-Json -Depth 8 |
                Set-Content -LiteralPath $tamperedPath -Encoding UTF8
            $tamperOutput = @(& python -c $validator $tamperedPath 2>&1)
            $tamperExitCode = $LASTEXITCODE
            $tamperExitCode | Should Not Be 0
            ($tamperOutput -join "`n") | Should Match "profile tool SHA-256 changed"
        }
        finally {
            $env:PYTHONDONTWRITEBYTECODE = $previousPythonBytecode
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }
}
