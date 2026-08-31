Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop

function Resolve-ResourceManagementExecutionProtocol {
    param([Parameter(Mandatory)]$Protocol)

    Assert-ResourceManagementSchemaProperties `
        -Value $Protocol `
        -RequiredNames @(
            'schema_version', 'protocol_kind', 'randomization_algorithm',
            'randomization_seed_sha256', 'order_receipt_sha256', 'cache_scope',
            'quiescence_policy_id') `
        -Label 'Resource-management execution protocol'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Protocol `
        -SchemaId 'zircon.resource-management.execution-protocol' `
        -Label 'Resource-management execution protocol' | Out-Null
    $randomizationAlgorithm = [string]$Protocol.randomization_algorithm
    if ($randomizationAlgorithm -cne 'fisher-yates-sha256-v1') {
        throw "Resource-management execution protocol randomization_algorithm '$randomizationAlgorithm' is unsupported."
    }
    $randomizationSeedSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Protocol.randomization_seed_sha256) `
        -Label 'Resource-management execution protocol randomization_seed_sha256'
    $orderReceiptSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Protocol.order_receipt_sha256) `
        -Label 'Resource-management execution protocol order_receipt_sha256'
    $cacheScope = [string]$Protocol.cache_scope
    if ($cacheScope -cne 'os+ddc+resource-index') {
        throw "Resource-management execution protocol cache_scope '$cacheScope' is unsupported."
    }
    $quiescencePolicyId = [string]$Protocol.quiescence_policy_id
    if ($quiescencePolicyId -cne 'mvp-resource-quiescence-v1') {
        throw "Resource-management execution protocol quiescence_policy_id '$quiescencePolicyId' is unsupported."
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        protocol_kind = 'zircon.resource-management-execution-protocol'
        randomization_algorithm = $randomizationAlgorithm
        randomization_seed_sha256 = $randomizationSeedSha256
        order_receipt_sha256 = $orderReceiptSha256
        cache_scope = $cacheScope
        quiescence_policy_id = $quiescencePolicyId
    }
}

function Resolve-ResourceManagementSampleExecutionProtocol {
    param(
        [Parameter(Mandatory)]$Protocol,
        [Parameter(Mandatory)][ValidateSet('cold-open', 'stable-generation', 'one-percent-change')][string]$ExpectedMode,
        [Parameter(Mandatory)][uint32]$ExpectedProcessId,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementSchemaProperties `
        -Value $Protocol `
        -RequiredNames @(
            'schema_version', 'protocol_kind', 'sequence_ordinal', 'cache_state',
            'cache_action', 'cache_receipt_sha256', 'quiescence_receipt_sha256',
            'quiescence_process_id') `
        -Label $Label
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Protocol `
        -SchemaId 'zircon.resource-management.sample-protocol' `
        -Label $Label | Out-Null
    $sequenceOrdinal = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $Protocol.sequence_ordinal `
        -Label "$Label sequence_ordinal"
    if ($sequenceOrdinal -lt 1 -or $sequenceOrdinal -gt [int]::MaxValue) {
        throw "$Label sequence_ordinal must be a positive 32-bit integer."
    }
    $expectedCacheState = if ($ExpectedMode -eq 'cold-open') { 'cold' } else { 'warm' }
    $expectedCacheAction = if ($ExpectedMode -eq 'cold-open') { 'purge' } else { 'prime' }
    $cacheState = [string]$Protocol.cache_state
    $cacheAction = [string]$Protocol.cache_action
    if ($cacheState -cne $expectedCacheState -or $cacheAction -cne $expectedCacheAction) {
        throw "$Label cache state/action differs from mode '$ExpectedMode'."
    }
    $cacheReceiptSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Protocol.cache_receipt_sha256) `
        -Label "$Label cache_receipt_sha256"
    $quiescenceReceiptSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Protocol.quiescence_receipt_sha256) `
        -Label "$Label quiescence_receipt_sha256"
    $quiescenceProcessId = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $Protocol.quiescence_process_id `
        -Label "$Label quiescence_process_id"
    if ($quiescenceProcessId -ne $ExpectedProcessId) {
        throw "$Label quiescence_process_id differs from its product process context."
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        protocol_kind = 'zircon.resource-management-sample-protocol'
        sequence_ordinal = [int]$sequenceOrdinal
        cache_state = $cacheState
        cache_action = $cacheAction
        cache_receipt_sha256 = $cacheReceiptSha256
        quiescence_receipt_sha256 = $quiescenceReceiptSha256
        quiescence_process_id = [uint32]$quiescenceProcessId
    }
}

function Assert-ResourceManagementExecutionProtocolSequence {
    param([Parameter(Mandatory)][object[]]$SampleProtocols)

    if ($SampleProtocols.Count -eq 0) {
        throw 'Resource-management execution protocol contains no sample sequence.'
    }
    $ordinals = [Collections.Generic.HashSet[int]]::new()
    $maximum = 0
    foreach ($protocol in $SampleProtocols) {
        $ordinal = [int]$protocol.sequence_ordinal
        if (-not $ordinals.Add($ordinal)) {
            throw "Resource-management execution protocol has duplicate sequence_ordinal $ordinal."
        }
        $maximum = [Math]::Max($maximum, $ordinal)
    }
    if ($ordinals.Count -ne $SampleProtocols.Count -or $maximum -ne $SampleProtocols.Count) {
        throw 'Resource-management execution protocol sequence_ordinal values must be contiguous from one.'
    }
}

function Assert-ResourceManagementExecutionProtocolsComparable {
    param(
        [Parameter(Mandatory)]$ApprovedBaseline,
        [Parameter(Mandatory)]$Candidate
    )

    $baselineKey = @(
        $ApprovedBaseline.randomization_algorithm,
        $ApprovedBaseline.cache_scope,
        $ApprovedBaseline.quiescence_policy_id
    ) -join [char]0
    $candidateKey = @(
        $Candidate.randomization_algorithm,
        $Candidate.cache_scope,
        $Candidate.quiescence_policy_id
    ) -join [char]0
    if (-not $baselineKey.Equals($candidateKey, [StringComparison]::Ordinal)) {
        throw 'Approved baseline and candidate execution protocol contracts differ.'
    }
}

Export-ModuleMember -Function @(
    'Assert-ResourceManagementExecutionProtocolSequence',
    'Assert-ResourceManagementExecutionProtocolsComparable',
    'Resolve-ResourceManagementExecutionProtocol',
    'Resolve-ResourceManagementSampleExecutionProtocol'
)
