$ErrorActionPreference = 'Stop'

Describe 'UI profile window resize structure gate' {
    BeforeAll {
        $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

        function Resolve-InteractionScenarioName {
            param([string]$ScenarioName)
            return $ScenarioName.Trim().ToLowerInvariant()
        }

        function Test-InteractionProcessEvidence {
            param(
                [object]$Interaction,
                [int64]$OperationCount,
                [double]$MaxCpuMsPerOperation
            )
            return $null -ne $Interaction -and $OperationCount -gt 0 -and $MaxCpuMsPerOperation -gt 0
        }

        function Get-UiCounterTotal {
            param(
                [object[]]$Counters,
                [string[]]$Names
            )
            return [double](
                $Counters |
                    Where-Object { $_.name -in $Names } |
                    Measure-Object -Property value -Sum
            ).Sum
        }

        . (Join-Path $repoRoot 'tools\ui-profile-counter-evidence.ps1')
    }

    It 'rejects a measured resize that rebuilds the host presentation' {
        $profileDir = Join-Path $TestDrive 'resize-structure-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

        [ordered]@{
            interaction = [ordered]@{
                requested_steps = 24
                completed_steps = 24
                restored_original_extent = $true
            }
        } | ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $profileDir 'ui_interaction_evidence.json') -Encoding UTF8

        $counters = @(
            [pscustomobject]@{ name = 'ui.window_resize.command_snapshot_build_count'; value = 1 },
            [pscustomobject]@{ name = 'ui.window_resize.command_snapshot_reuse_count'; value = 23 },
            [pscustomobject]@{ name = 'ui.window_resize.surface_reconfigure_count'; value = 24 },
            [pscustomobject]@{ name = 'ui.window_resize.gpu_image_vertices'; value = 144 },
            [pscustomobject]@{ name = 'ui.window_resize.gpu_image_prepare_cache_hits'; value = 24 },
            [pscustomobject]@{ name = 'ui.window_resize.visual_asset_cache_hit_count'; value = 8 },
            [pscustomobject]@{ name = 'ui.window_resize.svg_tree_cache_memory_hit_count'; value = 4 },
            [pscustomobject]@{ name = 'ui.window_resize.shell_drag_geometry_patch_count'; value = 24 },
            [pscustomobject]@{ name = 'ui.window_resize.shell_drag_node_patch_count'; value = 144 },
            [pscustomobject]@{ name = 'ui.window_resize.presentation_rebuild_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.shell_presentation_build_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.host_scene_build_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.pane_projection_build_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.presentation_structure_generation_change_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.template_projection_layout_measure_probe_node_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.window_resize.template_projection_layout_arrange_probe_node_count'; value = 192 },
            [pscustomobject]@{ name = 'ui.window_resize.asset_pointer_snapshot_clone_count'; value = 0 }
        )
        $writeTimeline = {
            [ordered]@{ counters = @($counters) } |
                ConvertTo-Json -Depth 4 |
                Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8
        }

        & $writeTimeline
        (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $true

        ($counters | Where-Object { $_.name -eq 'ui.window_resize.presentation_rebuild_count' }).value = 1
        & $writeTimeline
        (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false

        ($counters | Where-Object { $_.name -eq 'ui.window_resize.presentation_rebuild_count' }).value = 0
        foreach ($counterName in @(
                'ui.window_resize.shell_presentation_build_count',
                'ui.window_resize.host_scene_build_count',
                'ui.window_resize.pane_projection_build_count',
                'ui.window_resize.presentation_structure_generation_change_count'
            )) {
            ($counters | Where-Object { $_.name -eq $counterName }).value = 1
            & $writeTimeline
            (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
                Should Be $false
            ($counters | Where-Object { $_.name -eq $counterName }).value = 0
        }

        ($counters | Where-Object { $_.name -eq 'ui.window_resize.template_projection_layout_measure_probe_node_count' }).value = 1
        & $writeTimeline
        (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false
        ($counters | Where-Object { $_.name -eq 'ui.window_resize.template_projection_layout_measure_probe_node_count' }).value = 0

        ($counters | Where-Object { $_.name -eq 'ui.window_resize.template_projection_layout_arrange_probe_node_count' }).value = 1537
        & $writeTimeline
        (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false
        ($counters | Where-Object { $_.name -eq 'ui.window_resize.template_projection_layout_arrange_probe_node_count' }).value = 192

        ($counters | Where-Object { $_.name -eq 'ui.window_resize.asset_pointer_snapshot_clone_count' }).value = 1
        & $writeTimeline
        (Test-ZirconWindowResizeCounterGate -ProfileDir $profileDir -ScenarioName 'window_resize') |
            Should Be $false
    }
}
