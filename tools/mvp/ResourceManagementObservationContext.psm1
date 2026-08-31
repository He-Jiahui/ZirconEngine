Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop

function Get-ResourceManagementObservationContextString {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label,
        [ValidateRange(1, 1024)][int]$MaximumLength = 256
    )

    $text = [string](Get-ResourceManagementSchemaProperty -Value $Value -Name $Name -Label $Label)
    if ([string]::IsNullOrWhiteSpace($text) -or $text.Length -gt $MaximumLength) {
        throw "$Label '$Name' must be a non-empty string no longer than $MaximumLength characters."
    }
    return $text
}

function Assert-ResourceManagementObservationContextIdentifier {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -notmatch '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$') {
        throw "$Label must be one stable identifier."
    }
    return $Value
}

function Resolve-ResourceManagementObservationContext {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
    )

    Assert-ResourceManagementSchemaSha256 `
        -Value $ExpectedSourceFingerprint `
        -Label 'Expected observation source_fingerprint' | Out-Null
    Assert-ResourceManagementSchemaProperties `
        -Value $Context `
        -RequiredNames @('schema_version', 'context_kind', 'product_receipt', 'run', 'machine', 'collector') `
        -Label 'Resource-management observation context'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Context `
        -SchemaId 'zircon.resource-management.observation-context' `
        -Label 'Resource-management observation context' | Out-Null

    $productReceipt = Get-ResourceManagementSchemaProperty `
        -Value $Context -Name 'product_receipt' -Label 'Resource-management observation context'
    Assert-ResourceManagementSchemaProperties `
        -Value $productReceipt `
        -RequiredNames @(
            'schema_version', 'receipt_kind', 'receipt_id', 'source_fingerprint',
            'product_role', 'executable_sha256', 'build_set_id') `
        -Label 'Resource-management observation ProductReceipt'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $productReceipt `
        -SchemaId 'zircon.resource-management.product-receipt' `
        -Label 'Resource-management observation ProductReceipt' | Out-Null
    $receiptId = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$productReceipt.receipt_id) `
        -Label 'Resource-management observation ProductReceipt receipt_id'
    $sourceFingerprint = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$productReceipt.source_fingerprint) `
        -Label 'Resource-management observation ProductReceipt source_fingerprint'
    if (-not $sourceFingerprint.Equals($ExpectedSourceFingerprint, [StringComparison]::Ordinal)) {
        throw 'Resource-management observation ProductReceipt source_fingerprint differs from the observation source snapshot.'
    }
    $productRole = Get-ResourceManagementObservationContextString `
        -Value $productReceipt -Name 'product_role' -Label 'Resource-management observation ProductReceipt'
    if ($productRole -cne 'editor') {
        throw "Resource-management observation ProductReceipt product_role '$productRole' is unsupported."
    }
    $executableSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$productReceipt.executable_sha256) `
        -Label 'Resource-management observation ProductReceipt executable_sha256'
    $buildSetId = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$productReceipt.build_set_id) `
        -Label 'Resource-management observation ProductReceipt build_set_id'

    $run = Get-ResourceManagementSchemaProperty `
        -Value $Context -Name 'run' -Label 'Resource-management observation context'
    Assert-ResourceManagementSchemaProperties `
        -Value $run `
        -RequiredNames @('run_id') `
        -Label 'Resource-management observation run'
    $runId = Assert-ResourceManagementObservationContextIdentifier `
        -Value (Get-ResourceManagementObservationContextString -Value $run -Name 'run_id' -Label 'Resource-management observation run') `
        -Label 'Resource-management observation run_id'
    $machine = Get-ResourceManagementSchemaProperty `
        -Value $Context -Name 'machine' -Label 'Resource-management observation context'
    Assert-ResourceManagementSchemaProperties `
        -Value $machine `
        -RequiredNames @(
            'machine_id_sha256', 'cpu_model', 'logical_processor_count',
            'physical_memory_bytes', 'os_name', 'os_version', 'architecture') `
        -Label 'Resource-management observation machine'
    $machineIdSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$machine.machine_id_sha256) `
        -Label 'Resource-management observation machine_id_sha256'
    $cpuModel = Get-ResourceManagementObservationContextString `
        -Value $machine -Name 'cpu_model' -Label 'Resource-management observation machine'
    $logicalProcessorCount = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $machine.logical_processor_count `
        -Label 'Resource-management observation logical_processor_count'
    $physicalMemoryBytes = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $machine.physical_memory_bytes `
        -Label 'Resource-management observation physical_memory_bytes'
    if ($logicalProcessorCount -lt 1 -or $logicalProcessorCount -gt [uint16]::MaxValue -or
        $physicalMemoryBytes -lt 1) {
        throw 'Resource-management observation machine capacity values must be positive and bounded.'
    }
    $osName = Get-ResourceManagementObservationContextString `
        -Value $machine -Name 'os_name' -Label 'Resource-management observation machine'
    $osVersion = Get-ResourceManagementObservationContextString `
        -Value $machine -Name 'os_version' -Label 'Resource-management observation machine'
    $architecture = Get-ResourceManagementObservationContextString `
        -Value $machine -Name 'architecture' -Label 'Resource-management observation machine'
    if ($architecture -cnotin @('x64', 'arm64')) {
        throw "Resource-management observation machine architecture '$architecture' is unsupported."
    }

    $collector = Get-ResourceManagementSchemaProperty `
        -Value $Context -Name 'collector' -Label 'Resource-management observation context'
    Assert-ResourceManagementSchemaProperties `
        -Value $collector `
        -RequiredNames @('collector_id', 'collector_version', 'clock_domain') `
        -Label 'Resource-management observation collector'
    $collectorId = Assert-ResourceManagementObservationContextIdentifier `
        -Value (Get-ResourceManagementObservationContextString -Value $collector -Name 'collector_id' -Label 'Resource-management observation collector') `
        -Label 'Resource-management observation collector_id'
    $collectorVersion = Get-ResourceManagementObservationContextString `
        -Value $collector -Name 'collector_version' -Label 'Resource-management observation collector' -MaximumLength 64
    if ($collectorVersion -notmatch '^\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.-]+)?$') {
        throw 'Resource-management observation collector_version must be a semantic version.'
    }
    $clockDomain = Get-ResourceManagementObservationContextString `
        -Value $collector -Name 'clock_domain' -Label 'Resource-management observation collector'
    if ($clockDomain -cne 'zircon-profile-microseconds') {
        throw "Resource-management observation collector clock_domain '$clockDomain' is unsupported."
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        context_kind = 'zircon.resource-management-observation-context'
        product_receipt = [pscustomobject][ordered]@{
            schema_version = 1
            receipt_kind = 'zircon.mvp-product-receipt'
            receipt_id = $receiptId
            source_fingerprint = $sourceFingerprint
            product_role = $productRole
            executable_sha256 = $executableSha256
            build_set_id = $buildSetId
        }
        run = [pscustomobject][ordered]@{
            run_id = $runId
        }
        machine = [pscustomobject][ordered]@{
            machine_id_sha256 = $machineIdSha256
            cpu_model = $cpuModel
            logical_processor_count = [uint16]$logicalProcessorCount
            physical_memory_bytes = [uint64]$physicalMemoryBytes
            os_name = $osName
            os_version = $osVersion
            architecture = $architecture
        }
        collector = [pscustomobject][ordered]@{
            collector_id = $collectorId
            collector_version = $collectorVersion
            clock_domain = $clockDomain
        }
    }
}

function Resolve-ResourceManagementSampleProcessContext {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ResourceManagementSchemaProperties `
        -Value $Context `
        -RequiredNames @(
            'schema_version', 'context_kind', 'process_id', 'process_created_utc', 'trace_id',
            'first_frame_index', 'last_frame_index') `
        -Label $Label
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Context `
        -SchemaId 'zircon.resource-management.sample-process-context' `
        -Label $Label | Out-Null
    $processId = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $Context.process_id `
        -Label "$Label process_id"
    if ($processId -lt 1 -or $processId -gt [uint32]::MaxValue) {
        throw "$Label process_id must be a positive 32-bit integer."
    }
    $processCreatedValue = Get-ResourceManagementSchemaProperty `
        -Value $Context -Name 'process_created_utc' -Label $Label
    $processCreatedUtc = if ($processCreatedValue -is [DateTimeOffset]) {
        ([DateTimeOffset]$processCreatedValue).ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
    }
    elseif ($processCreatedValue -is [DateTime]) {
        ([DateTime]$processCreatedValue).ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
    }
    else {
        [string]$processCreatedValue
    }
    $parsedProcessCreatedUtc = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
            $processCreatedUtc,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$parsedProcessCreatedUtc
        ) -or $parsedProcessCreatedUtc.Offset -ne [TimeSpan]::Zero) {
        throw "$Label process_created_utc must be an ISO-8601 UTC timestamp."
    }
    $traceId = Assert-ResourceManagementObservationContextIdentifier `
        -Value (Get-ResourceManagementObservationContextString -Value $Context -Name 'trace_id' -Label $Label) `
        -Label "$Label trace_id"
    $firstFrameIndex = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $Context.first_frame_index `
        -Label "$Label first_frame_index"
    $lastFrameIndex = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value $Context.last_frame_index `
        -Label "$Label last_frame_index"
    if ($lastFrameIndex -lt $firstFrameIndex) {
        throw "$Label frame range is invalid."
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        context_kind = 'zircon.resource-management-sample-process-context'
        process_id = [uint32]$processId
        process_created_utc = $processCreatedUtc
        trace_id = $traceId
        first_frame_index = [uint64]$firstFrameIndex
        last_frame_index = [uint64]$lastFrameIndex
    }
}

function Assert-ResourceManagementSampleProcessLifecycle {
    param(
        [Parameter(Mandatory)][object[]]$ProcessContexts,
        [Parameter(Mandatory)][ValidateSet('fresh-process', 'same-process')][string]$ProcessLifecycle,
        [Parameter(Mandatory)][string]$Label
    )

    $processInstances = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $traceIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($context in $ProcessContexts) {
        [void]$processInstances.Add("$($context.process_id)$([char]0)$($context.process_created_utc)")
        if (-not $traceIds.Add([string]$context.trace_id)) {
            throw "$Label contains duplicate trace_id '$($context.trace_id)'."
        }
    }
    if ($ProcessLifecycle -eq 'fresh-process' -and $processInstances.Count -ne $ProcessContexts.Count) {
        throw "$Label violates its fresh-process lifecycle."
    }
    if ($ProcessLifecycle -eq 'same-process' -and $processInstances.Count -ne 1) {
        throw "$Label violates its same-process lifecycle."
    }
}

function Assert-ResourceManagementObservationContextsComparable {
    param(
        [Parameter(Mandatory)]$ApprovedBaseline,
        [Parameter(Mandatory)]$Candidate
    )

    $baselineMachine = @(
        $ApprovedBaseline.machine.machine_id_sha256,
        $ApprovedBaseline.machine.cpu_model,
        $ApprovedBaseline.machine.logical_processor_count,
        $ApprovedBaseline.machine.physical_memory_bytes,
        $ApprovedBaseline.machine.os_name,
        $ApprovedBaseline.machine.os_version,
        $ApprovedBaseline.machine.architecture
    ) -join [char]0
    $candidateMachine = @(
        $Candidate.machine.machine_id_sha256,
        $Candidate.machine.cpu_model,
        $Candidate.machine.logical_processor_count,
        $Candidate.machine.physical_memory_bytes,
        $Candidate.machine.os_name,
        $Candidate.machine.os_version,
        $Candidate.machine.architecture
    ) -join [char]0
    if (-not $baselineMachine.Equals($candidateMachine, [StringComparison]::Ordinal)) {
        throw 'Approved baseline and candidate observation context machine identities differ.'
    }
    $baselineCollector = @(
        $ApprovedBaseline.product_receipt.product_role,
        $ApprovedBaseline.collector.collector_id,
        $ApprovedBaseline.collector.collector_version,
        $ApprovedBaseline.collector.clock_domain
    ) -join [char]0
    $candidateCollector = @(
        $Candidate.product_receipt.product_role,
        $Candidate.collector.collector_id,
        $Candidate.collector.collector_version,
        $Candidate.collector.clock_domain
    ) -join [char]0
    if (-not $baselineCollector.Equals($candidateCollector, [StringComparison]::Ordinal)) {
        throw 'Approved baseline and candidate observation context collector contracts differ.'
    }
}

Export-ModuleMember -Function @(
    'Assert-ResourceManagementObservationContextsComparable',
    'Assert-ResourceManagementSampleProcessLifecycle',
    'Resolve-ResourceManagementObservationContext',
    'Resolve-ResourceManagementSampleProcessContext'
)
