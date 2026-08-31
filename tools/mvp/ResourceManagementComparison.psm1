Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementStatistics.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementWorkloadRegistry.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementObservationContext.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementExecutionProtocol.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementBaselineApproval.psm1') -ErrorAction Stop

function New-ResourceManagementOutlierReceipt {
    param(
        [Parameter(Mandatory)][double[]]$Values,
        [ValidateRange(0.1, 100.0)][double]$ModifiedZScoreThreshold = 3.5
    )

    $statistics = Get-ResourceManagementCohortStatistics `
        -Values $Values `
        -MinimumSampleCount 1 `
        -MaximumCoefficientOfVariation 1.0 `
        -MaximumRelativeMarginOfError 1.0
    $identified = [Collections.Generic.List[int]]::new()
    for ($index = 0; $index -lt $Values.Count; $index++) {
        $deviation = [Math]::Abs($Values[$index] - $statistics.median)
        $isOutlier = if ($statistics.median_absolute_deviation -eq 0.0) {
            $deviation -gt 0.0
        }
        else {
            ((0.6745 * $deviation) / $statistics.median_absolute_deviation) -gt $ModifiedZScoreThreshold
        }
        if ($isOutlier) {
            $identified.Add($index)
        }
    }

    return [pscustomobject][ordered]@{
        schema_version = 1
        policy = 'retain-all'
        detector = 'mad-modified-z-score'
        modified_z_score_threshold = $ModifiedZScoreThreshold
        input_count = $Values.Count
        output_count = $Values.Count
        identified_indices = $identified.ToArray()
        removed_indices = [int[]]@()
        retained_samples = [double[]]$Values.Clone()
    }
}

function Get-ResourceManagementComparisonEffect {
    param(
        [Parameter(Mandatory)]$Baseline,
        [Parameter(Mandatory)]$Candidate,
        [Parameter(Mandatory)][ValidateRange(0.0, [double]::MaxValue)][double]$MinimumAbsoluteEffectSize
    )

    $meanDifference = [double]$Candidate.mean - [double]$Baseline.mean
    $standardError = [Math]::Sqrt(
        (([double]$Baseline.sample_standard_deviation * [double]$Baseline.sample_standard_deviation) / [int]$Baseline.sample_count) +
        (([double]$Candidate.sample_standard_deviation * [double]$Candidate.sample_standard_deviation) / [int]$Candidate.sample_count))
    # The same conservative minimum-df Student-t critical value used by cohort confidence bounds.
    $confidenceMargin = 2.093 * $standardError
    $confidenceLower = $meanDifference - $confidenceMargin
    $confidenceUpper = $meanDifference + $confidenceMargin

    $degreesOfFreedom = [int]$Baseline.sample_count + [int]$Candidate.sample_count - 2
    $pooledVariance = if ($degreesOfFreedom -gt 0) {
        ((([int]$Baseline.sample_count - 1) * [Math]::Pow([double]$Baseline.sample_standard_deviation, 2)) +
            (([int]$Candidate.sample_count - 1) * [Math]::Pow([double]$Candidate.sample_standard_deviation, 2))) /
        $degreesOfFreedom
    }
    else {
        0.0
    }
    $pooledStandardDeviation = [Math]::Sqrt($pooledVariance)
    $effectStatus = 'measured'
    $hedgesG = $null
    $practicallySignificant = $false
    if ($pooledStandardDeviation -eq 0.0) {
        $effectStatus = if ($meanDifference -eq 0.0) { 'zero-effect-zero-variance' } else { 'unbounded-zero-variance' }
        $practicallySignificant = $meanDifference -gt 0.0
    }
    else {
        $smallSampleCorrection = 1.0 - (3.0 / ((4.0 * ([int]$Baseline.sample_count + [int]$Candidate.sample_count)) - 9.0))
        $hedgesG = ($meanDifference / $pooledStandardDeviation) * $smallSampleCorrection
        $practicallySignificant = $hedgesG -ge $MinimumAbsoluteEffectSize
    }

    return [pscustomobject][ordered]@{
        mean_difference = $meanDifference
        mean_difference_confidence_95_lower = $confidenceLower
        mean_difference_confidence_95_upper = $confidenceUpper
        statistically_significant_regression = $confidenceLower -gt 0.0
        standardized_effect_status = $effectStatus
        hedges_g = $hedgesG
        minimum_absolute_effect_size = $MinimumAbsoluteEffectSize
        practically_significant_regression = $practicallySignificant
    }
}

function Compare-ResourceManagementCohorts {
    param(
        [Parameter(Mandatory)][double[]]$BaselineValues,
        [Parameter(Mandatory)][double[]]$CandidateValues,
        [Parameter(Mandatory)][ValidateRange(0.0, [double]::MaxValue)][double]$MaximumAbsoluteIncrease,
        [Parameter(Mandatory)][ValidateRange(0.0, 1.0)][double]$MaximumRelativeIncrease,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MinimumSampleCount = 20,
        [ValidateRange(0.000001, 1.0)][double]$MaximumCoefficientOfVariation = 0.10,
        [ValidateRange(0.000001, 1.0)][double]$MaximumRelativeMarginOfError = 0.10,
        [ValidateRange(0.0, [double]::MaxValue)][double]$MinimumAbsoluteEffectSize = 0.20
    )

    $baseline = Get-ResourceManagementCohortStatistics `
        -Values $BaselineValues `
        -MinimumSampleCount $MinimumSampleCount `
        -MaximumCoefficientOfVariation $MaximumCoefficientOfVariation `
        -MaximumRelativeMarginOfError $MaximumRelativeMarginOfError
    $candidate = Get-ResourceManagementCohortStatistics `
        -Values $CandidateValues `
        -MinimumSampleCount $MinimumSampleCount `
        -MaximumCoefficientOfVariation $MaximumCoefficientOfVariation `
        -MaximumRelativeMarginOfError $MaximumRelativeMarginOfError
    $effect = Get-ResourceManagementComparisonEffect `
        -Baseline $baseline `
        -Candidate $candidate `
        -MinimumAbsoluteEffectSize $MinimumAbsoluteEffectSize

    $absoluteIncrease = [double]$candidate.median - [double]$baseline.median
    $relativeIncrease = $null
    $relativeChangeStatus = 'measured'
    $relativeExceeded = $false
    if ([double]$baseline.median -eq 0.0) {
        $relativeChangeStatus = 'not-applicable-zero-baseline'
    }
    else {
        $relativeIncrease = $absoluteIncrease / [double]$baseline.median
        $relativeExceeded = $relativeIncrease -gt $MaximumRelativeIncrease
    }
    $absoluteExceeded = $absoluteIncrease -gt $MaximumAbsoluteIncrease
    $budgetExceeded = $absoluteExceeded -or $relativeExceeded

    $diagnosticDecision = if ($baseline.noise_status -eq 'insufficient-samples' -or
        $candidate.noise_status -eq 'insufficient-samples') {
        'inconclusive-samples'
    }
    elseif ($baseline.noise_status -ne 'stable' -or $candidate.noise_status -ne 'stable') {
        'inconclusive-noise'
    }
    elseif (-not $budgetExceeded) {
        'within-budget'
    }
    elseif (-not $effect.statistically_significant_regression -or
        -not $effect.practically_significant_regression) {
        'inconclusive-effect'
    }
    else {
        'regression'
    }

    return [pscustomobject][ordered]@{
        schema_version = 1
        comparison_kind = 'resource-management-cohort'
        qualification_status = 'unverified'
        qualification_status_reason = 'no-trusted-report-qualification'
        diagnostic_decision = $diagnosticDecision
        baseline = $baseline
        candidate = $candidate
        outlier_receipts = [pscustomobject][ordered]@{
            baseline = New-ResourceManagementOutlierReceipt -Values $BaselineValues
            candidate = New-ResourceManagementOutlierReceipt -Values $CandidateValues
        }
        effect = $effect
        budget = [pscustomobject][ordered]@{
            maximum_absolute_increase = $MaximumAbsoluteIncrease
            maximum_relative_increase = $MaximumRelativeIncrease
            absolute_increase = $absoluteIncrease
            relative_change_status = $relativeChangeStatus
            relative_increase = $relativeIncrease
            absolute_exceeded = $absoluteExceeded
            relative_exceeded = $relativeExceeded
        }
    }
}

function Get-ResourceManagementComparisonProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    return Get-ResourceManagementSchemaProperty -Value $Value -Name $Name -Label $Label
}

function Assert-ResourceManagementComparisonProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$RequiredNames,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementSchemaProperties `
        -Value $Value `
        -RequiredNames $RequiredNames `
        -Label $Label
}

function Assert-ResourceManagementComparisonSha256 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    return Assert-ResourceManagementSchemaSha256 -Value $Value -Label $Label
}

function Test-ResourceManagementComparisonNumber {
    param([Parameter(Mandatory)]$Value)

    return Test-ResourceManagementSchemaJsonNumber -Value $Value
}

function ConvertTo-ResourceManagementComparisonNumber {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label,
        [double]$Maximum = [double]::MaxValue
    )

    return ConvertTo-ResourceManagementSchemaJsonNumber `
        -Value $Value `
        -Label $Label `
        -Maximum $Maximum `
        -InvalidRangeMessage "$Label must be a finite non-negative number no greater than $Maximum."
}

function Get-ResourceManagementComparisonQueryKey {
    param(
        [Parameter(Mandatory)]$Query,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementComparisonProperties `
        -Value $Query `
        -RequiredNames @('operation', 'query', 'offset', 'limit', 'elapsed_us', 'counters') `
        -Label $Label
    $operation = [string](Get-ResourceManagementComparisonProperty -Value $Query -Name 'operation' -Label $Label)
    $filter = Get-ResourceManagementComparisonProperty -Value $Query -Name 'query' -Label $Label
    Assert-ResourceManagementComparisonProperties `
        -Value $filter `
        -RequiredNames @('kind', 'state') `
        -Label "$Label filter"
    $kind = [string](Get-ResourceManagementComparisonProperty -Value $filter -Name 'kind' -Label "$Label filter")
    $state = [string](Get-ResourceManagementComparisonProperty -Value $filter -Name 'state' -Label "$Label filter")
    if ($operation -notin @('scan', 'page', 'asset-workspace-snapshot') -or
        $kind -ne 'Data' -or $state -ne 'any') {
        throw "$Label has an unsupported query identity."
    }
    $offset = $Query.offset
    $limit = $Query.limit
    if ($operation -eq 'page') {
        if ($null -eq $offset -or $null -eq $limit -or
            -not (Test-ResourceManagementComparisonNumber -Value $offset) -or
            -not (Test-ResourceManagementComparisonNumber -Value $limit) -or
            [double]$offset % 1 -ne 0 -or [double]$limit % 1 -ne 0 -or
            [double]$offset -lt 0 -or [double]$limit -lt 1) {
            throw "$Label has invalid page bounds."
        }
        return "$operation|$kind|$state|$([int]$offset)|$([int]$limit)"
    }
    if ($null -ne $offset -or $null -ne $limit) {
        throw "$Label must not declare page bounds for '$operation'."
    }
    return "$operation|$kind|$state||"
}

function Get-ResourceManagementComparisonStatisticsSamples {
    param(
        [Parameter(Mandatory)]$Statistics,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementComparisonProperties `
        -Value $Statistics `
        -RequiredNames @(
            'sample_count', 'raw_samples', 'min', 'median', 'p95', 'max', 'mean',
            'sample_standard_deviation', 'median_absolute_deviation',
            'coefficient_of_variation', 'confidence_95_lower', 'confidence_95_upper',
            'relative_margin_of_error', 'noise_status', 'total'
        ) `
        -Label $Label
    $samples = @(Get-ResourceManagementComparisonProperty -Value $Statistics -Name 'raw_samples' -Label $Label)
    if ($samples.Count -eq 0 -or [int]$Statistics.sample_count -ne $samples.Count) {
        throw "$Label raw sample count does not match sample_count."
    }
    [double[]]$values = @(
        foreach ($sample in $samples) {
            ConvertTo-ResourceManagementComparisonNumber -Value $sample -Label "$Label raw sample"
        }
    )
    return $values
}

function ConvertTo-ResourceManagementComparisonStatisticalPolicy {
    param(
        [Parameter(Mandatory)]$Policy,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementComparisonProperties `
        -Value $Policy `
        -RequiredNames @(
            'warmup_repetitions', 'measurement_repetitions', 'minimum_sample_count',
            'confidence_level', 'maximum_coefficient_of_variation',
            'maximum_relative_margin_of_error'
        ) `
        -Label $Label
    $warmup = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.warmup_repetitions -Label "$Label warmup_repetitions" -Maximum 10
    $measurements = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.measurement_repetitions -Label "$Label measurement_repetitions" -Maximum 50
    $minimumSamples = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.minimum_sample_count -Label "$Label minimum_sample_count" -Maximum 50
    $confidence = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.confidence_level -Label "$Label confidence_level" -Maximum 1
    $maximumCoefficientOfVariation = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.maximum_coefficient_of_variation `
        -Label "$Label maximum_coefficient_of_variation" `
        -Maximum 1
    $maximumRelativeMarginOfError = ConvertTo-ResourceManagementComparisonNumber `
        -Value $Policy.maximum_relative_margin_of_error `
        -Label "$Label maximum_relative_margin_of_error" `
        -Maximum 1
    if ($warmup % 1 -ne 0 -or $warmup -lt 1 -or
        $measurements % 1 -ne 0 -or $measurements -lt 20 -or
        $minimumSamples % 1 -ne 0 -or $minimumSamples -lt 20 -or
        $minimumSamples -gt $measurements -or $confidence -ne 0.95 -or
        $maximumCoefficientOfVariation -le 0 -or $maximumRelativeMarginOfError -le 0) {
        throw "$Label does not satisfy the supported statistical policy bounds."
    }
    return [pscustomobject][ordered]@{
        warmup_repetitions = [int]$warmup
        measurement_repetitions = [int]$measurements
        minimum_sample_count = [int]$minimumSamples
        confidence_level = $confidence
        maximum_coefficient_of_variation = $maximumCoefficientOfVariation
        maximum_relative_margin_of_error = $maximumRelativeMarginOfError
    }
}

function Get-ResourceManagementComparisonReportScenarios {
    param(
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementComparisonProperties `
        -Value $Report `
        -RequiredNames @(
            'schema_version', 'workload_family', 'measurement_status',
            'measurement_status_reason', 'workload_profile_id', 'workload_registry_receipt',
            'source_fingerprint', 'baseline_plan_sha256',
            'observation_sha256', 'observation_context', 'execution_protocol',
            'statistical_policy', 'scenarios'
        ) `
        -Label $Label
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Report `
        -SchemaId 'zircon.resource-management.baseline-report' `
        -Label $Label | Out-Null
    if ([string]$Report.measurement_status -ne 'unverified' -or
        [string]$Report.measurement_status_reason -ne 'untrusted-observation-context') {
        throw "$Label is not a supported fail-closed resource-management report."
    }
    $workloadProfileId = [string]$Report.workload_profile_id
    [void](Get-ResourceManagementWorkloadProfile -ProfileId $workloadProfileId)
    $workloadRegistryReceipt = Assert-ResourceManagementWorkloadRegistryReceipt `
        -Receipt $Report.workload_registry_receipt `
        -Label "$Label workload registry receipt"
    [void](Assert-ResourceManagementComparisonSha256 -Value ([string]$Report.source_fingerprint) -Label "$Label source_fingerprint")
    [void](Assert-ResourceManagementComparisonSha256 -Value ([string]$Report.baseline_plan_sha256) -Label "$Label baseline_plan_sha256")
    [void](Assert-ResourceManagementComparisonSha256 -Value ([string]$Report.observation_sha256) -Label "$Label observation_sha256")
    $observationContext = Resolve-ResourceManagementObservationContext `
        -Context $Report.observation_context `
        -ExpectedSourceFingerprint ([string]$Report.source_fingerprint)
    $executionProtocol = Resolve-ResourceManagementExecutionProtocol `
        -Protocol $Report.execution_protocol
    $statisticalPolicy = ConvertTo-ResourceManagementComparisonStatisticalPolicy `
        -Policy (Get-ResourceManagementComparisonProperty -Value $Report -Name 'statistical_policy' -Label $Label) `
        -Label "$Label statistical policy"
    $scenarios = @(Get-ResourceManagementComparisonProperty -Value $Report -Name 'scenarios' -Label $Label)
    if ($scenarios.Count -eq 0) {
        throw "$Label has no scenarios."
    }
    $index = @{}
    $allSampleExecutionProtocols = [Collections.Generic.List[object]]::new()
    foreach ($scenario in $scenarios) {
        Assert-ResourceManagementComparisonProperties `
            -Value $scenario `
            -RequiredNames @(
                'logical_id', 'mode', 'project_role', 'process_lifecycle', 'data_asset_count',
                'data_inventory_sha256', 'attempt_count', 'warmup_count', 'sample_count',
                'statistical_status', 'process_contexts', 'sample_protocols', 'process', 'queries'
            ) `
            -Label "$Label scenario"
        $logicalId = [string]$scenario.logical_id
        if ($logicalId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$' -or $index.ContainsKey($logicalId)) {
            throw "$Label has an invalid or duplicate scenario logical_id '$logicalId'."
        }
        $queries = @($scenario.queries)
        if ($queries.Count -eq 0) {
            throw "$Label scenario '$logicalId' has no queries."
        }
        if ([int]$scenario.warmup_count -ne $statisticalPolicy.warmup_repetitions -or
            [int]$scenario.sample_count -ne $statisticalPolicy.measurement_repetitions -or
            [int]$scenario.attempt_count -ne ($statisticalPolicy.warmup_repetitions + $statisticalPolicy.measurement_repetitions)) {
            throw "$Label scenario '$logicalId' does not match its statistical policy attempt counts."
        }
        $mode = [string]$scenario.mode
        $processLifecycle = [string]$scenario.process_lifecycle
        $expectedProcessLifecycle = if ($mode -eq 'stable-generation') { 'same-process' } else { 'fresh-process' }
        if ($mode -notin @('cold-open', 'stable-generation', 'one-percent-change') -or
            $processLifecycle -cne $expectedProcessLifecycle) {
            throw "$Label scenario '$logicalId' has an invalid process_lifecycle."
        }
        $processContextReceipts = @($scenario.process_contexts)
        if ($processContextReceipts.Count -ne [int]$scenario.attempt_count) {
            throw "$Label scenario '$logicalId' does not bind every sample process context."
        }
        $processContexts = [Collections.Generic.List[object]]::new()
        $processContextsByAttempt = @{}
        $seenAttempts = [Collections.Generic.HashSet[int]]::new()
        foreach ($receipt in $processContextReceipts) {
            Assert-ResourceManagementComparisonProperties `
                -Value $receipt `
                -RequiredNames @('attempt', 'sample_phase', 'process_context') `
                -Label "$Label scenario '$logicalId' process context receipt"
            $attemptNumber = ConvertTo-ResourceManagementComparisonNumber `
                -Value $receipt.attempt `
                -Label "$Label scenario '$logicalId' process context attempt" `
                -Maximum ([int]$scenario.attempt_count)
            if ($attemptNumber % 1 -ne 0 -or $attemptNumber -lt 1) {
                throw "$Label scenario '$logicalId' has an invalid process context attempt."
            }
            $attempt = [int]$attemptNumber
            if (-not $seenAttempts.Add($attempt)) {
                throw "$Label scenario '$logicalId' has an invalid or duplicate process context attempt."
            }
            $expectedPhase = if ($attempt -le $statisticalPolicy.warmup_repetitions) { 'warmup' } else { 'measurement' }
            if ([string]$receipt.sample_phase -cne $expectedPhase) {
                throw "$Label scenario '$logicalId' process context attempt $attempt has an invalid sample_phase."
            }
            $resolvedProcessContext = Resolve-ResourceManagementSampleProcessContext `
                -Context $receipt.process_context `
                -Label "$Label scenario '$logicalId' process context attempt $attempt"
            $processContexts.Add($resolvedProcessContext) | Out-Null
            $processContextsByAttempt[$attempt] = $resolvedProcessContext
        }
        Assert-ResourceManagementSampleProcessLifecycle `
            -ProcessContexts $processContexts.ToArray() `
            -ProcessLifecycle $processLifecycle `
            -Label "$Label scenario '$logicalId' process contexts"
        $sampleProtocolReceipts = @($scenario.sample_protocols)
        if ($sampleProtocolReceipts.Count -ne [int]$scenario.attempt_count) {
            throw "$Label scenario '$logicalId' does not bind every sample execution protocol."
        }
        $seenProtocolAttempts = [Collections.Generic.HashSet[int]]::new()
        foreach ($receipt in $sampleProtocolReceipts) {
            Assert-ResourceManagementComparisonProperties `
                -Value $receipt `
                -RequiredNames @('attempt', 'sample_phase', 'execution_protocol') `
                -Label "$Label scenario '$logicalId' sample protocol receipt"
            $attemptNumber = ConvertTo-ResourceManagementComparisonNumber `
                -Value $receipt.attempt `
                -Label "$Label scenario '$logicalId' sample protocol attempt" `
                -Maximum ([int]$scenario.attempt_count)
            if ($attemptNumber % 1 -ne 0 -or $attemptNumber -lt 1) {
                throw "$Label scenario '$logicalId' has an invalid sample protocol attempt."
            }
            $attempt = [int]$attemptNumber
            if (-not $seenProtocolAttempts.Add($attempt) -or -not $processContextsByAttempt.ContainsKey($attempt)) {
                throw "$Label scenario '$logicalId' has an invalid or duplicate sample protocol attempt."
            }
            $expectedPhase = if ($attempt -le $statisticalPolicy.warmup_repetitions) { 'warmup' } else { 'measurement' }
            if ([string]$receipt.sample_phase -cne $expectedPhase) {
                throw "$Label scenario '$logicalId' sample protocol attempt $attempt has an invalid sample_phase."
            }
            $resolvedSampleProtocol = Resolve-ResourceManagementSampleExecutionProtocol `
                -Protocol $receipt.execution_protocol `
                -ExpectedMode $mode `
                -ExpectedProcessId $processContextsByAttempt[$attempt].process_id `
                -Label "$Label scenario '$logicalId' sample protocol attempt $attempt"
            $allSampleExecutionProtocols.Add($resolvedSampleProtocol) | Out-Null
        }
        $queryIndex = @{}
        foreach ($query in $queries) {
            $queryKey = Get-ResourceManagementComparisonQueryKey -Query $query -Label "$Label scenario '$logicalId' query"
            if ($queryIndex.ContainsKey($queryKey)) {
                throw "$Label scenario '$logicalId' has a duplicate query '$queryKey'."
            }
            [void](Get-ResourceManagementComparisonStatisticsSamples `
                    -Statistics $query.elapsed_us `
                    -Label "$Label scenario '$logicalId' query '$queryKey' elapsed_us")
            $queryIndex[$queryKey] = $query
        }
        $index[$logicalId] = [pscustomobject]@{
            scenario = $scenario
            queries = $queryIndex
            process_contexts = $processContexts.ToArray()
        }
    }
    Assert-ResourceManagementExecutionProtocolSequence `
        -SampleProtocols $allSampleExecutionProtocols.ToArray()
    return [pscustomobject]@{
        scenarios = $index
        statistical_policy = $statisticalPolicy
        observation_context = $observationContext
        execution_protocol = $executionProtocol
        workload_profile_id = $workloadProfileId
        workload_registry_receipt = $workloadRegistryReceipt
    }
}

function Get-ResourceManagementComparisonBudgetIndex {
    param(
        [Parameter(Mandatory)]$Policy,
        [Parameter(Mandatory)][string]$ApprovedBaselineReportSha256,
        [Parameter(Mandatory)][string]$ApprovedBaselineReceiptSha256
    )

    Assert-ResourceManagementComparisonProperties `
        -Value $Policy `
        -RequiredNames @(
            'schema_version', 'policy_kind', 'approved_baseline_report_sha256',
            'approved_baseline_receipt_sha256', 'budgets', 'links'
        ) `
        -Label 'Resource-management comparison policy'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Policy `
        -SchemaId 'zircon.resource-management.comparison-policy' `
        -Label 'Resource-management comparison policy' | Out-Null
    $declaredReceiptSha256 = Assert-ResourceManagementComparisonSha256 `
        -Value ([string]$Policy.approved_baseline_receipt_sha256) `
        -Label 'Resource-management comparison approved baseline receipt SHA-256'
    if (-not $declaredReceiptSha256.Equals($ApprovedBaselineReceiptSha256, [StringComparison]::Ordinal)) {
        throw 'Resource-management comparison policy approved baseline receipt SHA-256 does not match the supplied receipt.'
    }
    $declaredBaselineSha256 = Assert-ResourceManagementComparisonSha256 `
        -Value ([string]$Policy.approved_baseline_report_sha256) `
        -Label 'Resource-management comparison approved baseline report SHA-256'
    if (-not $declaredBaselineSha256.Equals($ApprovedBaselineReportSha256, [StringComparison]::Ordinal)) {
        throw 'Resource-management comparison policy approved baseline report SHA-256 does not match the supplied report.'
    }
    Assert-ResourceManagementComparisonProperties `
        -Value $Policy.links `
        -RequiredNames @('trend_uri', 'bisect_uri') `
        -Label 'Resource-management comparison policy links'
    foreach ($name in @('trend_uri', 'bisect_uri')) {
        $uri = [string]$Policy.links.$name
        if ($uri.Length -gt 2048 -or $uri -notmatch '^https://[^\s]+$') {
            throw "Resource-management comparison policy link '$name' must be a bounded HTTPS URI."
        }
    }
    $budgets = @($Policy.budgets)
    if ($budgets.Count -eq 0) {
        throw 'Resource-management comparison policy has no query budget.'
    }
    $index = @{}
    foreach ($budget in $budgets) {
        Assert-ResourceManagementComparisonProperties `
            -Value $budget `
            -RequiredNames @(
                'scenario_logical_id', 'operation', 'query_kind', 'query_state', 'offset', 'limit',
                'maximum_absolute_increase', 'maximum_relative_increase',
                'minimum_absolute_effect_size'
            ) `
            -Label 'Resource-management comparison budget'
        $queryKey = "$([string]$budget.operation)|$([string]$budget.query_kind)|$([string]$budget.query_state)|"
        if ([string]$budget.operation -eq 'page') {
            if ($null -eq $budget.offset -or $null -eq $budget.limit) {
                throw 'Resource-management comparison page budget is missing bounds.'
            }
            $queryKey += "$([int]$budget.offset)|$([int]$budget.limit)"
        }
        elseif ($null -ne $budget.offset -or $null -ne $budget.limit) {
            throw 'Resource-management comparison non-page budget must not declare bounds.'
        }
        else {
            $queryKey += '|'
        }
        $key = "$([string]$budget.scenario_logical_id)$([char]0)$queryKey"
        if ($index.ContainsKey($key)) {
            throw "Resource-management comparison policy has a duplicate budget for '$key'."
        }
        $maximumAbsoluteIncrease = ConvertTo-ResourceManagementComparisonNumber `
            -Value $budget.maximum_absolute_increase `
            -Label 'Resource-management comparison maximum_absolute_increase'
        $maximumRelativeIncrease = ConvertTo-ResourceManagementComparisonNumber `
            -Value $budget.maximum_relative_increase `
            -Label 'Resource-management comparison maximum_relative_increase' `
            -Maximum 1.0
        $minimumAbsoluteEffectSize = ConvertTo-ResourceManagementComparisonNumber `
            -Value $budget.minimum_absolute_effect_size `
            -Label 'Resource-management comparison minimum_absolute_effect_size'
        $index[$key] = [pscustomobject]@{
            maximum_absolute_increase = $maximumAbsoluteIncrease
            maximum_relative_increase = $maximumRelativeIncrease
            minimum_absolute_effect_size = $minimumAbsoluteEffectSize
        }
    }
    return $index
}

function Compare-ResourceManagementReports {
    param(
        [Parameter(Mandatory)]$ApprovedBaselineReport,
        [Parameter(Mandatory)][string]$ApprovedBaselineReportSha256,
        [Parameter(Mandatory)]$ApprovedBaselineReceipt,
        [Parameter(Mandatory)][string]$ApprovedBaselineReceiptSha256,
        [Parameter(Mandatory)]$CandidateReport,
        [Parameter(Mandatory)][string]$CandidateReportSha256,
        [Parameter(Mandatory)]$Policy,
        [Parameter(Mandatory)][string]$PolicySha256
    )

    $approvedSha256 = Assert-ResourceManagementComparisonSha256 `
        -Value $ApprovedBaselineReportSha256 `
        -Label 'Approved baseline report SHA-256'
    $approvedReceiptSha256 = Assert-ResourceManagementComparisonSha256 `
        -Value $ApprovedBaselineReceiptSha256 `
        -Label 'Approved baseline receipt SHA-256'
    $candidateSha256 = Assert-ResourceManagementComparisonSha256 `
        -Value $CandidateReportSha256 `
        -Label 'Candidate report SHA-256'
    $policySha256Value = Assert-ResourceManagementComparisonSha256 `
        -Value $PolicySha256 `
        -Label 'Resource-management comparison policy SHA-256'
    $budgetIndex = Get-ResourceManagementComparisonBudgetIndex `
        -Policy $Policy `
        -ApprovedBaselineReportSha256 $approvedSha256 `
        -ApprovedBaselineReceiptSha256 $approvedReceiptSha256
    $approved = Get-ResourceManagementComparisonReportScenarios `
        -Report $ApprovedBaselineReport `
        -Label 'Approved baseline report'
    $candidate = Get-ResourceManagementComparisonReportScenarios `
        -Report $CandidateReport `
        -Label 'Candidate report'
    $approvalVerification = Resolve-ResourceManagementBaselineApproval `
        -Receipt $ApprovedBaselineReceipt `
        -ReceiptSha256 $approvedReceiptSha256 `
        -ApprovedBaselineReportSha256 $approvedSha256 `
        -WorkloadProfileId $approved.workload_profile_id `
        -TrustRegistrySnapshot (Get-ResourceManagementApprovalTrustRegistrySnapshot) `
        -VerificationTimeUtc ([DateTimeOffset]::UtcNow)
    $approvedPolicyKey = @(
        $approved.statistical_policy.PSObject.Properties | ForEach-Object { [string]$_.Value }
    ) -join '|'
    $candidatePolicyKey = @(
        $candidate.statistical_policy.PSObject.Properties | ForEach-Object { [string]$_.Value }
    ) -join '|'
    if (-not $approvedPolicyKey.Equals($candidatePolicyKey, [StringComparison]::Ordinal)) {
        throw 'Approved baseline and candidate report statistical policy values do not match.'
    }
    Assert-ResourceManagementObservationContextsComparable `
        -ApprovedBaseline $approved.observation_context `
        -Candidate $candidate.observation_context
    Assert-ResourceManagementExecutionProtocolsComparable `
        -ApprovedBaseline $approved.execution_protocol `
        -Candidate $candidate.execution_protocol
    if ([string]$approved.workload_profile_id -cne [string]$candidate.workload_profile_id -or
        [string]$approved.workload_registry_receipt.sha256 -cne [string]$candidate.workload_registry_receipt.sha256) {
        throw 'Approved baseline and candidate workload profile bindings differ.'
    }
    if ($approved.scenarios.Count -ne $candidate.scenarios.Count -or
        @($approved.scenarios.Keys | Where-Object { -not $candidate.scenarios.ContainsKey($_) }).Count -ne 0) {
        throw 'Approved baseline and candidate report scenario sets do not match.'
    }

    $comparisons = [Collections.Generic.List[object]]::new()
    $consumedBudgets = [Collections.Generic.HashSet[string]]::new()
    foreach ($logicalId in @($approved.scenarios.Keys | Sort-Object)) {
        $approvedScenario = $approved.scenarios[$logicalId]
        $candidateScenario = $candidate.scenarios[$logicalId]
        if ([string]$approvedScenario.scenario.mode -ne [string]$candidateScenario.scenario.mode -or
            [int]$approvedScenario.scenario.data_asset_count -ne [int]$candidateScenario.scenario.data_asset_count -or
            $approvedScenario.queries.Count -ne $candidateScenario.queries.Count -or
            @($approvedScenario.queries.Keys | Where-Object { -not $candidateScenario.queries.ContainsKey($_) }).Count -ne 0) {
            throw "Approved baseline and candidate report query set does not match for scenario '$logicalId'."
        }
        foreach ($queryKey in @($approvedScenario.queries.Keys | Sort-Object)) {
            $budgetKey = "$logicalId$([char]0)$queryKey"
            if (-not $budgetIndex.ContainsKey($budgetKey)) {
                throw "Resource-management comparison policy is missing a budget for '$logicalId/$queryKey'."
            }
            [void]$consumedBudgets.Add($budgetKey)
            $approvedQuery = $approvedScenario.queries[$queryKey]
            $candidateQuery = $candidateScenario.queries[$queryKey]
            $baselineValues = Get-ResourceManagementComparisonStatisticsSamples `
                -Statistics $approvedQuery.elapsed_us `
                -Label "Approved baseline '$logicalId/$queryKey' elapsed_us"
            $candidateValues = Get-ResourceManagementComparisonStatisticsSamples `
                -Statistics $candidateQuery.elapsed_us `
                -Label "Candidate '$logicalId/$queryKey' elapsed_us"
            $budget = $budgetIndex[$budgetKey]
            $comparison = Compare-ResourceManagementCohorts `
                -BaselineValues $baselineValues `
                -CandidateValues $candidateValues `
                -MaximumAbsoluteIncrease $budget.maximum_absolute_increase `
                -MaximumRelativeIncrease $budget.maximum_relative_increase `
                -MinimumSampleCount ([int]$approved.statistical_policy.minimum_sample_count) `
                -MaximumCoefficientOfVariation ([double]$approved.statistical_policy.maximum_coefficient_of_variation) `
                -MaximumRelativeMarginOfError ([double]$approved.statistical_policy.maximum_relative_margin_of_error) `
                -MinimumAbsoluteEffectSize $budget.minimum_absolute_effect_size
            $comparisons.Add([pscustomobject][ordered]@{
                    scenario_logical_id = $logicalId
                    mode = [string]$approvedScenario.scenario.mode
                    data_asset_count = [int]$approvedScenario.scenario.data_asset_count
                    operation = [string]$approvedQuery.operation
                    query = $approvedQuery.query
                    offset = $approvedQuery.offset
                    limit = $approvedQuery.limit
                    comparison = $comparison
                }) | Out-Null
        }
    }
    if ($consumedBudgets.Count -ne $budgetIndex.Count) {
        throw 'Resource-management comparison policy contains a budget not present in the report query set.'
    }

    $comparisonArray = $comparisons.ToArray()
    $regressionCount = @($comparisonArray | Where-Object { $_.comparison.diagnostic_decision -eq 'regression' }).Count
    $withinBudgetCount = @($comparisonArray | Where-Object { $_.comparison.diagnostic_decision -eq 'within-budget' }).Count
    $inconclusiveCount = $comparisonArray.Count - $regressionCount - $withinBudgetCount
    $diagnosticDecision = if ($regressionCount -gt 0) {
        'regression'
    }
    elseif ($inconclusiveCount -gt 0) {
        'inconclusive'
    }
    else {
        'within-budget'
    }

    return [pscustomobject][ordered]@{
        schema_version = 3
        report_kind = 'resource-management-comparison'
        qualification_status = 'unverified'
        qualification_status_reason = 'untrusted-observation-context'
        diagnostic_decision = $diagnosticDecision
        workload_profile_id = $approved.workload_profile_id
        workload_registry_receipt = $approved.workload_registry_receipt
        approved_baseline_report_sha256 = $approvedSha256
        approved_baseline_receipt_sha256 = $approvedReceiptSha256
        approval_verification = $approvalVerification
        candidate_report_sha256 = $candidateSha256
        policy_sha256 = $policySha256Value
        observation_contexts = [pscustomobject][ordered]@{
            approved_baseline = $approved.observation_context
            candidate = $candidate.observation_context
        }
        execution_protocols = [pscustomobject][ordered]@{
            approved_baseline = $approved.execution_protocol
            candidate = $candidate.execution_protocol
        }
        links = $Policy.links
        decision_summary = [pscustomobject][ordered]@{
            comparison_count = $comparisonArray.Count
            regression_count = $regressionCount
            within_budget_count = $withinBudgetCount
            inconclusive_count = $inconclusiveCount
        }
        comparisons = $comparisonArray
    }
}

function ConvertTo-ResourceManagementComparisonMarkdown {
    param([Parameter(Mandatory)]$Report)

    $lines = [Collections.Generic.List[string]]::new()
    $lines.Add('# Resource-management comparison')
    $lines.Add('')
    $lines.Add("- Qualification status: $($Report.qualification_status)")
    $lines.Add("- Qualification reason: $($Report.qualification_status_reason)")
    $lines.Add("- Approval verification: $($Report.approval_verification.verification_status) ($($Report.approval_verification.verification_reason))")
    $lines.Add("- Diagnostic decision: $($Report.diagnostic_decision)")
    $lines.Add("- Approved ProductReceipt: $($Report.observation_contexts.approved_baseline.product_receipt.receipt_id)")
    $lines.Add("- Candidate ProductReceipt: $($Report.observation_contexts.candidate.product_receipt.receipt_id)")
    $lines.Add("- Machine: $($Report.observation_contexts.candidate.machine.machine_id_sha256)")
    $lines.Add("- Collector: $($Report.observation_contexts.candidate.collector.collector_id) $($Report.observation_contexts.candidate.collector.collector_version)")
    $lines.Add("- Approved order receipt: $($Report.execution_protocols.approved_baseline.order_receipt_sha256)")
    $lines.Add("- Candidate order receipt: $($Report.execution_protocols.candidate.order_receipt_sha256)")
    $lines.Add("- Trend: $($Report.links.trend_uri)")
    $lines.Add("- Bisect: $($Report.links.bisect_uri)")
    $lines.Add('')
    $lines.Add('| Scenario | Query | Decision | Baseline median us | Candidate median us | Absolute increase | Relative increase | Hedges g |')
    $lines.Add('| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |')
    foreach ($entry in $Report.comparisons) {
        $queryLabel = if ($entry.operation -eq 'page') {
            "page offset=$($entry.offset) limit=$($entry.limit)"
        }
        else {
            [string]$entry.operation
        }
        $relativeIncrease = if ($null -eq $entry.comparison.budget.relative_increase) {
            $entry.comparison.budget.relative_change_status
        }
        else {
            [string]$entry.comparison.budget.relative_increase
        }
        $hedgesG = if ($null -eq $entry.comparison.effect.hedges_g) {
            $entry.comparison.effect.standardized_effect_status
        }
        else {
            [string]$entry.comparison.effect.hedges_g
        }
        $lines.Add("| $($entry.scenario_logical_id) | $queryLabel | $($entry.comparison.diagnostic_decision) | $($entry.comparison.baseline.median) | $($entry.comparison.candidate.median) | $($entry.comparison.budget.absolute_increase) | $relativeIncrease | $hedgesG |")
    }
    return ($lines -join [Environment]::NewLine) + [Environment]::NewLine
}

Export-ModuleMember -Function @(
    'Compare-ResourceManagementCohorts',
    'Compare-ResourceManagementReports',
    'ConvertTo-ResourceManagementComparisonMarkdown',
    'New-ResourceManagementOutlierReceipt'
)
