[CmdletBinding()]
param(
    [string]$BaselinePlanPath,
    [string]$ObservationPath,
    [string]$OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementJsonEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementWorkloadRegistry.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementObservationContext.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementExecutionProtocol.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementStatistics.psm1') -Force -ErrorAction Stop

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = New-MvpArtifactStoragePath `
        -NamespaceId 'resource-management-reports' `
        -InstanceId ([guid]::NewGuid().ToString('N'))
}

$script:ResourceManagementExpectedMeasurements = @{
    scan = @(
        'resource_management.scan.instances',
        'resource_management.scan.matching_rows',
        'resource_management.scan.rows_emitted',
        'resource_management.scan.shard_candidate_checks',
        'resource_management.scan.filtered_rows_skipped'
    )
    page = @(
        'resource_management.page.instances',
        'resource_management.page.matching_rows',
        'resource_management.page.candidate_rows',
        'resource_management.page.rows_returned',
        'resource_management.page.shard_candidate_checks',
        'resource_management.page.filtered_rows_skipped'
    )
    'asset-workspace-snapshot' = @(
        'asset_workspace.snapshot.instances',
        'asset_workspace.catalog_asset_count',
        'asset_workspace.visible_asset_count',
        'asset_workspace.row_by_locator.calls',
        'asset_workspace.row_by_locator.shard_probes',
        'asset_workspace.selection_lookup.calls',
        'asset_workspace.surface_clone.instances'
    )
}
$script:ResourceManagementMaximumBaselinePlanBytes = 4MB
$script:ResourceManagementMaximumObservationBytes = 64MB

function Get-ResourceManagementReportProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    return Get-ResourceManagementSchemaProperty -Value $Value -Name $Name -Label $Label
}

function Get-ResourceManagementReportOptionalProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    return Get-ResourceManagementSchemaOptionalProperty -Value $Value -Name $Name
}

function Get-ResourceManagementReportArrayProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    return Get-ResourceManagementSchemaArrayProperty -Value $Value -Name $Name -Label $Label
}

function Assert-ResourceManagementReportProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$RequiredNames,
        [string[]]$OptionalNames = @(),
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementSchemaProperties `
        -Value $Value `
        -RequiredNames $RequiredNames `
        -OptionalNames $OptionalNames `
        -Label $Label
}

function Assert-ResourceManagementReportSha256 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    return Assert-ResourceManagementSchemaSha256 -Value $Value -Label $Label
}

function Get-ResourceManagementReportJsonEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumBytes = $script:ResourceManagementMaximumObservationBytes
    )

    return Get-ResourceManagementJsonEvidence `
        -Path $Path `
        -Label $Label `
        -MaximumBytes $MaximumBytes
}

function Get-ResourceManagementReportStatistics {
    param(
        [Parameter(Mandatory)][double[]]$Values,
        [Parameter(Mandatory)]$StatisticalPolicy
    )

    return Get-ResourceManagementCohortStatistics `
        -Values $Values `
        -MinimumSampleCount $StatisticalPolicy.minimum_sample_count `
        -MaximumCoefficientOfVariation $StatisticalPolicy.maximum_coefficient_of_variation `
        -MaximumRelativeMarginOfError $StatisticalPolicy.maximum_relative_margin_of_error
}

function Test-ResourceManagementReportNumberType {
    param([Parameter(Mandatory)]$Value)

    return Test-ResourceManagementSchemaJsonNumber -Value $Value
}

function ConvertTo-ResourceManagementReportNumber {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    return ConvertTo-ResourceManagementSchemaJsonNumber `
        -Value $Value `
        -Label $Label `
        -InvalidRangeMessage "$Label must be a finite non-negative number."
}

function ConvertTo-ResourceManagementReportNonNegativeInteger {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    return ConvertTo-ResourceManagementSchemaNonNegativeInteger -Value $Value -Label $Label
}

function Get-ResourceManagementReportStatisticalPolicy {
    param([Parameter(Mandatory)]$Value)

    Assert-ResourceManagementReportProperties `
        -Value $Value `
        -RequiredNames @(
            'warmup_repetitions',
            'measurement_repetitions',
            'minimum_sample_count',
            'confidence_level',
            'maximum_coefficient_of_variation',
            'maximum_relative_margin_of_error') `
        -Label 'Baseline statistical policy'
    $warmupRepetitions = [int](ConvertTo-ResourceManagementReportNonNegativeInteger `
            -Value $Value.warmup_repetitions `
            -Label 'Baseline statistical policy warmup_repetitions')
    $measurementRepetitions = [int](ConvertTo-ResourceManagementReportNonNegativeInteger `
            -Value $Value.measurement_repetitions `
            -Label 'Baseline statistical policy measurement_repetitions')
    $minimumSampleCount = [int](ConvertTo-ResourceManagementReportNonNegativeInteger `
            -Value $Value.minimum_sample_count `
            -Label 'Baseline statistical policy minimum_sample_count')
    $confidenceLevel = ConvertTo-ResourceManagementReportNumber `
        -Value $Value.confidence_level `
        -Label 'Baseline statistical policy confidence_level'
    $maximumCoefficientOfVariation = ConvertTo-ResourceManagementReportNumber `
        -Value $Value.maximum_coefficient_of_variation `
        -Label 'Baseline statistical policy maximum_coefficient_of_variation'
    $maximumRelativeMarginOfError = ConvertTo-ResourceManagementReportNumber `
        -Value $Value.maximum_relative_margin_of_error `
        -Label 'Baseline statistical policy maximum_relative_margin_of_error'
    if ($warmupRepetitions -lt 1 -or $warmupRepetitions -gt 10 -or
        $measurementRepetitions -lt 20 -or $measurementRepetitions -gt 50 -or
        $minimumSampleCount -lt 20 -or $minimumSampleCount -gt $measurementRepetitions -or
        $confidenceLevel -ne 0.95 -or
        $maximumCoefficientOfVariation -le 0 -or $maximumCoefficientOfVariation -gt 0.25 -or
        $maximumRelativeMarginOfError -le 0 -or $maximumRelativeMarginOfError -gt 0.25) {
        throw 'Baseline statistical policy is outside the admitted warmup, sample, confidence, or noise bounds.'
    }
    return [pscustomobject][ordered]@{
        warmup_repetitions = $warmupRepetitions
        measurement_repetitions = $measurementRepetitions
        minimum_sample_count = $minimumSampleCount
        confidence_level = $confidenceLevel
        maximum_coefficient_of_variation = $maximumCoefficientOfVariation
        maximum_relative_margin_of_error = $maximumRelativeMarginOfError
    }
}

function Get-ResourceManagementReportQueryKey {
    param(
        [Parameter(Mandatory)]$Query,
        [Parameter(Mandatory)][string]$Label
    )

    $operation = [string](Get-ResourceManagementReportProperty -Value $Query -Name 'operation' -Label $Label)
    $filter = Get-ResourceManagementReportProperty -Value $Query -Name 'query' -Label $Label
    Assert-ResourceManagementReportProperties `
        -Value $filter `
        -RequiredNames @('kind', 'state') `
        -Label "$Label filter"
    $kind = [string](Get-ResourceManagementReportProperty -Value $filter -Name 'kind' -Label "$Label query")
    $state = [string](Get-ResourceManagementReportProperty -Value $filter -Name 'state' -Label "$Label query")
    if ($operation -notin @('scan', 'page', 'asset-workspace-snapshot') -or $kind -ne 'Data' -or $state -ne 'any') {
        throw "$Label has an unsupported resource-management query shape."
    }
    $offset = Get-ResourceManagementReportOptionalProperty -Value $Query -Name 'offset'
    $limit = Get-ResourceManagementReportOptionalProperty -Value $Query -Name 'limit'
    if ($operation -eq 'page') {
        if ($null -eq $offset -or $null -eq $limit -or [int]$offset -lt 0 -or [int]$limit -lt 1) {
            throw "$Label page query requires a non-negative offset and positive limit."
        }
        return "$operation|$kind|$state|$([int]$offset)|$([int]$limit)"
    }
    if ($null -ne $offset -or $null -ne $limit) {
        throw "$Label must not declare page bounds for '$operation'."
    }
    return "$operation|$kind|$state||"
}

function Get-ResourceManagementReportQueryDescription {
    param(
        [Parameter(Mandatory)]$Query,
        [Parameter(Mandatory)][string]$Key
    )

    $filter = $Query.query
    $result = [ordered]@{
        operation = [string]$Query.operation
        query = [ordered]@{
            kind = [string]$filter.kind
            state = [string]$filter.state
        }
    }
    if ([string]$Query.operation -eq 'page') {
        $result['offset'] = [int]$Query.offset
        $result['limit'] = [int]$Query.limit
    }
    $result['key'] = $Key
    return $result
}

function Assert-ResourceManagementReportExpectedMeasurements {
    param(
        [Parameter(Mandatory)][string]$Operation,
        [Parameter(Mandatory)][string[]]$CounterNames,
        [Parameter(Mandatory)][string]$Label
    )

    $expected = @($script:ResourceManagementExpectedMeasurements[$Operation])
    if ($null -eq $expected -or $CounterNames.Count -ne $expected.Count -or
        @($expected | Where-Object { $_ -notin $CounterNames }).Count -gt 0) {
        throw "$Label does not declare the required $Operation measurement contract."
    }
}

function Get-ResourceManagementReportPlanScenarios {
    param([Parameter(Mandatory)]$BaselinePlan)

    Assert-ResourceManagementReportProperties `
        -Value $BaselinePlan `
        -RequiredNames @(
            'schema_version', 'workload_family', 'workload_profile_id',
            'workload_registry_receipt', 'source_fingerprint', 'resource_kind',
            'statistical_policy', 'scenarios') `
        -Label 'Baseline plan'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $BaselinePlan `
        -SchemaId 'zircon.resource-management.baseline-plan' `
        -Label 'Baseline plan' | Out-Null
    $workloadRegistryReceipt = Assert-ResourceManagementWorkloadRegistryReceipt `
        -Receipt (Get-ResourceManagementReportProperty `
            -Value $BaselinePlan `
            -Name 'workload_registry_receipt' `
            -Label 'Baseline plan') `
        -Label 'Baseline plan workload registry receipt'
    $workloadProfileId = [string](Get-ResourceManagementReportProperty `
            -Value $BaselinePlan `
            -Name 'workload_profile_id' `
            -Label 'Baseline plan')
    $workloadProfile = Get-ResourceManagementWorkloadProfile -ProfileId $workloadProfileId
    $resourceKind = [string](Get-ResourceManagementReportProperty `
            -Value $BaselinePlan -Name 'resource_kind' -Label 'Baseline plan')
    if ($workloadProfile.asset_kinds -cnotcontains $resourceKind) {
        throw 'Baseline plan has an unsupported schema.'
    }
    $sourceFingerprint = Assert-ResourceManagementReportSha256 `
        -Value ([string](Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'source_fingerprint' -Label 'Baseline plan')) `
        -Label 'Baseline plan source_fingerprint'
    $statisticalPolicy = Get-ResourceManagementReportStatisticalPolicy `
        -Value (Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'statistical_policy' -Label 'Baseline plan')
    $scenarioMap = @{}
    foreach ($scenario in @(Get-ResourceManagementReportArrayProperty -Value $BaselinePlan -Name 'scenarios' -Label 'Baseline plan')) {
        Assert-ResourceManagementReportProperties `
            -Value $scenario `
            -RequiredNames @('logical_id', 'mode', 'project_role', 'process_lifecycle', 'data_asset_count', 'data_inventory_sha256', 'required_repetitions', 'queries') `
            -OptionalNames @('project_id', 'project_manifest_sha256', 'required_generation_relation', 'change_mode', 'change_percent', 'changed_asset_count', 'changed_virtual_paths', 'resource_kind', 'data_virtual_prefix', 'data_source_pattern', 'change_set_manifest_sha256') `
            -Label 'Baseline scenario'
        $logicalId = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'logical_id' -Label 'Baseline scenario')
        if ([string]::IsNullOrWhiteSpace($logicalId) -or $scenarioMap.ContainsKey($logicalId)) {
            throw "Baseline plan has an empty or duplicate logical_id '$logicalId'."
        }
        $dataAssetCount = [int](Get-ResourceManagementReportProperty -Value $scenario -Name 'data_asset_count' -Label "Baseline scenario '$logicalId'")
        $repeatCount = [int](Get-ResourceManagementReportProperty -Value $scenario -Name 'required_repetitions' -Label "Baseline scenario '$logicalId'")
        $inventory = Assert-ResourceManagementReportSha256 `
            -Value ([string](Get-ResourceManagementReportProperty -Value $scenario -Name 'data_inventory_sha256' -Label "Baseline scenario '$logicalId'")) `
            -Label "Baseline scenario '$logicalId' data_inventory_sha256"
        $expectedRepetitions = $statisticalPolicy.warmup_repetitions + $statisticalPolicy.measurement_repetitions
        if ($dataAssetCount -lt 1 -or $dataAssetCount -gt 100000 -or
            $dataAssetCount -lt $workloadProfile.minimum_asset_count -or
            $dataAssetCount -gt $workloadProfile.maximum_asset_count -or
            $repeatCount -ne $expectedRepetitions) {
            throw "Baseline scenario '$logicalId' has an invalid scale or repetition count."
        }
        $mode = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'mode' -Label "Baseline scenario '$logicalId'")
        $projectRole = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'project_role' -Label "Baseline scenario '$logicalId'")
        if (($mode -in @('cold-open', 'stable-generation') -and $projectRole -ne 'baseline') -or
            ($mode -eq 'one-percent-change' -and $projectRole -ne 'changed') -or
            $mode -notin @('cold-open', 'stable-generation', 'one-percent-change')) {
            throw "Baseline scenario '$logicalId' has an invalid mode/project role pairing."
        }
        $processLifecycle = [string](Get-ResourceManagementReportProperty `
                -Value $scenario `
                -Name 'process_lifecycle' `
                -Label "Baseline scenario '$logicalId'")
        $expectedProcessLifecycle = if ($mode -eq 'stable-generation') { 'same-process' } else { 'fresh-process' }
        if ($processLifecycle -cne $expectedProcessLifecycle) {
            throw "Baseline scenario '$logicalId' process_lifecycle '$processLifecycle' differs from expected '$expectedProcessLifecycle'."
        }
        $queries = @{}
        foreach ($query in @(Get-ResourceManagementReportArrayProperty -Value $scenario -Name 'queries' -Label "Baseline scenario '$logicalId'")) {
            $key = Get-ResourceManagementReportQueryKey -Query $query -Label "Baseline scenario '$logicalId' query"
            Assert-ResourceManagementReportProperties `
                -Value $query `
                -RequiredNames @('operation', 'query', 'expected_measurements') `
                -OptionalNames @('offset', 'limit') `
                -Label "Baseline scenario '$logicalId' query '$key'"
            if ($queries.ContainsKey($key)) {
                throw "Baseline scenario '$logicalId' has a duplicate query '$key'."
            }
            $counterNames = @((Get-ResourceManagementReportArrayProperty -Value $query -Name 'expected_measurements' -Label "Baseline scenario '$logicalId' query") | ForEach-Object { [string]$_ })
            if ($counterNames.Count -eq 0 -or @($counterNames | Select-Object -Unique).Count -ne $counterNames.Count) {
                throw "Baseline scenario '$logicalId' query '$key' has invalid expected measurements."
            }
            Assert-ResourceManagementReportExpectedMeasurements `
                -Operation ([string]$query.operation) `
                -CounterNames $counterNames `
                -Label "Baseline scenario '$logicalId' query '$key'"
            $queries[$key] = [pscustomobject]@{
                description = Get-ResourceManagementReportQueryDescription -Query $query -Key $key
                counter_names = $counterNames
            }
        }
        $queryOperations = @($queries.Values | ForEach-Object { [string]$_.description.operation } | Select-Object -Unique)
        if (@($queryOperations | Where-Object { $_ -notin $workloadProfile.query_mix }).Count -ne 0) {
            throw "Baseline scenario '$logicalId' query mix differs from workload profile '$workloadProfileId'."
        }
        $scenarioMap[$logicalId] = [pscustomobject]@{
            logical_id = $logicalId
            mode = $mode
            project_role = $projectRole
            process_lifecycle = $processLifecycle
            data_asset_count = $dataAssetCount
            data_inventory_sha256 = $inventory
            required_repetitions = $repeatCount
            queries = $queries
        }
    }
    return [pscustomobject]@{
        workload_profile_id = $workloadProfileId
        workload_registry_receipt = $workloadRegistryReceipt
        source_fingerprint = $sourceFingerprint
        statistical_policy = $statisticalPolicy
        scenarios = $scenarioMap
    }
}

function ConvertTo-ResourceManagementBaselineReport {
    param(
        [Parameter(Mandatory)]$BaselinePlan,
        [Parameter(Mandatory)][string]$BaselinePlanSha256,
        [Parameter(Mandatory)]$Observation,
        [Parameter(Mandatory)][string]$ObservationSha256
    )

    $plan = Get-ResourceManagementReportPlanScenarios -BaselinePlan $BaselinePlan
    Assert-ResourceManagementReportSha256 -Value $BaselinePlanSha256 -Label 'Baseline plan SHA-256' | Out-Null
    Assert-ResourceManagementReportSha256 -Value $ObservationSha256 -Label 'Observation SHA-256' | Out-Null
    Assert-ResourceManagementReportProperties `
        -Value $Observation `
        -RequiredNames @(
            'schema_version', 'workload_family', 'source_fingerprint',
            'baseline_plan_sha256', 'observation_context', 'execution_protocol', 'samples') `
        -Label 'Observation manifest'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Observation `
        -SchemaId 'zircon.resource-management.observation' `
        -Label 'Observation manifest' | Out-Null
    $observationFingerprint = Assert-ResourceManagementReportSha256 `
        -Value ([string](Get-ResourceManagementReportProperty -Value $Observation -Name 'source_fingerprint' -Label 'Observation manifest')) `
        -Label 'Observation manifest source_fingerprint'
    if (-not $observationFingerprint.Equals($plan.source_fingerprint, [StringComparison]::Ordinal)) {
        throw 'Observation manifest belongs to a different source snapshot than the baseline plan.'
    }
    $declaredPlanSha256 = Assert-ResourceManagementReportSha256 `
        -Value ([string](Get-ResourceManagementReportProperty -Value $Observation -Name 'baseline_plan_sha256' -Label 'Observation manifest')) `
        -Label 'Observation manifest baseline_plan_sha256'
    if (-not $declaredPlanSha256.Equals($BaselinePlanSha256, [StringComparison]::Ordinal)) {
        throw 'Observation manifest belongs to a different baseline plan.'
    }
    $observationContext = Resolve-ResourceManagementObservationContext `
        -Context (Get-ResourceManagementReportProperty `
            -Value $Observation `
            -Name 'observation_context' `
            -Label 'Observation manifest') `
        -ExpectedSourceFingerprint $observationFingerprint
    $executionProtocol = Resolve-ResourceManagementExecutionProtocol `
        -Protocol (Get-ResourceManagementReportProperty `
            -Value $Observation `
            -Name 'execution_protocol' `
            -Label 'Observation manifest')

    $samplesByScenario = @{}
    $processContextsByScenario = @{}
    $executionProtocolsByScenario = @{}
    $allSampleExecutionProtocols = [Collections.Generic.List[object]]::new()
    foreach ($sample in @(Get-ResourceManagementReportArrayProperty -Value $Observation -Name 'samples' -Label 'Observation manifest')) {
        Assert-ResourceManagementReportProperties `
            -Value $sample `
            -RequiredNames @('logical_id', 'attempt', 'sample_phase', 'data_inventory_sha256', 'process_context', 'execution_protocol', 'process', 'queries') `
            -Label 'Observation sample'
        $logicalId = [string](Get-ResourceManagementReportProperty -Value $sample -Name 'logical_id' -Label 'Observation sample')
        if (-not $plan.scenarios.ContainsKey($logicalId)) {
            throw "Observation sample references an unknown baseline scenario '$logicalId'."
        }
        $attemptNumber = ConvertTo-ResourceManagementReportNonNegativeInteger `
            -Value (Get-ResourceManagementReportProperty -Value $sample -Name 'attempt' -Label "Observation sample '$logicalId'") `
            -Label "Observation sample '$logicalId' attempt"
        $scenario = $plan.scenarios[$logicalId]
        if ($attemptNumber -lt 1 -or $attemptNumber -gt $scenario.required_repetitions) {
            throw "Observation sample '$logicalId' attempt $attemptNumber is outside the required repetition budget of 1..$($scenario.required_repetitions)."
        }
        $attempt = [int]$attemptNumber
        $samplePhase = [string](Get-ResourceManagementReportProperty `
                -Value $sample `
                -Name 'sample_phase' `
                -Label "Observation sample '$logicalId'")
        $expectedSamplePhase = if ($attempt -le $plan.statistical_policy.warmup_repetitions) {
            'warmup'
        }
        else {
            'measurement'
        }
        if ($samplePhase -ne $expectedSamplePhase) {
            throw "Observation sample '$logicalId' attempt $attempt has sample_phase '$samplePhase'; expected sample_phase '$expectedSamplePhase'."
        }
        $inventory = Assert-ResourceManagementReportSha256 `
            -Value ([string](Get-ResourceManagementReportProperty -Value $sample -Name 'data_inventory_sha256' -Label "Observation sample '$logicalId'")) `
            -Label "Observation sample '$logicalId' data_inventory_sha256"
        if (-not $inventory.Equals($scenario.data_inventory_sha256, [StringComparison]::Ordinal)) {
            throw "Observation sample '$logicalId' belongs to a different data inventory."
        }
        if (-not $samplesByScenario.ContainsKey($logicalId)) {
            $samplesByScenario[$logicalId] = @{}
            $processContextsByScenario[$logicalId] = @{}
            $executionProtocolsByScenario[$logicalId] = @{}
        }
        if ($samplesByScenario[$logicalId].ContainsKey($attempt)) {
            throw "Observation sample '$logicalId' has duplicate attempt $attempt."
        }
        $samplesByScenario[$logicalId][$attempt] = $sample
        $resolvedProcessContext = Resolve-ResourceManagementSampleProcessContext `
            -Context (Get-ResourceManagementReportProperty `
                -Value $sample `
                -Name 'process_context' `
                -Label "Observation sample '$logicalId'") `
            -Label "Observation sample '$logicalId' process context"
        $processContextsByScenario[$logicalId][$attempt] = $resolvedProcessContext
        $resolvedExecutionProtocol = Resolve-ResourceManagementSampleExecutionProtocol `
            -Protocol (Get-ResourceManagementReportProperty `
                -Value $sample `
                -Name 'execution_protocol' `
                -Label "Observation sample '$logicalId'") `
            -ExpectedMode $scenario.mode `
            -ExpectedProcessId $resolvedProcessContext.process_id `
            -Label "Observation sample '$logicalId' execution protocol"
        $executionProtocolsByScenario[$logicalId][$attempt] = $resolvedExecutionProtocol
        $allSampleExecutionProtocols.Add($resolvedExecutionProtocol) | Out-Null
    }
    Assert-ResourceManagementExecutionProtocolSequence `
        -SampleProtocols $allSampleExecutionProtocols.ToArray()

    $scenarioReports = [Collections.Generic.List[object]]::new()
    foreach ($scenario in @($plan.scenarios.Values | Sort-Object logical_id)) {
        if (-not $samplesByScenario.ContainsKey($scenario.logical_id)) {
            throw "Observation manifest is missing baseline scenario '$($scenario.logical_id)'."
        }
        $attempts = @($samplesByScenario[$scenario.logical_id].Keys | Sort-Object)
        $expectedAttempts = @(1..$scenario.required_repetitions)
        if ($attempts.Count -ne $expectedAttempts.Count -or @($expectedAttempts | Where-Object { $_ -notin $attempts }).Count -gt 0) {
            throw "Observation manifest does not contain every required attempt for scenario '$($scenario.logical_id)'."
        }
        $scenarioProcessContexts = @(
            $expectedAttempts | ForEach-Object { $processContextsByScenario[$scenario.logical_id][$_] }
        )
        Assert-ResourceManagementSampleProcessLifecycle `
            -ProcessContexts $scenarioProcessContexts `
            -ProcessLifecycle $scenario.process_lifecycle `
            -Label "Observation scenario '$($scenario.logical_id)' process contexts"
        $reportedProcessContexts = @(
            foreach ($attempt in $expectedAttempts) {
                [pscustomobject][ordered]@{
                    attempt = $attempt
                    sample_phase = [string]$samplesByScenario[$scenario.logical_id][$attempt].sample_phase
                    process_context = $processContextsByScenario[$scenario.logical_id][$attempt]
                }
            }
        )
        $reportedSampleProtocols = @(
            foreach ($attempt in $expectedAttempts) {
                [pscustomobject][ordered]@{
                    attempt = $attempt
                    sample_phase = [string]$samplesByScenario[$scenario.logical_id][$attempt].sample_phase
                    execution_protocol = $executionProtocolsByScenario[$scenario.logical_id][$attempt]
                }
            }
        )

        $processCpu = [Collections.Generic.List[double]]::new()
        $workingSet = [Collections.Generic.List[double]]::new()
        $allocationProxy = [Collections.Generic.List[double]]::new()
        $querySamples = @{}
        foreach ($queryKey in $scenario.queries.Keys) {
            $querySamples[$queryKey] = [pscustomobject]@{
                elapsed_us = [Collections.Generic.List[double]]::new()
                counters = @{}
            }
            foreach ($counterName in $scenario.queries[$queryKey].counter_names) {
                $querySamples[$queryKey].counters[$counterName] = [Collections.Generic.List[double]]::new()
            }
        }

        foreach ($attempt in $expectedAttempts) {
            $sample = $samplesByScenario[$scenario.logical_id][$attempt]
            $sampleProcessContext = $processContextsByScenario[$scenario.logical_id][$attempt]
            $isMeasurement = [string]$sample.sample_phase -eq 'measurement'
            $process = Get-ResourceManagementReportProperty -Value $sample -Name 'process' -Label "Observation sample '$($scenario.logical_id)'"
            Assert-ResourceManagementReportProperties `
                -Value $process `
                -RequiredNames @('cpu_time_ms', 'peak_working_set_bytes', 'allocation_proxy_bytes') `
                -Label "Observation sample '$($scenario.logical_id)' process"
            $processCpuValue = ConvertTo-ResourceManagementReportNumber `
                -Value (Get-ResourceManagementReportProperty -Value $process -Name 'cpu_time_ms' -Label "Observation sample '$($scenario.logical_id)' process") `
                -Label "Observation sample '$($scenario.logical_id)' cpu_time_ms"
            $workingSetValue = ConvertTo-ResourceManagementReportNumber `
                -Value (Get-ResourceManagementReportProperty -Value $process -Name 'peak_working_set_bytes' -Label "Observation sample '$($scenario.logical_id)' process") `
                -Label "Observation sample '$($scenario.logical_id)' peak_working_set_bytes"
            $allocationProxyValue = ConvertTo-ResourceManagementReportNumber `
                -Value (Get-ResourceManagementReportProperty -Value $process -Name 'allocation_proxy_bytes' -Label "Observation sample '$($scenario.logical_id)' process") `
                -Label "Observation sample '$($scenario.logical_id)' allocation_proxy_bytes"
            if ($isMeasurement) {
                $processCpu.Add($processCpuValue)
                $workingSet.Add($workingSetValue)
                $allocationProxy.Add($allocationProxyValue)
            }

            $observedQueries = @{}
            foreach ($query in @(Get-ResourceManagementReportArrayProperty -Value $sample -Name 'queries' -Label "Observation sample '$($scenario.logical_id)'")) {
                $queryKey = Get-ResourceManagementReportQueryKey -Query $query -Label "Observation sample '$($scenario.logical_id)' query"
                if (-not $scenario.queries.ContainsKey($queryKey)) {
                    throw "Observation sample '$($scenario.logical_id)' contains an unknown query '$queryKey'."
                }
                if ($observedQueries.ContainsKey($queryKey)) {
                    throw "Observation sample '$($scenario.logical_id)' has duplicate query '$queryKey'."
                }
                $observedQueries[$queryKey] = $query
            }
            foreach ($queryKey in $scenario.queries.Keys) {
                if (-not $observedQueries.ContainsKey($queryKey)) {
                    throw "Observation sample '$($scenario.logical_id)' is missing required query '$queryKey'."
                }
                $query = $observedQueries[$queryKey]
                $frameIndex = Get-ResourceManagementReportOptionalProperty -Value $query -Name 'frame_index'
                $timestampUs = Get-ResourceManagementReportOptionalProperty -Value $query -Name 'timestamp_us'
                if ($null -eq $frameIndex -or $null -eq $timestampUs) {
                    throw "Observation sample '$($scenario.logical_id)' query '$queryKey' is missing its profiling frame association."
                }
                $resolvedFrameIndex = ConvertTo-ResourceManagementReportNonNegativeInteger `
                    -Value $frameIndex `
                    -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' profiling frame association frame_index"
                [void](ConvertTo-ResourceManagementReportNonNegativeInteger `
                        -Value $timestampUs `
                        -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' profiling frame association timestamp_us")
                if ($resolvedFrameIndex -lt $sampleProcessContext.first_frame_index -or
                    $resolvedFrameIndex -gt $sampleProcessContext.last_frame_index) {
                    throw "Observation sample '$($scenario.logical_id)' query '$queryKey' profiling frame association is outside the collector frame range."
                }
                Assert-ResourceManagementReportProperties `
                    -Value $query `
                    -RequiredNames @('operation', 'query', 'elapsed_us', 'counters', 'frame_index', 'timestamp_us') `
                    -OptionalNames @('offset', 'limit') `
                    -Label "Observation sample '$($scenario.logical_id)' query '$queryKey'"
                $elapsedValue = ConvertTo-ResourceManagementReportNumber `
                    -Value (Get-ResourceManagementReportProperty -Value $query -Name 'elapsed_us' -Label "Observation sample '$($scenario.logical_id)' query '$queryKey'") `
                    -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' elapsed_us"
                if ($isMeasurement) {
                    $querySamples[$queryKey].elapsed_us.Add($elapsedValue)
                }
                $counters = Get-ResourceManagementReportProperty -Value $query -Name 'counters' -Label "Observation sample '$($scenario.logical_id)' query '$queryKey'"
                foreach ($counterName in $scenario.queries[$queryKey].counter_names) {
                    $counter = Get-ResourceManagementReportOptionalProperty -Value $counters -Name $counterName
                    if ($null -eq $counter) {
                        throw "Observation sample '$($scenario.logical_id)' query '$queryKey' is missing required counter '$counterName'."
                    }
                    $counterValue = ConvertTo-ResourceManagementReportNumber `
                        -Value $counter `
                        -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' counter '$counterName'"
                    if ($isMeasurement) {
                        $querySamples[$queryKey].counters[$counterName].Add($counterValue)
                    }
                }
                Assert-ResourceManagementReportProperties `
                    -Value $counters `
                    -RequiredNames @($scenario.queries[$queryKey].counter_names) `
                    -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' counters"
            }
        }

        $statisticalPolicy = $plan.statistical_policy
        $primaryNoiseStatuses = [Collections.Generic.List[string]]::new()
        $processCpuStatistics = Get-ResourceManagementReportStatistics `
            -Values $processCpu.ToArray() `
            -StatisticalPolicy $statisticalPolicy
        $workingSetStatistics = Get-ResourceManagementReportStatistics `
            -Values $workingSet.ToArray() `
            -StatisticalPolicy $statisticalPolicy
        $allocationProxyStatistics = Get-ResourceManagementReportStatistics `
            -Values $allocationProxy.ToArray() `
            -StatisticalPolicy $statisticalPolicy
        $primaryNoiseStatuses.Add([string]$processCpuStatistics.noise_status)
        $primaryNoiseStatuses.Add([string]$workingSetStatistics.noise_status)
        $primaryNoiseStatuses.Add([string]$allocationProxyStatistics.noise_status)

        $queryReports = [Collections.Generic.List[object]]::new()
        foreach ($queryKey in @($scenario.queries.Keys | Sort-Object)) {
            $counters = [ordered]@{}
            foreach ($counterName in @($scenario.queries[$queryKey].counter_names | Sort-Object)) {
                $counters[$counterName] = Get-ResourceManagementReportStatistics `
                    -Values $querySamples[$queryKey].counters[$counterName].ToArray() `
                    -StatisticalPolicy $statisticalPolicy
            }
            $description = $scenario.queries[$queryKey].description
            $offset = if ($description.operation -eq 'page') { [int]$description.offset } else { $null }
            $limit = if ($description.operation -eq 'page') { [int]$description.limit } else { $null }
            $elapsedStatistics = Get-ResourceManagementReportStatistics `
                -Values $querySamples[$queryKey].elapsed_us.ToArray() `
                -StatisticalPolicy $statisticalPolicy
            $primaryNoiseStatuses.Add([string]$elapsedStatistics.noise_status)
            $queryReports.Add([pscustomobject][ordered]@{
                    operation = $description.operation
                    query = $description.query
                    offset = $offset
                    limit = $limit
                    elapsed_us = $elapsedStatistics
                    counters = $counters
                }) | Out-Null
        }
        $statisticalStatus = if ($primaryNoiseStatuses.Contains('insufficient-samples')) {
            'insufficient-samples'
        }
        elseif ($primaryNoiseStatuses.Contains('unstable')) {
            'unstable'
        }
        else {
            'stable'
        }
        $scenarioReports.Add([pscustomobject][ordered]@{
                logical_id = $scenario.logical_id
                mode = $scenario.mode
                project_role = $scenario.project_role
                process_lifecycle = $scenario.process_lifecycle
                data_asset_count = $scenario.data_asset_count
                data_inventory_sha256 = $scenario.data_inventory_sha256
                attempt_count = $expectedAttempts.Count
                warmup_count = $statisticalPolicy.warmup_repetitions
                sample_count = $statisticalPolicy.measurement_repetitions
                statistical_status = $statisticalStatus
                process_contexts = $reportedProcessContexts
                sample_protocols = $reportedSampleProtocols
                process = [ordered]@{
                    cpu_time_ms = $processCpuStatistics
                    peak_working_set_bytes = $workingSetStatistics
                    allocation_proxy_bytes = $allocationProxyStatistics
                }
                queries = $queryReports.ToArray()
            }) | Out-Null
    }

    return [pscustomobject][ordered]@{
        schema_version = 4
        workload_family = 'resource-management-query'
        workload_profile_id = $plan.workload_profile_id
        workload_registry_receipt = $plan.workload_registry_receipt
        # A structurally valid caller manifest is diagnostic input, not
        # evidence from a trusted product observation producer.
        measurement_status = 'unverified'
        measurement_status_reason = 'untrusted-observation-context'
        source_fingerprint = $plan.source_fingerprint
        baseline_plan_sha256 = $BaselinePlanSha256
        observation_sha256 = $ObservationSha256
        observation_context = $observationContext
        execution_protocol = $executionProtocol
        statistical_policy = $plan.statistical_policy
        scenarios = $scenarioReports.ToArray()
    }
}

function Assert-ResourceManagementBaselineReportOutputDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'resource-management-reports'
    if ([IO.Directory]::Exists($storage.operation_path) -or [IO.File]::Exists($storage.operation_path)) {
        throw "Resource-management baseline report output must not already exist: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Write-ResourceManagementBaselineReportFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Content
    )

    $stream = [IO.FileStream]::new($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Content)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function ConvertTo-ResourceManagementBaselineReportMarkdown {
    param([Parameter(Mandatory)]$Report)

    $lines = [Collections.Generic.List[string]]::new()
    $lines.Add('# Resource-management baseline report')
    $lines.Add('')
    $lines.Add("- Measurement status: $($Report.measurement_status)")
    $lines.Add("- Product receipt: $($Report.observation_context.product_receipt.receipt_id)")
    $lines.Add("- Run: $($Report.observation_context.run.run_id)")
    $lines.Add("- Machine: $($Report.observation_context.machine.machine_id_sha256)")
    $lines.Add("- Collector: $($Report.observation_context.collector.collector_id) $($Report.observation_context.collector.collector_version)")
    $lines.Add("- Order receipt: $($Report.execution_protocol.order_receipt_sha256)")
    $lines.Add("- Cache scope: $($Report.execution_protocol.cache_scope)")
    $lines.Add("- Statistical policy: warmup=$($Report.statistical_policy.warmup_repetitions), samples=$($Report.statistical_policy.measurement_repetitions), confidence=95%")
    $lines.Add("- Scenario count: $($Report.scenarios.Count)")
    $lines.Add('')
    $lines.Add('| Scenario | Resources | Warmup | Samples | Noise | Query | Median us | P95 us |')
    $lines.Add('| --- | ---: | ---: | ---: | --- | --- | ---: | ---: |')
    foreach ($scenario in $Report.scenarios) {
        foreach ($query in $scenario.queries) {
            $label = if ($query.operation -eq 'page') {
                "page offset=$($query.offset) limit=$($query.limit)"
            }
            else {
                [string]$query.operation
            }
            $lines.Add("| $($scenario.logical_id) | $($scenario.data_asset_count) | $($scenario.warmup_count) | $($scenario.sample_count) | $($scenario.statistical_status) | $label | $($query.elapsed_us.median) | $($query.elapsed_us.p95) |")
        }
    }
    return ($lines -join [Environment]::NewLine) + [Environment]::NewLine
}

function Invoke-ResourceManagementBaselineReport {
    param(
        [Parameter(Mandatory)][string]$BaselinePlanPath,
        [Parameter(Mandatory)][string]$ObservationPath,
        [Parameter(Mandatory)][string]$OutputDirectory
    )

    $planEvidence = Get-ResourceManagementReportJsonEvidence `
        -Path $BaselinePlanPath `
        -Label 'Resource-management baseline plan' `
        -MaximumBytes $script:ResourceManagementMaximumBaselinePlanBytes
    $observationEvidence = Get-ResourceManagementReportJsonEvidence `
        -Path $ObservationPath `
        -Label 'Resource-management baseline observation manifest' `
        -MaximumBytes $script:ResourceManagementMaximumObservationBytes
    $report = ConvertTo-ResourceManagementBaselineReport `
        -BaselinePlan $planEvidence.json `
        -BaselinePlanSha256 $planEvidence.sha256 `
        -Observation $observationEvidence.json `
        -ObservationSha256 $observationEvidence.sha256
    $output = Assert-ResourceManagementBaselineReportOutputDirectory -Path $OutputDirectory
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($output.OperationalPath)) | Out-Null
    $stagingPath = "$($output.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($stagingPath) | Out-Null
        $jsonPath = Join-ZirconWindowsPath -Path $stagingPath -ChildPath 'resource-management-baseline-report.json'
        $markdownPath = Join-ZirconWindowsPath -Path $stagingPath -ChildPath 'resource-management-baseline-report.md'
        Write-ResourceManagementBaselineReportFileNew -Path $jsonPath -Content ($report | ConvertTo-Json -Depth 24)
        Write-ResourceManagementBaselineReportFileNew -Path $markdownPath -Content (ConvertTo-ResourceManagementBaselineReportMarkdown -Report $report)
        [IO.Directory]::Move($stagingPath, $output.OperationalPath)
    }
    catch {
        if ([IO.Directory]::Exists($stagingPath)) {
            [IO.Directory]::Delete($stagingPath, $true)
        }
        throw
    }
    return [pscustomobject]@{
        report_path = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $output.OperationalPath -ChildPath 'resource-management-baseline-report.json')).DisplayPath
        markdown_path = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $output.OperationalPath -ChildPath 'resource-management-baseline-report.md')).DisplayPath
        scenario_count = $report.scenarios.Count
        source_fingerprint = $report.source_fingerprint
    }
}

if ($env:RESOURCE_MANAGEMENT_BASELINE_REPORT_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($BaselinePlanPath)) {
        throw '-BaselinePlanPath is required for resource-management baseline reporting.'
    }
    if ([string]::IsNullOrWhiteSpace($ObservationPath)) {
        throw '-ObservationPath is required for resource-management baseline reporting.'
    }
    Invoke-ResourceManagementBaselineReport `
        -BaselinePlanPath $BaselinePlanPath `
        -ObservationPath $ObservationPath `
        -OutputDirectory $OutputDirectory
}
