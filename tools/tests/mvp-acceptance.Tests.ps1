Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$driver = Join-Path $PSScriptRoot '..\mvp\Invoke-MvpAcceptance.ps1'

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$workflowPath = Join-Path $PSScriptRoot '..\..\.github\workflows\mvp-editor-windows.yml'
$workflowSource = Get-Content -LiteralPath $workflowPath -Raw
Assert-True `
    ($workflowSource -match 'Copy-Item -LiteralPath \$evidenceRoot -Destination \$artifactRoot -Recurse -Force') `
    'Windows MVP workflow must upload the complete detached EvidenceRoot instead of a partial staging projection.'

function Write-FixtureJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    [IO.File]::WriteAllText(
        $Path,
        ($Value | ConvertTo-Json -Depth 8),
        [Text.UTF8Encoding]::new($false)
    )
}

function Copy-FixtureProductRuns {
    param([Parameter(Mandatory)]$Runs)

    $json = ConvertTo-Json -InputObject $Runs -Depth 12
    $decoded = $json | ConvertFrom-Json
    return $decoded
}

function Get-FixtureFileEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$RelativePath
    )

    return [ordered]@{
        path = $RelativePath.Replace('\', '/')
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
        size_bytes = (Get-Item -LiteralPath $Path).Length
    }
}

function New-FixtureAutomationProcessEvidence {
    param(
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][string]$RequestPath,
        [Parameter(Mandatory)][string]$RequestRelativePath,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$EvidenceLabel
    )

    $normalizedReport = $Report | ConvertTo-Json -Depth 16 | ConvertFrom-Json
    $logsRoot = Join-Path $StagingRoot 'logs'
    $diagnosticsRoot = Join-Path $logsRoot "$EvidenceLabel.diagnostics"
    New-Item -ItemType Directory -Force -Path $logsRoot, $diagnosticsRoot | Out-Null
    $stdoutPath = Join-Path $logsRoot "$EvidenceLabel.stdout.log"
    $stderrPath = Join-Path $logsRoot "$EvidenceLabel.stderr.log"
    $diagnosticPath = Join-Path $diagnosticsRoot 'fixture.log'
    [IO.File]::WriteAllText($stdoutPath, ($normalizedReport | ConvertTo-Json -Depth 16), [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($stderrPath, "$EvidenceLabel stderr`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($diagnosticPath, "$EvidenceLabel diagnostics`n", [Text.UTF8Encoding]::new($false))
    $normalizedReport | Add-Member -NotePropertyName 'automation_request' -NotePropertyValue (Get-FixtureFileEvidence -Path $RequestPath -RelativePath $RequestRelativePath)
    $normalizedReport | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-FixtureFileEvidence -Path $stdoutPath -RelativePath "logs/$EvidenceLabel.stdout.log")
    $normalizedReport | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-FixtureFileEvidence -Path $stderrPath -RelativePath "logs/$EvidenceLabel.stderr.log")
    $normalizedReport | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @(
        Get-FixtureFileEvidence -Path $diagnosticPath -RelativePath "logs/$EvidenceLabel.diagnostics/fixture.log"
    )
    $normalizedReport | Add-Member -NotePropertyName 'exit_code' -NotePropertyValue 0
    return $normalizedReport
}

function Write-FixtureVisiblePng {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$AfterAuthoring
    )

    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new(16, 16)
    try {
        for ($y = 0; $y -lt 16; $y++) {
            for ($x = 0; $x -lt 16; $x++) {
                $bitmap.SetPixel($x, $y, $(if ($x -lt 8) {
                    [Drawing.Color]::Black
                }
                elseif ($AfterAuthoring) {
                    [Drawing.Color]::FromArgb(255, 48, 192, 112)
                }
                else {
                    [Drawing.Color]::FromArgb(255, 64, 128, 255)
                }))
            }
        }
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

function Write-FixtureBlankPng {
    param([Parameter(Mandatory)][string]$Path)

    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new(16, 16)
    try {
        for ($y = 0; $y -lt 16; $y++) {
            for ($x = 0; $x -lt 16; $x++) {
                $bitmap.SetPixel($x, $y, [Drawing.Color]::Black)
            }
        }
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

$fixtureRoot = Join-Path ([IO.Path]::GetTempPath()) (
    'zircon_mvp_acceptance_' + [guid]::NewGuid().ToString('N')
)

try {
    $stagingRoot = Join-Path $fixtureRoot 'staging'
    $evidenceRoot = Join-Path $fixtureRoot 'evidence'
    New-Item -ItemType Directory -Force -Path (Join-Path $stagingRoot 'project') | Out-Null
    $projectManifestPath = Join-Path $stagingRoot 'project\zircon-project.toml'
    [IO.File]::WriteAllText($projectManifestPath, "name = 'Fixture'`n", [Text.UTF8Encoding]::new($false))

    $stagingManifestEntry = [ordered]@{
        logical_id = 'project/zircon-project.toml'
        target_relative_path = 'project/zircon-project.toml'
        sha256 = (Get-FileHash -LiteralPath $projectManifestPath -Algorithm SHA256).Hash
        size_bytes = (Get-Item -LiteralPath $projectManifestPath).Length
    }
    $stagingManifestFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        toolchain = 'rustc 1.89.0 (fixture)'
        target = 'x86_64-pc-windows-msvc'
        staged_at_utc = '2026-07-29T00:00:00Z'
        entries = @($stagingManifestEntry)
    }
    $authoringRequestPath = Join-Path $stagingRoot 'authoring\automation.json'
    $reopenRequestPath = Join-Path $stagingRoot 'reopen\automation.json'
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $authoringRequestPath), (Split-Path -Parent $reopenRequestPath) | Out-Null
    [IO.File]::WriteAllText($authoringRequestPath, '{"bindings":["authoring"]}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($reopenRequestPath, '{"bindings":["reopen"]}', [Text.UTF8Encoding]::new($false))
    $stagingManifestFixture.entries += @(
        [ordered]@{
            logical_id = 'authoring-automation-request'
            target_relative_path = 'authoring/automation.json'
            sha256 = (Get-FileHash -LiteralPath $authoringRequestPath -Algorithm SHA256).Hash
            size_bytes = (Get-Item -LiteralPath $authoringRequestPath).Length
        },
        [ordered]@{
            logical_id = 'reopen-automation-request'
            target_relative_path = 'reopen/automation.json'
            sha256 = (Get-FileHash -LiteralPath $reopenRequestPath -Algorithm SHA256).Hash
            size_bytes = (Get-Item -LiteralPath $reopenRequestPath).Length
        }
    )
    $stagingManifestPath = Join-Path $stagingRoot 'staging-manifest.json'
    Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            },
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 2
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 2
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 2
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    })

    $result = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $evidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -Json
    ) | ConvertFrom-Json

    Assert-True ($result.run_id -eq 'fixture-stage') 'Acceptance output lost the staging run identity.'
    Assert-True ($result.source_fingerprint -eq 'fixture-source-fingerprint') 'Acceptance output lost the source fingerprint.'
    Assert-True ($result.render_backend -eq 'fixture-wgpu') 'Acceptance output lost the stable runtime render backend.'
    Assert-True ($result.render_adapter -eq 'Fixture WGPU Adapter') 'Acceptance output lost the stable runtime render adapter.'
    Assert-True ($result.render_adapter_type -eq 'discrete_gpu') 'Acceptance output lost the stable runtime adapter type.'
    Assert-True ($result.render_device_limits.max_bind_groups -eq 5) 'Acceptance output lost negotiated device-limit evidence.'
    Assert-True ($result.staged_project_root -eq 'project') 'Acceptance output lost the canonical staged project root.'
    Assert-True ($result.staging_manifest_sha256 -match '^[0-9A-F]{64}$') 'Acceptance output did not bind the staging manifest hash.'
    Assert-True ($result.startup_summary_sha256 -match '^[0-9A-F]{64}$') 'Acceptance output did not bind the startup summary hash.'
    Assert-True ($result.product_runs.Count -eq 4) 'Acceptance output lost repeated staged product evidence.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'manifest.json') -PathType Leaf) 'Acceptance output did not write manifest.json.'

    $manifest = Get-Content -Raw (Join-Path $evidenceRoot 'manifest.json') | ConvertFrom-Json
    Assert-True ($manifest.run_id -eq 'fixture-stage') 'Evidence manifest lost the staging run identity.'
    Assert-True ($manifest.source_fingerprint -eq 'fixture-source-fingerprint') 'Evidence manifest lost the source fingerprint.'
    Assert-True ($manifest.toolchain -eq 'rustc 1.89.0 (fixture)') 'Evidence manifest lost the staged Rust toolchain.'
    Assert-True ($manifest.target -eq 'x86_64-pc-windows-msvc') 'Evidence manifest lost the staged Rust target.'
    Assert-True ($manifest.render_backend -eq 'fixture-wgpu') 'Evidence manifest lost the stable runtime render backend.'
    Assert-True ($manifest.render_adapter -eq 'Fixture WGPU Adapter') 'Evidence manifest lost the stable runtime render adapter.'
    Assert-True ($manifest.render_adapter_type -eq 'discrete_gpu') 'Evidence manifest lost the stable runtime adapter type.'
    Assert-True ($manifest.render_device_limits.max_storage_buffer_binding_size -eq 134217728) 'Evidence manifest lost negotiated device-limit evidence.'
    Assert-True ($manifest.staging_manifest_sha256 -eq $result.staging_manifest_sha256) 'Evidence manifest is not bound to the staging manifest hash.'
    Assert-True ($manifest.startup_summary_sha256 -eq $result.startup_summary_sha256) 'Evidence manifest is not bound to the startup summary hash.'
    Assert-True ($manifest.evidence_layout_version -eq 1) 'Evidence manifest did not declare its self-contained layout.'
    Assert-True ($manifest.staging_manifest -eq 'staging-manifest.json') 'Evidence manifest did not retain a local staging manifest path.'
    Assert-True ($manifest.startup_summary -eq 'startup-summary.json') 'Evidence manifest did not retain a local startup summary path.'
    Assert-True (@($manifest.evidence_files).Count -ge 3) 'Evidence manifest did not inventory its copied source evidence.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'staging-manifest.json') -PathType Leaf) 'Evidence package did not copy its staging manifest.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'startup-summary.json') -PathType Leaf) 'Evidence package did not copy its startup summary.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'project/zircon-project.toml') -PathType Leaf) 'Evidence package did not copy its canonical project.'

    $detachedStagingRoot = Join-Path $fixtureRoot 'staging-detached'
    Move-Item -LiteralPath $stagingRoot -Destination $detachedStagingRoot
    try {
        foreach ($evidenceFile in @($manifest.evidence_files)) {
            $evidencePath = Join-Path $evidenceRoot ([string]$evidenceFile.path)
            Assert-True (Test-Path -LiteralPath $evidencePath -PathType Leaf) "Detached evidence file '$($evidenceFile.path)' is missing."
            Assert-True ((Get-FileHash -LiteralPath $evidencePath -Algorithm SHA256).Hash -eq $evidenceFile.sha256) "Detached evidence file '$($evidenceFile.path)' has a hash mismatch."
            Assert-True ((Get-Item -LiteralPath $evidencePath).Length -eq $evidenceFile.size_bytes) "Detached evidence file '$($evidenceFile.path)' has a size mismatch."
        }
    }
    finally {
        Move-Item -LiteralPath $detachedStagingRoot -Destination $stagingRoot
    }

    $f0StartupProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($runtimeRun in @($f0StartupProducts | Where-Object { $_.product -eq 'runtime' })) {
        [void]$runtimeRun.PSObject.Properties.Remove('runtime_product_diagnostics')
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $f0StartupProducts
    })
    $f0EvidenceRoot = Join-Path $fixtureRoot 'evidence-f0-no-runtime-diagnostics'
    $f0Result = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $f0EvidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -Json
    ) | ConvertFrom-Json
    Assert-True ($null -eq $f0Result.render_backend) 'F0 startup evidence should not require a persisted-scene render backend.'
    Assert-True ($null -eq $f0Result.render_adapter) 'F0 startup evidence should not require a persisted-scene render adapter.'
    Assert-True ($null -eq $f0Result.render_device_limits) 'F0 startup evidence should not require negotiated device-limit evidence.'
    $f0Manifest = Get-Content -LiteralPath (Join-Path $f0EvidenceRoot 'manifest.json') -Raw | ConvertFrom-Json
    Assert-True ($null -eq $f0Manifest.render_backend) 'F0 evidence manifest should preserve the absence of runtime diagnostics.'
    Assert-True ($null -eq $f0Manifest.render_adapter) 'F0 evidence manifest should preserve the absence of runtime adapter diagnostics.'
    Assert-True ($null -eq $f0Manifest.render_device_limits) 'F0 evidence manifest should preserve the absence of device-limit diagnostics.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $manifest.product_runs
    })

    $renderBackendDriftSummary = Get-Content -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -Raw | ConvertFrom-Json
    $renderBackendDriftRuntime = @($renderBackendDriftSummary.products | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    Assert-True ($renderBackendDriftRuntime.Count -eq 1) 'Backend drift fixture requires a second runtime run.'
    $renderBackendDriftRuntime[0].runtime_product_diagnostics.render_backend = 'fixture-vulkan'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $renderBackendDriftSummary
    $renderBackendDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-render-backend-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $renderBackendDriftRejected = $_.Exception.Message -match "'render_backend' differs between attempts|disagree on render_backend"
    }
    finally {
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            products = $manifest.product_runs
        })
    }
    Assert-True $renderBackendDriftRejected 'Acceptance did not reject render-backend drift across runtime runs.'

    $missingToolchainManifest = $stagingManifestFixture | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    [void]$missingToolchainManifest.PSObject.Properties.Remove('toolchain')
    Write-FixtureJson -Path $stagingManifestPath -Value $missingToolchainManifest
    $missingToolchainRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-missing-toolchain') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $missingToolchainRejected = $_.Exception.Message -match "Staging manifest is missing 'toolchain'"
    }
    finally {
        Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    }
    Assert-True $missingToolchainRejected 'Acceptance did not reject staging evidence without toolchain provenance.'

    $runtimeDiagnostics = $manifest.product_runs | Where-Object { $_.product -eq 'runtime' } | Select-Object -First 1 -ExpandProperty runtime_product_diagnostics
    Assert-True ($runtimeDiagnostics.input_keyboard_release_count -eq 1) 'Evidence manifest lost runtime input evidence.'
    Assert-True ($runtimeDiagnostics.render_adapter -eq 'Fixture WGPU Adapter') 'Evidence manifest lost the adapter identity.'
    Assert-True ($runtimeDiagnostics.render_adapter_type -eq 'discrete_gpu') 'Evidence manifest lost the adapter type.'
    Assert-True ($runtimeDiagnostics.device_max_bind_groups -eq 5) 'Evidence manifest lost actual device limits.'

    $productEvidenceRuns = Copy-FixtureProductRuns -Runs $manifest.product_runs
    Assert-True (
        $productEvidenceRuns.Count -eq $manifest.product_runs.Count
    ) 'Product evidence fixture copy must retain every staged product run.'
    $logsRoot = Join-Path $stagingRoot 'logs'
    $capturesRoot = Join-Path $stagingRoot 'captures'
    New-Item -ItemType Directory -Force -Path $logsRoot, $capturesRoot | Out-Null
    foreach ($productRun in $productEvidenceRuns) {
        $prefix = "$($productRun.product)-$($productRun.attempt)"
        $stdoutPath = Join-Path $logsRoot "$prefix.stdout.log"
        $stderrPath = Join-Path $logsRoot "$prefix.stderr.log"
        $diagnosticPath = Join-Path $logsRoot "$prefix.diagnostic.log"
        [IO.File]::WriteAllText($stdoutPath, "$prefix stdout`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($stderrPath, "$prefix stderr`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($diagnosticPath, "$prefix diagnostic`n", [Text.UTF8Encoding]::new($false))
        $productRun | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-FixtureFileEvidence -Path $stdoutPath -RelativePath "logs/$prefix.stdout.log")
        $productRun | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-FixtureFileEvidence -Path $stderrPath -RelativePath "logs/$prefix.stderr.log")
        $productRun | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @(
            Get-FixtureFileEvidence -Path $diagnosticPath -RelativePath "logs/$prefix.diagnostic.log"
        )
        if ($productRun.product -eq 'runtime') {
            $capturePath = Join-Path $capturesRoot "$prefix.png"
            Write-FixtureVisiblePng -Path $capturePath
            $productRun | Add-Member -NotePropertyName 'frame_capture' -NotePropertyValue ([ordered]@{
                path = "captures/$prefix.png"
                sha256 = (Get-FileHash -LiteralPath $capturePath -Algorithm SHA256).Hash
                size_bytes = (Get-Item -LiteralPath $capturePath).Length
                width = 16
                height = 16
                non_background_pixels = 128
                non_transparent_pixels = 256
            })
        }
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceRuns
    })
    $productEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProductEvidence `
            -Json
    ) | ConvertFrom-Json
    Assert-True ($productEvidence.product_runs.Count -eq 4) 'Acceptance output lost independently verified staged product evidence.'

    $missingProjectCreationRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'missing-project-creation') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $missingProjectCreationRejected = $_.Exception.Message -match 'project_creation'
    }
    Assert-True $missingProjectCreationRejected 'Acceptance did not reject a fixed F5 request without staged project-creation evidence.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-project-creation'))) 'Missing project-creation evidence left a partial evidence root.'

    $productEvidenceHashMismatch = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidenceHashMismatch[0].stdout.sha256 = ('0' * 64)
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceHashMismatch
    })
    $productEvidenceHashMismatchRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-hash-mismatch') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceHashMismatchRejected = $_.Exception.Message -match 'stdout hash mismatch'
    }
    Assert-True $productEvidenceHashMismatchRejected 'Product evidence with a stdout hash mismatch was not rejected.'

    $productEvidencePathEscape = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidencePathEscape[0].stderr.path = '../outside-staging.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidencePathEscape
    })
    $productEvidencePathEscapeRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-path-escape') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidencePathEscapeRejected = $_.Exception.Message -match 'unsafe relative path'
    }
    Assert-True $productEvidencePathEscapeRejected 'Product evidence with a path escape was not rejected.'

    $productEvidenceMissingFile = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidenceMissingFile[0].diagnostic_logs[0].path = 'logs/missing.diagnostic.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceMissingFile
    })
    $productEvidenceMissingFileRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-missing-file') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceMissingFileRejected = $_.Exception.Message -match 'does not exist in the staging root'
    }
    Assert-True $productEvidenceMissingFileRejected 'Product evidence with a missing diagnostic file was not rejected.'

    $productEvidencePngMetadataMismatch = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidencePngMetadataMismatch[0].frame_capture.non_background_pixels = 0
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidencePngMetadataMismatch
    })
    $productEvidencePngMetadataMismatchRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-png-metadata') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidencePngMetadataMismatchRejected = $_.Exception.Message -match 'non_background_pixels.*differs'
    }
    Assert-True $productEvidencePngMetadataMismatchRejected 'Product evidence with mismatched PNG metadata was not rejected.'

    $blankCapturePath = Join-Path $capturesRoot 'runtime-blank.png'
    Write-FixtureBlankPng -Path $blankCapturePath
    $productEvidenceBlankPng = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $blankRuntime = @($productEvidenceBlankPng | Where-Object { $_.product -eq 'runtime' })[0]
    $blankRuntime.frame_capture.path = 'captures/runtime-blank.png'
    $blankRuntime.frame_capture.sha256 = (Get-FileHash -LiteralPath $blankCapturePath -Algorithm SHA256).Hash
    $blankRuntime.frame_capture.size_bytes = (Get-Item -LiteralPath $blankCapturePath).Length
    $blankRuntime.frame_capture.non_background_pixels = 0
    $blankRuntime.frame_capture.non_transparent_pixels = 256
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceBlankPng
    })
    $productEvidenceBlankPngRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-blank-png') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceBlankPngRejected = $_.Exception.Message -match 'blank or visually insufficient'
    }
    Assert-True $productEvidenceBlankPngRejected 'Product evidence with a blank PNG was not rejected.'

    $createdProjectRoot = Join-Path $stagingRoot 'project\ZirconMvpFixture'
    New-Item -ItemType Directory -Force -Path $createdProjectRoot | Out-Null
    $creationLogsRoot = Join-Path $stagingRoot 'logs'
    $creationDiagnosticsRoot = Join-Path $creationLogsRoot 'editor-create.diagnostics'
    New-Item -ItemType Directory -Force -Path $creationDiagnosticsRoot | Out-Null
    $creationStdoutPath = Join-Path $creationLogsRoot 'editor-create.stdout.log'
    $creationStderrPath = Join-Path $creationLogsRoot 'editor-create.stderr.log'
    $creationDiagnosticPath = Join-Path $creationDiagnosticsRoot 'fixture.log'
    [IO.File]::WriteAllText($creationStdoutPath, "created`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($creationStderrPath, "created stderr`n", [Text.UTF8Encoding]::new($false))
    $encodedCreatedProjectRoot = [Uri]::EscapeDataString($createdProjectRoot)
    $creationDiagnosticText =
        "editor_first_frame_presented`neditor_process_teardown_complete`n" +
        "editor_product_frame_diagnostics project_path=$encodedCreatedProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0`n" +
        "editor_project_open result=completed project_root=$encodedCreatedProjectRoot manifest_identity=Fixture%20Project%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n"
    [IO.File]::WriteAllText(
        $creationDiagnosticPath,
        $creationDiagnosticText,
        [Text.UTF8Encoding]::new($false)
    )
    $creationCapturePath = Join-Path $capturesRoot 'editor-before-edit.png'
    Write-FixtureVisiblePng -Path $creationCapturePath
    $projectCreationFixture = [ordered]@{
        exit_code = 0
        first_frame_presented = $true
        teardown_complete = $true
        stdout = Get-FixtureFileEvidence -Path $creationStdoutPath -RelativePath 'logs/editor-create.stdout.log'
        stderr = Get-FixtureFileEvidence -Path $creationStderrPath -RelativePath 'logs/editor-create.stderr.log'
        diagnostic_logs = @(Get-FixtureFileEvidence -Path $creationDiagnosticPath -RelativePath 'logs/editor-create.diagnostics/fixture.log')
        editor_window_capture = [ordered]@{
            path = 'captures/editor-before-edit.png'
            sha256 = (Get-FileHash -LiteralPath $creationCapturePath -Algorithm SHA256).Hash
            size_bytes = (Get-Item -LiteralPath $creationCapturePath).Length
            width = 16
            height = 16
            non_background_pixels = 128
            non_transparent_pixels = 256
        }
        editor_product_diagnostics = [ordered]@{
            project_path = 'project/ZirconMvpFixture'
            selected_node_id = 3
            selected_node_name = 'Cube'
            inspector_translation_x = '0'
            inspector_translation_y = '0'
            inspector_translation_z = '0'
        }
        project_open = [ordered]@{
            project_root = 'project/ZirconMvpFixture'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            registry_asset_count = 4
            registry_ready_asset_count = 4
            registry_failed_asset_count = 0
            registry_diagnostic_count = 0
            project_generation = 1
            project_generation_publish_epoch = 1
            catalog_asset_count = 4
            settings_source = 'persisted-v1'
        }
    }
    $createdProjectProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($productRun in $createdProjectProducts) {
        $productRun.project = 'project/ZirconMvpFixture'
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $projectCreationFixture
        products = $createdProjectProducts
    })
    $createdProjectEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'created-project-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence `
            -Json
    ) | ConvertFrom-Json
    Assert-True ($createdProjectEvidence.staged_project_root -eq 'project/ZirconMvpFixture') 'Acceptance did not preserve the canonical created-project relative root.'
    Assert-True ($createdProjectEvidence.project_creation.exit_code -eq 0) 'Acceptance did not preserve verified staged editor project-creation evidence.'
    Assert-True ($createdProjectEvidence.project_creation.project_open.manifest_identity -eq 'Fixture Project@v1') 'Acceptance did not preserve the editor project-open manifest identity.'

    [IO.File]::AppendAllText($creationDiagnosticPath, "tampered`n", [Text.UTF8Encoding]::new($false))
    $projectCreationDiagnosticTamperRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-creation-diagnostic-tamper') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $projectCreationDiagnosticTamperRejected = $_.Exception.Message -match 'Project creation diagnostic log hash mismatch'
    }
    finally {
        [IO.File]::WriteAllText($creationDiagnosticPath, $creationDiagnosticText, [Text.UTF8Encoding]::new($false))
    }
    Assert-True $projectCreationDiagnosticTamperRejected 'Acceptance did not reject a modified project-creation diagnostic log.'

    $unicodeProjectName = -join ([char[]]@(0x9879, 0x76EE))
    $unicodeProjectPathSegment = -join ([char[]]@(0x8DEF, 0x5F84))
    $unicodeProjectRelativeRoot = "project/$unicodeProjectName $unicodeProjectPathSegment"
    $unicodeProjectRoot = Join-Path $stagingRoot $unicodeProjectRelativeRoot
    New-Item -ItemType Directory -Force -Path $unicodeProjectRoot | Out-Null
    $unicodeDiagnosticsRoot = Join-Path $creationLogsRoot 'editor-create-unicode.diagnostics'
    New-Item -ItemType Directory -Force -Path $unicodeDiagnosticsRoot | Out-Null
    $unicodeDiagnosticPath = Join-Path $unicodeDiagnosticsRoot 'fixture.log'
    $encodedUnicodeProjectRoot = [Uri]::EscapeDataString($unicodeProjectRoot)
    [IO.File]::WriteAllText(
        $unicodeDiagnosticPath,
        "editor_first_frame_presented`neditor_process_teardown_complete`n" +
        "editor_project_open result=completed project_root=$encodedUnicodeProjectRoot manifest_identity=%E9%A1%B9%E7%9B%AE%20MVP%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n",
        [Text.UTF8Encoding]::new($false)
    )
    $unicodeProjectCreation = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-Json
    $unicodeProjectCreation.diagnostic_logs = @(Get-FixtureFileEvidence -Path $unicodeDiagnosticPath -RelativePath 'logs/editor-create-unicode.diagnostics/fixture.log')
    $unicodeProjectCreation.project_open.project_root = $unicodeProjectRelativeRoot
    $unicodeProjectCreation.project_open.manifest_identity = "$unicodeProjectName MVP@v1"
    $unicodeProjectProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($productRun in $unicodeProjectProducts) {
        $productRun.project = $unicodeProjectRelativeRoot
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = $unicodeProjectRelativeRoot
        project_creation = $unicodeProjectCreation
        products = $unicodeProjectProducts
    })
    $unicodeProjectEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'unicode-created-project-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence `
            -Json
    ) | ConvertFrom-Json
    Assert-True ($unicodeProjectEvidence.staged_project_root -eq $unicodeProjectRelativeRoot) 'Acceptance did not preserve a UTF-8 staged project root.'
    Assert-True ($unicodeProjectEvidence.project_creation.project_open.manifest_identity -eq "$unicodeProjectName MVP@v1") 'Acceptance did not preserve a UTF-8 project-open manifest identity.'

    $invalidProjectOpenEvidence = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-Json
    $invalidProjectOpenEvidence.project_open.scene_uri = 'res://scenes/not-main.scene.toml'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $invalidProjectOpenEvidence
        products = $createdProjectProducts
    })
    $invalidProjectOpenEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'invalid-project-open-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $invalidProjectOpenEvidenceRejected = $_.Exception.Message -match 'project_open.*scene_uri|scene_uri.*project_open'
    }
    Assert-True $invalidProjectOpenEvidenceRejected 'Acceptance did not reject a tampered project-open diagnostic summary.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $projectCreationFixture
        products = $createdProjectProducts
    })

    $missingAuthoringEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-authoring-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation | Out-Null
    }
    catch {
        $missingAuthoringEvidenceRejected = $_.Exception.Message -match 'authoring_automation'
    }
    Assert-True $missingAuthoringEvidenceRejected 'Acceptance with required authoring evidence did not reject a startup summary without an automation report.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-authoring-evidence'))) 'Missing authoring evidence left a partial evidence root.'

    $authoringAutomationFixture = [ordered]@{
        project_path = 'project'
        project_identity = 'fixture-project'
        manifest_identity = 'Fixture Project@v1'
        scene_uri = 'res://scenes/main.scene.toml'
        selected_model_resource_id = 'fixture-cube-model-resource'
        selected_material_resource_id = 'fixture-default-material-resource'
        opened_project_inspection_generation = 1
        records = @(
            [ordered]@{
                binding_path = 'Hierarchy/SelectCube:onClick'
                source = 'Cli'
                operation_id = $null
                transaction_id = $null
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'Inspector/TransformPositionXCommit:onSubmit'
                source = 'Cli'
                operation_id = 'inspector.field.apply_batch'
                transaction_id = 1
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'WorkbenchMenuBar/SaveProject:onClick'
                source = 'Cli'
                operation_id = 'file.project.save'
                transaction_id = $null
                save_generation = 2
            }
        )
        snapshot = [ordered]@{
            project_open = $true
            scene_entry_count = 3
            selected_node_id = 3
            selected_node_name = 'Cube'
            inspector_translation = @('42', '0', '0')
        }
    }
    $reopenAutomationFixture = @(
        [ordered]@{
            project_path = 'project'
            project_identity = 'fixture-project'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            selected_model_resource_id = 'fixture-cube-model-resource'
            selected_material_resource_id = 'fixture-default-material-resource'
            opened_project_inspection_generation = 1
            records = @(
                [ordered]@{
                    binding_path = 'Hierarchy/SelectCube:onClick'
                    source = 'Cli'
                }
            )
            snapshot = [ordered]@{
                project_open = $true
                scene_entry_count = 3
                selected_node_id = 3
                selected_node_name = 'Cube'
                inspector_translation = @('42', '0', '0')
            }
        },
        [ordered]@{
            project_path = 'project'
            project_identity = 'fixture-project'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            selected_model_resource_id = 'fixture-cube-model-resource'
            selected_material_resource_id = 'fixture-default-material-resource'
            opened_project_inspection_generation = 1
            records = @(
                [ordered]@{
                    binding_path = 'Hierarchy/SelectCube:onClick'
                    source = 'Cli'
                }
            )
            snapshot = [ordered]@{
                project_open = $true
                scene_entry_count = 3
                selected_node_id = 3
                selected_node_name = 'Cube'
                inspector_translation = @('42', '0', '0')
            }
        }
    )
    $authoringAutomationFixture = New-FixtureAutomationProcessEvidence `
        -Report $authoringAutomationFixture `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring'
    $baselineAutomationFixture = Copy-FixtureProductRuns -Runs @($reopenAutomationFixture[0])
    $baselineAutomationFixture.snapshot.inspector_translation[0] = '0'
    $baselineAutomationFixture = New-FixtureAutomationProcessEvidence `
        -Report $baselineAutomationFixture `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-baseline'
    $reopenAutomationOne = New-FixtureAutomationProcessEvidence `
        -Report $reopenAutomationFixture[0] `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-reopen-1'
    $reopenAutomationTwo = New-FixtureAutomationProcessEvidence `
        -Report $reopenAutomationFixture[1] `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-reopen-2'
    $reopenAutomationFixture = @($reopenAutomationOne, $reopenAutomationTwo)
    Assert-True ($authoringAutomationFixture -isnot [array]) 'Authoring process evidence fixture emitted multiple pipeline values instead of one report.'
    Assert-True ($null -ne $authoringAutomationFixture.PSObject.Properties['records']) 'Authoring process evidence fixture lost the original binding records before serialization.'
    $capturedAuthoringReportFixture = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-Json
    Assert-True ($null -ne $capturedAuthoringReportFixture.PSObject.Properties['records']) 'Authoring process evidence fixture did not serialize the original binding records into stdout.'
    $authoringProductRuns = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $afterAuthoringRuntime = Copy-FixtureProductRuns -Runs @($manifest.product_runs | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    $afterAuthoringRuntime.attempt = 3
    $afterAuthoringRuntime.runtime_product_diagnostics.frame_index = 3
    $authoringProductRuns += $afterAuthoringRuntime
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $authoringStartupFixture = Get-Content -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -Raw | ConvertFrom-Json
    Assert-True ($null -ne $authoringStartupFixture.PSObject.Properties['reopen_automation']) 'Authoring acceptance fixture did not serialize its repeated reopen reports.'
    Assert-True ($null -ne $authoringStartupFixture.authoring_automation.PSObject.Properties['records']) 'Authoring acceptance fixture did not retain authoring records in its startup summary.'
    Assert-True (@($authoringStartupFixture.authoring_automation.records).Count -eq 3) 'Authoring acceptance fixture did not retain all authoring records in its startup summary.'
    try {
        $authoringEvidence = @(
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot (Join-Path $fixtureRoot 'authoring-evidence') `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -RequireAuthoringAutomation `
                -RequireReopenAutomation `
                -Json
        ) | ConvertFrom-Json
    }
    catch {
        throw "Authoring automation positive fixture was rejected: $($_.Exception.Message)"
    }
    Assert-True ($null -ne $authoringEvidence.authoring_automation) 'Acceptance output lost required authoring automation evidence.'
    Assert-True ($authoringEvidence.authoring_automation.records.Count -eq 3) 'Acceptance output lost the normal authoring binding sequence.'
    Assert-True ($authoringEvidence.reopen_automation.Count -eq 2) 'Acceptance output lost the independent reopened-project reports.'

    $f5CreationDiagnosticPath = Join-Path $creationLogsRoot 'editor-create-f5.diagnostics\fixture.log'
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $f5CreationDiagnosticPath) | Out-Null
    $encodedF5ProjectRoot = [Uri]::EscapeDataString((Join-Path $stagingRoot 'project'))
    [IO.File]::WriteAllText(
        $f5CreationDiagnosticPath,
        "editor_first_frame_presented`neditor_process_teardown_complete`n" +
        "editor_product_frame_diagnostics project_path=$encodedF5ProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0`n" +
        "editor_project_open result=completed project_root=$encodedF5ProjectRoot manifest_identity=Fixture%20Project%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n",
        [Text.UTF8Encoding]::new($false)
    )
    $f5ProjectCreation = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-Json
    $f5ProjectCreation.diagnostic_logs = @(Get-FixtureFileEvidence -Path $f5CreationDiagnosticPath -RelativePath 'logs/editor-create-f5.diagnostics/fixture.log')
    $f5ProjectCreation.project_open.project_root = 'project'
    $f5ProjectCreation.editor_product_diagnostics.project_path = 'project'
    $afterReopenCapturePath = Join-Path $capturesRoot 'editor-after-reopen.png'
    Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    $f5ProductRuns = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $f5ReopenedEditor = @($f5ProductRuns | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    $f5ReopenedEditor | Add-Member -NotePropertyName 'editor_window_capture' -NotePropertyValue ([ordered]@{
        path = 'captures/editor-after-reopen.png'
        sha256 = (Get-FileHash -LiteralPath $afterReopenCapturePath -Algorithm SHA256).Hash
        size_bytes = (Get-Item -LiteralPath $afterReopenCapturePath).Length
        width = 16
        height = 16
        non_background_pixels = 128
        non_transparent_pixels = 256
    })
    $f5ReopenedEditor | Add-Member -NotePropertyName 'editor_product_diagnostics' -NotePropertyValue ([ordered]@{
        project_path = 'project'
        selected_node_id = 3
        selected_node_name = 'Cube'
        inspector_translation_x = '42'
        inspector_translation_y = '0'
        inspector_translation_z = '0'
    })
    $f5AfterAuthoringRuntime = Copy-FixtureProductRuns -Runs @($f5ProductRuns | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    $f5AfterAuthoringRuntime.attempt = 3
    $f5AfterAuthoringRuntime.runtime_product_diagnostics.frame_index = 3
    $f5ProductRuns += $f5AfterAuthoringRuntime
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5Evidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-editor-window-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence `
            -Json
    ) | ConvertFrom-Json
    Assert-True ($f5Evidence.project_creation.editor_window_capture.path -eq 'captures/editor-before-edit.png') 'F5 acceptance output lost the created-project editor window evidence.'
    Assert-True ($f5Evidence.project_identity.project_root -eq 'project') 'F5 evidence manifest lost the canonical project root.'
    Assert-True ($f5Evidence.project_identity.project_identity -eq 'fixture-project') 'F5 evidence manifest lost the runtime project identity.'
    Assert-True ($f5Evidence.project_identity.manifest_identity -eq 'Fixture Project@v1') 'F5 evidence manifest lost the editor manifest identity.'
    Assert-True ($f5Evidence.project_identity.scene_uri -eq 'res://scenes/main.scene.toml') 'F5 evidence manifest lost the canonical scene URI.'
    Assert-True ($f5Evidence.project_identity.model_resource_id -eq 'fixture-cube-model-resource') 'F5 evidence manifest lost the selected Cube model reference.'
    Assert-True ($f5Evidence.project_identity.material_resource_id -eq 'fixture-default-material-resource') 'F5 evidence manifest lost the selected Cube material reference.'
    Assert-True ($f5Evidence.baseline_automation.snapshot.inspector_translation[0] -eq '0') 'F5 evidence manifest lost the pre-authoring Cube baseline.'
    $f5EvidenceEditorRun = @(
        $f5Evidence.product_runs | Where-Object {
            $_.product -eq 'editor' -and
            $null -ne $_.PSObject.Properties['editor_window_capture'] -and
            $null -ne $_.editor_window_capture
        }
    )[0]
    Assert-True ($f5EvidenceEditorRun.editor_window_capture.path -eq 'captures/editor-after-reopen.png') 'F5 acceptance output lost the reopened editor window evidence.'

    $f5IdenticalCaptures = Copy-FixtureProductRuns -Runs $f5ProductRuns
    $f5IdenticalEditor = @($f5IdenticalCaptures | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    Copy-Item -LiteralPath $creationCapturePath -Destination $afterReopenCapturePath -Force
    $f5IdenticalEditor.editor_window_capture.sha256 = (Get-FileHash -LiteralPath $afterReopenCapturePath -Algorithm SHA256).Hash
    $f5IdenticalEditor.editor_window_capture.size_bytes = (Get-Item -LiteralPath $afterReopenCapturePath).Length
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5IdenticalCaptures
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5IdenticalCapturesRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-identical-editor-captures') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5IdenticalCapturesRejected = $_.Exception.Message -match 'window captures are identical'
    }
    finally {
        Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    }
    Assert-True $f5IdenticalCapturesRejected 'F5 acceptance did not reject identical before/after editor window captures.'

    $f5ReferenceDrift = Copy-FixtureProductRuns -Runs $reopenAutomationFixture
    $f5ReferenceDrift[1].selected_material_resource_id = 'replacement-material-resource'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $f5ReferenceDrift
    })
    $f5ReferenceDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-project-reference-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5ReferenceDriftRejected = $_.Exception.Message -match 'selected_material_resource_id.*(captured stdout report|pre-authoring baseline)'
    }
    Assert-True $f5ReferenceDriftRejected 'F5 acceptance did not reject a replaced Cube material reference after reopen.'

    $f5DriftedAuthoringReport = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-Json
    $f5DriftedAuthoringReport.selected_model_resource_id = 'replacement-model-resource'
    $f5DriftedAuthoringReport.selected_material_resource_id = 'replacement-material-resource'
    $f5DriftedAuthoring = New-FixtureAutomationProcessEvidence `
        -Report $f5DriftedAuthoringReport `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-reference-drift'
    $f5DriftedReopens = @()
    for ($index = 1; $index -le 2; $index++) {
        $report = Get-Content -LiteralPath (Join-Path $stagingRoot "logs\editor-reopen-$index.stdout.log") -Raw | ConvertFrom-Json
        $report.selected_model_resource_id = 'replacement-model-resource'
        $report.selected_material_resource_id = 'replacement-material-resource'
        $f5DriftedReopens += New-FixtureAutomationProcessEvidence `
            -Report $report `
            -RequestPath $reopenRequestPath `
            -RequestRelativePath 'reopen/automation.json' `
            -StagingRoot $stagingRoot `
            -EvidenceLabel "editor-reopen-reference-drift-$index"
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $f5DriftedAuthoring
        reopen_automation = $f5DriftedReopens
    })
    $f5FirstAuthoringReferenceDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-first-authoring-reference-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5FirstAuthoringReferenceDriftRejected = $_.Exception.Message -match 'selected_(model|material)_resource_id differs from the pre-authoring baseline'
    }
    Assert-True $f5FirstAuthoringReferenceDriftRejected 'F5 acceptance did not reject model/material replacement first introduced by authoring.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })

    $f5MissingBeforeEditCapture = $f5ProjectCreation | ConvertTo-Json -Depth 12 | ConvertFrom-Json
    $f5MissingBeforeEditCapture.PSObject.Properties.Remove('editor_window_capture')
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5MissingBeforeEditCapture
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5MissingBeforeEditCaptureRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-missing-before-edit-capture') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5MissingBeforeEditCaptureRejected = $_.Exception.Message -match 'editor_window_capture'
    }
    Assert-True $f5MissingBeforeEditCaptureRejected 'F5 acceptance did not reject a missing created-project editor PNG.'

    Remove-Item -LiteralPath $afterReopenCapturePath -Force -ErrorAction Stop
    try {
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            project_creation = $f5ProjectCreation
            products = $f5ProductRuns
            baseline_automation = $baselineAutomationFixture
            authoring_automation = $authoringAutomationFixture
            reopen_automation = $reopenAutomationFixture
        })
        $f5MissingReopenedCaptureFileRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot (Join-Path $fixtureRoot 'f5-missing-reopened-capture-file') `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $f5MissingReopenedCaptureFileRejected = $_.Exception.Message -match 'does not exist in the staging root'
        }
        Assert-True $f5MissingReopenedCaptureFileRejected 'F5 acceptance did not reject a missing reopened editor PNG file with retained metadata.'
    }
    finally {
        Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    }

    $f5EditorCaptureDrift = Copy-FixtureProductRuns -Runs $f5ProductRuns
    $f5EditorCaptureDriftRun = @(
        $f5EditorCaptureDrift | Where-Object {
            $_.product -eq 'editor' -and
            $null -ne $_.PSObject.Properties['editor_window_capture'] -and
            $null -ne $_.editor_window_capture
        }
    )[0]
    $f5EditorCaptureDriftRun.editor_window_capture.non_background_pixels = 0
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5EditorCaptureDrift
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5EditorCaptureDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-editor-window-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5EditorCaptureDriftRejected = $_.Exception.Message -match 'editor window capture.*non_background_pixels.*differs'
    }
    Assert-True $f5EditorCaptureDriftRejected 'F5 acceptance did not reject mismatched reopened editor PNG evidence.'

    $authoringRequestHashDrift = $authoringAutomationFixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
    $authoringRequestHashDrift.automation_request.sha256 = ('0' * 64)
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $authoringRequestHashDrift
        reopen_automation = $reopenAutomationFixture
    })
    $authoringRequestHashDriftRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'authoring-request-hash-drift') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $authoringRequestHashDriftRejected = $_.Exception.Message -match 'request hash mismatch'
    }
    Assert-True $authoringRequestHashDriftRejected 'Acceptance did not reject authoring evidence detached from its staged request hash.'

    $authoringStdoutDrift = $authoringAutomationFixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
    $authoringStdoutDrift.snapshot.inspector_translation[0] = '41'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $authoringStdoutDrift
        reopen_automation = $reopenAutomationFixture
    })
    $authoringStdoutDriftRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'authoring-stdout-drift') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $authoringStdoutDriftRejected = $_.Exception.Message -match 'differs from its captured stdout report'
    }
    Assert-True $authoringStdoutDriftRejected 'Acceptance did not reject authoring evidence detached from its captured process stdout.'

    $reopenTranslationDrift = Copy-FixtureProductRuns -Runs $reopenAutomationFixture
    $reopenTranslationDrift[1].snapshot.inspector_translation[0] = '43'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenTranslationDrift
    })
    $reopenTranslationDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'reopen-translation-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation `
            -RequireReopenAutomation | Out-Null
    }
    catch {
        $reopenTranslationDriftRejected = $_.Exception.Message -match 'differs'
    }
    Assert-True $reopenTranslationDriftRejected 'Acceptance evidence with a reopened Inspector transform drift was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'reopen-translation-drift-evidence'))) 'Reopen transform drift left a partial evidence root.'

    $projectIdentityDriftProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $projectIdentityDriftRuntimes = @($projectIdentityDriftProducts | Where-Object { $_.product -eq 'runtime' })
    Assert-True ($projectIdentityDriftRuntimes.Count -eq 2) "Project identity drift fixture requires two runtime attempts; manifest_products=$(@($manifest.product_runs).Count) cloned_products=$($projectIdentityDriftProducts.Count) runtime_attempts=$($projectIdentityDriftRuntimes.Count)."
    $projectIdentityDriftRuntime = $projectIdentityDriftRuntimes[1]
    $projectIdentityDriftRuntime.runtime_product_diagnostics.project_identity = 'fixture-project-drifted'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $projectIdentityDriftProducts
    })
    $projectIdentityDriftRejected = $false
    $projectIdentityDriftMessage = '<no error>'
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-identity-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $projectIdentityDriftMessage = $_.Exception.Message
        $projectIdentityDriftRejected = $projectIdentityDriftMessage -match 'project_identity.*differs'
    }
    Assert-True $projectIdentityDriftRejected "Acceptance evidence with runtime project identity drift was not rejected; observed='$projectIdentityDriftMessage'."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'project-identity-drift-evidence'))) 'Project identity drift left a partial evidence root.'

    $projectPathDriftProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $projectPathDriftEditor = @($projectPathDriftProducts | Where-Object { $_.product -eq 'editor' })[1]
    $projectPathDriftEditor.project = 'project-drifted'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $projectPathDriftProducts
    })
    $projectPathDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-path-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $projectPathDriftRejected = $_.Exception.Message -match 'staged project root.*differs'
    }
    Assert-True $projectPathDriftRejected 'Acceptance evidence with a product project-path drift was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'project-path-drift-evidence'))) 'Project-path drift left a partial evidence root.'

    $duplicateAttemptProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $duplicateAttemptRuntime = @($duplicateAttemptProducts | Where-Object { $_.product -eq 'runtime' })[1]
    $duplicateAttemptRuntime.attempt = 1
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $duplicateAttemptProducts
    })
    $duplicateAttemptRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'duplicate-attempt-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $duplicateAttemptRejected = $_.Exception.Message -match 'duplicate attempt'
    }
    Assert-True $duplicateAttemptRejected 'Acceptance evidence with duplicate runtime attempts was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'duplicate-attempt-evidence'))) 'Duplicate attempt evidence left a partial evidence root.'

    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @($manifest.product_runs | Where-Object { $_.attempt -eq 1 })
    })
    $singleAttemptRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'single-attempt-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $singleAttemptRejected = $_.Exception.Message -match 'at least two successful.*runtime.*editor'
    }
    Assert-True $singleAttemptRejected 'Acceptance evidence without two successful runtime and editor runs was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'single-attempt-evidence'))) 'Single-attempt acceptance input left a partial evidence root.'

    $nestedEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $stagingRoot 'acceptance-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $nestedEvidenceRejected = $_.Exception.Message -match 'outside StagingRoot'
    }
    Assert-True $nestedEvidenceRejected 'Acceptance evidence nested under the staging root was not rejected.'

    [IO.File]::WriteAllText($projectManifestPath, "name = 'Mutated'`n", [Text.UTF8Encoding]::new($false))
    $stagingMutationRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'mutated-staging-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $stagingMutationRejected = $_.Exception.Message -match 'hash mismatch'
    }
    Assert-True $stagingMutationRejected 'Acceptance did not reject a staged file that diverged from its manifest hash.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'mutated-staging-evidence'))) 'Staging-integrity failure left a partial evidence root.'
    [IO.File]::WriteAllText($projectManifestPath, "name = 'Fixture'`n", [Text.UTF8Encoding]::new($false))

    $duplicateManifest = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_at_utc = '2026-07-29T00:00:00Z'
        entries = @($stagingManifestEntry, $stagingManifestEntry)
    }
    Write-FixtureJson -Path $stagingManifestPath -Value $duplicateManifest
    $duplicateEntryRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'duplicate-entry-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $duplicateEntryRejected = $_.Exception.Message -match 'duplicate target_relative_path'
    }
    Assert-True $duplicateEntryRejected 'Acceptance did not reject duplicate staging target paths.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'duplicate-entry-evidence'))) 'Duplicate staging entries left a partial evidence root.'
    Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture

    $missingRuntimeDiagnosticsFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $missingRuntimeDiagnosticsFixture
    $missingRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProductEvidence | Out-Null
    }
    catch {
        $missingRuntimeDiagnosticsRejected = $_.Exception.Message -match 'runtime_product_diagnostics'
    }
    Assert-True $missingRuntimeDiagnosticsRejected 'Acceptance evidence without runtime diagnostics was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-runtime-diagnostics-evidence'))) 'Missing runtime diagnostics left a partial evidence root.'

    $zeroInputRuntimeDiagnosticsFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 0
                }
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $zeroInputRuntimeDiagnosticsFixture
    $zeroInputRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'zero-input-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $zeroInputRuntimeDiagnosticsRejected = $_.Exception.Message -match 'input_keyboard_release_count'
    }
    Assert-True $zeroInputRuntimeDiagnosticsRejected 'Acceptance evidence with zero runtime input consumption was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'zero-input-runtime-diagnostics-evidence'))) 'Zero runtime input diagnostics left a partial evidence root.'

    $zeroInputRuntimeDiagnosticsFixture.products[0].runtime_product_diagnostics.input_keyboard_release_count = 1
    $zeroInputRuntimeDiagnosticsFixture.products[0].runtime_product_diagnostics.material_fallback_count = 1
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $zeroInputRuntimeDiagnosticsFixture
    $materialFallbackRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'material-fallback-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $materialFallbackRuntimeDiagnosticsRejected = $_.Exception.Message -match 'material_fallback_count'
    }
    Assert-True $materialFallbackRuntimeDiagnosticsRejected 'Acceptance evidence with runtime material fallback usage was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'material-fallback-runtime-diagnostics-evidence'))) 'Material fallback runtime diagnostics left a partial evidence root.'

    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            }
        )
    })
    $missingEditorRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-editor-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $missingEditorRejected = $_.Exception.Message -match 'runtime and editor'
    }
    Assert-True $missingEditorRejected 'Acceptance evidence without a successful editor product run was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-editor-evidence'))) 'Rejected acceptance input left a partial evidence root.'

    Write-Host 'MVP acceptance manifest contract passed'
}
finally {
    if (Test-Path -LiteralPath $fixtureRoot) {
        Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
    }
}
