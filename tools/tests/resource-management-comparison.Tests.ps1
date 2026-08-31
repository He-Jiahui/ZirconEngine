$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$comparisonModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementComparison.psm1'
$statisticsModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementStatistics.psm1'
$comparisonWriter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementComparisonReport.ps1'
$observationContextModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementObservationContext.psm1'
$executionProtocolModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementExecutionProtocol.psm1'
$baselineReporter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1'
$schemaRegistry = Join-Path $repoRoot 'tools\mvp\resource-management-schema-registry.json'
$schemaRegistryModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementSchemaRegistry.psm1'
$workloadRegistry = Join-Path $repoRoot 'tools\mvp\resource-management-workload-registry.json'
$workloadRegistryModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementWorkloadRegistry.psm1'
$approvalTrustRegistry = Join-Path $repoRoot 'tools\mvp\resource-management-approval-trust-registry.json'
$approvalModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementBaselineApproval.psm1'
$artifactStorageModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'

Import-Module $schemaRegistryModule -ErrorAction Stop
Import-Module $workloadRegistryModule -ErrorAction Stop
Import-Module $comparisonModule -Force -ErrorAction Stop
Import-Module $statisticsModule -Force -ErrorAction Stop
Import-Module $artifactStorageModule -Force -ErrorAction Stop

function New-ResourceManagementComparisonSamples {
    param(
        [Parameter(Mandatory)][double]$Center,
        [double]$Spread = 0.5
    )

    return [double[]]@(
        1..20 | ForEach-Object {
            $Center + (((($_ - 1) % 5) - 2) * $Spread)
        }
    )
}

function New-ResourceManagementComparisonReportFixture {
    param(
        [Parameter(Mandatory)][double]$Center,
        [string]$SourceFingerprint = ('A' * 64)
    )

    $workloadSnapshot = Get-ResourceManagementWorkloadRegistrySnapshot
    $policy = [pscustomobject][ordered]@{
        warmup_repetitions = 3
        measurement_repetitions = 20
        minimum_sample_count = 20
        confidence_level = 0.95
        maximum_coefficient_of_variation = 0.10
        maximum_relative_margin_of_error = 0.10
    }
    $samples = New-ResourceManagementComparisonSamples -Center $Center
    $statistics = Get-ResourceManagementCohortStatistics `
        -Values $samples `
        -MinimumSampleCount 20 `
        -MaximumCoefficientOfVariation 0.10 `
        -MaximumRelativeMarginOfError 0.10
    return [pscustomobject][ordered]@{
        schema_version = 4
        workload_family = 'resource-management-query'
        workload_profile_id = 'json-data-flat-v1'
        workload_registry_receipt = $workloadSnapshot.receipt
        measurement_status = 'unverified'
        measurement_status_reason = 'untrusted-observation-context'
        source_fingerprint = $SourceFingerprint
        baseline_plan_sha256 = 'B' * 64
        observation_sha256 = 'C' * 64
        observation_context = [pscustomobject][ordered]@{
            schema_version = 1
            context_kind = 'zircon.resource-management-observation-context'
            product_receipt = [pscustomobject][ordered]@{
                schema_version = 1
                receipt_kind = 'zircon.mvp-product-receipt'
                receipt_id = 'E' * 64
                source_fingerprint = $SourceFingerprint
                product_role = 'editor'
                executable_sha256 = 'F' * 64
                build_set_id = '1' * 64
            }
            run = [pscustomobject][ordered]@{
                run_id = 'resource-comparison-run-001'
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
        statistical_policy = $policy
        scenarios = @(
            [pscustomobject][ordered]@{
                logical_id = 'data-000001-cold-open'
                mode = 'cold-open'
                project_role = 'baseline'
                process_lifecycle = 'fresh-process'
                data_asset_count = 1
                data_inventory_sha256 = 'D' * 64
                attempt_count = 23
                warmup_count = 3
                sample_count = 20
                statistical_status = 'stable'
                process_contexts = @(
                    1..23 | ForEach-Object {
                        [pscustomobject][ordered]@{
                            attempt = $_
                            sample_phase = if ($_ -le 3) { 'warmup' } else { 'measurement' }
                            process_context = [pscustomobject][ordered]@{
                                schema_version = 1
                                context_kind = 'zircon.resource-management-sample-process-context'
                                process_id = 4200 + $_
                                process_created_utc = ('2026-08-26T09:00:{0:D2}.0000000Z' -f $_)
                                trace_id = ('resource-trace-{0:D3}' -f $_)
                                first_frame_index = 0
                                last_frame_index = 22
                            }
                        }
                    }
                )
                sample_protocols = @(
                    1..23 | ForEach-Object {
                        [pscustomobject][ordered]@{
                            attempt = $_
                            sample_phase = if ($_ -le 3) { 'warmup' } else { 'measurement' }
                            execution_protocol = [pscustomobject][ordered]@{
                                schema_version = 1
                                protocol_kind = 'zircon.resource-management-sample-protocol'
                                sequence_ordinal = $_
                                cache_state = 'cold'
                                cache_action = 'purge'
                                cache_receipt_sha256 = ('{0:X64}' -f $_)
                                quiescence_receipt_sha256 = ('{0:X64}' -f (100 + $_))
                                quiescence_process_id = 4200 + $_
                            }
                        }
                    }
                )
                process = [pscustomobject][ordered]@{
                    cpu_time_ms = $statistics
                    peak_working_set_bytes = $statistics
                    allocation_proxy_bytes = $statistics
                }
                queries = @(
                    [pscustomobject][ordered]@{
                        operation = 'scan'
                        query = [pscustomobject][ordered]@{ kind = 'Data'; state = 'any' }
                        offset = $null
                        limit = $null
                        elapsed_us = $statistics
                        counters = [pscustomobject][ordered]@{}
                    }
                )
            }
        )
    }
}

function New-ResourceManagementComparisonPolicyFixture {
    return [pscustomobject][ordered]@{
        schema_version = 1
        policy_kind = 'resource-management-comparison'
        approved_baseline_report_sha256 = '1' * 64
        approved_baseline_receipt_sha256 = '2' * 64
        budgets = @(
            [pscustomobject][ordered]@{
                scenario_logical_id = 'data-000001-cold-open'
                operation = 'scan'
                query_kind = 'Data'
                query_state = 'any'
                offset = $null
                limit = $null
                maximum_absolute_increase = 5.0
                maximum_relative_increase = 0.05
                minimum_absolute_effect_size = 0.20
            }
        )
        links = [pscustomobject][ordered]@{
            trend_uri = 'https://perf.example.invalid/resource-management/trend'
            bisect_uri = 'https://perf.example.invalid/resource-management/bisect'
        }
    }
}

function New-ResourceManagementApprovalReceiptFixture {
    return [pscustomobject][ordered]@{
        schema_version = 2
        receipt_kind = 'zircon.resource-management-baseline-approval'
        promotion_id = 'resource-baseline-promotion-001'
        evidence_set_id = '7' * 64
        review_id = 'performance-review-001'
        issuer_id = 'performance-review'
        key_id = 'performance-review-2026'
        issued_utc = '2026-08-26T08:00:00.0000000Z'
        expires_utc = '2026-09-26T08:00:00.0000000Z'
        approved_baseline_report_sha256 = '1' * 64
        workload_profile_id = 'json-data-flat-v1'
        retention_class = 'accepted-baseline'
        retention_until_utc = '2027-09-26T08:00:00.0000000Z'
        legal_security_scrub_receipt_sha256 = '8' * 64
        supersedes_promotion_receipt_sha256 = $null
        decision = 'approved'
        signature_algorithm = 'rsa-pss-sha256'
        signature_base64 = 'AA=='
    }
}

function New-ResourceManagementApprovalTrustRegistryFixture {
    param(
        [Parameter(Mandatory)][string]$PublicKeySpkiBase64,
        [ValidateSet('active', 'disabled')][string]$Status = 'active',
        [string[]]$RevokedReceiptSha256 = @()
    )

    return [pscustomobject][ordered]@{
        schema_version = 1
        registry_kind = 'zircon.resource-management-approval-trust-registry'
        issuers = @(
            [pscustomobject][ordered]@{
                issuer_id = 'performance-review'
                key_id = 'performance-review-2026'
                signature_algorithm = 'rsa-pss-sha256'
                public_key_spki_base64 = $PublicKeySpkiBase64
                not_before_utc = '2026-08-01T00:00:00.0000000Z'
                not_after_utc = '2027-08-01T00:00:00.0000000Z'
                status = $Status
            }
        )
        revoked_receipt_sha256 = @($RevokedReceiptSha256)
    }
}

Describe 'Resource-management baseline comparison' {
    It 'reports a significant practical regression when either budget is exceeded' {
        $comparison = Compare-ResourceManagementCohorts `
            -BaselineValues (New-ResourceManagementComparisonSamples -Center 100) `
            -CandidateValues (New-ResourceManagementComparisonSamples -Center 125) `
            -MaximumAbsoluteIncrease 5 `
            -MaximumRelativeIncrease 0.05

        $comparison.schema_version | Should Be 1
        $comparison.comparison_kind | Should Be 'resource-management-cohort'
        $comparison.qualification_status | Should Be 'unverified'
        $comparison.qualification_status_reason | Should Be 'no-trusted-report-qualification'
        $comparison.diagnostic_decision | Should Be 'regression'
        $comparison.budget.absolute_exceeded | Should Be $true
        $comparison.budget.relative_exceeded | Should Be $true
        $comparison.effect.statistically_significant_regression | Should Be $true
        $comparison.effect.practically_significant_regression | Should Be $true
        $comparison.effect.hedges_g | Should BeGreaterThan 1
    }

    It 'keeps a statistically detectable change within budget when neither budget is exceeded' {
        $comparison = Compare-ResourceManagementCohorts `
            -BaselineValues (New-ResourceManagementComparisonSamples -Center 100) `
            -CandidateValues (New-ResourceManagementComparisonSamples -Center 102) `
            -MaximumAbsoluteIncrease 5 `
            -MaximumRelativeIncrease 0.05

        $comparison.diagnostic_decision | Should Be 'within-budget'
        $comparison.budget.absolute_exceeded | Should Be $false
        $comparison.budget.relative_exceeded | Should Be $false
        $comparison.effect.statistically_significant_regression | Should Be $true
    }

    It 'does not issue a regression decision for unstable cohorts' {
        $baseline = [double[]]@(1..20 | ForEach-Object { if (($_ % 2) -eq 0) { 10 } else { 1000 } })
        $candidate = [double[]]@(1..20 | ForEach-Object { if (($_ % 2) -eq 0) { 20 } else { 1200 } })

        $comparison = Compare-ResourceManagementCohorts `
            -BaselineValues $baseline `
            -CandidateValues $candidate `
            -MaximumAbsoluteIncrease 5 `
            -MaximumRelativeIncrease 0.05

        $comparison.diagnostic_decision | Should Be 'inconclusive-noise'
        $comparison.baseline.noise_status | Should Be 'unstable'
        $comparison.candidate.noise_status | Should Be 'unstable'
    }

    It 'does not issue a regression decision below the required cohort size' {
        $comparison = Compare-ResourceManagementCohorts `
            -BaselineValues ([double[]]@(100, 100, 100)) `
            -CandidateValues ([double[]]@(120, 120, 120)) `
            -MaximumAbsoluteIncrease 5 `
            -MaximumRelativeIncrease 0.05

        $comparison.diagnostic_decision | Should Be 'inconclusive-samples'
        $comparison.baseline.noise_status | Should Be 'insufficient-samples'
        $comparison.candidate.noise_status | Should Be 'insufficient-samples'
    }

    It 'uses the absolute budget and marks relative change not applicable for a zero baseline' {
        $comparison = Compare-ResourceManagementCohorts `
            -BaselineValues ([double[]](1..20 | ForEach-Object { 0 })) `
            -CandidateValues ([double[]](1..20 | ForEach-Object { 10 })) `
            -MaximumAbsoluteIncrease 5 `
            -MaximumRelativeIncrease 0.05

        $comparison.budget.relative_change_status | Should Be 'not-applicable-zero-baseline'
        $comparison.budget.relative_increase | Should Be $null
        $comparison.budget.absolute_exceeded | Should Be $true
        $comparison.effect.statistically_significant_regression | Should Be $true
        $comparison.diagnostic_decision | Should Be 'regression'
    }

    It 'emits a retain-all MAD outlier receipt without mutating raw samples' {
        $values = [double[]]@((1..19 | ForEach-Object { 100 }) + 1000)

        $receipt = New-ResourceManagementOutlierReceipt -Values $values

        $receipt.schema_version | Should Be 1
        $receipt.policy | Should Be 'retain-all'
        $receipt.detector | Should Be 'mad-modified-z-score'
        $receipt.input_count | Should Be 20
        $receipt.output_count | Should Be 20
        $receipt.identified_indices | Should Be @(19)
        $receipt.removed_indices.Count | Should Be 0
        $receipt.retained_samples | Should Be $values
    }

    It 'aggregates exact report queries into one fail-closed machine-readable regression decision' {
        $report = Compare-ResourceManagementReports `
            -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
            -ApprovedBaselineReportSha256 ('1' * 64) `
            -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
            -ApprovedBaselineReceiptSha256 ('2' * 64) `
            -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 125 -SourceFingerprint ('E' * 64)) `
            -CandidateReportSha256 ('3' * 64) `
            -Policy (New-ResourceManagementComparisonPolicyFixture) `
            -PolicySha256 ('4' * 64)

        $report.schema_version | Should Be 3
        $report.report_kind | Should Be 'resource-management-comparison'
        $report.qualification_status | Should Be 'unverified'
        $report.qualification_status_reason | Should Be 'untrusted-observation-context'
        $report.diagnostic_decision | Should Be 'regression'
        $report.decision_summary.comparison_count | Should Be 1
        $report.decision_summary.regression_count | Should Be 1
        $report.approved_baseline_report_sha256 | Should Be ('1' * 64)
        $report.approved_baseline_receipt_sha256 | Should Be ('2' * 64)
        $report.approval_verification.verification_status | Should Be 'unverified'
        $report.approval_verification.verification_reason | Should Be 'approval-issuer-not-trusted'
        $report.candidate_report_sha256 | Should Be ('3' * 64)
        $report.policy_sha256 | Should Be ('4' * 64)
        $report.workload_profile_id | Should Be 'json-data-flat-v1'
        $report.workload_registry_receipt.sha256 | Should Match '^[0-9A-F]{64}$'
        $report.observation_contexts.approved_baseline.product_receipt.receipt_id | Should Be ('E' * 64)
        $report.observation_contexts.candidate.run.run_id | Should Be 'resource-comparison-run-001'
        $report.execution_protocols.approved_baseline.order_receipt_sha256 | Should Be ('4' * 64)
        $report.comparisons[0].comparison.diagnostic_decision | Should Be 'regression'
        $report.links.trend_uri | Should Match '^https://'
    }

    It 'reports within-budget only as a diagnostic decision' {
        $report = Compare-ResourceManagementReports `
            -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
            -ApprovedBaselineReportSha256 ('1' * 64) `
            -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
            -ApprovedBaselineReceiptSha256 ('2' * 64) `
            -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 102) `
            -CandidateReportSha256 ('3' * 64) `
            -Policy (New-ResourceManagementComparisonPolicyFixture) `
            -PolicySha256 ('4' * 64)

        $report.diagnostic_decision | Should Be 'within-budget'
        $report.qualification_status | Should Be 'unverified'
        $report.decision_summary.within_budget_count | Should Be 1
    }

    It 'rejects a policy that does not bind the supplied approved baseline report' {
        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('9' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 102) `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'approved baseline report SHA-256'
    }

    It 'rejects a policy that does not bind the supplied approved baseline receipt' {
        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('8' * 64) `
                -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 102) `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'approved baseline receipt SHA-256'
    }

    It 'rejects missing or duplicate query budgets' {
        foreach ($budgetMutation in @('missing', 'duplicate')) {
            $policy = New-ResourceManagementComparisonPolicyFixture
            if ($budgetMutation -eq 'missing') {
                $policy.budgets = @()
            }
            else {
                $policy.budgets = @($policy.budgets[0], $policy.budgets[0])
            }
            {
                Compare-ResourceManagementReports `
                    -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                    -ApprovedBaselineReportSha256 ('1' * 64) `
                    -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                    -ApprovedBaselineReceiptSha256 ('2' * 64) `
                    -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 102) `
                    -CandidateReportSha256 ('3' * 64) `
                    -Policy $policy `
                    -PolicySha256 ('4' * 64)
            } | Should Throw 'budget'
        }
    }

    It 'rejects candidate query identity drift before comparing samples' {
        $candidate = New-ResourceManagementComparisonReportFixture -Center 102
        $candidate.scenarios[0].queries[0].query.kind = 'Mesh'

        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport $candidate `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'query'
    }

    It 'rejects a candidate measured under a different statistical policy' {
        $candidate = New-ResourceManagementComparisonReportFixture -Center 102
        $candidate.statistical_policy.minimum_sample_count = 19

        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport $candidate `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'statistical policy'
    }

    It 'rejects candidate machine or collector drift before comparing cohorts' {
        foreach ($mutation in @('machine', 'collector')) {
            $candidate = New-ResourceManagementComparisonReportFixture -Center 102
            if ($mutation -eq 'machine') {
                $candidate.observation_context.machine.logical_processor_count = 8
            }
            else {
                $candidate.observation_context.collector.collector_version = '2.0.0'
            }
            {
                Compare-ResourceManagementReports `
                    -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                    -ApprovedBaselineReportSha256 ('1' * 64) `
                    -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                    -ApprovedBaselineReceiptSha256 ('2' * 64) `
                    -CandidateReport $candidate `
                    -CandidateReportSha256 ('3' * 64) `
                    -Policy (New-ResourceManagementComparisonPolicyFixture) `
                    -PolicySha256 ('4' * 64)
            } | Should Throw 'observation context'
        }
    }

    It 'rejects unrandomized or malformed candidate execution protocol evidence' {
        $unrandomized = New-ResourceManagementComparisonReportFixture -Center 102
        $unrandomized.execution_protocol.randomization_algorithm = 'ordered'
        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport $unrandomized `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'randomization_algorithm'

        $fractionalAttempt = New-ResourceManagementComparisonReportFixture -Center 102
        $fractionalAttempt.scenarios[0].process_contexts[0].attempt = 0.5
        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport $fractionalAttempt `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'process context attempt'
    }

    It 'rejects candidate workload profile drift before cohort comparison' {
        $candidate = New-ResourceManagementComparisonReportFixture -Center 102
        $candidate.workload_profile_id = 'other-profile'

        {
            Compare-ResourceManagementReports `
                -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
                -ApprovedBaselineReceiptSha256 ('2' * 64) `
                -CandidateReport $candidate `
                -CandidateReportSha256 ('3' * 64) `
                -Policy (New-ResourceManagementComparisonPolicyFixture) `
                -PolicySha256 ('4' * 64)
        } | Should Throw 'workload profile'
    }

    It 'routes observation context validation through one shared authority' {
        $comparisonSource = Get-Content -Raw $comparisonModule
        $reporterSource = Get-Content -Raw $baselineReporter

        $observationContextModule | Should Exist
        $executionProtocolModule | Should Exist
        $comparisonSource | Should Match 'ResourceManagementObservationContext\.psm1'
        $reporterSource | Should Match 'ResourceManagementObservationContext\.psm1'
        $comparisonSource | Should Match 'Resolve-ResourceManagementObservationContext'
        $reporterSource | Should Match 'Resolve-ResourceManagementObservationContext'
        $comparisonSource | Should Match 'ResourceManagementExecutionProtocol\.psm1'
        $reporterSource | Should Match 'ResourceManagementExecutionProtocol\.psm1'
        $comparisonSource | Should Match 'Resolve-ResourceManagementExecutionProtocol'
        $reporterSource | Should Match 'Resolve-ResourceManagementExecutionProtocol'
    }

    It 'renders the diagnostic decision and operator links without upgrading qualification' {
        $report = Compare-ResourceManagementReports `
            -ApprovedBaselineReport (New-ResourceManagementComparisonReportFixture -Center 100) `
            -ApprovedBaselineReportSha256 ('1' * 64) `
            -ApprovedBaselineReceipt (New-ResourceManagementApprovalReceiptFixture) `
            -ApprovedBaselineReceiptSha256 ('2' * 64) `
            -CandidateReport (New-ResourceManagementComparisonReportFixture -Center 125) `
            -CandidateReportSha256 ('3' * 64) `
            -Policy (New-ResourceManagementComparisonPolicyFixture) `
            -PolicySha256 ('4' * 64)

        $markdown = ConvertTo-ResourceManagementComparisonMarkdown -Report $report

        $markdown | Should Match 'Qualification status: unverified'
        $markdown | Should Match 'Approval verification: unverified \(approval-issuer-not-trusted\)'
        $markdown | Should Match 'Diagnostic decision: regression'
        $markdown | Should Match 'Trend: https://perf\.example\.invalid/'
        $markdown | Should Match 'Bisect: https://perf\.example\.invalid/'
        $markdown | Should Match 'data-000001-cold-open'
    }

    It 'atomically publishes machine-readable and Markdown comparison artifacts' {
        $baselinePath = Join-Path $TestDrive 'approved-baseline-report.json'
        $candidatePath = Join-Path $TestDrive 'candidate-report.json'
        $approvalReceiptPath = Join-Path $TestDrive 'approved-baseline-receipt.json'
        $policyPath = Join-Path $TestDrive 'comparison-policy.json'
        [IO.File]::WriteAllText(
            $baselinePath,
            (New-ResourceManagementComparisonReportFixture -Center 100 | ConvertTo-Json -Depth 20),
            [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText(
            $candidatePath,
            (New-ResourceManagementComparisonReportFixture -Center 125 | ConvertTo-Json -Depth 20),
            [Text.UTF8Encoding]::new($false))
        $approvalReceipt = New-ResourceManagementApprovalReceiptFixture
        $approvalReceipt.approved_baseline_report_sha256 = (Get-FileHash -LiteralPath $baselinePath -Algorithm SHA256).Hash
        [IO.File]::WriteAllText(
            $approvalReceiptPath,
            ($approvalReceipt | ConvertTo-Json -Depth 10),
            [Text.UTF8Encoding]::new($false))
        $policy = New-ResourceManagementComparisonPolicyFixture
        $policy.approved_baseline_report_sha256 = (Get-FileHash -LiteralPath $baselinePath -Algorithm SHA256).Hash
        $policy.approved_baseline_receipt_sha256 = (Get-FileHash -LiteralPath $approvalReceiptPath -Algorithm SHA256).Hash
        [IO.File]::WriteAllText(
            $policyPath,
            ($policy | ConvertTo-Json -Depth 10),
            [Text.UTF8Encoding]::new($false))
        $outputRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'resource-management-comparisons' `
            -InstanceId ('comparison-test-' + [guid]::NewGuid().ToString('N'))
        try {
            $result = & $comparisonWriter `
                -ApprovedBaselineReportPath $baselinePath `
                -ApprovedBaselineReceiptPath $approvalReceiptPath `
                -CandidateReportPath $candidatePath `
                -PolicyPath $policyPath `
                -OutputDirectory $outputRoot

            $result.diagnostic_decision | Should Be 'regression'
            $result.qualification_status | Should Be 'unverified'
            [IO.File]::Exists((Join-Path $outputRoot 'resource-management-comparison.json')) | Should Be $true
            [IO.File]::Exists((Join-Path $outputRoot 'resource-management-comparison.md')) | Should Be $true
            $published = Get-Content -Raw (Join-Path $outputRoot 'resource-management-comparison.json') | ConvertFrom-Json
            $published.diagnostic_decision | Should Be 'regression'
            $published.qualification_status | Should Be 'unverified'
            $published.approval_verification.verification_reason | Should Be 'approval-issuer-not-trusted'
            @(
                Get-ChildItem ([IO.Path]::GetDirectoryName($outputRoot)) -Directory |
                    Where-Object { $_.Name -like ([IO.Path]::GetFileName($outputRoot) + '.partial-*') }
            ).Count | Should Be 0
        }
        finally {
            if ([IO.Directory]::Exists($outputRoot)) {
                [IO.Directory]::Delete($outputRoot, $true)
            }
        }
    }

    It 'rejects comparison output outside the approved root or over an existing target' {
        $baselinePath = Join-Path $TestDrive 'root-check-baseline.json'
        $candidatePath = Join-Path $TestDrive 'root-check-candidate.json'
        $approvalReceiptPath = Join-Path $TestDrive 'root-check-approval-receipt.json'
        $policyPath = Join-Path $TestDrive 'root-check-policy.json'
        [IO.File]::WriteAllText($baselinePath, '{}', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($candidatePath, '{}', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($approvalReceiptPath, '{}', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($policyPath, '{}', [Text.UTF8Encoding]::new($false))

        {
            & $comparisonWriter `
                -ApprovedBaselineReportPath $baselinePath `
                -ApprovedBaselineReceiptPath $approvalReceiptPath `
                -CandidateReportPath $candidatePath `
                -PolicyPath $policyPath `
                -OutputDirectory 'C:\ZirconBuilds\mvp-resource-management-comparison-rejected'
        } | Should Throw 'outside the approved'

        $occupiedRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'resource-management-comparisons' `
            -InstanceId ('comparison-occupied-' + [guid]::NewGuid().ToString('N'))
        try {
            [IO.Directory]::CreateDirectory($occupiedRoot) | Out-Null
            {
                & $comparisonWriter `
                    -ApprovedBaselineReportPath $baselinePath `
                    -ApprovedBaselineReceiptPath $approvalReceiptPath `
                    -CandidateReportPath $candidatePath `
                    -PolicyPath $policyPath `
                    -OutputDirectory $occupiedRoot
            } | Should Throw 'must not already exist'
        }
        finally {
            if ([IO.Directory]::Exists($occupiedRoot)) {
                [IO.Directory]::Delete($occupiedRoot, $true)
            }
        }
    }

    It 'routes both resource report producers through one bounded JSON evidence authority' {
        $baselineReporter = Get-Content -Raw (Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1')
        $comparisonReporter = Get-Content -Raw $comparisonWriter
        $evidenceAuthority = Join-Path $repoRoot 'tools\mvp\ResourceManagementJsonEvidence.psm1'

        $evidenceAuthority | Should Exist
        foreach ($source in @($baselineReporter, $comparisonReporter)) {
            $source | Should Match 'Get-ResourceManagementJsonEvidence'
            $source | Should Not Match '\[IO\.File\]::Open'
            $source | Should Not Match 'ComputeHash'
            $source | Should Not Match '\[IO\.File\]::ReadAllBytes'
        }
        $comparisonReporter | Should Match 'ApprovedBaselineReceiptPath'
        $comparisonReporter | Should Match "Label 'Approved resource-management baseline approval receipt'"
        $comparisonReporter | Should Match 'ResourceManagementComparisonMaximumReceiptBytes'
    }

    It 'routes exact-property and JSON-number validation through one resource schema authority' {
        $baselineReporter = Get-Content -Raw (Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1')
        $comparisonModuleSource = Get-Content -Raw $comparisonModule
        $schemaAuthority = Join-Path $repoRoot 'tools\mvp\ResourceManagementSchema.psm1'

        $schemaAuthority | Should Exist
        foreach ($source in @($baselineReporter, $comparisonModuleSource)) {
            $source | Should Match 'ResourceManagementSchema\.psm1'
            $source | Should Match 'Assert-ResourceManagementSchemaProperties'
            $source | Should Match 'ConvertTo-ResourceManagementSchemaJsonNumber'
            $source | Should Not Match '\[Type\]::GetTypeCode'
        }
    }

    It 'loads one bounded exact registry for every resource schema identity' {
        $schemaRegistry | Should Exist
        $schemaRegistryModule | Should Exist
        $snapshot = Get-ResourceManagementSchemaRegistrySnapshot
        $snapshot.receipt.schema_version | Should Be 1
        $snapshot.receipt.registry_kind | Should Be 'zircon.resource-management-schema-registry'
        $snapshot.receipt.schema_count | Should Be 19
        $snapshot.receipt.bytes | Should BeGreaterThan 0
        $snapshot.receipt.sha256 | Should Match '^[0-9A-F]{64}$'
        $snapshot.schemas['zircon.resource-management.observation'].current_version | Should Be 3
        $snapshot.schemas['zircon.resource-management.comparison-report'].current_version | Should Be 3
        $snapshot.schemas['zircon.resource-management.approval-receipt'].current_version | Should Be 2
        $snapshot.schemas['zircon.resource-management.approval-verification'].current_version | Should Be 2
        $snapshot.schemas['zircon.resource-management.sample-process-context'].identity_value |
            Should Be 'zircon.resource-management-sample-process-context'
    }

    It 'rejects unregistered, stale, future, or mismatched schema identities' {
        $valid = [pscustomobject][ordered]@{
            schema_version = 1
            protocol_kind = 'zircon.resource-management-sample-protocol'
        }

        Assert-ResourceManagementRegisteredSchemaIdentity `
            -Value $valid `
            -SchemaId 'zircon.resource-management.sample-protocol' | Should Be $valid
        {
            Assert-ResourceManagementRegisteredSchemaIdentity `
                -Value $valid `
                -SchemaId 'zircon.resource-management.missing'
        } | Should Throw 'unregistered'

        $stale = [pscustomobject][ordered]@{
            schema_version = 0
            protocol_kind = 'zircon.resource-management-sample-protocol'
        }
        {
            Assert-ResourceManagementRegisteredSchemaIdentity `
                -Value $stale `
                -SchemaId 'zircon.resource-management.sample-protocol'
        } | Should Throw 'stale'

        $future = [pscustomobject][ordered]@{
            schema_version = 2
            protocol_kind = 'zircon.resource-management-sample-protocol'
        }
        {
            Assert-ResourceManagementRegisteredSchemaIdentity `
                -Value $future `
                -SchemaId 'zircon.resource-management.sample-protocol'
        } | Should Throw 'future'

        $mismatched = [pscustomobject][ordered]@{
            schema_version = 1
            protocol_kind = 'zircon.resource-management.other'
        }
        {
            Assert-ResourceManagementRegisteredSchemaIdentity `
                -Value $mismatched `
                -SchemaId 'zircon.resource-management.sample-protocol'
        } | Should Throw 'differs from registered'
    }

    It 'routes resource artifact validators through the registered schema authority' {
        $registrySource = Get-Content -Raw $schemaRegistryModule
        $observationSource = Get-Content -Raw $observationContextModule
        $executionSource = Get-Content -Raw $executionProtocolModule
        $comparisonSource = Get-Content -Raw $comparisonModule
        $reporterSource = Get-Content -Raw $baselineReporter

        $registrySource | Should Match 'Get-ResourceManagementJsonEvidence'
        $registrySource | Should Match 'MaximumBytes 65536'
        $observationSource | Should Match 'ResourceManagementSchemaRegistry\.psm1'
        $executionSource | Should Match 'ResourceManagementSchemaRegistry\.psm1'
        $comparisonSource | Should Match 'ResourceManagementSchemaRegistry\.psm1'
        $reporterSource | Should Match 'ResourceManagementSchemaRegistry\.psm1'
        $observationSource | Should Match 'zircon\.resource-management\.sample-process-context'
    }

    It 'loads one bounded extensible resource workload profile registry' {
        $workloadRegistry | Should Exist
        $workloadRegistryModule | Should Exist
        $snapshot = Get-ResourceManagementWorkloadRegistrySnapshot
        $snapshot.receipt.registry_kind | Should Be 'zircon.resource-management-workload-registry'
        $snapshot.receipt.profile_count | Should Be 1
        $snapshot.receipt.sha256 | Should Match '^[0-9A-F]{64}$'
        $profile = Get-ResourceManagementWorkloadProfile -ProfileId 'json-data-flat-v1'
        $profile.asset_kinds | Should Be @('Data')
        $profile.dependency_graph_shape | Should Be 'none'
        $profile.tag_cardinality | Should Be 0
        $profile.query_mix | Should Be @('scan', 'page', 'asset-workspace-snapshot')
        $profile.minimum_asset_count | Should Be 1
        $profile.maximum_asset_count | Should Be 100000

        (Get-Content -Raw $baselineReporter) | Should Match 'ResourceManagementWorkloadRegistry\.psm1'
        (Get-Content -Raw $comparisonModule) | Should Match 'ResourceManagementWorkloadRegistry\.psm1'
        (Get-Content -Raw (Join-Path $repoRoot 'tools\mvp\New-ResourceManagementBaselinePlan.ps1')) |
            Should Match 'ResourceManagementWorkloadRegistry\.psm1'
    }

    It 'loads a bounded empty-by-default approval trust registry' {
        $approvalTrustRegistry | Should Exist
        $approvalModule | Should Exist
        Import-Module $approvalModule -ErrorAction Stop

        $snapshot = Get-ResourceManagementApprovalTrustRegistrySnapshot
        $snapshot.receipt.registry_kind | Should Be 'zircon.resource-management-approval-trust-registry'
        $snapshot.receipt.issuer_count | Should Be 0
        $snapshot.receipt.revoked_receipt_count | Should Be 0
        $snapshot.receipt.sha256 | Should Match '^[0-9A-F]{64}$'
    }

    It 'verifies an approved baseline receipt with a registered RSA-PSS issuer' {
        Import-Module $approvalModule -ErrorAction Stop
        $rsa = [Security.Cryptography.RSA]::Create(2048)
        try {
            $publicKey = [Convert]::ToBase64String($rsa.ExportSubjectPublicKeyInfo())
            $trustSnapshot = Resolve-ResourceManagementApprovalTrustRegistry `
                -Registry (New-ResourceManagementApprovalTrustRegistryFixture -PublicKeySpkiBase64 $publicKey) `
                -RegistryBytes 1 `
                -RegistrySha256 ('A' * 64)
            $receipt = New-ResourceManagementApprovalReceiptFixture
            $payload = Get-ResourceManagementApprovalCanonicalPayloadBytes -Receipt $receipt
            $signature = $rsa.SignData(
                $payload,
                [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.RSASignaturePadding]::Pss)
            $receipt.signature_base64 = [Convert]::ToBase64String($signature)

            $verification = Resolve-ResourceManagementBaselineApproval `
                -Receipt $receipt `
                -ReceiptSha256 ('E' * 64) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -WorkloadProfileId 'json-data-flat-v1' `
                -TrustRegistrySnapshot $trustSnapshot `
                -VerificationTimeUtc ([DateTimeOffset]'2026-08-27T00:00:00Z')

            $verification.schema_version | Should Be 2
            $verification.verification_kind | Should Be 'zircon.resource-management-baseline-approval-verification'
            $verification.verification_status | Should Be 'verified'
            $verification.verification_reason | Should Be 'signature-and-policy-verified'
            $verification.issuer_id | Should Be 'performance-review'
            $verification.key_id | Should Be 'performance-review-2026'
            $verification.promotion_id | Should Be 'resource-baseline-promotion-001'
            $verification.evidence_set_id | Should Be ('7' * 64)
            $verification.retention_class | Should Be 'accepted-baseline'
        }
        finally {
            $rsa.Dispose()
        }
    }

    It 'rejects tampering and keeps revoked, expired, or disabled approvals unverified' {
        Import-Module $approvalModule -ErrorAction Stop
        $rsa = [Security.Cryptography.RSA]::Create(2048)
        try {
            $publicKey = [Convert]::ToBase64String($rsa.ExportSubjectPublicKeyInfo())
            $receipt = New-ResourceManagementApprovalReceiptFixture
            $payload = Get-ResourceManagementApprovalCanonicalPayloadBytes -Receipt $receipt
            $receipt.signature_base64 = [Convert]::ToBase64String($rsa.SignData(
                    $payload,
                    [Security.Cryptography.HashAlgorithmName]::SHA256,
                    [Security.Cryptography.RSASignaturePadding]::Pss))
            $activeRegistry = New-ResourceManagementApprovalTrustRegistryFixture -PublicKeySpkiBase64 $publicKey
            $activeSnapshot = Resolve-ResourceManagementApprovalTrustRegistry `
                -Registry $activeRegistry `
                -RegistryBytes 1 `
                -RegistrySha256 ('A' * 64)

            $tampered = $receipt.PSObject.Copy()
            $tampered.approved_baseline_report_sha256 = '9' * 64
            {
                Resolve-ResourceManagementBaselineApproval `
                    -Receipt $tampered `
                    -ReceiptSha256 ('E' * 64) `
                    -ApprovedBaselineReportSha256 ('9' * 64) `
                    -WorkloadProfileId 'json-data-flat-v1' `
                    -TrustRegistrySnapshot $activeSnapshot `
                    -VerificationTimeUtc ([DateTimeOffset]'2026-08-27T00:00:00Z')
            } | Should Throw 'signature'

            $revokedSnapshot = Resolve-ResourceManagementApprovalTrustRegistry `
                -Registry (New-ResourceManagementApprovalTrustRegistryFixture `
                    -PublicKeySpkiBase64 $publicKey `
                    -RevokedReceiptSha256 @('E' * 64)) `
                -RegistryBytes 1 `
                -RegistrySha256 ('B' * 64)
            $revoked = Resolve-ResourceManagementBaselineApproval `
                -Receipt $receipt `
                -ReceiptSha256 ('E' * 64) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -WorkloadProfileId 'json-data-flat-v1' `
                -TrustRegistrySnapshot $revokedSnapshot `
                -VerificationTimeUtc ([DateTimeOffset]'2026-08-27T00:00:00Z')
            $revoked.verification_status | Should Be 'unverified'
            $revoked.verification_reason | Should Be 'approval-receipt-revoked'

            $expired = Resolve-ResourceManagementBaselineApproval `
                -Receipt $receipt `
                -ReceiptSha256 ('E' * 64) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -WorkloadProfileId 'json-data-flat-v1' `
                -TrustRegistrySnapshot $activeSnapshot `
                -VerificationTimeUtc ([DateTimeOffset]'2026-10-01T00:00:00Z')
            $expired.verification_status | Should Be 'unverified'
            $expired.verification_reason | Should Be 'approval-receipt-expired'

            $disabledSnapshot = Resolve-ResourceManagementApprovalTrustRegistry `
                -Registry (New-ResourceManagementApprovalTrustRegistryFixture `
                    -PublicKeySpkiBase64 $publicKey `
                    -Status 'disabled') `
                -RegistryBytes 1 `
                -RegistrySha256 ('C' * 64)
            $disabled = Resolve-ResourceManagementBaselineApproval `
                -Receipt $receipt `
                -ReceiptSha256 ('E' * 64) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -WorkloadProfileId 'json-data-flat-v1' `
                -TrustRegistrySnapshot $disabledSnapshot `
                -VerificationTimeUtc ([DateTimeOffset]'2026-08-27T00:00:00Z')
            $disabled.verification_status | Should Be 'unverified'
            $disabled.verification_reason | Should Be 'approval-issuer-disabled'
        }
        finally {
            $rsa.Dispose()
        }
    }

    It 'binds review, retention, scrub, and supersedes governance into the signed promotion receipt' {
        Import-Module $approvalModule -ErrorAction Stop
        $receipt = New-ResourceManagementApprovalReceiptFixture
        $receipt.supersedes_promotion_receipt_sha256 = '6' * 64

        $payload = Get-ResourceManagementApprovalCanonicalPayloadBytes -Receipt $receipt
        [Text.Encoding]::UTF8.GetString($payload) | Should Match 'resource-baseline-promotion-001'
        [Text.Encoding]::UTF8.GetString($payload) | Should Match ('6' * 64)

        $mutationErrors = @(
            foreach ($mutation in @('retention-class', 'retention-window', 'scrub-receipt')) {
            $invalid = $receipt.PSObject.Copy()
            if ($mutation -eq 'retention-class') {
                $invalid.retention_class = 'ci-ephemeral'
            }
            elseif ($mutation -eq 'retention-window') {
                $invalid.retention_until_utc = '2026-09-01T08:00:00.0000000Z'
            }
            else {
                $invalid.legal_security_scrub_receipt_sha256 = 'not-a-digest'
            }
                try {
                    Get-ResourceManagementApprovalCanonicalPayloadBytes -Receipt $invalid | Out-Null
                }
                catch {
                    $_.Exception.Message
                }
            }
        )
        $mutationErrors.Count | Should Be 3

        $selfReferential = $receipt.PSObject.Copy()
        $selfReferential.supersedes_promotion_receipt_sha256 = 'E' * 64
        {
            Resolve-ResourceManagementBaselineApproval `
                -Receipt $selfReferential `
                -ReceiptSha256 ('E' * 64) `
                -ApprovedBaselineReportSha256 ('1' * 64) `
                -WorkloadProfileId 'json-data-flat-v1' `
                -TrustRegistrySnapshot (Get-ResourceManagementApprovalTrustRegistrySnapshot) `
                -VerificationTimeUtc ([DateTimeOffset]'2026-08-27T00:00:00Z')
        } | Should Throw 'supersede itself'
    }

}
