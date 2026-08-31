$ErrorActionPreference = 'Stop'

Describe 'UI profile idle-hover paint submission gate' {
    BeforeAll {
        $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

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
        $captureSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\ui-profile-capture.ps1') -Raw
    }

    It 'is part of the measured capture acceptance chain' {
        $captureSource | Should Match 'function Test-IdleHoverPaintSubmissionCounterGate'
        $captureSource | Should Match 'Test-ZirconIdleHoverPaintSubmissionCounterGate'
        $captureSource | Should Match '\$idleHoverPaintSubmissionOk = Test-IdleHoverPaintSubmissionCounterGate'
        $captureSource | Should Match '\$stableVisualAssetCacheOk -and \$idleHoverPaintSubmissionOk -and \$surfaceFramePublicationOk'
    }

    It 'ignores scenarios outside idle hover' {
        (Test-ZirconIdleHoverPaintSubmissionCounterGate `
                -ProfileDir (Join-Path $TestDrive 'missing') `
                -ScenarioName 'window_resize') | Should Be $true
    }

    It 'requires both timeline and submitted-input outcome evidence' {
        $profileDir = Join-Path $TestDrive 'missing-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

        (Test-ZirconIdleHoverPaintSubmissionCounterGate `
                -ProfileDir $profileDir `
                -ScenarioName 'idle_hover') | Should Be $false
    }

    It 'accepts bounded damage painting and rejects structural or unbounded work' {
        $profileDir = Join-Path $TestDrive 'hover-paint-evidence'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

        [ordered]@{
            schema_version = 5
            submitted_count = 2
            retryable_no_submit_count = 0
            damaged_input_outcome_count = 4
            present_batch_count = 2
            present_batch_damaged_count = 4
            typed_input_outcome_complete = $true
        } | ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $profileDir 'ui_surface_present_outcomes.json') -Encoding UTF8

        $counters = @(
            [pscustomobject]@{ name = 'ui.idle_hover.presentation_rebuild_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.presentation_structure_generation_change_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.presentation_generation_read_count'; value = 2 },
            [pscustomobject]@{ name = 'ui.idle_hover.presentation_snapshot_read_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.chrome_command_full_rebuild_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.chrome_command_patch_count'; value = 4 },
            [pscustomobject]@{ name = 'ui.idle_hover.full_paint_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.region_paint_count'; value = 2 },
            [pscustomobject]@{ name = 'ui.idle_hover.painted_pixels'; value = 200000 },
            [pscustomobject]@{ name = 'ui.idle_hover.presented_surface_pixels'; value = 2000000 },
            [pscustomobject]@{ name = 'ui.idle_hover.workbench_paint_index_query_count'; value = 4 },
            [pscustomobject]@{ name = 'ui.idle_hover.workbench_paint_index_candidate_count'; value = 96 },
            [pscustomobject]@{ name = 'ui.idle_hover.template_node_visit_count'; value = 128 },
            [pscustomobject]@{ name = 'ui.idle_hover.template_node_clone_count'; value = 4 },
            [pscustomobject]@{ name = 'ui.idle_hover.template_node_damage_reject_count'; value = 24 },
            [pscustomobject]@{ name = 'ui.idle_hover.fallback_sort_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.paint_index.query_scratch_growth_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.software_fallback_present_count'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_batch_plan_builds'; value = 1 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_batch_plan_cache_hits'; value = 1 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_command_visibility_scans'; value = 96 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_vertex_buffer_creates'; value = 2 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_text_shapes'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_text_renderer_builds'; value = 0 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_text_renderer_cache_hits'; value = 8 },
            [pscustomobject]@{ name = 'ui.idle_hover.gpu_text_prepare_failures'; value = 0 }
        )
        $writeTimeline = {
            [ordered]@{ counters = @($counters) } |
                ConvertTo-Json -Depth 4 |
                Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8
        }
        $assertGate = {
            & $writeTimeline
            Test-ZirconIdleHoverPaintSubmissionCounterGate `
                -ProfileDir $profileDir `
                -ScenarioName 'idle_hover'
        }

        (& $assertGate) | Should Be $true

        foreach ($mutation in @(
                @{ name = 'ui.idle_hover.presentation_rebuild_count'; value = 1 },
                @{ name = 'ui.idle_hover.presentation_structure_generation_change_count'; value = 1 },
                @{ name = 'ui.idle_hover.presentation_snapshot_read_count'; value = 1 },
                @{ name = 'ui.idle_hover.chrome_command_full_rebuild_count'; value = 1 },
                @{ name = 'ui.idle_hover.full_paint_count'; value = 1 },
                @{ name = 'ui.idle_hover.fallback_sort_count'; value = 1 },
                @{ name = 'ui.paint_index.query_scratch_growth_count'; value = 1 },
                @{ name = 'ui.idle_hover.software_fallback_present_count'; value = 1 },
                @{ name = 'ui.idle_hover.workbench_paint_index_candidate_count'; value = 1025 },
                @{ name = 'ui.idle_hover.template_node_visit_count'; value = 513 },
                @{ name = 'ui.idle_hover.gpu_batch_plan_builds'; value = 3 },
                @{ name = 'ui.idle_hover.gpu_command_visibility_scans'; value = 513 },
                @{ name = 'ui.idle_hover.gpu_vertex_buffer_creates'; value = 3 },
                @{ name = 'ui.idle_hover.gpu_text_shapes'; value = 1 },
                @{ name = 'ui.idle_hover.gpu_text_renderer_builds'; value = 1 },
                @{ name = 'ui.idle_hover.gpu_text_prepare_failures'; value = 1 }
            )) {
            $counter = $counters | Where-Object { $_.name -eq $mutation.name }
            $original = $counter.value
            $counter.value = $mutation.value
            (& $assertGate) | Should Be $false
            $counter.value = $original
        }

        ($counters | Where-Object { $_.name -eq 'ui.idle_hover.painted_pixels' }).value = 1000001
        (& $assertGate) | Should Be $false
    }
}
