$script:ProfileScript = Join-Path $PSScriptRoot "..\zircon_profile_shader_pbr_viewer.ps1"
$script:ProfileSource = Get-Content -LiteralPath $script:ProfileScript -Raw
. $script:ProfileScript -ViewerExe "E:\ZirconBuilds\fixture\zircon_shader_pbr_viewer.exe" -HdriPath "E:\fixtures\profile.hdr" -BuildProvenance "E:\ZirconBuilds\fixture\viewer-build-provenance.json"
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
                Remove-Item -LiteralPath $junctionPath -Force
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

    It "binds warm-cache HDRI loading code into viewer provenance" {
        $criticalSources = @(Get-ZirconShaderPbrProfileCriticalSourcePaths)

        foreach ($relativePath in @(
            "zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs",
            "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs",
            "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs",
            "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs",
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
            "zircon_runtime/src/core/framework/render/environment/skybox.rs",
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
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs"
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
        $script:ProfileSource | Should Match "renderdoc.dll"
        $script:ProfileSource | Should Match "zircon_validate_shader_pbr_renderdoc_replay.py"
        $script:ProfileSource | Should Match "Get-ZirconProfileGitMetadata"
        $script:ProfileSource | Should Match '\[string\]\$BuildProvenance'
        $script:ProfileSource | Should Match "zircon_managed_viewer_artifact_provenance"
        $script:ProfileSource | Should Match "does not bind the requested viewer binary"
        $script:ProfileSource | Should Match "FileAttributes]::ReparsePoint"
        $script:ProfileSource | Should Match "WprTimeoutSeconds"
        $script:ProfileSource | Should Match "-cancel"
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
}
