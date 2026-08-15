$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$reporter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1'
$originalTestMode = $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE

try {
    $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE = '1'
    . $reporter
}
finally {
    $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE = $originalTestMode
}

function New-ResourceManagementBaselineReportPlanFixture {
    return [pscustomobject][ordered]@{
        schema_version = 1
        workload_family = 'resource-management-query'
        source_fingerprint = 'A' * 64
        resource_kind = 'Data'
        scenarios = @(
            [pscustomobject][ordered]@{
                logical_id = 'data-000001-cold-open'
                mode = 'cold-open'
                project_role = 'baseline'
                data_asset_count = 1
                data_inventory_sha256 = 'B' * 64
                required_repetitions = 3
                queries = @(
                    [pscustomobject][ordered]@{
                        operation = 'scan'
                        query = [pscustomobject][ordered]@{ kind = 'Data'; state = 'any' }
                        expected_measurements = @(
                            'resource_management.scan.instances',
                            'resource_management.scan.matching_rows',
                            'resource_management.scan.rows_emitted',
                            'resource_management.scan.shard_candidate_checks',
                            'resource_management.scan.filtered_rows_skipped'
                        )
                    }
                )
            }
        )
    }
}

function New-ResourceManagementBaselineObservationFixture {
    param(
        [switch]$OmitRequiredCounter,
        [switch]$OmitFrameAssociation,
        [switch]$InvalidFrameAssociation,
        [switch]$EmptyFrameAssociation,
        [switch]$BooleanFrameAssociation
    )

    $samples = foreach ($attempt in 1..3) {
        $counters = [ordered]@{
            'resource_management.scan.instances' = 1
            'resource_management.scan.matching_rows' = 1
            'resource_management.scan.rows_emitted' = 1
            'resource_management.scan.shard_candidate_checks' = 64
            'resource_management.scan.filtered_rows_skipped' = 0
        }
        if ($OmitRequiredCounter) {
            $counters.Remove('resource_management.scan.shard_candidate_checks')
        }
        $query = [ordered]@{
            operation = 'scan'
            query = [pscustomobject][ordered]@{ kind = 'Data'; state = 'any' }
            elapsed_us = 100 * $attempt
            counters = [pscustomobject]$counters
        }
        if (-not $OmitFrameAssociation) {
            $query.frame_index = if ($InvalidFrameAssociation) {
                0.5
            }
            elseif ($EmptyFrameAssociation) {
                ''
            }
            elseif ($BooleanFrameAssociation) {
                $true
            }
            else {
                $attempt - 1
            }
            $query.timestamp_us = 1000 * $attempt
        }
        [pscustomobject][ordered]@{
            logical_id = 'data-000001-cold-open'
            attempt = $attempt
            data_inventory_sha256 = 'B' * 64
            process = [pscustomobject][ordered]@{
                cpu_time_ms = $attempt
                peak_working_set_bytes = 1048576
                allocation_proxy_bytes = 256
            }
            queries = @([pscustomobject]$query)
        }
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        workload_family = 'resource-management-query'
        source_fingerprint = 'A' * 64
        baseline_plan_sha256 = 'C' * 64
        samples = @($samples)
    }
}

Describe 'Resource-management baseline report' {
    It 'aggregates every required attempt and query without serializing physical project paths' {
        $report = ConvertTo-ResourceManagementBaselineReport `
            -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
            -BaselinePlanSha256 ('C' * 64) `
            -Observation (New-ResourceManagementBaselineObservationFixture) `
            -ObservationSha256 ('D' * 64)
        $serialized = $report | ConvertTo-Json -Depth 20
        $scenario = $report.scenarios[0]

        $report.schema_version | Should Be 1
        $report.workload_family | Should Be 'resource-management-query'
        $report.measurement_status | Should Be 'measured'
        $report.source_fingerprint | Should Be ('A' * 64)
        $report.baseline_plan_sha256 | Should Be ('C' * 64)
        $scenario.logical_id | Should Be 'data-000001-cold-open'
        $scenario.attempt_count | Should Be 3
        $scenario.queries[0].elapsed_us.median | Should Be 200
        $scenario.queries[0].counters['resource_management.scan.shard_candidate_checks'].median | Should Be 64
        $serialized | Should Not Match '[A-Za-z]:\\'
    }

    It 'rejects a required counter missing from any observation attempt' {
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation (New-ResourceManagementBaselineObservationFixture -OmitRequiredCounter) `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'missing required counter'
    }

    It 'rejects an observation query without its profiling frame association' {
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation (New-ResourceManagementBaselineObservationFixture -OmitFrameAssociation) `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'profiling frame association'
    }

    It 'rejects a non-integral profiling frame association' {
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation (New-ResourceManagementBaselineObservationFixture -InvalidFrameAssociation) `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'profiling frame association'
    }

    It 'rejects empty or boolean profiling frame associations' {
        foreach ($fixtureSwitch in @('EmptyFrameAssociation', 'BooleanFrameAssociation')) {
            {
                $arguments = @{}
                $arguments[$fixtureSwitch] = $true
                ConvertTo-ResourceManagementBaselineReport `
                    -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                    -BaselinePlanSha256 ('C' * 64) `
                    -Observation (New-ResourceManagementBaselineObservationFixture @arguments) `
                    -ObservationSha256 ('D' * 64)
            } | Should Throw 'profiling frame association'
        }
    }

    It 'rejects report output roots outside the approved E drive evidence directory' {
        { Assert-ResourceManagementBaselineReportOutputDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-reports\report' } |
            Should Throw 'mvp-resource-management-reports'
    }
}
