$script:PressureScript = Join-Path $PSScriptRoot "..\ui-window-metrics-pane-clone-pressure.ps1"
if (Test-Path -LiteralPath $script:PressureScript) {
    . $script:PressureScript
}

Describe "UI window-metrics semantic pane clone pressure" {
    It "exports a source-bound lower-bound resize pressure receipt" {
        Get-Command Export-ZirconUiWindowMetricsPaneClonePressure -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-window-metrics-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $scenePath = Join-Path $fixtureRoot `
                "zircon_editor\src\ui\retained_host\ui\apply_presentation\scene_conversion.rs"
            $hostRootPath = Join-Path $fixtureRoot `
                "zircon_editor\src\ui\retained_host\host_contract\data\host_root.rs"
            New-Item -ItemType Directory -Path (Split-Path -Parent $scenePath) -Force |
                Out-Null
            New-Item -ItemType Directory -Path (Split-Path -Parent $hostRootPath) -Force |
                Out-Null

            @'
fn geometry(current: &HostWindowSceneData) {
    keep(current.left_dock.pane.clone());
    keep(current.document_dock.pane.clone());
    keep(current.right_dock.pane.clone());
    keep(current.bottom_dock.pane.clone());
    keep(candidate.active_pane.clone());
}
'@ | Set-Content -LiteralPath $scenePath -Encoding UTF8
            @'
fn apply_to(current: &HostWindowPresentationData) {
    next.host_scene_data.left_dock.pane = current.host_scene_data.left_dock.pane.clone();
    next.host_scene_data.document_dock.pane = current.host_scene_data.document_dock.pane.clone();
    next.host_scene_data.right_dock.pane = current.host_scene_data.right_dock.pane.clone();
    next.host_scene_data.bottom_dock.pane = current.host_scene_data.bottom_dock.pane.clone();
}
'@ | Set-Content -LiteralPath $hostRootPath -Encoding UTF8

            $receipt = Export-ZirconUiWindowMetricsPaneClonePressure `
                -RepoRoot $fixtureRoot `
                -OutputDirectory $outputRoot `
                -FrameCount 120 `
                -FloatingWindowCount 2 `
                -EstimatedPanePayloadBytes 1048576 `
                -ExpectedFloatingWindowCloneSites 1

            $report = Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            $report.schema_version | Should Be 1
            $report.source_binding.manifest_sha256.Length | Should Be 64
            $report.source_evidence.fixed_dock_clone_sites.scene_conversion | Should Be 4
            $report.source_evidence.fixed_dock_clone_sites.geometry_apply | Should Be 4
            $report.source_evidence.floating_window_clone_site_count | Should Be 1
            $report.legacy_model.semantic_pane_clones_per_frame | Should Be 10
            $report.legacy_model.semantic_pane_clones_total | Should Be 1200
            $report.legacy_model.estimated_semantic_clone_bytes_total | Should Be 1258291200
            $report.target_model.semantic_pane_clones_per_frame | Should Be 0
            $report.target_model.semantic_pane_clones_total | Should Be 0
            $report.target_model.estimated_semantic_clone_bytes_total | Should Be 0
            $report.modeled_reduction.semantic_pane_clone_count | Should Be 1200
            Test-Path -LiteralPath $receipt.json_path | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "rejects unexpected clone-source drift instead of rewriting the baseline" {
        $fixtureRoot = "E:\zircon-profiles\pester-ui-window-metrics-drift-$([guid]::NewGuid().ToString('N'))"
        try {
            $scenePath = Join-Path $fixtureRoot `
                "zircon_editor\src\ui\retained_host\ui\apply_presentation\scene_conversion.rs"
            $hostRootPath = Join-Path $fixtureRoot `
                "zircon_editor\src\ui\retained_host\host_contract\data\host_root.rs"
            New-Item -ItemType Directory -Path (Split-Path -Parent $scenePath) -Force |
                Out-Null
            New-Item -ItemType Directory -Path (Split-Path -Parent $hostRootPath) -Force |
                Out-Null

            @'
fn geometry(current: &HostWindowSceneData) {
    keep(current.left_dock.pane.clone());
    keep(current.document_dock.pane.clone());
    keep(current.right_dock.pane.clone());
    keep(candidate.active_pane.clone());
}
'@ | Set-Content -LiteralPath $scenePath -Encoding UTF8
            @'
fn apply_to(current: &HostWindowPresentationData) {
    next.host_scene_data.left_dock.pane = current.host_scene_data.left_dock.pane.clone();
    next.host_scene_data.document_dock.pane = current.host_scene_data.document_dock.pane.clone();
    next.host_scene_data.right_dock.pane = current.host_scene_data.right_dock.pane.clone();
    next.host_scene_data.bottom_dock.pane = current.host_scene_data.bottom_dock.pane.clone();
}
'@ | Set-Content -LiteralPath $hostRootPath -Encoding UTF8

            {
                Export-ZirconUiWindowMetricsPaneClonePressure `
                    -RepoRoot $fixtureRoot `
                    -OutputDirectory (Join-Path $fixtureRoot "evidence") `
                    -ExpectedSceneDockCloneSites 4 `
                    -ExpectedGeometryApplyCloneSites 4 `
                    -ExpectedFloatingWindowCloneSites 1
            } | Should Throw "WindowMetrics pane clone source contract drift: scene_conversion=3 expected=4; geometry_apply=4 expected=4; floating=1 expected=1."
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "rejects output paths on the system drive" {
        Get-Command Export-ZirconUiWindowMetricsPaneClonePressure -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        {
            Export-ZirconUiWindowMetricsPaneClonePressure `
                -RepoRoot "E:\Git\ZirconEngine" `
                -OutputDirectory "C:\temp\zircon-ui-pressure"
        } | Should Throw "UI pressure evidence must be written outside the system drive (C:\)."
    }
}
