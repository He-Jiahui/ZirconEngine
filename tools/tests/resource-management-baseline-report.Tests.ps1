$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$reporter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1'
$evidenceModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementJsonEvidence.psm1'
$observationContextModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementObservationContext.psm1'
$originalTestMode = $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE

try {
    $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE = '1'
    . $reporter
}
finally {
    $env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE = $originalTestMode
}

function New-ResourceManagementBaselineReportPlanFixture {
    $workloadSnapshot = Get-ResourceManagementWorkloadRegistrySnapshot
    return [pscustomobject][ordered]@{
        schema_version = 3
        workload_family = 'resource-management-query'
        workload_profile_id = 'json-data-flat-v1'
        workload_registry_receipt = $workloadSnapshot.receipt
        source_fingerprint = 'A' * 64
        resource_kind = 'Data'
        statistical_policy = [pscustomobject][ordered]@{
            warmup_repetitions = 3
            measurement_repetitions = 20
            minimum_sample_count = 20
            confidence_level = 0.95
            maximum_coefficient_of_variation = 0.10
            maximum_relative_margin_of_error = 0.10
        }
        scenarios = @(
            [pscustomobject][ordered]@{
                logical_id = 'data-000001-cold-open'
                mode = 'cold-open'
                project_role = 'baseline'
                process_lifecycle = 'fresh-process'
                data_asset_count = 1
                data_inventory_sha256 = 'B' * 64
                required_repetitions = 23
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

    $samples = foreach ($attempt in 1..23) {
        $isWarmup = $attempt -le 3
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
            elapsed_us = if ($isWarmup) { 1000 - (100 * $attempt) } else { 100 + ((($attempt % 5) - 2) * 0.5) }
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
            sample_phase = if ($isWarmup) { 'warmup' } else { 'measurement' }
            data_inventory_sha256 = 'B' * 64
            process_context = [pscustomobject][ordered]@{
                schema_version = 1
                context_kind = 'zircon.resource-management-sample-process-context'
                process_id = 4200 + $attempt
                process_created_utc = ('2026-08-26T09:00:{0:D2}.0000000Z' -f $attempt)
                trace_id = ('resource-trace-{0:D3}' -f $attempt)
                first_frame_index = 0
                last_frame_index = 22
            }
            execution_protocol = [pscustomobject][ordered]@{
                schema_version = 1
                protocol_kind = 'zircon.resource-management-sample-protocol'
                sequence_ordinal = $attempt
                cache_state = 'cold'
                cache_action = 'purge'
                cache_receipt_sha256 = ('{0:X64}' -f $attempt)
                quiescence_receipt_sha256 = ('{0:X64}' -f (100 + $attempt))
                quiescence_process_id = 4200 + $attempt
            }
            process = [pscustomobject][ordered]@{
                cpu_time_ms = if ($isWarmup) { 100 - $attempt } else { 10 + ((($attempt % 5) - 2) * 0.05) }
                peak_working_set_bytes = 1048576
                allocation_proxy_bytes = 256
            }
            queries = @([pscustomobject]$query)
        }
    }
    return [pscustomobject][ordered]@{
        schema_version = 3
        workload_family = 'resource-management-query'
        source_fingerprint = 'A' * 64
        baseline_plan_sha256 = 'C' * 64
        observation_context = [pscustomobject][ordered]@{
            schema_version = 1
            context_kind = 'zircon.resource-management-observation-context'
            product_receipt = [pscustomobject][ordered]@{
                schema_version = 1
                receipt_kind = 'zircon.mvp-product-receipt'
                receipt_id = 'E' * 64
                source_fingerprint = 'A' * 64
                product_role = 'editor'
                executable_sha256 = 'F' * 64
                build_set_id = '1' * 64
            }
            run = [pscustomobject][ordered]@{
                run_id = 'resource-baseline-run-001'
            }
            machine = [pscustomobject][ordered]@{
                machine_id_sha256 = '2' * 64
                cpu_model = 'Zircon Test CPU'
                logical_processor_count = 16
                physical_memory_bytes = 34359738368
                os_name = 'Windows'
                os_version = '10.0.26100'
                architecture = 'x64'
            }
            collector = [pscustomobject][ordered]@{
                collector_id = 'zircon-resource-profile'
                collector_version = '1.0.0'
                clock_domain = 'zircon-profile-microseconds'
            }
        }
        execution_protocol = [pscustomobject][ordered]@{
            schema_version = 1
            protocol_kind = 'zircon.resource-management-execution-protocol'
            randomization_algorithm = 'fisher-yates-sha256-v1'
            randomization_seed_sha256 = '3' * 64
            order_receipt_sha256 = '4' * 64
            cache_scope = 'os+ddc+resource-index'
            quiescence_policy_id = 'mvp-resource-quiescence-v1'
        }
        samples = @($samples)
    }
}

Describe 'Resource-management baseline report' {
    It 'encodes JSON evidence SHA-256 values through one fixed-size uppercase buffer' {
        $evidencePath = Join-Path $TestDrive 'resource-management-evidence.json'
        [IO.File]::WriteAllBytes($evidencePath, ([Text.UTF8Encoding]::new($false)).GetBytes('{"fixture":true}'))

        $evidence = Get-ResourceManagementReportJsonEvidence `
            -Path $evidencePath `
            -Label 'Resource-management evidence fixture'
        $reporterSource = Get-Content -Raw $reporter
        $evidenceSource = Get-Content -Raw $evidenceModule

        $evidence.sha256 | Should Be 'FFBC2DFC402782325DA71132100E74FF511D1585DD80E4EA196ED4BCACE3FEF2'
        $evidenceSource | Should Match '\[char\[\]\]::new\(\$hashBytes.Length \* 2\)'
        $evidenceSource | Should Not Match 'ComputeHash\(\$bytes\) \| ForEach-Object'
        $evidenceSource | Should Not Match "ToString\('X2'\)"
        $reporterSource | Should Match 'Get-ResourceManagementJsonEvidence'
        $reporterSource | Should Not Match '\[IO\.File\]::Open'
        $reporterSource | Should Not Match 'ComputeHash'
        $reporterSource | Should Not Match '\[IO\.File\]::ReadAllBytes'
    }

    It 'rejects JSON evidence above its caller-owned byte budget' {
        $evidencePath = Join-Path $TestDrive 'oversized-resource-observation.json'
        [IO.File]::WriteAllText(
            $evidencePath,
            ('{"payload":"' + ('x' * 64) + '"}'),
            [Text.UTF8Encoding]::new($false))

        {
            Get-ResourceManagementReportJsonEvidence `
                -Path $evidencePath `
                -Label 'Oversized resource observation' `
                -MaximumBytes 32
        } | Should Throw 'byte budget of 32 bytes'
    }

    It 'aggregates every required attempt and query without serializing physical project paths' {
        $report = ConvertTo-ResourceManagementBaselineReport `
            -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
            -BaselinePlanSha256 ('C' * 64) `
            -Observation (New-ResourceManagementBaselineObservationFixture) `
            -ObservationSha256 ('D' * 64)
        $serialized = $report | ConvertTo-Json -Depth 20
        $scenario = $report.scenarios[0]

        $report.schema_version | Should Be 4
        $report.workload_family | Should Be 'resource-management-query'
        $report.workload_profile_id | Should Be 'json-data-flat-v1'
        $report.workload_registry_receipt.sha256 | Should Match '^[0-9A-F]{64}$'
        $report.measurement_status | Should Be 'unverified'
        $report.measurement_status_reason | Should Be 'untrusted-observation-context'
        $report.source_fingerprint | Should Be ('A' * 64)
        $report.baseline_plan_sha256 | Should Be ('C' * 64)
        $report.observation_context.product_receipt.receipt_id | Should Be ('E' * 64)
        $report.observation_context.run.run_id | Should Be 'resource-baseline-run-001'
        $report.observation_context.machine.machine_id_sha256 | Should Be ('2' * 64)
        $scenario.process_lifecycle | Should Be 'fresh-process'
        $scenario.process_contexts.Count | Should Be 23
        $scenario.process_contexts[0].process_context.process_id | Should Be 4201
        $scenario.process_contexts[0].process_context.trace_id | Should Be 'resource-trace-001'
        $report.execution_protocol.randomization_algorithm | Should Be 'fisher-yates-sha256-v1'
        $scenario.sample_protocols.Count | Should Be 23
        $scenario.sample_protocols[0].execution_protocol.cache_action | Should Be 'purge'
        $scenario.sample_protocols[0].execution_protocol.quiescence_process_id | Should Be 4201
        $scenario.logical_id | Should Be 'data-000001-cold-open'
        $scenario.attempt_count | Should Be 23
        $scenario.warmup_count | Should Be 3
        $scenario.sample_count | Should Be 20
        $scenario.statistical_status | Should Be 'stable'
        $scenario.queries[0].elapsed_us.sample_count | Should Be 20
        $scenario.queries[0].elapsed_us.raw_samples.Count | Should Be 20
        $scenario.queries[0].elapsed_us.raw_samples[0] | Should Be 101
        $scenario.queries[0].elapsed_us.median | Should Be 100
        $scenario.queries[0].elapsed_us.noise_status | Should Be 'stable'
        $scenario.queries[0].elapsed_us.confidence_95_lower | Should BeLessThan 100
        $scenario.queries[0].elapsed_us.confidence_95_upper | Should BeGreaterThan 100
        $scenario.queries[0].counters['resource_management.scan.shard_candidate_checks'].median | Should Be 64
        $serialized | Should Not Match '[A-Za-z]:\\'
    }

    It 'rejects an observation context whose ProductReceipt belongs to another source' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation.observation_context.product_receipt.source_fingerprint = '9' * 64

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'ProductReceipt source_fingerprint'
    }

    It 'rejects undeclared context fields and invalid process or frame identities' {
        $unknownField = New-ResourceManagementBaselineObservationFixture
        $unknownField.observation_context.machine |
            Add-Member -NotePropertyName 'trusted' -NotePropertyValue $true
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $unknownField `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'unexpected property'

        $invalidProcess = New-ResourceManagementBaselineObservationFixture
        $invalidProcess.samples[0].process_context.process_created_utc = 'not-utc'
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $invalidProcess `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'process_created_utc'

        $invalidFrame = New-ResourceManagementBaselineObservationFixture
        $invalidFrame.samples[0].process_context.last_frame_index = 0
        $invalidFrame.samples[0].process_context.first_frame_index = 1
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $invalidFrame `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'frame range'
    }

    It 'enforces fresh-process uniqueness and same-process reuse from the baseline plan' {
        $freshObservation = New-ResourceManagementBaselineObservationFixture
        $freshObservation.samples[1].process_context.process_id = $freshObservation.samples[0].process_context.process_id
        $freshObservation.samples[1].process_context.process_created_utc = $freshObservation.samples[0].process_context.process_created_utc
        $freshObservation.samples[1].execution_protocol.quiescence_process_id = $freshObservation.samples[0].process_context.process_id
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $freshObservation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'fresh-process'

        $stablePlan = New-ResourceManagementBaselineReportPlanFixture
        $stablePlan.scenarios[0].logical_id = 'data-000001-stable-generation'
        $stablePlan.scenarios[0].mode = 'stable-generation'
        $stablePlan.scenarios[0].process_lifecycle = 'same-process'
        $stableObservation = New-ResourceManagementBaselineObservationFixture
        foreach ($sample in $stableObservation.samples) {
            $sample.logical_id = 'data-000001-stable-generation'
            $sample.execution_protocol.cache_state = 'warm'
            $sample.execution_protocol.cache_action = 'prime'
        }
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan $stablePlan `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $stableObservation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'same-process'

        foreach ($sample in $stableObservation.samples) {
            $sample.process_context.process_id = 4242
            $sample.process_context.process_created_utc = '2026-08-26T09:00:00.0000000Z'
            $sample.execution_protocol.cache_state = 'warm'
            $sample.execution_protocol.cache_action = 'prime'
            $sample.execution_protocol.quiescence_process_id = 4242
        }
        $stableReport = ConvertTo-ResourceManagementBaselineReport `
            -BaselinePlan $stablePlan `
            -BaselinePlanSha256 ('C' * 64) `
            -Observation $stableObservation `
            -ObservationSha256 ('D' * 64)
        $stableReport.scenarios[0].process_contexts.Count | Should Be 23
    }

    It 'rejects unrandomized order, duplicate sequence, cache-state drift, or cross-process quiescence' {
        $unrandomized = New-ResourceManagementBaselineObservationFixture
        $unrandomized.execution_protocol.randomization_algorithm = 'ordered'
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $unrandomized `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'randomization_algorithm'

        $duplicateSequence = New-ResourceManagementBaselineObservationFixture
        $duplicateSequence.samples[1].execution_protocol.sequence_ordinal = 1
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $duplicateSequence `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'sequence_ordinal'

        $cacheDrift = New-ResourceManagementBaselineObservationFixture
        $cacheDrift.samples[0].execution_protocol.cache_state = 'warm'
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $cacheDrift `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'cache state/action'

        $crossProcess = New-ResourceManagementBaselineObservationFixture
        $crossProcess.samples[0].execution_protocol.quiescence_process_id = 9999
        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $crossProcess `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'quiescence_process_id'
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

    It 'rejects an undeclared observation manifest property' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation | Add-Member -NotePropertyName 'trusted' -NotePropertyValue $true

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'unexpected property'
    }

    It 'rejects an undeclared observation counter' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation.samples[0].queries[0].counters |
            Add-Member -NotePropertyName 'resource_management.scan.injected' -NotePropertyValue 1

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'unexpected property'
    }

    It 'rejects a string disguised as a process metric' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation.samples[0].process.cpu_time_ms = '1'

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'JSON number'
    }

    It 'rejects an attempt above the plan-owned repetition budget' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation.samples[22].attempt = 24

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'outside the required repetition budget'
    }

    It 'rejects a sample whose warmup or measurement phase disagrees with the plan' {
        $observation = New-ResourceManagementBaselineObservationFixture
        $observation.samples[0].sample_phase = 'measurement'

        {
            ConvertTo-ResourceManagementBaselineReport `
                -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
                -BaselinePlanSha256 ('C' * 64) `
                -Observation $observation `
                -ObservationSha256 ('D' * 64)
        } | Should Throw 'expected sample_phase'
    }

    It 'marks a high-variance measurement cohort unstable without including warmups' {
        $observation = New-ResourceManagementBaselineObservationFixture
        foreach ($sample in @($observation.samples | Where-Object { $_.sample_phase -eq 'measurement' })) {
            $sample.queries[0].elapsed_us = if (($sample.attempt % 2) -eq 0) { 1 } else { 1000 }
        }

        $report = ConvertTo-ResourceManagementBaselineReport `
            -BaselinePlan (New-ResourceManagementBaselineReportPlanFixture) `
            -BaselinePlanSha256 ('C' * 64) `
            -Observation $observation `
            -ObservationSha256 ('D' * 64)

        $statistics = $report.scenarios[0].queries[0].elapsed_us
        $report.scenarios[0].statistical_status | Should Be 'unstable'
        $statistics.noise_status | Should Be 'unstable'
        $statistics.sample_count | Should Be 20
        @($statistics.raw_samples | Where-Object { $_ -eq 700 }).Count | Should Be 0
    }

    It 'marks a cohort below the policy-owned sample minimum as insufficient' {
        $statistics = Get-ResourceManagementCohortStatistics `
            -Values ([double[]]@(10, 10, 10)) `
            -MinimumSampleCount 20 `
            -MaximumCoefficientOfVariation 0.10 `
            -MaximumRelativeMarginOfError 0.10

        $statistics.sample_count | Should Be 3
        $statistics.raw_samples.Count | Should Be 3
        $statistics.noise_status | Should Be 'insufficient-samples'
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

    It 'rejects report output roots outside the registered artifact storage roots' {
        { Assert-ResourceManagementBaselineReportOutputDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-report-rejected' } |
            Should Throw 'outside the approved'
    }
}
