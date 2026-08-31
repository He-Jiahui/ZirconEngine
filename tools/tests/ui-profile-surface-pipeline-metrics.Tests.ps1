$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:SurfacePipelineMetrics =
    Join-Path $script:RepoRoot "tools\ui-profile-surface-pipeline-metrics.ps1"

if (Test-Path -LiteralPath $script:SurfacePipelineMetrics) {
    . $script:SurfacePipelineMetrics
}

function Write-SurfacePipelineTimelineFixture {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProfileDir,
        [Parameter(Mandatory = $true)]
        [object[]]$Counters
    )

    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    [ordered]@{
        counters = $Counters
        spans = @()
    } | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $ProfileDir "timeline.zrtrace.json") -Encoding UTF8
}

function New-RuntimeCounter {
    param([string]$Name, [double]$Value)

    return [pscustomobject]@{
        stream = "runtime"
        name = $Name
        value = $Value
    }
}

Describe "UI surface pipeline metrics contract" {
    It "exports percentile stage durations and authoritative work totals" {
        Get-Command Export-ZirconUiSurfacePipelineMetrics -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $profileDir = Join-Path $TestDrive "complete"
        Write-SurfacePipelineTimelineFixture -ProfileDir $profileDir -Counters @(
            New-RuntimeCounter "ui.surface_rebuild.total_elapsed_us" 200
            New-RuntimeCounter "ui.surface_rebuild.total_elapsed_us" 400
            New-RuntimeCounter "ui.surface_rebuild.layout_elapsed_us" 50
            New-RuntimeCounter "ui.surface_rebuild.layout_elapsed_us" 100
            New-RuntimeCounter "ui.surface_rebuild.post_layout_elapsed_us" 20
            New-RuntimeCounter "ui.surface_rebuild.base_picking_elapsed_us" 10
            New-RuntimeCounter "ui.surface_rebuild.render_extract_elapsed_us" 60
            New-RuntimeCounter "ui.surface_projected_hit.rebuild_elapsed_us" 15
            New-RuntimeCounter "ui.surface_projected_hit.patch_elapsed_us" 5
            New-RuntimeCounter "ui.navigation_index.rebuild_elapsed_us" 25
            New-RuntimeCounter "ui.surface_frame.publication_elapsed_us" 35
            New-RuntimeCounter "ui.surface_frame.publication_elapsed_us" 45
            New-RuntimeCounter "ui.surface_rebuild.dirty_node_count" 2
            New-RuntimeCounter "ui.surface_rebuild.dirty_node_count" 4
        )

        $path = Export-ZirconUiSurfacePipelineMetrics -ProfileDir $profileDir
        $metrics = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json

        $path | Should Be (Join-Path $profileDir "ui_surface_pipeline_metrics.json")
        $metrics.schema_version | Should Be 1
        $metrics.stage_duration_us.surface_rebuild.sample_count | Should Be 2
        $metrics.stage_duration_us.surface_rebuild.p50 | Should Be 200
        $metrics.stage_duration_us.surface_rebuild.p95 | Should Be 400
        $metrics.stage_duration_us.surface_rebuild.max | Should Be 400
        $metrics.stage_duration_us.surface_rebuild.sum | Should Be 600
        $metrics.stage_duration_us.frame_publication.sample_count | Should Be 2
        $metrics.work.dirty_node_count.sample_count | Should Be 2
        $metrics.work.dirty_node_count.sum | Should Be 6
    }

    It "keeps an unobserved stage empty instead of manufacturing zero work" {
        $profileDir = Join-Path $TestDrive "partial"
        Write-SurfacePipelineTimelineFixture -ProfileDir $profileDir -Counters @(
            New-RuntimeCounter "ui.surface_rebuild.total_elapsed_us" 125
        )

        $path = Export-ZirconUiSurfacePipelineMetrics -ProfileDir $profileDir
        $metrics = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json

        $metrics.stage_duration_us.surface_rebuild.sample_count | Should Be 1
        $metrics.stage_duration_us.navigation_rebuild.sample_count | Should Be 0
        $metrics.stage_duration_us.navigation_rebuild.p50 | Should BeNullOrEmpty
        $metrics.work.dirty_node_count.sample_count | Should Be 0
    }
}
