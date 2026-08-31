Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpProcessQualificationContextSchemaVersion = 1
$script:MvpProcessQualificationContextKind = 'zircon.mvp-process-qualification-context'
$script:MvpProcessQualificationContextLowerHexDigits = [char[]]'0123456789abcdef'

function ConvertTo-MvpProcessQualificationLowerHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpProcessQualificationContextLowerHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpProcessQualificationContextLowerHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpProcessQualificationSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProcessQualificationLowerHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Assert-MvpProcessQualificationExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value -or $Value -is [Array]) {
        throw "$Label must contain one object."
    }
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $ExpectedNames) {
        [void]$expected.Add($name)
    }
    $actual = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($property in $Value.PSObject.Properties) {
        if (-not $expected.Contains($property.Name)) {
            throw "$Label contains unknown property '$($property.Name)'."
        }
        [void]$actual.Add($property.Name)
    }
    foreach ($name in $ExpectedNames) {
        if (-not $actual.Contains($name)) {
            throw "$Label is missing required property '$name'."
        }
    }
}

function Assert-MvpProcessQualificationIdentifier {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        throw "$Label '$Value' must be one stable lowercase identifier."
    }
    return $Value
}

function Get-MvpProcessQualificationCanonicalPayload {
    param(
        [Parameter(Mandatory)][ValidateSet('unqualified_missing_product_receipt', 'pending_observation')][string]$QualificationStatus,
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][string]$SourceFingerprint,
        [AllowNull()][string]$BuildSetId,
        [Parameter(Mandatory)][string]$ScenarioRegistryId,
        [Parameter(Mandatory)][string]$ScenarioRegistrySha256,
        [Parameter(Mandatory)][string]$ScenarioId,
        [Parameter(Mandatory)][string]$ScenarioVariant,
        [AllowEmptyCollection()][Parameter(Mandatory)][string[]]$ProductReceiptIds
    )

    return [ordered]@{
        schema_version = $script:MvpProcessQualificationContextSchemaVersion
        context_kind = $script:MvpProcessQualificationContextKind
        qualification_status = $QualificationStatus
        run_id = $RunId
        source_fingerprint = $SourceFingerprint
        build_set_id = $BuildSetId
        scenario_registry_id = $ScenarioRegistryId
        scenario_registry_sha256 = $ScenarioRegistrySha256
        scenario_id = $ScenarioId
        scenario_variant = $ScenarioVariant
        product_receipt_ids = @($ProductReceiptIds)
    }
}

function Get-MvpProcessQualificationContextId {
    param([Parameter(Mandatory)]$CanonicalPayload)

    $json = $CanonicalPayload | ConvertTo-Json -Depth 4 -Compress
    return Get-MvpProcessQualificationSha256 -Bytes ([Text.UTF8Encoding]::new($false, $true).GetBytes($json))
}

function Assert-MvpProcessQualificationContext {
    param(
        [Parameter(Mandatory)]$Context,
        [string]$ExpectedRunId
    )

    $expectedNames = @(
        'schema_version', 'context_kind', 'context_id', 'qualification_status',
        'run_id', 'source_fingerprint', 'build_set_id', 'scenario_registry_id',
        'scenario_registry_sha256', 'scenario_id', 'scenario_variant', 'product_receipt_ids'
    )
    Assert-MvpProcessQualificationExactProperties `
        -Value $Context `
        -ExpectedNames $expectedNames `
        -Label 'MVP process qualification context'
    if ([int]$Context.schema_version -ne $script:MvpProcessQualificationContextSchemaVersion -or
        [string]$Context.context_kind -cne $script:MvpProcessQualificationContextKind) {
        throw 'MVP process qualification context has an incompatible schema identity.'
    }
    $runId = [string]$Context.run_id
    if ($runId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "MVP process qualification context run_id '$runId' is invalid."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedRunId) -and $runId -cne $ExpectedRunId) {
        throw "MVP process qualification context run_id '$runId' differs from expected '$ExpectedRunId'."
    }
    $sourceFingerprint = [string]$Context.source_fingerprint
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'MVP process qualification context source_fingerprint must be an uppercase SHA-256.'
    }
    $buildSetId = if ($null -eq $Context.build_set_id) { $null } else { [string]$Context.build_set_id }
    if ($null -ne $buildSetId -and $buildSetId -notmatch '^[0-9A-F]{64}$') {
        throw 'MVP process qualification context build_set_id must be null or an uppercase SHA-256.'
    }
    $registryId = Assert-MvpProcessQualificationIdentifier `
        -Value ([string]$Context.scenario_registry_id) `
        -Label 'MVP process qualification context scenario_registry_id'
    $registrySha256 = [string]$Context.scenario_registry_sha256
    if ($registrySha256 -notmatch '^[0-9a-f]{64}$') {
        throw 'MVP process qualification context scenario_registry_sha256 must be a lowercase SHA-256.'
    }
    $scenarioId = Assert-MvpProcessQualificationIdentifier `
        -Value ([string]$Context.scenario_id) `
        -Label 'MVP process qualification context scenario_id'
    $scenarioVariant = Assert-MvpProcessQualificationIdentifier `
        -Value ([string]$Context.scenario_variant) `
        -Label 'MVP process qualification context scenario_variant'
    $receiptIds = [Collections.Generic.List[string]]::new()
    $seenReceiptIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($receiptIdEntry in @($Context.product_receipt_ids)) {
        $receiptId = [string]$receiptIdEntry
        if ($receiptId -notmatch '^[0-9A-F]{64}$') {
            throw "MVP process qualification context ProductReceipt ID '$receiptId' must be an uppercase SHA-256."
        }
        if (-not $seenReceiptIds.Add($receiptId)) {
            throw "MVP process qualification context contains duplicate ProductReceipt ID '$receiptId'."
        }
        $receiptIds.Add($receiptId) | Out-Null
    }
    $qualificationStatus = [string]$Context.qualification_status
    $expectedStatus = if ($receiptIds.Count -eq 0) {
        'unqualified_missing_product_receipt'
    }
    else {
        'pending_observation'
    }
    if ($qualificationStatus -cne $expectedStatus) {
        throw "MVP process qualification context qualification_status '$qualificationStatus' differs from expected '$expectedStatus'."
    }
    $canonicalPayload = Get-MvpProcessQualificationCanonicalPayload `
        -QualificationStatus $qualificationStatus `
        -RunId $runId `
        -SourceFingerprint $sourceFingerprint `
        -BuildSetId $buildSetId `
        -ScenarioRegistryId $registryId `
        -ScenarioRegistrySha256 $registrySha256 `
        -ScenarioId $scenarioId `
        -ScenarioVariant $scenarioVariant `
        -ProductReceiptIds $receiptIds.ToArray()
    $expectedContextId = Get-MvpProcessQualificationContextId -CanonicalPayload $canonicalPayload
    if ([string]$Context.context_id -cne $expectedContextId) {
        throw 'MVP process qualification context context_id differs from its canonical payload.'
    }
    return $Context
}

function New-MvpProcessQualificationContext {
    param(
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][string]$SourceFingerprint,
        [AllowNull()][string]$BuildSetId,
        [Parameter(Mandatory)]$ScenarioRegistryReceipt,
        [Parameter(Mandatory)]$ScenarioRegistration,
        [Parameter(Mandatory)][string]$ScenarioVariant,
        [AllowEmptyCollection()][string[]]$ProductReceiptIds = @()
    )

    Assert-MvpProcessQualificationExactProperties `
        -Value $ScenarioRegistryReceipt `
        -ExpectedNames @('schema_version', 'registry_kind', 'registry_id', 'scenario_count', 'bytes', 'sha256') `
        -Label 'MVP scenario registry receipt'
    if ([int]$ScenarioRegistryReceipt.schema_version -ne 1 -or
        [string]$ScenarioRegistryReceipt.registry_kind -cne 'zircon.mvp-scenario-registry') {
        throw 'MVP scenario registry receipt has an incompatible schema identity.'
    }
    $registryId = Assert-MvpProcessQualificationIdentifier `
        -Value ([string]$ScenarioRegistryReceipt.registry_id) `
        -Label 'MVP scenario registry receipt registry_id'
    $registrySha256 = [string]$ScenarioRegistryReceipt.sha256
    if ($registrySha256 -notmatch '^[0-9a-f]{64}$') {
        throw 'MVP scenario registry receipt sha256 must be a lowercase SHA-256.'
    }
    Assert-MvpProcessQualificationExactProperties `
        -Value $ScenarioRegistration `
        -ExpectedNames @(
            'scenario_id', 'capability_id', 'owner', 'roles', 'liveness_scenario',
            'automation_request', 'steps', 'progress_event_ids', 'oracle_ids', 'artifact_ids', 'variants',
            'execution_policy'
        ) `
        -Label 'MVP scenario registration'
    $scenarioId = Assert-MvpProcessQualificationIdentifier `
        -Value ([string]$ScenarioRegistration.scenario_id) `
        -Label 'MVP scenario registration scenario_id'
    $variantRegistered = @($ScenarioRegistration.variants | Where-Object { [string]$_ -ceq $ScenarioVariant }).Count -eq 1
    if (-not $variantRegistered) {
        throw "MVP process qualification context variant '$ScenarioVariant' is not registered for scenario '$scenarioId'."
    }
    $normalizedReceiptIds = [Collections.Generic.List[string]]::new()
    $seenReceiptIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($receiptIdEntry in @($ProductReceiptIds)) {
        $receiptId = ([string]$receiptIdEntry).ToUpperInvariant()
        if ($receiptId -notmatch '^[0-9A-F]{64}$') {
            throw "MVP process qualification context ProductReceipt ID '$receiptIdEntry' must be a SHA-256."
        }
        if (-not $seenReceiptIds.Add($receiptId)) {
            throw "MVP process qualification context contains duplicate ProductReceipt ID '$receiptId'."
        }
        $normalizedReceiptIds.Add($receiptId) | Out-Null
    }
    $qualificationStatus = if ($normalizedReceiptIds.Count -eq 0) {
        'unqualified_missing_product_receipt'
    }
    else {
        'pending_observation'
    }
    $canonicalPayload = Get-MvpProcessQualificationCanonicalPayload `
        -QualificationStatus $qualificationStatus `
        -RunId $RunId `
        -SourceFingerprint $SourceFingerprint `
        -BuildSetId $BuildSetId `
        -ScenarioRegistryId $registryId `
        -ScenarioRegistrySha256 $registrySha256 `
        -ScenarioId $scenarioId `
        -ScenarioVariant $ScenarioVariant `
        -ProductReceiptIds $normalizedReceiptIds.ToArray()
    $context = [pscustomobject][ordered]@{
        schema_version = $canonicalPayload.schema_version
        context_kind = $canonicalPayload.context_kind
        context_id = Get-MvpProcessQualificationContextId -CanonicalPayload $canonicalPayload
        qualification_status = $canonicalPayload.qualification_status
        run_id = $canonicalPayload.run_id
        source_fingerprint = $canonicalPayload.source_fingerprint
        build_set_id = $canonicalPayload.build_set_id
        scenario_registry_id = $canonicalPayload.scenario_registry_id
        scenario_registry_sha256 = $canonicalPayload.scenario_registry_sha256
        scenario_id = $canonicalPayload.scenario_id
        scenario_variant = $canonicalPayload.scenario_variant
        product_receipt_ids = @($canonicalPayload.product_receipt_ids)
    }
    return Assert-MvpProcessQualificationContext -Context $context -ExpectedRunId $RunId
}

function Get-MvpProcessQualificationContextSetCanonicalPayload {
    param(
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][ValidateSet('unqualified_missing_product_receipt', 'pending_observation')][string]$QualificationStatus,
        [AllowEmptyCollection()][Parameter(Mandatory)][object[]]$Entries
    )

    return [ordered]@{
        schema_version = 1
        receipt_kind = 'zircon.mvp-process-qualification-context-set'
        run_id = $RunId
        context_count = $Entries.Count
        qualification_status = $QualificationStatus
        entries = @($Entries)
    }
}

function Assert-MvpProcessQualificationContextSetReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [string]$ExpectedRunId
    )

    Assert-MvpProcessQualificationExactProperties `
        -Value $Receipt `
        -ExpectedNames @(
            'schema_version', 'receipt_kind', 'run_id', 'context_count',
            'qualification_status', 'entries', 'sha256'
        ) `
        -Label 'MVP process qualification context-set receipt'
    if ([int]$Receipt.schema_version -ne 1 -or
        [string]$Receipt.receipt_kind -cne 'zircon.mvp-process-qualification-context-set') {
        throw 'MVP process qualification context-set receipt has an incompatible schema identity.'
    }
    $runId = [string]$Receipt.run_id
    if ($runId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "MVP process qualification context-set receipt run_id '$runId' is invalid."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedRunId) -and $runId -cne $ExpectedRunId) {
        throw "MVP process qualification context-set receipt run_id '$runId' differs from expected '$ExpectedRunId'."
    }
    $entries = @($Receipt.entries)
    if ($entries.Count -eq 0 -or $entries.Count -gt 64 -or [int]$Receipt.context_count -ne $entries.Count) {
        throw 'MVP process qualification context-set receipt context_count differs from its bounded entries.'
    }
    $seenContextIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seenScenarioVariants = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $containsUnqualifiedContext = $false
    $canonicalEntries = [Collections.Generic.List[object]]::new()
    foreach ($entry in $entries) {
        Assert-MvpProcessQualificationExactProperties `
            -Value $entry `
            -ExpectedNames @('context_id', 'scenario_id', 'scenario_variant', 'qualification_status') `
            -Label 'MVP process qualification context-set entry'
        $contextId = [string]$entry.context_id
        if ($contextId -notmatch '^[0-9a-f]{64}$') {
            throw "MVP process qualification context-set entry context_id '$contextId' is invalid."
        }
        if (-not $seenContextIds.Add($contextId)) {
            throw "MVP process qualification context-set receipt contains duplicate context_id '$contextId'."
        }
        $scenarioId = Assert-MvpProcessQualificationIdentifier `
            -Value ([string]$entry.scenario_id) `
            -Label 'MVP process qualification context-set entry scenario_id'
        $scenarioVariant = Assert-MvpProcessQualificationIdentifier `
            -Value ([string]$entry.scenario_variant) `
            -Label 'MVP process qualification context-set entry scenario_variant'
        $scenarioVariantKey = $scenarioId + "`n" + $scenarioVariant
        if (-not $seenScenarioVariants.Add($scenarioVariantKey)) {
            throw "MVP process qualification context-set receipt contains duplicate scenario/variant '$scenarioId/$scenarioVariant'."
        }
        $entryStatus = [string]$entry.qualification_status
        if ($entryStatus -cnotin @('unqualified_missing_product_receipt', 'pending_observation')) {
            throw "MVP process qualification context-set entry has unsupported status '$entryStatus'."
        }
        if ($entryStatus -ceq 'unqualified_missing_product_receipt') {
            $containsUnqualifiedContext = $true
        }
        $canonicalEntries.Add([pscustomobject][ordered]@{
                context_id = $contextId
                scenario_id = $scenarioId
                scenario_variant = $scenarioVariant
                qualification_status = $entryStatus
            }) | Out-Null
    }
    $expectedStatus = if ($containsUnqualifiedContext) {
        'unqualified_missing_product_receipt'
    }
    else {
        'pending_observation'
    }
    if ([string]$Receipt.qualification_status -cne $expectedStatus) {
        throw "MVP process qualification context-set receipt status differs from expected '$expectedStatus'."
    }
    $canonicalPayload = Get-MvpProcessQualificationContextSetCanonicalPayload `
        -RunId $runId `
        -QualificationStatus $expectedStatus `
        -Entries $canonicalEntries.ToArray()
    $expectedSha256 = Get-MvpProcessQualificationSha256 `
        -Bytes ([Text.UTF8Encoding]::new($false, $true).GetBytes(
            ($canonicalPayload | ConvertTo-Json -Depth 6 -Compress)))
    if ([string]$Receipt.sha256 -cne $expectedSha256) {
        throw 'MVP process qualification context-set receipt sha256 differs from its canonical payload.'
    }
    return $Receipt
}

function Get-MvpProcessQualificationContextSetReceipt {
    param(
        [AllowEmptyCollection()][Parameter(Mandatory)][object[]]$Contexts,
        [Parameter(Mandatory)][string]$ExpectedRunId
    )

    if ($Contexts.Count -eq 0 -or $Contexts.Count -gt 64) {
        throw 'MVP process qualification context set must contain 1..64 contexts.'
    }
    $entries = [Collections.Generic.List[object]]::new()
    $seenContextIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $containsUnqualifiedContext = $false
    foreach ($context in $Contexts) {
        $validatedContext = Assert-MvpProcessQualificationContext `
            -Context $context `
            -ExpectedRunId $ExpectedRunId
        $contextId = [string]$validatedContext.context_id
        if (-not $seenContextIds.Add($contextId)) {
            throw "MVP process qualification context set contains duplicate context_id '$contextId'."
        }
        if ([string]$validatedContext.qualification_status -ceq 'unqualified_missing_product_receipt') {
            $containsUnqualifiedContext = $true
        }
        $entries.Add([pscustomobject][ordered]@{
                context_id = $contextId
                scenario_id = [string]$validatedContext.scenario_id
                scenario_variant = [string]$validatedContext.scenario_variant
                qualification_status = [string]$validatedContext.qualification_status
            }) | Out-Null
    }
    $qualificationStatus = if ($containsUnqualifiedContext) {
        'unqualified_missing_product_receipt'
    }
    else {
        'pending_observation'
    }
    $canonicalPayload = Get-MvpProcessQualificationContextSetCanonicalPayload `
        -RunId $ExpectedRunId `
        -QualificationStatus $qualificationStatus `
        -Entries $entries.ToArray()
    $sha256 = Get-MvpProcessQualificationSha256 `
        -Bytes ([Text.UTF8Encoding]::new($false, $true).GetBytes(
            ($canonicalPayload | ConvertTo-Json -Depth 6 -Compress)))
    $receipt = [pscustomobject][ordered]@{
        schema_version = $canonicalPayload.schema_version
        receipt_kind = $canonicalPayload.receipt_kind
        run_id = $canonicalPayload.run_id
        context_count = $canonicalPayload.context_count
        qualification_status = $canonicalPayload.qualification_status
        entries = @($canonicalPayload['entries'])
        sha256 = $sha256
    }
    return Assert-MvpProcessQualificationContextSetReceipt `
        -Receipt $receipt `
        -ExpectedRunId $ExpectedRunId
}

Export-ModuleMember -Function @(
    'New-MvpProcessQualificationContext',
    'Assert-MvpProcessQualificationContext',
    'Get-MvpProcessQualificationContextSetReceipt',
    'Assert-MvpProcessQualificationContextSetReceipt'
)
