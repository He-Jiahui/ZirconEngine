$script:HierarchyFilterMetrics = Join-Path $PSScriptRoot "..\ui-profile-hierarchy-filter-metrics.ps1"
$script:ProfileCaptureScript = Join-Path $PSScriptRoot "..\ui-profile-capture.ps1"
$script:ProfileCaptureManifest = Join-Path $PSScriptRoot "..\profile-capture-manifest.ps1"
$script:HierarchyFilterMetricsFixtureRoot = Join-Path `
    'E:\zircon-profiles' `
    ("test-fixtures\hierarchy-filter-metrics-" + [Guid]::NewGuid().ToString('N'))

. $script:HierarchyFilterMetrics

function Write-HierarchyFilterMetricsTimeline {
    param(
        [string]$ProfileDir,
        [switch]$OmitVisibleRowCount
    )

    $counters = @(
        [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_projection_invocation_count'; value = 1 },
        [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_source_row_count'; value = 1000 },
        [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_name_match_count'; value = 0 },
        [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_ancestor_link_count'; value = 0 },
        [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_visible_row_count'; value = 0 }
    )
    if ($OmitVisibleRowCount) {
        $counters = @($counters | Where-Object { $_.name -ne 'hierarchy_filter_visible_row_count' })
    }
    [ordered]@{
        spans = @(
            [ordered]@{ stream = 'editor'; category = 'hierarchy'; name = 'filter_projection'; duration_us = 12 },
            [ordered]@{ stream = 'editor'; category = 'hierarchy'; name = 'filter_projection'; duration_us = 48 }
        )
        counters = $counters
    } | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $ProfileDir 'timeline.zrtrace.json') -Encoding UTF8
}

Describe "ui-profile hierarchy filter metrics" {
    BeforeAll {
        New-Item -ItemType Directory -Force -Path $script:HierarchyFilterMetricsFixtureRoot | Out-Null
    }

    AfterAll {
        if (Test-Path -LiteralPath $script:HierarchyFilterMetricsFixtureRoot) {
            Remove-Item -LiteralPath $script:HierarchyFilterMetricsFixtureRoot -Recurse -Force
        }
    }

    It "exports projection timing and retains zero-valued hierarchy counters" {
        $profileDir = Join-Path $script:HierarchyFilterMetricsFixtureRoot 'zero-match'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        Write-HierarchyFilterMetricsTimeline -ProfileDir $profileDir

        $metrics = Export-ZirconHierarchyFilterMetrics -ProfileDir $profileDir

        $metrics.projection_duration_us.sample_count | Should Be 2
        $metrics.projection_duration_us.p50 | Should Be 12
        $metrics.projection_duration_us.p95 | Should Be 48
        $metrics.counters.Count | Should Be 5
        (@($metrics.counters | Where-Object {
                    $_.name -eq 'hierarchy_filter_name_match_count'
                })[0].values.min) | Should Be 0
        (Test-ZirconHierarchyFilterMetricsGate -ProfileDir $profileDir -ScenarioName 'hierarchy_filter') |
            Should Be $true
    }

    It "rejects a hierarchy filter trace that omits a structural counter" {
        $profileDir = Join-Path $script:HierarchyFilterMetricsFixtureRoot 'missing-counter'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        Write-HierarchyFilterMetricsTimeline -ProfileDir $profileDir -OmitVisibleRowCount
        Export-ZirconHierarchyFilterMetrics -ProfileDir $profileDir | Out-Null

        (Test-ZirconHierarchyFilterMetricsGate -ProfileDir $profileDir -ScenarioName 'hierarchy_filter') |
            Should Be $false
    }

    It "does not treat absent trace values as zero-valued metric evidence" {
        $profileDir = Join-Path $script:HierarchyFilterMetricsFixtureRoot 'absent-values'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [ordered]@{
            spans = @(
                [ordered]@{ stream = 'editor'; category = 'hierarchy'; name = 'filter_projection'; duration_us = 25 },
                [ordered]@{ stream = 'editor'; category = 'hierarchy'; name = 'filter_projection' }
            )
            counters = @(
                [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_projection_invocation_count'; value = 1 },
                [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_source_row_count'; value = 1000 },
                [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_name_match_count'; value = 0 },
                [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_ancestor_link_count'; value = 0 },
                [ordered]@{ stream = 'editor'; name = 'hierarchy_filter_visible_row_count' }
            )
        } | ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        $metrics = Export-ZirconHierarchyFilterMetrics -ProfileDir $profileDir

        $metrics.projection_duration_us.sample_count | Should Be 1
        ($metrics.counters | Where-Object {
                $_.name -eq 'hierarchy_filter_visible_row_count'
            }).Count | Should Be 0
        (Test-ZirconHierarchyFilterMetricsGate -ProfileDir $profileDir -ScenarioName 'hierarchy_filter') |
            Should Be $false
    }

    It "rejects nonfinite and negative metric values without dropping legitimate zeroes" {
        $summary = Get-ZirconHierarchyFilterMetricSummary -Values @(0, 25, -1, 'NaN', 'Infinity')

        $summary.sample_count | Should Be 2
        $summary.min | Should Be 0
        $summary.max | Should Be 25
    }

    It "binds hierarchy metrics export and strict gate into the managed capture manifest" {
        $capture = Get-Content -LiteralPath $script:ProfileCaptureScript -Raw
        $manifest = Get-Content -LiteralPath $script:ProfileCaptureManifest -Raw

        $capture | Should Match 'ui-profile-hierarchy-filter-metrics\.ps1'
        $capture | Should Match 'Export-ZirconHierarchyFilterMetrics'
        $capture | Should Match 'Test-ZirconHierarchyFilterMetricsGate'
        $capture | Should Match '\$hierarchyFilterMetricsOk'
        $manifest | Should Match 'tools/ui-profile-hierarchy-filter-metrics\.ps1'
    }

    It "keeps the metrics helper inside the exact source-bound tool set" {
        . $script:ProfileCaptureManifest

        $toolPaths = @(Get-ZirconProfileCaptureToolPaths)
        $expectedToolPaths = @(
            'tools/ui-profile-capture.ps1',
            'tools/ui-profile-scenarios.ps1',
            'tools/ui-profile-latency-evidence.ps1',
            'tools/ui-profile-process-evidence.ps1',
            'tools/ui-profile-counter-evidence.ps1',
            'tools/ui-profile-workbench-pointer-evidence.ps1',
            'tools/ui-profile-native-resize.ps1',
            'tools/ui-profile-hierarchy-filter-input.ps1',
            'tools/ui-profile-hierarchy-filter-metrics.ps1',
            'tools/ui-profile-scale-fixture.ps1',
            'tools/ui-profile-surface-pipeline-metrics.ps1',
            'tools/ui-profile-chrome-paint-metrics.ps1',
            'tools/ui-profile-machine-manifest.ps1',
            'tools/performance-machine-manifest.ps1',
            'tools/profile-capture-paths.ps1',
            'tools/ui-profile-product-directory.ps1',
            'tools/profile-capture-manifest.ps1'
        )

        $toolPaths.Count | Should Be $expectedToolPaths.Count
        @($toolPaths | Where-Object { $_ -notin $expectedToolPaths }).Count | Should Be 0
        @($expectedToolPaths | Where-Object { $_ -notin $toolPaths }).Count | Should Be 0
    }
}
