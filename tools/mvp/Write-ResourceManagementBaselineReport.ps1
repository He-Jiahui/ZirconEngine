[CmdletBinding()]
param(
    [string]$BaselinePlanPath,
    [string]$ObservationPath,
    [string]$OutputDirectory = (Join-Path 'E:\ZirconBuilds\mvp-resource-management-reports' ([guid]::NewGuid().ToString('N')))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

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

function Get-ResourceManagementReportProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Get-ResourceManagementReportOptionalProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function Get-ResourceManagementReportArrayProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $items = @(Get-ResourceManagementReportProperty -Value $Value -Name $Name -Label $Label)
    if ($items.Count -eq 0) {
        throw "$Label has no '$Name'."
    }
    return $items
}

function Assert-ResourceManagementReportSha256 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -notmatch '^[0-9A-F]{64}$') {
        throw "$Label must be an uppercase SHA-256 value."
    }
    return $Value
}

function Get-ResourceManagementReportJsonEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "$Label does not exist: $($resolution.DisplayPath)"
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($resolution.OperationalPath)
    if ($bytes.Length -eq 0) {
        throw "$Label is empty: $($resolution.DisplayPath)"
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $sha256 = -join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
    try {
        $text = ([Text.UTF8Encoding]::new($false)).GetString($bytes)
        if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) {
            $text = $text.Substring(1)
        }
        $json = $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "$Label is not valid JSON: $($resolution.DisplayPath): $($_.Exception.Message)"
    }
    return [pscustomobject]@{
        json = $json
        sha256 = $sha256
        display_path = $resolution.DisplayPath
    }
}

function Get-ResourceManagementReportStatistics {
    param([Parameter(Mandatory)][double[]]$Values)

    if ($Values.Count -eq 0) {
        throw 'Cannot aggregate zero resource-management baseline samples.'
    }
    $sorted = [double[]]$Values.Clone()
    [Array]::Sort($sorted)
    $total = 0.0
    foreach ($value in $sorted) {
        $total += $value
    }
    $percentileIndex = [int][Math]::Ceiling(($sorted.Count - 1) * 0.95)
    return [pscustomobject][ordered]@{
        sample_count = $sorted.Count
        min = $sorted[0]
        median = $sorted[[int][Math]::Ceiling(($sorted.Count - 1) * 0.5)]
        p95 = $sorted[$percentileIndex]
        max = $sorted[$sorted.Count - 1]
        mean = $total / $sorted.Count
        total = $total
    }
}

function ConvertTo-ResourceManagementReportNumber {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        $number = [double]$Value
    }
    catch {
        throw "$Label must be numeric."
    }
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or $number -lt 0) {
        throw "$Label must be a finite non-negative number."
    }
    return $number
}

function ConvertTo-ResourceManagementReportNonNegativeInteger {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [bool] -or ($Value -is [string] -and [string]::IsNullOrWhiteSpace($Value))) {
        throw "$Label must be a non-negative integer."
    }
    try {
        $number = [decimal]$Value
    }
    catch {
        throw "$Label must be a non-negative integer."
    }
    if ($number -lt 0 -or [decimal]::Truncate($number) -ne $number -or $number -gt [decimal][uint64]::MaxValue) {
        throw "$Label must be a non-negative integer."
    }
    return [uint64]$number
}

function Get-ResourceManagementReportQueryKey {
    param(
        [Parameter(Mandatory)]$Query,
        [Parameter(Mandatory)][string]$Label
    )

    $operation = [string](Get-ResourceManagementReportProperty -Value $Query -Name 'operation' -Label $Label)
    $filter = Get-ResourceManagementReportProperty -Value $Query -Name 'query' -Label $Label
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

    if ([int](Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'schema_version' -Label 'Baseline plan') -ne 1 -or
        [string](Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'workload_family' -Label 'Baseline plan') -ne 'resource-management-query' -or
        [string](Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'resource_kind' -Label 'Baseline plan') -ne 'Data') {
        throw 'Baseline plan has an unsupported schema.'
    }
    $sourceFingerprint = Assert-ResourceManagementReportSha256 `
        -Value ([string](Get-ResourceManagementReportProperty -Value $BaselinePlan -Name 'source_fingerprint' -Label 'Baseline plan')) `
        -Label 'Baseline plan source_fingerprint'
    $scenarioMap = @{}
    foreach ($scenario in @(Get-ResourceManagementReportArrayProperty -Value $BaselinePlan -Name 'scenarios' -Label 'Baseline plan')) {
        $logicalId = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'logical_id' -Label 'Baseline scenario')
        if ([string]::IsNullOrWhiteSpace($logicalId) -or $scenarioMap.ContainsKey($logicalId)) {
            throw "Baseline plan has an empty or duplicate logical_id '$logicalId'."
        }
        $dataAssetCount = [int](Get-ResourceManagementReportProperty -Value $scenario -Name 'data_asset_count' -Label "Baseline scenario '$logicalId'")
        $repeatCount = [int](Get-ResourceManagementReportProperty -Value $scenario -Name 'required_repetitions' -Label "Baseline scenario '$logicalId'")
        $inventory = Assert-ResourceManagementReportSha256 `
            -Value ([string](Get-ResourceManagementReportProperty -Value $scenario -Name 'data_inventory_sha256' -Label "Baseline scenario '$logicalId'")) `
            -Label "Baseline scenario '$logicalId' data_inventory_sha256"
        if ($dataAssetCount -lt 1 -or $dataAssetCount -gt 100000 -or $repeatCount -lt 3 -or $repeatCount -gt 20) {
            throw "Baseline scenario '$logicalId' has an invalid scale or repetition count."
        }
        $mode = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'mode' -Label "Baseline scenario '$logicalId'")
        $projectRole = [string](Get-ResourceManagementReportProperty -Value $scenario -Name 'project_role' -Label "Baseline scenario '$logicalId'")
        if (($mode -in @('cold-open', 'stable-generation') -and $projectRole -ne 'baseline') -or
            ($mode -eq 'one-percent-change' -and $projectRole -ne 'changed') -or
            $mode -notin @('cold-open', 'stable-generation', 'one-percent-change')) {
            throw "Baseline scenario '$logicalId' has an invalid mode/project role pairing."
        }
        $queries = @{}
        foreach ($query in @(Get-ResourceManagementReportArrayProperty -Value $scenario -Name 'queries' -Label "Baseline scenario '$logicalId'")) {
            $key = Get-ResourceManagementReportQueryKey -Query $query -Label "Baseline scenario '$logicalId' query"
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
        $scenarioMap[$logicalId] = [pscustomobject]@{
            logical_id = $logicalId
            mode = $mode
            project_role = $projectRole
            data_asset_count = $dataAssetCount
            data_inventory_sha256 = $inventory
            required_repetitions = $repeatCount
            queries = $queries
        }
    }
    return [pscustomobject]@{
        source_fingerprint = $sourceFingerprint
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
    if ([int](Get-ResourceManagementReportProperty -Value $Observation -Name 'schema_version' -Label 'Observation manifest') -ne 1 -or
        [string](Get-ResourceManagementReportProperty -Value $Observation -Name 'workload_family' -Label 'Observation manifest') -ne 'resource-management-query') {
        throw 'Observation manifest has an unsupported schema.'
    }
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

    $samplesByScenario = @{}
    foreach ($sample in @(Get-ResourceManagementReportArrayProperty -Value $Observation -Name 'samples' -Label 'Observation manifest')) {
        $logicalId = [string](Get-ResourceManagementReportProperty -Value $sample -Name 'logical_id' -Label 'Observation sample')
        if (-not $plan.scenarios.ContainsKey($logicalId)) {
            throw "Observation sample references an unknown baseline scenario '$logicalId'."
        }
        $attempt = [int](Get-ResourceManagementReportProperty -Value $sample -Name 'attempt' -Label "Observation sample '$logicalId'")
        if ($attempt -lt 1) {
            throw "Observation sample '$logicalId' has an invalid attempt."
        }
        $scenario = $plan.scenarios[$logicalId]
        $inventory = Assert-ResourceManagementReportSha256 `
            -Value ([string](Get-ResourceManagementReportProperty -Value $sample -Name 'data_inventory_sha256' -Label "Observation sample '$logicalId'")) `
            -Label "Observation sample '$logicalId' data_inventory_sha256"
        if (-not $inventory.Equals($scenario.data_inventory_sha256, [StringComparison]::Ordinal)) {
            throw "Observation sample '$logicalId' belongs to a different data inventory."
        }
        if (-not $samplesByScenario.ContainsKey($logicalId)) {
            $samplesByScenario[$logicalId] = @{}
        }
        if ($samplesByScenario[$logicalId].ContainsKey($attempt)) {
            throw "Observation sample '$logicalId' has duplicate attempt $attempt."
        }
        $samplesByScenario[$logicalId][$attempt] = $sample
    }

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
            $process = Get-ResourceManagementReportProperty -Value $sample -Name 'process' -Label "Observation sample '$($scenario.logical_id)'"
            $processCpu.Add((ConvertTo-ResourceManagementReportNumber `
                    -Value (Get-ResourceManagementReportProperty -Value $process -Name 'cpu_time_ms' -Label "Observation sample '$($scenario.logical_id)' process") `
                    -Label "Observation sample '$($scenario.logical_id)' cpu_time_ms"))
            $workingSet.Add((ConvertTo-ResourceManagementReportNumber `
                    -Value (Get-ResourceManagementReportProperty -Value $process -Name 'peak_working_set_bytes' -Label "Observation sample '$($scenario.logical_id)' process") `
                    -Label "Observation sample '$($scenario.logical_id)' peak_working_set_bytes"))
            $allocationProxy.Add((ConvertTo-ResourceManagementReportNumber `
                    -Value (Get-ResourceManagementReportProperty -Value $process -Name 'allocation_proxy_bytes' -Label "Observation sample '$($scenario.logical_id)' process") `
                    -Label "Observation sample '$($scenario.logical_id)' allocation_proxy_bytes"))

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
                [void](ConvertTo-ResourceManagementReportNonNegativeInteger `
                        -Value $frameIndex `
                        -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' profiling frame association frame_index")
                [void](ConvertTo-ResourceManagementReportNonNegativeInteger `
                        -Value $timestampUs `
                        -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' profiling frame association timestamp_us")
                $querySamples[$queryKey].elapsed_us.Add((ConvertTo-ResourceManagementReportNumber `
                        -Value (Get-ResourceManagementReportProperty -Value $query -Name 'elapsed_us' -Label "Observation sample '$($scenario.logical_id)' query '$queryKey'") `
                        -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' elapsed_us"))
                $counters = Get-ResourceManagementReportProperty -Value $query -Name 'counters' -Label "Observation sample '$($scenario.logical_id)' query '$queryKey'"
                foreach ($counterName in $scenario.queries[$queryKey].counter_names) {
                    $counter = Get-ResourceManagementReportOptionalProperty -Value $counters -Name $counterName
                    if ($null -eq $counter) {
                        throw "Observation sample '$($scenario.logical_id)' query '$queryKey' is missing required counter '$counterName'."
                    }
                    $querySamples[$queryKey].counters[$counterName].Add((ConvertTo-ResourceManagementReportNumber `
                            -Value $counter `
                            -Label "Observation sample '$($scenario.logical_id)' query '$queryKey' counter '$counterName'"))
                }
            }
        }

        $queryReports = [Collections.Generic.List[object]]::new()
        foreach ($queryKey in @($scenario.queries.Keys | Sort-Object)) {
            $counters = [ordered]@{}
            foreach ($counterName in @($scenario.queries[$queryKey].counter_names | Sort-Object)) {
                $counters[$counterName] = Get-ResourceManagementReportStatistics -Values $querySamples[$queryKey].counters[$counterName].ToArray()
            }
            $description = $scenario.queries[$queryKey].description
            $offset = if ($description.operation -eq 'page') { [int]$description.offset } else { $null }
            $limit = if ($description.operation -eq 'page') { [int]$description.limit } else { $null }
            $queryReports.Add([pscustomobject][ordered]@{
                    operation = $description.operation
                    query = $description.query
                    offset = $offset
                    limit = $limit
                    elapsed_us = Get-ResourceManagementReportStatistics -Values $querySamples[$queryKey].elapsed_us.ToArray()
                    counters = $counters
                }) | Out-Null
        }
        $scenarioReports.Add([pscustomobject][ordered]@{
                logical_id = $scenario.logical_id
                mode = $scenario.mode
                project_role = $scenario.project_role
                data_asset_count = $scenario.data_asset_count
                data_inventory_sha256 = $scenario.data_inventory_sha256
                attempt_count = $expectedAttempts.Count
                process = [ordered]@{
                    cpu_time_ms = Get-ResourceManagementReportStatistics -Values $processCpu.ToArray()
                    peak_working_set_bytes = Get-ResourceManagementReportStatistics -Values $workingSet.ToArray()
                    allocation_proxy_bytes = Get-ResourceManagementReportStatistics -Values $allocationProxy.ToArray()
                }
                queries = $queryReports.ToArray()
            }) | Out-Null
    }

    return [pscustomobject][ordered]@{
        schema_version = 1
        workload_family = 'resource-management-query'
        measurement_status = 'measured'
        source_fingerprint = $plan.source_fingerprint
        baseline_plan_sha256 = $BaselinePlanSha256
        observation_sha256 = $ObservationSha256
        scenarios = $scenarioReports.ToArray()
    }
}

function Assert-ResourceManagementBaselineReportOutputDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if ($resolution.DisplayPath -notmatch '^E:\\ZirconBuilds\\mvp-resource-management-reports\\(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "Resource-management baseline report output must resolve under E:\ZirconBuilds\mvp-resource-management-reports\<session>: $($resolution.DisplayPath)"
    }
    if ([IO.Directory]::Exists($resolution.OperationalPath) -or [IO.File]::Exists($resolution.OperationalPath)) {
        throw "Resource-management baseline report output must not already exist: $($resolution.DisplayPath)"
    }
    return $resolution
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
    $lines.Add("- Scenario count: $($Report.scenarios.Count)")
    $lines.Add('')
    $lines.Add('| Scenario | Resources | Attempts | Query | Median us | P95 us |')
    $lines.Add('| --- | ---: | ---: | --- | ---: | ---: |')
    foreach ($scenario in $Report.scenarios) {
        foreach ($query in $scenario.queries) {
            $label = if ($query.operation -eq 'page') {
                "page offset=$($query.offset) limit=$($query.limit)"
            }
            else {
                [string]$query.operation
            }
            $lines.Add("| $($scenario.logical_id) | $($scenario.data_asset_count) | $($scenario.attempt_count) | $label | $($query.elapsed_us.median) | $($query.elapsed_us.p95) |")
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

    $planEvidence = Get-ResourceManagementReportJsonEvidence -Path $BaselinePlanPath -Label 'Resource-management baseline plan'
    $observationEvidence = Get-ResourceManagementReportJsonEvidence -Path $ObservationPath -Label 'Resource-management baseline observation manifest'
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
