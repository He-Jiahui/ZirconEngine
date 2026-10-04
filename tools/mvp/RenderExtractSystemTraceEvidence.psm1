Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$moduleRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $moduleRepoRoot 'tools\maintenance\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractBaselineEvidence.psm1') -Force -DisableNameChecking -ErrorAction Stop

function Get-RenderExtractSystemTraceProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label,
        [switch]$Optional
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        if ($Optional) {
            return $null
        }
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function ConvertTo-RenderExtractSystemTraceUtcTime {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        if ($Value -is [DateTimeOffset]) {
            return ([DateTimeOffset]$Value).ToUniversalTime()
        }
        if ($Value -is [DateTime]) {
            return ([DateTimeOffset]([DateTime]$Value)).ToUniversalTime()
        }
        return [DateTimeOffset]::Parse(
            [string]$Value,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind
        ).ToUniversalTime()
    }
    catch {
        throw "$Label is not a round-trip UTC timestamp: $($_.Exception.Message)"
    }
}

function Test-RenderExtractSystemTracePathWithinDirectory {
    param(
        [Parameter(Mandatory)][string]$CandidatePath,
        [Parameter(Mandatory)][string]$RootPath
    )

    $root = $RootPath.TrimEnd('\', '/')
    $prefix = $root + [IO.Path]::DirectorySeparatorChar
    return $CandidatePath.Equals($root, [StringComparison]::OrdinalIgnoreCase) -or
        $CandidatePath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
}

function Get-RenderExtractSystemTraceFileEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Kind,
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][Int64]$ProcessId,
        [Parameter(Mandatory)][string]$Profile
    )

    if (-not [IO.File]::Exists($Path)) {
        throw "Render-extract system trace evidence does not exist: $Path"
    }
    $item = [IO.FileInfo]::new($Path)
    if ($item.Length -le 0) {
        throw "Render-extract system trace evidence is empty: $Path"
    }
    return [ordered]@{
        logical_id = $LogicalId
        attempt = $Attempt
        kind = $Kind
        profile = $Profile
        process_id = $ProcessId
        path = $item.FullName
        bytes = [Int64]$item.Length
        sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash
    }
}

function Assert-RenderExtractSystemTraceDeterministicPath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$TracesRoot,
        [Parameter(Mandatory)][string]$ExpectedName,
        [Parameter(Mandatory)][string]$Label
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not (Test-RenderExtractSystemTracePathWithinDirectory `
            -CandidatePath $resolution.OperationalPath `
            -RootPath $TracesRoot)) {
        throw "$Label is outside this evidence session."
    }
    $expected = [IO.Path]::GetFullPath((Join-Path $TracesRoot $ExpectedName))
    if (-not $resolution.OperationalPath.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label does not match its deterministic session id (observed '$($resolution.OperationalPath)', expected '$expected')."
    }
    return $resolution.OperationalPath
}

function Get-RenderExtractSystemTraceEvidenceForRun {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][Int64]$ProcessId,
        [Parameter(Mandatory)][string]$InvocationId,
        [Parameter(Mandatory)][string]$TracesRoot
    )

    $traceValue = Get-RenderExtractSystemTraceProperty `
        -Value $Run `
        -Name 'system_trace_etl' `
        -Label "Baseline run '$LogicalId' attempt $Attempt" `
        -Optional
    if ([string]::IsNullOrWhiteSpace([string]$traceValue)) {
        return [pscustomobject][ordered]@{ artifacts = @(); profile = $null; status = 'not_measured' }
    }

    $profileValue = Get-RenderExtractSystemTraceProperty `
        -Value $Run `
        -Name 'system_trace_profile' `
        -Label "Baseline run '$LogicalId' attempt $Attempt" `
        -Optional
    $profile = if ([string]::IsNullOrWhiteSpace([string]$profileValue)) {
        'cpu'
    }
    else {
        ([string]$profileValue).ToLowerInvariant()
    }
    if ($profile -notin @('cpu', 'heap')) {
        throw "Baseline run '$LogicalId' attempt $Attempt has unsupported system_trace_profile '$profile'."
    }
    $resolvedTracesRoot = (Resolve-ZirconWindowsPath -Path $TracesRoot).OperationalPath
    $invocationTracesRoot = Join-ZirconWindowsPath -Path $resolvedTracesRoot -ChildPath $InvocationId
    $sessionId = "$LogicalId-$Attempt"
    $traceName = if ($profile -eq 'heap') { "$sessionId.heap.etl" } else { "$sessionId.etl" }
    $tracePath = Assert-RenderExtractSystemTraceDeterministicPath `
        -Path ([string]$traceValue) `
        -TracesRoot $invocationTracesRoot `
        -ExpectedName $traceName `
        -Label "Baseline run '$LogicalId' attempt $Attempt system trace"
    $traceEvidence = Get-RenderExtractSystemTraceFileEvidence `
        -Path $tracePath `
        -Kind 'system_trace_etl' `
        -LogicalId $LogicalId `
        -Attempt $Attempt `
        -ProcessId $ProcessId `
        -Profile $profile

    if ([string]::IsNullOrWhiteSpace([string]$profileValue)) {
        return [pscustomobject][ordered]@{
            artifacts = @($traceEvidence)
            profile = $profile
            status = 'raw_only'
        }
    }

    $analysisValue = Get-RenderExtractSystemTraceProperty `
        -Value $Run `
        -Name 'system_trace_analysis' `
        -Label "Baseline run '$LogicalId' attempt $Attempt"
    $receiptValue = Get-RenderExtractSystemTraceProperty `
        -Value $Run `
        -Name 'system_trace_receipt' `
        -Label "Baseline run '$LogicalId' attempt $Attempt"
    $analysisName = if ($profile -eq 'heap') {
        "$sessionId.heap-product-allocation-stacks.txt"
    }
    else {
        "$sessionId.cpu-product-sampled-stacks.txt"
    }
    $receiptName = "$sessionId.$profile-wpr-capture.json"
    $analysisPath = Assert-RenderExtractSystemTraceDeterministicPath `
        -Path ([string]$analysisValue) `
        -TracesRoot $invocationTracesRoot `
        -ExpectedName $analysisName `
        -Label "Baseline run '$LogicalId' attempt $Attempt system trace analysis"
    $receiptPath = Assert-RenderExtractSystemTraceDeterministicPath `
        -Path ([string]$receiptValue) `
        -TracesRoot $invocationTracesRoot `
        -ExpectedName $receiptName `
        -Label "Baseline run '$LogicalId' attempt $Attempt system trace receipt"
    $analysisKind = "system_trace_$($profile)_analysis"
    $analysisEvidence = Get-RenderExtractSystemTraceFileEvidence `
        -Path $analysisPath `
        -Kind $analysisKind `
        -LogicalId $LogicalId `
        -Attempt $Attempt `
        -ProcessId $ProcessId `
        -Profile $profile
    $receiptSnapshot = Read-RenderExtractJsonEvidence `
        -Path $receiptPath `
        -Label "Baseline run '$LogicalId' attempt $Attempt system trace receipt"
    $receiptEvidence = [ordered]@{
        logical_id = $LogicalId
        attempt = $Attempt
        kind = 'system_trace_receipt'
        profile = $profile
        process_id = $ProcessId
        path = $receiptSnapshot.path
        bytes = $receiptSnapshot.bytes
        sha256 = $receiptSnapshot.sha256
    }
    $receipt = $receiptSnapshot.json
    $receiptSchemaVersion = [int](Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'schema_version' -Label 'System trace receipt')
    $evidenceKind = [string](Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'evidence_kind' -Label 'System trace receipt')
    $receiptProfile = [string](Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'profile' -Label 'System trace receipt')
    $traceStartedAtUtc = Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'started_at_utc' -Label 'System trace receipt'
    $attribution = Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'attribution' -Label 'System trace receipt'
    $receiptTrace = Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'trace' -Label 'System trace receipt'
    $receiptAnalysis = Get-RenderExtractSystemTraceProperty -Value $receipt -Name 'analysis' -Label 'System trace receipt'
    $range = Get-RenderExtractSystemTraceProperty -Value $attribution -Name 'trace_range' -Label 'System trace receipt attribution'
    $processStartedAtUtc = Get-RenderExtractSystemTraceProperty -Value $attribution -Name 'process_started_at_utc' -Label 'System trace receipt attribution'
    $processEndedAtUtc = Get-RenderExtractSystemTraceProperty -Value $attribution -Name 'process_ended_at_utc' -Label 'System trace receipt attribution'
    $runStartedAtUtc = Get-RenderExtractSystemTraceProperty -Value $Run -Name 'started_at_utc' -Label "Baseline run '$LogicalId' attempt $Attempt"
    $runEndedAtUtc = Get-RenderExtractSystemTraceProperty -Value $Run -Name 'ended_at_utc' -Label "Baseline run '$LogicalId' attempt $Attempt"
    $culture = [Globalization.CultureInfo]::InvariantCulture
    $traceStarted = ConvertTo-RenderExtractSystemTraceUtcTime -Value $traceStartedAtUtc -Label 'System trace start'
    $processStarted = ConvertTo-RenderExtractSystemTraceUtcTime -Value $processStartedAtUtc -Label 'System trace product start'
    $processEnded = ConvertTo-RenderExtractSystemTraceUtcTime -Value $processEndedAtUtc -Label 'System trace product end'
    $runStarted = ConvertTo-RenderExtractSystemTraceUtcTime -Value $runStartedAtUtc -Label 'Baseline run start'
    $runEnded = ConvertTo-RenderExtractSystemTraceUtcTime -Value $runEndedAtUtc -Label 'Baseline run end'
    $rangeStarted = ConvertTo-RenderExtractSystemTraceUtcTime -Value $range.start_at_utc -Label 'System trace range start'
    $rangeEnded = ConvertTo-RenderExtractSystemTraceUtcTime -Value $range.end_at_utc -Label 'System trace range end'
    $expectedXperfStart = $processStarted.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", $culture)
    $expectedXperfEnd = $processEnded.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", $culture)
    $receiptTracePath = (Resolve-ZirconWindowsPath -Path ([string]$receiptTrace.path)).OperationalPath
    $receiptAnalysisPath = (Resolve-ZirconWindowsPath -Path ([string]$receiptAnalysis.path)).OperationalPath
    $expectedEvidenceKind = if ($profile -eq 'cpu') {
        'windows_product_sampled_cpu_stacks'
    }
    else {
        'windows_product_heap_allocation_stacks'
    }
    if ($receiptSchemaVersion -ne 1 -or
        $evidenceKind -ne $expectedEvidenceKind -or
        -not $receiptProfile.Equals($profile, [StringComparison]::OrdinalIgnoreCase) -or
        [string]$attribution.scope -ne 'product_process' -or
        [Int64]$attribution.process_id -ne $ProcessId -or
        -not [bool]$attribution.process_lifetime_range_applied -or
        $traceStarted -gt $processStarted -or
        $processStarted -lt $runStarted -or
        $processEnded -gt $runEnded -or
        [string]$range.format -ne 'utc_wall_clock' -or
        $rangeStarted -ne $processStarted -or
        $rangeEnded -ne $processEnded -or
        [string]$range.xperf_start -ne $expectedXperfStart -or
        [string]$range.xperf_end -ne $expectedXperfEnd -or
        -not $receiptTracePath.Equals($tracePath, [StringComparison]::OrdinalIgnoreCase) -or
        [Int64]$receiptTrace.bytes -ne [Int64]$traceEvidence.bytes -or
        -not ([string]$receiptTrace.sha256).Equals($traceEvidence.sha256, [StringComparison]::OrdinalIgnoreCase) -or
        -not $receiptAnalysisPath.Equals($analysisPath, [StringComparison]::OrdinalIgnoreCase) -or
        [Int64]$receiptAnalysis.bytes -ne [Int64]$analysisEvidence.bytes -or
        -not ([string]$receiptAnalysis.sha256).Equals($analysisEvidence.sha256, [StringComparison]::OrdinalIgnoreCase)) {
        throw ("Baseline run '$LogicalId' attempt $Attempt system trace receipt does not bind its profile, process lifetime, ETL, and analysis artifacts. " +
            "schema=$receiptSchemaVersion evidence=$evidenceKind/$expectedEvidenceKind profile=$receiptProfile/$profile scope=$($attribution.scope) pid=$($attribution.process_id)/$ProcessId lifetime=$($attribution.process_lifetime_range_applied) " +
            "trace_start=$traceStarted process=$processStarted-$processEnded run=$runStarted-$runEnded range=$rangeStarted-$rangeEnded format=$($range.format) xperf=$($range.xperf_start)/$expectedXperfStart,$($range.xperf_end)/$expectedXperfEnd " +
            "trace_path=$($receiptTrace.path)/$tracePath trace_bytes=$($receiptTrace.bytes)/$($traceEvidence.bytes) trace_sha=$($receiptTrace.sha256)/$($traceEvidence.sha256) " +
            "analysis_path=$($receiptAnalysis.path)/$analysisPath analysis_bytes=$($receiptAnalysis.bytes)/$($analysisEvidence.bytes) analysis_sha=$($receiptAnalysis.sha256)/$($analysisEvidence.sha256)")
    }
    return [pscustomobject][ordered]@{
        artifacts = @($traceEvidence, $analysisEvidence, $receiptEvidence)
        profile = $profile
        status = 'measured'
    }
}

function Get-RenderExtractSystemTraceMeasurementCoverage {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Records,
        [Parameter(Mandatory)][ValidateSet('cpu', 'heap')][string]$Profile
    )

    $profileRecords = @($Records | Where-Object { $_.profile -eq $Profile })
    $measuredCount = @($profileRecords | Where-Object { $_.status -eq 'measured' }).Count
    $status = if ($profileRecords.Count -gt 0 -and $measuredCount -eq $profileRecords.Count) {
        'measured'
    }
    elseif ($measuredCount -gt 0) {
        'partial'
    }
    else {
        'not_measured'
    }
    return [ordered]@{
        status = $status
        source = if ($Profile -eq 'cpu') {
            'PID and process-lifetime-filtered xperf sampled CPU stacks'
        }
        else {
            'PID and process-lifetime-filtered xperf Heap allocation stacks sorted by total allocation'
        }
    }
}

function Get-RenderExtractCpuTimelineCoverage {
    param([Parameter(Mandatory)][AllowEmptyCollection()][object[]]$SystemTraceRecords)

    $heapMeasured = @($SystemTraceRecords | Where-Object {
            $_.profile -eq 'heap' -and $_.status -eq 'measured'
        }).Count -gt 0
    if ($heapMeasured) {
        return [ordered]@{
            status = 'instrumented_not_baseline'
            source = 'timeline.zrtrace.json frame and span samples'
            reason = 'WPR Heap tracking was active; frame and CPU timeline values are retained for correlation but cannot be used as a performance baseline.'
        }
    }
    return [ordered]@{
        status = 'measured'
        source = 'timeline.zrtrace.json frame and span samples'
    }
}

Export-ModuleMember -Function @(
    'Get-RenderExtractSystemTraceEvidenceForRun',
    'Get-RenderExtractSystemTraceMeasurementCoverage',
    'Get-RenderExtractCpuTimelineCoverage'
)
