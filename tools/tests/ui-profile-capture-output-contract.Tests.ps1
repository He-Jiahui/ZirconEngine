$script:ProfileCaptureScript = Join-Path $PSScriptRoot "..\ui-profile-capture.ps1"
$script:ProfileCapturePaths = Join-Path $PSScriptRoot "..\profile-capture-paths.ps1"
$script:ProfileCaptureManifest = Join-Path $PSScriptRoot "..\profile-capture-manifest.ps1"
$script:ProfileNativeInteraction = Join-Path $PSScriptRoot "..\ui-profile-native-resize.ps1"
$script:ProfileLatencyEvidence = Join-Path $PSScriptRoot "..\ui-profile-latency-evidence.ps1"
$script:ProfileProcessEvidence = Join-Path $PSScriptRoot "..\ui-profile-process-evidence.ps1"
$script:ProfileScaleFixture = Join-Path $PSScriptRoot "..\ui-profile-scale-fixture.ps1"
$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:ProfileCaptureSource = @(
    Get-Content -LiteralPath $script:ProfileCaptureScript -Raw
    Get-Content -LiteralPath $script:ProfileLatencyEvidence -Raw
    Get-Content -LiteralPath $script:ProfileProcessEvidence -Raw
) -join "`n"
$script:ProfileNativeInteractionSource = Get-Content -LiteralPath $script:ProfileNativeInteraction -Raw
$script:HierarchyPointerRebuildSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs') -Raw
$script:NativeScrollDispatchSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/entry.rs') -Raw
$script:HierarchyScrollDispatchSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/pane/native/hierarchy.rs') -Raw
$script:WelcomeScrollDispatchSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/pane/native/welcome.rs') -Raw
$script:WelcomePointerRebuildSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs') -Raw
$script:UiPerfSource = Get-Content -LiteralPath (Join-Path $script:RepoRoot 'zircon_editor/src/ui/retained_host/ui_perf.rs') -Raw
if (Test-Path -LiteralPath $script:ProfileCapturePaths) {
    . $script:ProfileCapturePaths
}
if (Test-Path -LiteralPath $script:ProfileCaptureManifest) {
    . $script:ProfileCaptureManifest
}
if (Test-Path -LiteralPath $script:ProfileLatencyEvidence) {
    . $script:ProfileLatencyEvidence
}
if (Test-Path -LiteralPath $script:ProfileProcessEvidence) {
    . $script:ProfileProcessEvidence
}

function New-ProfileManifestTestRepository {
    param(
        [string]$Root,
        [string]$OmitCriticalSource = "",
        [string]$OmitCaptureTool = ""
    )

    New-Item -ItemType Directory -Force -Path $Root | Out-Null
    & git.exe -C $Root init --quiet
    if ($LASTEXITCODE -ne 0) {
        throw "Could not initialize manifest test repository."
    }
    & git.exe -C $Root config user.email "zircon-profile-test@example.invalid"
    & git.exe -C $Root config user.name "Zircon Profile Test"
    foreach ($relativePath in (Get-ZirconProfileCriticalSourcePaths)) {
        if ($relativePath -eq $OmitCriticalSource) {
            continue
        }
        $path = Join-Path $Root $relativePath
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
        Set-Content -LiteralPath $path -Value $relativePath -Encoding ASCII
    }
    foreach ($relativePath in @(
        'tools/ui-profile-capture.ps1',
        'tools/ui-profile-latency-evidence.ps1',
        'tools/ui-profile-process-evidence.ps1',
        'tools/ui-profile-native-resize.ps1',
        'tools/ui-profile-scale-fixture.ps1',
        'tools/profile-capture-paths.ps1',
        'tools/profile-capture-manifest.ps1'
    )) {
        if ($relativePath -eq $OmitCaptureTool) {
            continue
        }
        $path = Join-Path $Root $relativePath
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
        Set-Content -LiteralPath $path -Value $relativePath -Encoding ASCII
    }
    & git.exe -C $Root add --all
    & git.exe -C $Root commit --quiet -m "profile fixture"
    if ($LASTEXITCODE -ne 0) {
        throw "Could not commit manifest test repository."
    }
    return $Root
}

Describe "ui-profile-capture output contract" {
    It "accepts only E drive profile output roots" {
        Get-Command Resolve-ZirconProfileOutputRoot -ErrorAction SilentlyContinue | Should Not BeNullOrEmpty

        Resolve-ZirconProfileOutputRoot -RepoRoot 'E:\Git\ZirconEngine' -Path 'E:\zircon-profiles' |
            Should Be 'E:\zircon-profiles'
        Resolve-ZirconProfileOutputRoot -RepoRoot 'E:\Git\ZirconEngine' -Path 'E:\zircon-profiles\run-001' |
            Should Be 'E:\zircon-profiles\run-001'

        foreach ($unsafePath in @(
            'target\zircon-profiles',
            'E:\zircon-profiles-sibling',
            'E:\zircon-profiles\..\target'
        )) {
            { Resolve-ZirconProfileOutputRoot -RepoRoot 'E:\Git\ZirconEngine' -Path $unsafePath } |
                Should Throw 'Profile output root must resolve beneath E:\zircon-profiles.'
        }
    }

    It "requires a coordinator-managed external profiling target" {
        $script:ProfileCaptureSource | Should Match "CARGO_TARGET_DIR must be set"
        $script:ProfileCaptureSource | Should Not Match [regex]::Escape('Join-Path $RepoRoot "target\\profiling"')
    }

    It "requires a managed profiling build instead of invoking Cargo directly" {
        $script:ProfileCaptureSource | Should Match "requires a managed profiling build"
        $script:ProfileCaptureSource | Should Not Match "cargo build -p zircon_runtime"
        $script:ProfileCaptureSource | Should Not Match "cargo build -p zircon_app"
    }

    It "records CPU evidence and source-binds configurable interaction volumes" {
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoClickCount = 0'
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoClickDelayMs = 4'
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoWheelCount = 0'
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoWheelDelayMs = 2'
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoResizeStepCount = 24'
        $script:ProfileCaptureSource | Should Match '\[int\]\$AutoResizeDelayMs = 40'
        $script:ProfileCaptureSource | Should Match 'Invoke-PointerClickStorm'
        $script:ProfileCaptureSource | Should Match 'Invoke-PointerWheelStorm'
        $script:ProfileCaptureSource | Should Match 'Get-LiveGeometryScrollTargets'
        $script:ProfileCaptureSource | Should Match 'Get-LiveGeometryInteractionTargets'
        $script:ProfileCaptureSource | Should Match 'ui_profile_geometry\.json'
        $script:ProfileNativeInteractionSource | Should Match '\[object\[\]\]\$Targets'
        $script:ProfileNativeInteractionSource | Should Match 'target_id = \[string\]\$_\.target_id'
        $script:ProfileNativeInteractionSource | Should Match 'target_kind = \[string\]\$_\.target_kind'
        $script:ProfileNativeInteractionSource | Should Match 'target_surface = \[string\]\$_\.target_surface'
        $script:ProfileNativeInteractionSource | Should Match 'targets = \$targetEvidence'
        $script:ProfileNativeInteractionSource | Should Match 'mouse_event\(0x0800'
        $script:ProfileNativeInteractionSource | Should Match 'alternating_direction = \$true'
        $script:ProfileCaptureSource | Should Match 'auto_click_count = \$AutoClickCount'
        $script:ProfileCaptureSource | Should Match 'auto_click_delay_ms = \$AutoClickDelayMs'
        $script:ProfileCaptureSource | Should Match 'auto_wheel_count = \$AutoWheelCount'
        $script:ProfileCaptureSource | Should Match 'auto_wheel_delay_ms = \$AutoWheelDelayMs'
        $script:ProfileCaptureSource | Should Match 'auto_resize_step_count = \$AutoResizeStepCount'
        $script:ProfileCaptureSource | Should Match 'auto_resize_delay_ms = \$AutoResizeDelayMs'
        $script:ProfileCaptureSource | Should Match '\$completedClicks -ne \$requestedClicks'
        $script:ProfileCaptureSource | Should Match '\$completedMoves -ne \$requestedMoves'
        $script:ProfileCaptureSource | Should Match '\$completedWheelEvents -ne \$requestedWheelEvents'
        $script:ProfileCaptureSource | Should Match 'function Test-HierarchyScrollCounterGate'
        $script:ProfileCaptureSource | Should Match 'function Test-WelcomeRecentScrollCounterGate'
        $script:ProfileCaptureSource | Should Match 'function Get-WelcomeRecentScrollTargets'
        $script:ProfileCaptureSource | Should Match 'viewport_toolbar_click'
        $script:ProfileCaptureSource | Should Match '\$viewportToolbarControlsOnly = \$normalizedScenario -eq "viewport_toolbar_click"'
        $script:ProfileCaptureSource | Should Match 'target_kind -ne "viewport_toolbar_control"'
        $script:ProfileCaptureSource | Should Match '\$interactionKind = if \(\$normalizedScenario -in @\("hierarchy_scroll", "welcome_recent_scroll"\)\)'
        $script:ProfileCaptureSource | Should Match 'switch \(\$interactionKind\)'
        $script:ProfileCaptureSource | Should Match 'ui\.idle_hover\.hierarchy_scroll_dispatch_count'
        $script:ProfileCaptureSource | Should Match 'ui\.idle_hover\.hierarchy_surface_rebuild_count'
        $script:ProfileCaptureSource | Should Match 'ui\.idle_hover\.hierarchy_row_insert_count'
        $script:ProfileCaptureSource | Should Match 'ui\.idle_hover\.hierarchy_dispatcher_rebuild_count'
        $script:ProfileCaptureSource | Should Match 'ui\.idle_hover\.hierarchy_route_map_rebuild_count'
        foreach ($counterName in @(
                'welcome_recent_scroll_dispatch_count',
                'welcome_recent_surface_rebuild_count',
                'welcome_recent_authority_rebuild_count',
                'welcome_recent_row_insert_count',
                'welcome_recent_geometry_patch_count',
                'welcome_recent_dispatcher_rebuild_count',
                'welcome_recent_route_map_rebuild_count'
            )) {
            $script:ProfileCaptureSource | Should Match "ui\.idle_hover\.$counterName"
        }
        $script:NativeScrollDispatchSource | Should Not Match 'UiPerfCounter::HierarchyScrollDispatchCount'
        $script:HierarchyScrollDispatchSource | Should Match 'UiPerfCounter::HierarchyScrollDispatchCount'
        $script:HierarchyPointerRebuildSource | Should Match 'UiPerfCounter::HierarchySurfaceRebuildCount'
        $script:HierarchyPointerRebuildSource | Should Match 'UiPerfCounter::HierarchyRowInsertCount'
        $script:HierarchyPointerRebuildSource | Should Match 'UiPerfCounter::HierarchyDispatcherRebuildCount'
        $script:HierarchyPointerRebuildSource | Should Match 'UiPerfCounter::HierarchyRouteMapRebuildCount'
        foreach ($counter in @(
                'HierarchyScrollDispatchCount',
                'HierarchySurfaceRebuildCount',
                'HierarchyRowInsertCount',
                'HierarchyDispatcherRebuildCount',
                'HierarchyRouteMapRebuildCount'
            )) {
            $script:UiPerfSource | Should Match "UiPerfCounter::$counter"
        }
        $script:WelcomeScrollDispatchSource | Should Match 'UiPerfCounter::WelcomeRecentScrollDispatchCount'
        foreach ($counter in @(
                'WelcomeRecentSurfaceRebuildCount',
                'WelcomeRecentAuthorityRebuildCount',
                'WelcomeRecentRowInsertCount',
                'WelcomeRecentGeometryPatchCount',
                'WelcomeRecentDispatcherRebuildCount',
                'WelcomeRecentRouteMapRebuildCount'
            )) {
            $script:WelcomePointerRebuildSource | Should Match "UiPerfCounter::$counter"
        }
        foreach ($counter in @(
                'WelcomeRecentScrollDispatchCount',
                'WelcomeRecentSurfaceRebuildCount',
                'WelcomeRecentAuthorityRebuildCount',
                'WelcomeRecentRowInsertCount',
                'WelcomeRecentGeometryPatchCount',
                'WelcomeRecentDispatcherRebuildCount',
                'WelcomeRecentRouteMapRebuildCount'
            )) {
            $script:UiPerfSource | Should Match "UiPerfCounter::$counter"
        }
        $script:ProfileNativeInteractionSource | Should Match 'processor_time_delta_ms'
        $script:ProfileNativeInteractionSource | Should Match 'cpu_core_utilization_percent'
        $script:ProfileNativeInteractionSource | Should Match 'cpu_system_utilization_percent'
        $script:ProfileCaptureSource | Should Match 'WithinProcessWarmupPresentCount = 1'
        $script:ProfileCaptureSource | Should Match 'WithinProcessQuiescenceSeconds = 2'
        $script:ProfileCaptureSource | Should Not Match 'CachePrimeRunCount'
        $script:ProfileCaptureSource | Should Match 'MeasuredRunCount = 3'
        $script:ProfileCaptureSource | Should Not Match "'cache_prime'"
        $script:ProfileCaptureSource | Should Match 'run_phase = \$runPhase'
        $script:ProfileCaptureSource | Should Match "'within_process_warm_measure'"
        $script:ProfileCaptureSource | Should Match 'within_process_warmup = \$withinProcessWarmupPresentCount -gt 0'
        $script:ProfileCaptureSource | Should Match 'ZIRCON_PROFILE_WITHIN_PROCESS_WARMUP_PRESENTS'
        $script:ProfileCaptureSource | Should Match 'Complete-ZirconProcessQuiescenceEvidence'
        $script:ProfileCaptureSource | Should Match 'within_process_quiescence_seconds = \$WithinProcessQuiescenceSeconds'
        $script:ProfileCaptureSource | Should Match 'RunQuiescenceSeconds'
    }

    It "waits for the current process measurement epoch before automatic input" {
        $script:ProfileCaptureSource | Should Match 'ui_profile_measurement_ready\.json'
        $script:ProfileCaptureSource | Should Match 'function Wait-ProfileMeasurementReady'
        $script:ProfileCaptureSource | Should Match '\[int\]\$ready\.process_id -eq \$Process\.Id'
        $afterInteraction = $script:ProfileCaptureSource.Split(
            'function Invoke-AutoScenarioInteraction', 2)[1]
        $interaction = $afterInteraction.Split(
            'function Invoke-SoftbufferScreenshotCapture', 2)[0]
        $ready = $interaction.IndexOf('Wait-ProfileMeasurementReady')
        $foreground = $interaction.IndexOf('[ZirconProfileCaptureNative]::SetForegroundWindow')

        $ready | Should BeGreaterThan -1
        $foreground | Should BeGreaterThan $ready
    }

    It "exports damage coverage and host invalidation decision evidence" {
        $script:ProfileCaptureSource | Should Match 'presented_surface_pixels'
        $script:ProfileCaptureSource | Should Match 'damage_coverage_percent'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_transaction_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_scope_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_legacy_dirty_transaction_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_full_target_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_shell_content_target_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_workbench_projection_target_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_view_presentation_target_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_window_metrics_target_count'
        $script:ProfileCaptureSource | Should Match 'host_invalidation_paint_only_target_count'
    }

    It "requires source-bound GPU timestamp evidence for rendered scenarios" {
        $script:ProfileCaptureSource | Should Match 'gpu_timestamp_supported_present_count'
        $script:ProfileCaptureSource | Should Match 'gpu_time_sample_count'
        $script:ProfileCaptureSource | Should Match 'gpu_time_p50_us'
        $script:ProfileCaptureSource | Should Match 'gpu_time_p95_us'
        $script:ProfileCaptureSource | Should Match 'gpu_time_max_us'
        $script:ProfileCaptureSource | Should Match 'gpu_profile_latency_max_frames'
    }

    It "rejects internally inconsistent host invalidation and damage evidence" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @('Resolve-InteractionScenarioName', 'Show-UiScenarioEvidence')) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'scenario-consistency'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $scenario = [ordered]@{
            scenario = 'click'
            frame_count = 1
            dirty_paint_only_count = 1
            redraw_region_count = 1
            redraw_full_frame_count = 0
            host_invalidation_transaction_count = 1
            host_invalidation_scope_count = 1
            host_invalidation_legacy_dirty_transaction_count = 0
            host_invalidation_full_target_count = 0
            host_invalidation_shell_content_target_count = 0
            host_invalidation_workbench_projection_target_count = 0
            host_invalidation_view_presentation_target_count = 0
            host_invalidation_window_metrics_target_count = 0
            host_invalidation_paint_only_target_count = 1
            slow_path_rebuild_count = 0
            painted_pixels = 100
            presented_surface_pixels = 1000
            gpu_draw_calls = 1
            gpu_visible_commands = 2
            gpu_visible_draw_items = 2
            gpu_timestamp_supported_present_count = 1
            gpu_time_sample_count = 1
            gpu_time_p50_us = 120
            gpu_time_p95_us = 120
            gpu_time_max_us = 120
            gpu_profile_latency_max_frames = 2
            software_fallback_present_count = 0
        }
        $writeReport = {
            [ordered]@{ scenarios = @([pscustomobject]$scenario); alerts = @() } |
                ConvertTo-Json -Depth 5 |
                Set-Content -LiteralPath (Join-Path $profileDir 'ui_hotspots.json') -Encoding UTF8
        }

        & $writeReport
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $true

        $scenario.host_invalidation_transaction_count = 2
        $scenario.host_invalidation_scope_count = 2
        $scenario.host_invalidation_full_target_count = 1
        & $writeReport
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'click') |
            Should Be $true

        $scenario.host_invalidation_transaction_count = 1
        $scenario.host_invalidation_scope_count = 1
        $scenario.host_invalidation_full_target_count = 0
        $scenario.presented_surface_pixels = 0
        & $writeReport
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $scenario.presented_surface_pixels = 50
        & $writeReport
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $scenario.presented_surface_pixels = 1000
        $scenario.gpu_time_sample_count = 0
        & $writeReport
        (Show-UiScenarioEvidence -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false
    }

    It "rejects incomplete click storms and accepts a complete source-bound run" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Resolve-InteractionScenarioName',
                'Test-InteractionProcessEvidence',
                'Test-UiInteractionEvidenceGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'click-storm-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $artifactPath = Join-Path $profileDir 'ui_interaction_evidence.json'
        $AutoClickCount = 1000
        $AutoPointerMoveCount = 0
        $interaction = [ordered]@{
            scenario = 'pointer_click_storm'
            requested_clicks = 1000
            completed_clicks = 999
            processor_time_delta_ms = 250
        }
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $interaction.completed_clicks = 1000
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $interaction.used_geometry = $true
        $interaction.targets = @(
            [ordered]@{
                target_id = 'template.document.MaterialLabButtons'
                target_kind = 'template_control'
                target_surface = 'document'
                source = 'ui_profile_geometry.json'
            }
        )
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $interaction.elapsed_ms = 4000
        $interaction.process_id = 4242
        $interaction.cpu_core_utilization_percent = 6.25
        $interaction.cpu_system_utilization_percent = 0.78125
        $interaction.logical_processor_count = 8
        $interaction.start_working_set_bytes = 1000000
        $interaction.end_working_set_bytes = 1100000
        $interaction.peak_working_set_bytes = 1200000
        $interaction.start_private_bytes = 800000
        $interaction.end_private_bytes = 850000
        $interaction.peak_private_bytes = 900000
        $interaction.quiescence_process_id = 4242
        $interaction.quiescence_requested_ms = 2000
        $interaction.quiescence_elapsed_ms = 2050
        $interaction.quiescence_working_set_bytes = 1100000
        $interaction.quiescence_private_bytes = 850000
        $interaction.quiescence_sampled = $true
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $true

        $interaction.peak_working_set_bytes = 1000000
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $interaction.peak_working_set_bytes = 1200000
        $interaction.used_geometry = $false
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $interaction.used_geometry = $true
        $interaction.targets[0].target_id = 'viewport_toolbar.document.SetDisplayMode'
        $interaction.targets[0].target_kind = 'viewport_toolbar_control'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8
        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'viewport_toolbar_click') |
            Should Be $true

        $interaction.targets[0].target_kind = 'template_control'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8
        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'viewport_toolbar_click') |
            Should Be $false
    }

    It "rejects pointer storms that are not bound to live geometry targets" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Resolve-InteractionScenarioName',
                'Test-InteractionProcessEvidence',
                'Test-UiInteractionEvidenceGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'pointer-storm-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $artifactPath = Join-Path $profileDir 'ui_interaction_evidence.json'
        $AutoClickCount = 0
        $AutoPointerMoveCount = 1000
        $interaction = [ordered]@{
            scenario = 'pointer_move_storm'
            requested_moves = 1000
            completed_moves = 1000
            processor_time_delta_ms = 180
        }
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_hover') |
            Should Be $false

        $interaction.used_geometry = $true
        $interaction.targets = @(
            [ordered]@{
                target_id = 'template.document.MaterialLabButtons'
                target_kind = 'template_control'
                target_surface = 'document'
                source = 'ui_profile_geometry.json'
            },
            [ordered]@{
                target_id = 'template.document.MaterialLabCheckboxes'
                target_kind = 'template_control'
                target_surface = 'document'
                source = 'ui_profile_geometry.json'
            }
        )
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_hover') |
            Should Be $false

        $interaction.elapsed_ms = 2000
        $interaction.process_id = 4242
        $interaction.cpu_core_utilization_percent = 9
        $interaction.cpu_system_utilization_percent = 1.125
        $interaction.logical_processor_count = 8
        $interaction.start_working_set_bytes = 1000000
        $interaction.end_working_set_bytes = 1050000
        $interaction.peak_working_set_bytes = 1100000
        $interaction.start_private_bytes = 800000
        $interaction.end_private_bytes = 825000
        $interaction.peak_private_bytes = 850000
        $interaction.quiescence_process_id = 4242
        $interaction.quiescence_requested_ms = 2000
        $interaction.quiescence_elapsed_ms = 2050
        $interaction.quiescence_working_set_bytes = 1050000
        $interaction.quiescence_private_bytes = 825000
        $interaction.quiescence_sampled = $true
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_hover') |
            Should Be $true

        $interaction.targets[1].source = 'ratio_fallback'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_hover') |
            Should Be $false
    }

    It "rejects incomplete wheel storms and accepts source-bound process evidence" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Resolve-InteractionScenarioName',
                'Test-InteractionProcessEvidence',
                'Test-UiInteractionEvidenceGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'wheel-storm-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $artifactPath = Join-Path $profileDir 'ui_interaction_evidence.json'
        $AutoClickCount = 0
        $AutoPointerMoveCount = 0
        $AutoWheelCount = 1000
        $interaction = [ordered]@{
            scenario = 'pointer_wheel_storm'
            process_id = 4242
            requested_wheel_events = 1000
            completed_wheel_events = 999
            wheel_delta = 120
            used_geometry = $true
            targets = @(
                [ordered]@{
                    target_id = 'layout.left_region'
                    target_kind = 'pane_region'
                    target_surface = 'left'
                    source = 'ui_profile_geometry.json'
                }
            )
            elapsed_ms = 2000
            processor_time_delta_ms = 180
            cpu_core_utilization_percent = 9
            cpu_system_utilization_percent = 1.125
            logical_processor_count = 8
            start_working_set_bytes = 1000000
            end_working_set_bytes = 1050000
            peak_working_set_bytes = 1100000
            start_private_bytes = 800000
            end_private_bytes = 825000
            peak_private_bytes = 850000
            quiescence_process_id = 4242
            quiescence_requested_ms = 2000
            quiescence_elapsed_ms = 2050
            quiescence_working_set_bytes = 1050000
            quiescence_private_bytes = 825000
            quiescence_sampled = $true
        }
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false

        $interaction.completed_wheel_events = 1000
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $true

        $interaction.targets[0].target_id = 'welcome.recent.viewport'
        $interaction.targets[0].target_kind = 'welcome_recent_viewport'
        $interaction.targets[0].target_surface = 'document'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8
        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'welcome_recent_scroll') |
            Should Be $true

        $interaction.targets[0].target_id = 'layout.left_region'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8
        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'welcome_recent_scroll') |
            Should Be $false

        $interaction.targets[0].target_id = 'layout.left_region'
        $interaction.targets[0].target_kind = 'pane_region'
        $interaction.targets[0].target_surface = 'left'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        $AutoPointerMoveCount = 1000
        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $true
        $AutoPointerMoveCount = 0

        $interaction.targets[0].source = 'ratio_fallback'
        [ordered]@{ interaction = $interaction } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $artifactPath -Encoding UTF8

        (Test-UiInteractionEvidenceGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false
    }

    It "requires routed hierarchy scroll counters with zero retained-authority rebuild work" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Resolve-InteractionScenarioName',
                'Test-InteractionProcessEvidence',
                'Test-HierarchyScrollCounterGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        (Resolve-InteractionScenarioName -ScenarioName 'hierarchy_scroll') | Should Be 'idle_hover'

        $profileDir = Join-Path $TestDrive 'hierarchy-scroll-counters'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [ordered]@{
            interaction = [ordered]@{
                scenario = 'pointer_wheel_storm'
                process_id = 4242
                requested_wheel_events = 1000
                completed_wheel_events = 1000
                elapsed_ms = 2000
                processor_time_delta_ms = 180
                cpu_core_utilization_percent = 9
                cpu_system_utilization_percent = 1.125
                logical_processor_count = 8
                start_working_set_bytes = 1000000
                end_working_set_bytes = 1050000
                peak_working_set_bytes = 1100000
                start_private_bytes = 800000
                end_private_bytes = 825000
                peak_private_bytes = 850000
                quiescence_process_id = 4242
                quiescence_requested_ms = 2000
                quiescence_elapsed_ms = 2050
                quiescence_working_set_bytes = 1050000
                quiescence_private_bytes = 825000
                quiescence_sampled = $true
            }
        } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $profileDir 'ui_interaction_evidence.json') -Encoding UTF8
        [ordered]@{ counters = @() } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-HierarchyScrollCounterGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false

        $counters = @(
            [ordered]@{ name = 'ui.idle_hover.hierarchy_scroll_dispatch_count'; value = 1000 },
            [ordered]@{ name = 'ui.idle_hover.hierarchy_surface_rebuild_count'; value = 1000 },
            [ordered]@{ name = 'ui.idle_hover.hierarchy_row_insert_count'; value = 50000 },
            [ordered]@{ name = 'ui.idle_hover.hierarchy_dispatcher_rebuild_count'; value = 1000 },
            [ordered]@{ name = 'ui.idle_hover.hierarchy_route_map_rebuild_count'; value = 1000 }
        )
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-HierarchyScrollCounterGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false

        for ($index = 1; $index -lt $counters.Count; $index++) {
            $counters[$index].value = 0
        }
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-HierarchyScrollCounterGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $true

        $counters[0].value = 999
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-HierarchyScrollCounterGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false

        $counters[0].value = 1000
        $counters[4].value = 999
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-HierarchyScrollCounterGate -ProfileDir $profileDir -ScenarioName 'hierarchy_scroll') |
            Should Be $false
    }

    It "requires routed welcome recent counters with zero retained-authority rebuild work" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Test-InteractionProcessEvidence',
                'Test-WelcomeRecentScrollCounterGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'welcome-recent-scroll-counters'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [ordered]@{
            interaction = [ordered]@{
                scenario = 'pointer_wheel_storm'
                process_id = 4242
                requested_wheel_events = 1000
                completed_wheel_events = 1000
                elapsed_ms = 2000
                processor_time_delta_ms = 180
                cpu_core_utilization_percent = 9
                cpu_system_utilization_percent = 1.125
                logical_processor_count = 8
                start_working_set_bytes = 1000000
                end_working_set_bytes = 1050000
                peak_working_set_bytes = 1100000
                start_private_bytes = 800000
                end_private_bytes = 825000
                peak_private_bytes = 850000
                quiescence_process_id = 4242
                quiescence_requested_ms = 2000
                quiescence_elapsed_ms = 2050
                quiescence_working_set_bytes = 1050000
                quiescence_private_bytes = 825000
                quiescence_sampled = $true
            }
        } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $profileDir 'ui_interaction_evidence.json') -Encoding UTF8

        $counters = @(
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_scroll_dispatch_count'; value = 1000 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_surface_rebuild_count'; value = 0 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_authority_rebuild_count'; value = 0 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_row_insert_count'; value = 0 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_geometry_patch_count'; value = 0 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_dispatcher_rebuild_count'; value = 0 },
            [ordered]@{ name = 'ui.idle_hover.welcome_recent_route_map_rebuild_count'; value = 0 }
        )
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        (Test-WelcomeRecentScrollCounterGate -ProfileDir $profileDir -ScenarioName 'welcome_recent_scroll') |
            Should Be $true

        $counters[1].value = 1
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8
        (Test-WelcomeRecentScrollCounterGate -ProfileDir $profileDir -ScenarioName 'welcome_recent_scroll') |
            Should Be $false

        $counters[1].value = 0
        $counters[0].value = 999
        [ordered]@{ counters = $counters } |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8
        (Test-WelcomeRecentScrollCounterGate -ProfileDir $profileDir -ScenarioName 'welcome_recent_scroll') |
            Should Be $false
    }

    It "fails resize evidence when visual assets or GPU images churn" {
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.gpu_image_upload_writes'
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.gpu_image_cache_admission_rejects'
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.visual_asset_cache_miss_count'
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.svg_tree_cache_miss_count'
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.duplicate_size_suppressed_count'
        $script:ProfileCaptureSource | Should Match 'ui\.window_resize\.duplicate_scale_suppressed_count'
        $script:ProfileCaptureSource | Should Match '\$imageUploadCount -le 1'
        $script:ProfileCaptureSource | Should Match '\$visualMissCount -eq 0'
        $script:ProfileCaptureSource | Should Match '\$svgMissCount -eq 0'
    }

    It "requires positive GPU image cache evidence during native resize" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0

        foreach ($functionName in @(
                'Resolve-InteractionScenarioName',
                'Test-InteractionProcessEvidence',
                'Test-WindowResizeCounterGate'
            )) {
            $functionAst = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $functionAst | Should Not BeNullOrEmpty
            Invoke-Expression $functionAst.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'resize-gpu-image-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [ordered]@{
            interaction = [ordered]@{
                process_id = 4242
                requested_steps = 24
                completed_steps = 24
                restored_original_extent = $true
                elapsed_ms = 3000
                processor_time_delta_ms = 750
                cpu_core_utilization_percent = 25
                cpu_system_utilization_percent = 3.125
                logical_processor_count = 8
                start_working_set_bytes = 1000000
                end_working_set_bytes = 1100000
                peak_working_set_bytes = 1200000
                start_private_bytes = 800000
                end_private_bytes = 850000
                peak_private_bytes = 900000
                quiescence_process_id = 4242
                quiescence_requested_ms = 2000
                quiescence_elapsed_ms = 2050
                quiescence_working_set_bytes = 1100000
                quiescence_private_bytes = 850000
                quiescence_sampled = $true
            }
        } | ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $profileDir 'ui_interaction_evidence.json') -Encoding UTF8

        $counters = @(
            [pscustomobject]@{ name = 'ui.window_resize.command_snapshot_build_count'; value = 1 },
            [pscustomobject]@{ name = 'ui.window_resize.command_snapshot_reuse_count'; value = 23 },
            [pscustomobject]@{ name = 'ui.window_resize.surface_reconfigure_count'; value = 24 }
        )
        $writeTimeline = {
            [ordered]@{ counters = @($counters) } |
                ConvertTo-Json -Depth 4 |
                Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8
        }

        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false

        $counters += [pscustomobject]@{
            name = 'ui.window_resize.gpu_image_vertices'
            value = 144
        }
        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false

        $counters += [pscustomobject]@{
            name = 'ui.window_resize.gpu_image_prepare_cache_hits'
            value = 24
        }
        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false

        $counters += [pscustomobject]@{
            name = 'ui.window_resize.visual_asset_cache_hit_count'
            value = 8
        }
        $counters += [pscustomobject]@{
            name = 'ui.window_resize.svg_tree_cache_memory_hit_count'
            value = 4
        }
        $counters += [pscustomobject]@{
            name = 'ui.window_resize.shell_drag_geometry_patch_count'
            value = 24
        }
        $counters += [pscustomobject]@{
            name = 'ui.window_resize.shell_drag_node_patch_count'
            value = 144
        }
        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $true

        $counters += [pscustomobject]@{
            name = 'ui.window_resize.shell_drag_authority_rebuild_count'
            value = 1
        }
        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false

        $counters[-1].value = 0
        ($counters | Where-Object { $_.name -eq 'ui.window_resize.shell_drag_geometry_patch_count' }).value = 0
        & $writeTimeline
        (Test-WindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false
    }

    It "rejects full visual cache invalidation for the non-visual asset refresh fixture" {
        $script:ProfileCaptureSource | Should Match 'ui\.asset_refresh\.visual_asset_targeted_invalidation_count'
        $script:ProfileCaptureSource | Should Match 'ui\.asset_refresh\.svg_tree_targeted_invalidation_count'
        $script:ProfileCaptureSource | Should Match '\$fullInvalidationCount -gt 0'
        $script:ProfileCaptureSource | Should Not Match 'return \$targetedInvalidationCount -gt 0'
    }

    It "exports watcher lag reconciliation work instead of hiding cache scans" {
        $script:ProfileCaptureSource | Should Match 'visual_asset_reconcile_source_visit_count'
        $script:ProfileCaptureSource | Should Match 'visual_asset_reconciled_invalidation_count'
        $script:ProfileCaptureSource | Should Match 'svg_tree_reconcile_source_visit_count'
        $script:ProfileCaptureSource | Should Match 'svg_tree_reconciled_invalidation_count'
    }

    It "exports compiled projection and vertex upload evidence" {
        $script:ProfileCaptureSource | Should Match 'gpu_compiled_draw_items'
        $script:ProfileCaptureSource | Should Match 'gpu_batch_plan_build_count'
        $script:ProfileCaptureSource | Should Match 'gpu_batch_plan_cache_hit_count'
        $script:ProfileCaptureSource | Should Match 'gpu_vertex_buffer_create_count'
        $script:ProfileCaptureSource | Should Match 'gpu_vertex_upload_bytes'
        $script:ProfileCaptureSource | Should Match 'gpu_retained_cache_copy_bytes'
    }

    It "exports submitted and retryable surface present outcomes" {
        $script:ProfileCaptureSource | Should Match 'ui\.surface\.submitted_count'
        $script:ProfileCaptureSource | Should Match 'ui\.surface\.retryable_no_submit_count'
        $script:ProfileCaptureSource | Should Match 'ui\.surface\.retry_backoff_ms'
        $script:ProfileCaptureSource | Should Match 'ui_surface_present_outcomes\.json'
        $script:ProfileCaptureSource | Should Match 'input_to_damage_sample_count'
        $script:ProfileCaptureSource | Should Match 'input_to_damage_p95_us'
        $script:ProfileCaptureSource | Should Match 'damage_to_submit_sample_count'
        $script:ProfileCaptureSource | Should Match 'damage_to_submit_p95_us'
        $script:ProfileCaptureSource | Should Match 'retry_backoff_sample_count'
    }

    It "summarizes surface present outcomes without counting retries as submissions" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0
        $exportFunction = $ast.Find({
                param($node)
                $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                    $node.Name -eq 'Export-UiSurfacePresentOutcomeEvidence'
            }, $true)
        $exportFunction | Should Not BeNullOrEmpty
        Invoke-Expression $exportFunction.Extent.Text

        $profileDir = Join-Path $TestDrive 'surface-outcomes'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [pscustomobject]@{
            recorder_retention = @(
                [pscustomobject]@{
                    frames = [pscustomobject]@{
                        capacity = 8; written = 0; overwritten = 0; retained = 0
                        oldest_sequence = $null; newest_sequence = $null
                    }
                    spans = [pscustomobject]@{
                        capacity = 16; written = 0; overwritten = 0; retained = 0
                        oldest_sequence = $null; newest_sequence = $null
                    }
                    counters = [pscustomobject]@{
                        capacity = 32; written = 13; overwritten = 0; retained = 13
                        oldest_sequence = 0; newest_sequence = 12
                    }
                }
            )
            counters = @(
                [pscustomobject]@{ name = 'ui.surface.submitted_count'; value = 2; timestamp_us = 100 },
                [pscustomobject]@{ name = 'ui.surface.retryable_no_submit_count'; value = 1; timestamp_us = 101 },
                [pscustomobject]@{ name = 'ui.surface.retry_backoff_ms'; value = 8; timestamp_us = 102 },
                [pscustomobject]@{ name = 'ui.surface.retry_backoff_ms'; value = 16; timestamp_us = 103 },
                [pscustomobject]@{ name = 'ui.input.outcome.damaged_sequence'; value = 1; timestamp_us = 110 },
                [pscustomobject]@{ name = 'ui.click.input_to_damage_us'; value = 80; timestamp_us = 110 },
                [pscustomobject]@{ name = 'ui.input.outcome.intentionally_no_damage_sequence'; value = 2; timestamp_us = 111 },
                [pscustomobject]@{ name = 'ui.input.outcome.damaged_sequence'; value = 3; timestamp_us = 112 },
                [pscustomobject]@{ name = 'ui.click.input_to_damage_us'; value = 140; timestamp_us = 112 },
                [pscustomobject]@{ name = 'ui.input.present_batch.first_sequence'; value = 1; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.input.present_batch.last_sequence'; value = 3; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.input.present_batch.damaged_count'; value = 2; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.click.damage_to_submit_us'; value = 180; timestamp_us = 120 }
            )
        } | ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        Export-UiSurfacePresentOutcomeEvidence -ProfileDir $profileDir

        $artifact = Get-Content -LiteralPath (Join-Path $profileDir 'ui_surface_present_outcomes.json') -Raw |
            ConvertFrom-Json
        $artifact.schema_version | Should Be 5
        $artifact.retention_complete | Should Be $true
        $artifact.counter_overwritten | Should Be 0
        $artifact.submitted_count | Should Be 2
        $artifact.retryable_no_submit_count | Should Be 1
        $artifact.input_to_damage_sample_count | Should Be 2
        $artifact.input_to_damage_p50_us | Should Be 80
        $artifact.input_to_damage_p95_us | Should Be 140
        $artifact.input_to_damage_p99_us | Should Be 140
        $artifact.input_to_damage_max_us | Should Be 140
        $artifact.damage_to_submit_sample_count | Should Be 1
        $artifact.damage_to_submit_p50_us | Should Be 180
        $artifact.damage_to_submit_p95_us | Should Be 180
        $artifact.damage_to_submit_p99_us | Should Be 180
        $artifact.damage_to_submit_max_us | Should Be 180
        $artifact.retry_backoff_sample_count | Should Be 2
        $artifact.retry_backoff_min_ms | Should Be 8
        $artifact.retry_backoff_max_ms | Should Be 16
        $artifact.retry_observed | Should Be $true
        $artifact.input_outcome_count | Should Be 3
        $artifact.damaged_input_outcome_count | Should Be 2
        $artifact.present_batch_count | Should Be 1
        $artifact.typed_input_outcome_complete | Should Be $true
    }

    It "requires both monotonic input and submit latency stages for interaction storms" {
        $tokens = $null
        $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            $script:ProfileCaptureScript,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0
        foreach ($functionName in @('Resolve-InteractionScenarioName', 'Test-UiSurfaceLatencyEvidenceGate')) {
            $definition = $ast.Find({
                    param($node)
                    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                        $node.Name -eq $functionName
                }, $true)
            $definition | Should Not BeNullOrEmpty
            Invoke-Expression $definition.Extent.Text
        }

        $profileDir = Join-Path $TestDrive 'latency-gate'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        $script:AutoClickCount = 1000
        $script:AutoPointerMoveCount = 0
        $artifact = [ordered]@{
            schema_version = 5
            retention_source_count = 1
            retention_complete = $true
            frame_overwritten = 0
            span_overwritten = 0
            counter_overwritten = 0
            input_outcome_count = 4
            damaged_input_outcome_count = 4
            intentionally_no_damage_input_outcome_count = 0
            rejected_input_outcome_count = 0
            present_batch_count = 4
            present_batch_damaged_count = 4
            typed_input_outcome_complete = $true
            input_to_damage_sample_count = 0
            input_to_damage_p50_us = 100
            input_to_damage_p95_us = 120
            input_to_damage_p99_us = 140
            input_to_damage_max_us = 160
            damage_to_submit_sample_count = 4
            damage_to_submit_p50_us = 140
            damage_to_submit_p95_us = 180
            damage_to_submit_p99_us = 200
            damage_to_submit_max_us = 220
        }
        $writeArtifact = {
            $artifact | ConvertTo-Json -Depth 4 |
                Set-Content -LiteralPath (Join-Path $profileDir 'ui_surface_present_outcomes.json') -Encoding UTF8
        }

        & $writeArtifact
        (Test-UiSurfaceLatencyEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $artifact.input_to_damage_sample_count = 4
        $artifact.input_to_damage_p95_us = 90
        & $writeArtifact
        (Test-UiSurfaceLatencyEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $false

        $artifact.input_to_damage_p95_us = 120
        & $writeArtifact
        (Test-UiSurfaceLatencyEvidenceGate -ProfileDir $profileDir -ScenarioName 'material_lab_click') |
            Should Be $true
    }

    It "keeps generated profile projects out of the repository target directory" {
        $script:ProfileCaptureSource | Should Match "profile-projects"
        $script:ProfileCaptureSource | Should Not Match "target\\zircon-profile-projects"
    }

    It "source-binds exact hierarchy N and actual wheel operations before the measured process starts" {
        $script:ProfileCaptureSource | Should Match '\[int\]\$HierarchyLogicalNodeCount = 0'
        $script:ProfileCaptureSource | Should Not Match '\$HierarchyDeltaNodeCount'
        $script:ProfileCaptureSource | Should Match 'New-ZirconUiHierarchyScaleFixture'
        $script:ProfileCaptureSource | Should Match 'Get-ScenarioRequestedWheelOperationCount'
        $script:ProfileCaptureSource | Should Match '-InputFixture \$inputFixture'
        $script:ProfileCaptureSource | Should Match 'hierarchy_logical_node_count = \$HierarchyLogicalNodeCount'
        $script:ProfileCaptureSource | Should Match 'requested_wheel_operation_count = \$requestedWheelOperationCount'
        $materializeIndex = $script:ProfileCaptureSource.LastIndexOf('$inputFixture = New-ZirconUiHierarchyScaleFixture')
        $enableCaptureIndex = $script:ProfileCaptureSource.LastIndexOf('$env:ZIRCON_PROFILE_CAPTURE = "1"')
        $exportManifestIndex = $script:ProfileCaptureSource.LastIndexOf('$sourceManifest = Export-ZirconProfileCaptureManifest')
        ($materializeIndex -ge 0) | Should Be $true
        ($materializeIndex -lt $enableCaptureIndex) | Should Be $true
        ($enableCaptureIndex -lt $exportManifestIndex) | Should Be $true
    }

    It "source-binds a real importable asset catalog scale input" {
        $script:ProfileCaptureSource | Should Match '\[int\]\$AssetCatalogItemCount = 0'
        $script:ProfileCaptureSource | Should Match 'New-ZirconUiAssetCatalogScaleFixture'
        $script:ProfileCaptureSource | Should Match 'asset_catalog_item_count = \$AssetCatalogItemCount'
        $script:ProfileCaptureSource | Should Match 'Asset catalog scale inputs are valid only for the asset_refresh scenario'
        $script:ProfileCaptureSource | Should Match '\$normalizedScenario -eq "asset_refresh" -and \$AssetCatalogItemCount -gt 0'
        $script:ProfileCaptureSource | Should Match 'return @\("--project", \$projectRoot\)'
        $script:ProfileCaptureSource | Should Match 'profile_catalog_asset_000001\.json'

        $materializeIndex = $script:ProfileCaptureSource.LastIndexOf('$inputFixture = New-ZirconUiAssetCatalogScaleFixture')
        $enableCaptureIndex = $script:ProfileCaptureSource.LastIndexOf('$env:ZIRCON_PROFILE_CAPTURE = "1"')
        $exportManifestIndex = $script:ProfileCaptureSource.LastIndexOf('$sourceManifest = Export-ZirconProfileCaptureManifest')
        ($materializeIndex -ge 0) | Should Be $true
        ($materializeIndex -lt $enableCaptureIndex) | Should Be $true
        ($enableCaptureIndex -lt $exportManifestIndex) | Should Be $true
    }

    It "exports verification screenshots beneath docs tests editor" {
        $script:ProfileCaptureSource | Should Match "profile-captures"
        $script:ProfileCaptureSource | Should Match "Export-VerificationScreenshots"
    }

    It "writes a source and binary bound manifest before capture" {
        $trackedSource = Join-Path $script:RepoRoot 'zircon_editor\src\ui\retained_host\app\host_lifecycle\recompute.rs'
        $editorExe = Join-Path $TestDrive 'editor.exe'
        $runtimeDll = Join-Path $TestDrive 'runtime.dll'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII

        $manifestPath = Export-ZirconProfileCaptureManifest `
            -ProfileDir (Join-Path $TestDrive 'profile') `
            -RepoRoot $script:RepoRoot `
            -OutputRoot 'E:\zircon-profiles' `
            -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
            -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
            -SessionId 'test-session' `
            -ScenarioName 'click' `
            -EditorExe $editorExe `
            -RuntimeDll $runtimeDll `
            -CaptureOptions @{ max_frames = 64 }

        (Test-Path -LiteralPath $manifestPath) | Should Be $true
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.schema_version | Should Be 2
        $manifest.session_id | Should Be 'test-session'
        $manifest.scenario | Should Be 'click'
        $manifest.capture.output_root | Should Be 'E:\zircon-profiles'
        $manifest.input_fixture | Should BeNullOrEmpty
        $manifest.capture.tool_files.Count | Should Be 7
        $captureToolPaths = @($manifest.capture.tool_files.relative_path)
        ($captureToolPaths -contains 'tools/ui-profile-capture.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/ui-profile-latency-evidence.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/ui-profile-process-evidence.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/ui-profile-native-resize.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/ui-profile-scale-fixture.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/profile-capture-paths.ps1') | Should Be $true
        ($captureToolPaths -contains 'tools/profile-capture-manifest.ps1') | Should Be $true
        $nativeInteractionTool = $manifest.capture.tool_files |
            Where-Object { $_.relative_path -eq 'tools/ui-profile-native-resize.ps1' } |
            Select-Object -First 1
        $nativeInteractionTool.sha256 |
            Should Be ((Get-FileHash -LiteralPath $script:ProfileNativeInteraction -Algorithm SHA256).Hash.ToLowerInvariant())
        $manifest.repository.critical_source_files.Count | Should Be 103
        $manifest.repository.critical_source_files[0].relative_path |
            Should Be 'zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs'
        $criticalSourcePaths = @($manifest.repository.critical_source_files.relative_path)
        foreach ($path in @(
                'zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs',
                'zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs',
                'zircon_editor/src/ui/retained_host/host_contract/window/event_loop/profile_capture.rs',
                'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs',
                'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs'
            )) {
            ($criticalSourcePaths -contains $path) | Should Be $true
        }
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry/body.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/entry.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_index.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/pixels.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/rendering.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/cache.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/ui_perf/counter_batch.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs') |
            Should Be $true
        foreach ($assetPointerSource in @(
            'zircon_editor/src/ui/retained_host/asset_pointer/common.rs',
            'zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs',
            'zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs',
            'zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs'
        )) {
            ($criticalSourcePaths -contains $assetPointerSource) | Should Be $true
        }
        foreach ($menuPointerSource in @(
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_dispatch_event.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_popup_items.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs',
            'zircon_editor/src/ui/retained_host/menu_pointer/register_handled_pointer_node.rs'
        )) {
            ($criticalSourcePaths -contains $menuPointerSource) | Should Be $true
        }
        foreach ($welcomePointerSource in @(
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_click.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_move.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_scroll.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_project_route.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs',
            'zircon_editor/src/ui/retained_host/welcome_recent_pointer/register_handled_pointer_node.rs'
        )) {
            ($criticalSourcePaths -contains $welcomePointerSource) | Should Be $true
        }
        foreach ($profileGeometrySource in @(
            'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs',
            'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_profile_controls.rs',
            'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/pane.rs',
            'zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/geometry.rs'
        )) {
            ($criticalSourcePaths -contains $profileGeometrySource) | Should Be $true
        }
        foreach ($viewportToolbarSource in @(
            'zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs',
            'zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/viewport_toolbar.rs',
            'zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/new.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs',
            'zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs'
        )) {
            ($criticalSourcePaths -contains $viewportToolbarSource) | Should Be $true
        }
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_scroll.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_click.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_move.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/register_handled_pointer_node.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/route_at_point.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs') |
            Should Be $true
        foreach ($shellDragSource in @(
            'zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs',
            'zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs',
            'zircon_editor/src/ui/retained_host/shell_pointer/common.rs',
            'zircon_editor/src/ui/retained_host/shell_pointer/drag_frames.rs',
            'zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs',
            'zircon_editor/src/ui/retained_host/shell_pointer/node_ids.rs'
        )) {
            ($criticalSourcePaths -contains $shellDragSource) | Should Be $true
        }
        ($criticalSourcePaths -contains 'zircon_runtime/src/ui/tree/node/scroll.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/ui/layout/pass/incremental.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/window/event_wake.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs') |
            Should Be $true
        foreach ($presenterLifecycleSource in @(
            'zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs',
            'zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs',
            'zircon_editor/src/ui/retained_host/host_contract/presenter/runtime_factory.rs',
            'zircon_editor/src/ui/retained_host/viewport/presenter_factory.rs'
        )) {
            ($criticalSourcePaths -contains $presenterLifecycleSource) | Should Be $true
        }
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_editor/src/ui/retained_host/app/assets/refresh.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/ui/surface/surface/rebuild.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime_interface/src/profiling.rs') |
            Should Be $true
        ($criticalSourcePaths -contains 'zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs') |
            Should Be $true
        $manifest.repository.critical_source_files[0].sha256 |
            Should Be ((Get-FileHash -LiteralPath $trackedSource -Algorithm SHA256).Hash.ToLowerInvariant())
        $manifest.binaries.editor.sha256 |
            Should Be ((Get-FileHash -LiteralPath $editorExe -Algorithm SHA256).Hash.ToLowerInvariant())
        $manifest.binaries.runtime.sha256 |
            Should Be ((Get-FileHash -LiteralPath $runtimeDll -Algorithm SHA256).Hash.ToLowerInvariant())
    }

    It "fails closed when Git metadata cannot be read" {
        $profileDir = Join-Path $TestDrive 'git-metadata-failure'
        $failure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $profileDir `
                -RepoRoot $TestDrive `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'git-failure' `
                -ScenarioName 'click' `
                -EditorExe (Join-Path $TestDrive 'editor.exe') `
                -RuntimeDll (Join-Path $TestDrive 'runtime.dll') `
                -CaptureOptions @{}
        }
        catch {
            $failure = $_
        }
        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match '^Source-bound profile capture requires a readable Git revision for:'
        (Test-Path -LiteralPath (Join-Path $profileDir 'source_manifest.json')) | Should Be $false
    }

    It "fails closed when Git status cannot be read" {
        $repoRoot = New-ProfileManifestTestRepository -Root (Join-Path $TestDrive 'status-failure-repository')
        $editorExe = Join-Path $TestDrive 'status-editor.exe'
        $runtimeDll = Join-Path $TestDrive 'status-runtime.dll'
        $profileDir = Join-Path $TestDrive 'git-status-failure'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII
        Set-Content -LiteralPath (Join-Path $repoRoot '.git\index') -Value 'invalid index' -Encoding ASCII

        $failure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $profileDir `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'git-status-failure' `
                -ScenarioName 'click' `
                -EditorExe $editorExe `
                -RuntimeDll $runtimeDll `
                -CaptureOptions @{}
        }
        catch {
            $failure = $_
        }
        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match '^Source-bound profile capture requires readable Git working-tree status for:'
        (Test-Path -LiteralPath (Join-Path $profileDir 'source_manifest.json')) | Should Be $false
    }

    It "fails closed when a required manifest file is missing" {
        $missingSource = (Get-ZirconProfileCriticalSourcePaths)[0]
        $repoRoot = New-ProfileManifestTestRepository `
            -Root (Join-Path $TestDrive 'source-failure-repository') `
            -OmitCriticalSource $missingSource
        $editorExe = Join-Path $TestDrive 'source-editor.exe'
        $runtimeDll = Join-Path $TestDrive 'source-runtime.dll'
        $profileDir = Join-Path $TestDrive 'source-failure'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII

        $sourceFailure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $profileDir `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'source-failure' `
                -ScenarioName 'click' `
                -EditorExe $editorExe `
                -RuntimeDll $runtimeDll `
                -CaptureOptions @{}
        }
        catch {
            $sourceFailure = $_
        }
        $sourceFailure | Should Not BeNullOrEmpty
        $sourceFailure.Exception.Message | Should Match '^Source-bound profile capture requires critical source file'
        (Test-Path -LiteralPath (Join-Path $profileDir 'source_manifest.json')) | Should Be $false
    }

    It "fails closed when a required capture tool is missing" {
        $missingTool = 'tools/ui-profile-native-resize.ps1'
        $repoRoot = New-ProfileManifestTestRepository `
            -Root (Join-Path $TestDrive 'tool-failure-repository') `
            -OmitCaptureTool $missingTool
        $editorExe = Join-Path $TestDrive 'tool-editor.exe'
        $runtimeDll = Join-Path $TestDrive 'tool-runtime.dll'
        $profileDir = Join-Path $TestDrive 'tool-failure'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII

        $toolFailure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $profileDir `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'tool-failure' `
                -ScenarioName 'click' `
                -EditorExe $editorExe `
                -RuntimeDll $runtimeDll `
                -CaptureOptions @{}
        }
        catch {
            $toolFailure = $_
        }
        $toolFailure | Should Not BeNullOrEmpty
        $toolFailure.Exception.Message | Should Match '^Source-bound profile capture requires capture tool'
        (Test-Path -LiteralPath (Join-Path $profileDir 'source_manifest.json')) | Should Be $false
    }

    It "fails closed when an editor or Runtime binary fingerprint is missing" {
        $repoRoot = New-ProfileManifestTestRepository -Root (Join-Path $TestDrive 'binary-failure-repository')
        $editorExe = Join-Path $TestDrive 'binary-editor.exe'
        $runtimeDll = Join-Path $TestDrive 'binary-runtime.dll'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII

        $missingEditorProfileDir = Join-Path $TestDrive 'missing-editor-binary'
        $missingEditorFailure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $missingEditorProfileDir `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'missing-editor' `
                -ScenarioName 'click' `
                -EditorExe (Join-Path $TestDrive 'missing-editor.exe') `
                -RuntimeDll $runtimeDll `
                -CaptureOptions @{}
        }
        catch {
            $missingEditorFailure = $_
        }
        $missingEditorFailure | Should Not BeNullOrEmpty
        $missingEditorFailure.Exception.Message | Should Match '^Source-bound profile capture requires editor binary fingerprint:'
        (Test-Path -LiteralPath (Join-Path $missingEditorProfileDir 'source_manifest.json')) | Should Be $false

        $missingRuntimeProfileDir = Join-Path $TestDrive 'missing-runtime-binary'
        $missingRuntimeFailure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir $missingRuntimeProfileDir `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'missing-runtime' `
                -ScenarioName 'click' `
                -EditorExe $editorExe `
                -RuntimeDll (Join-Path $TestDrive 'missing-runtime.dll') `
                -CaptureOptions @{}
        }
        catch {
            $missingRuntimeFailure = $_
        }
        $missingRuntimeFailure | Should Not BeNullOrEmpty
        $missingRuntimeFailure.Exception.Message | Should Match '^Source-bound profile capture requires Runtime binary fingerprint:'
        (Test-Path -LiteralPath (Join-Path $missingRuntimeProfileDir 'source_manifest.json')) | Should Be $false
    }

    It "fails closed when profiling binaries predate a critical source change" {
        $repoRoot = New-ProfileManifestTestRepository -Root (Join-Path $TestDrive 'stale-binary-repository')
        $editorExe = Join-Path $TestDrive 'stale-editor.exe'
        $runtimeDll = Join-Path $TestDrive 'stale-runtime.dll'
        Set-Content -LiteralPath $editorExe -Value 'editor binary' -Encoding ASCII
        Set-Content -LiteralPath $runtimeDll -Value 'runtime binary' -Encoding ASCII
        $staleWrite = (Get-Date).AddDays(-2)
        (Get-Item -LiteralPath $editorExe).LastWriteTimeUtc = $staleWrite.ToUniversalTime()
        (Get-Item -LiteralPath $runtimeDll).LastWriteTimeUtc = $staleWrite.ToUniversalTime()

        $failure = $null
        try {
            Export-ZirconProfileCaptureManifest `
                -ProfileDir (Join-Path $TestDrive 'stale-binary-profile') `
                -RepoRoot $repoRoot `
                -OutputRoot 'E:\zircon-profiles' `
                -VerificationScreenshotRoot 'E:\Git\ZirconEngine\docs\tests\editor\profile-captures' `
                -TargetDir 'E:\cargo-targets\zircon-editor-profile' `
                -SessionId 'stale-binary' `
                -ScenarioName 'window_resize' `
                -EditorExe $editorExe `
                -RuntimeDll $runtimeDll `
                -CaptureOptions @{}
        }
        catch {
            $failure = $_
        }
        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message |
            Should Match '^Source-bound profile capture requires binaries built after the newest critical source change:'
    }
}
