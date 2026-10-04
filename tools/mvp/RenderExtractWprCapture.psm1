Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Resolve-RenderExtractWprProfile {
    param(
        [switch]$UseWpr,
        [switch]$UseWprHeap
    )

    if ($UseWpr -and $UseWprHeap) {
        throw '-UseWpr and -UseWprHeap are mutually exclusive; CPU and Heap traces require separate runs.'
    }
    if ($UseWprHeap) {
        return 'Heap'
    }
    if ($UseWpr) {
        return 'CPU'
    }
    return $null
}

function Invoke-RenderExtractWprNativeTool {
    param(
        [Parameter(Mandatory)][string]$FilePath,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    & $FilePath @Arguments | Out-Null
    return $LASTEXITCODE
}

function Assert-RenderExtractWprEvidencePath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not [IO.Path]::IsPathRooted($Path)) {
        throw "$Label must be an absolute path."
    }
    $resolved = [IO.Path]::GetFullPath($Path)
    $drive = [IO.Path]::GetPathRoot($resolved).TrimEnd('\')
    if ($drive -notin @('D:', 'E:', 'F:')) {
        throw "$Label must stay below D:, E:, or F:."
    }
    return $resolved
}

function Get-RenderExtractWprFileEvidence {
    param([Parameter(Mandatory)][string]$Path)

    $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    if ($item.Length -le 0) {
        throw "WPR evidence is empty: '$($item.FullName)'."
    }
    return [ordered]@{
        path = $item.FullName
        bytes = [Int64]$item.Length
        sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash
    }
}

function Write-RenderExtractWprReceiptNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Content
    )

    try {
        $stream = [IO.FileStream]::new(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
    }
    catch [IO.IOException] {
        throw "Refusing to overwrite existing render-extract WPR receipt: $Path"
    }
    try {
        [byte[]]$bytes = [Text.UTF8Encoding]::new($false).GetBytes($Content)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function ConvertTo-RenderExtractWprTraceRange {
    param(
        [Parameter(Mandatory)][string]$ProcessStartedAtUtc,
        [Parameter(Mandatory)][string]$ProcessEndedAtUtc
    )

    $styles = [Globalization.DateTimeStyles]::RoundtripKind
    $culture = [Globalization.CultureInfo]::InvariantCulture
    $processStarted = [DateTimeOffset]::Parse($ProcessStartedAtUtc, $culture, $styles).ToUniversalTime()
    $processEnded = [DateTimeOffset]::Parse($ProcessEndedAtUtc, $culture, $styles).ToUniversalTime()
    if ($processEnded -lt $processStarted) {
        throw 'WPR product process completion precedes its start.'
    }
    return [ordered]@{
        format = 'utc_wall_clock'
        start_at_utc = $processStarted.ToString('o')
        end_at_utc = $processEnded.ToString('o')
        xperf_start = $processStarted.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", $culture)
        xperf_end = $processEnded.ToString("yyyy/MM/dd:HH:mm:ss.fffffff'+UTC'", $culture)
    }
}

function Start-RenderExtractWprCapture {
    param(
        [Parameter(Mandatory)][string]$TemporaryDirectory,
        [Parameter(Mandatory)]
        [ValidateSet('CPU', 'Heap')]
        [string]$Profile
    )

    $temporaryPath = Assert-RenderExtractWprEvidencePath `
        -Path $TemporaryDirectory `
        -Label 'WPR recording temporary directory'
    if (-not [IO.Directory]::Exists($temporaryPath)) {
        throw "WPR recording temporary directory does not exist: $temporaryPath"
    }
    $wpr = Get-Command wpr.exe -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $wpr) {
        $switchName = if ($Profile -eq 'Heap') { 'UseWprHeap' } else { 'UseWpr' }
        throw "-$switchName requires wpr.exe on the Windows PATH."
    }
    $arguments = @('-start', $Profile, '-filemode', '-recordtempto', $temporaryPath)
    $startedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    $exitCode = Invoke-RenderExtractWprNativeTool -FilePath $wpr.Source -Arguments $arguments
    if ($exitCode -ne 0) {
        throw "WPR could not start the $Profile capture; exit code $exitCode."
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        profile = $Profile
        temporary_directory = $temporaryPath
        wpr_path = $wpr.Source
        started_at_utc = $startedAtUtc
    }
}

function Stop-RenderExtractWprCapture {
    param(
        [Parameter(Mandatory)]$Capture,
        [Parameter(Mandatory)][string]$TracePath,
        [Parameter(Mandatory)][string]$AnalysisPath,
        [Parameter(Mandatory)][string]$ReceiptPath,
        [Int64]$ProcessId,
        [string]$ProcessStartedAtUtc,
        [string]$ProcessEndedAtUtc
    )

    $trace = Assert-RenderExtractWprEvidencePath -Path $TracePath -Label 'WPR ETL path'
    $analysis = Assert-RenderExtractWprEvidencePath -Path $AnalysisPath -Label 'WPR analysis path'
    $receipt = Assert-RenderExtractWprEvidencePath -Path $ReceiptPath -Label 'WPR receipt path'
    $profile = [string]$Capture.profile
    if ($profile -notin @('CPU', 'Heap')) {
        throw "Unsupported render-extract WPR profile '$profile'."
    }

    $stopExitCode = Invoke-RenderExtractWprNativeTool `
        -FilePath ([string]$Capture.wpr_path) `
        -Arguments @('-stop', $trace)
    if ($stopExitCode -ne 0) {
        throw "WPR could not stop the $profile capture at '$trace'; exit code $stopExitCode."
    }
    $traceEvidence = Get-RenderExtractWprFileEvidence -Path $trace

    $hasProcessId = $PSBoundParameters.ContainsKey('ProcessId')
    $hasStarted = -not [string]::IsNullOrWhiteSpace($ProcessStartedAtUtc)
    $hasEnded = -not [string]::IsNullOrWhiteSpace($ProcessEndedAtUtc)
    if (($hasProcessId -or $hasStarted -or $hasEnded) -and
        -not ($hasProcessId -and $hasStarted -and $hasEnded)) {
        throw 'WPR product attribution requires process id, start time, and completion time together.'
    }

    $range = $null
    $analysisEvidence = $null
    $analysisArguments = $null
    if ($hasProcessId -and $hasStarted -and $hasEnded) {
        if ($ProcessId -le 0) {
            throw 'WPR product attribution requires a positive process id.'
        }
        $range = ConvertTo-RenderExtractWprTraceRange `
            -ProcessStartedAtUtc $ProcessStartedAtUtc `
            -ProcessEndedAtUtc $ProcessEndedAtUtc
        $xperf = Get-Command xperf.exe -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -eq $xperf) {
            throw 'Render-extract WPR analysis requires xperf.exe on the Windows PATH.'
        }
        $analysisArguments = if ($profile -eq 'CPU') {
            @(
                '-i', $trace,
                '-symbols',
                '-target', 'machine',
                '-o', $analysis,
                '-a', 'stack',
                '-pid', [string]$ProcessId,
                '-range', [string]$range.xperf_start, [string]$range.xperf_end,
                '-event', 'PROFILE',
                '-butterfly', '1'
            )
        }
        else {
            @(
                '-i', $trace,
                '-symbols',
                '-target', 'machine',
                '-o', $analysis,
                '-a', 'heap',
                '-pid', [string]$ProcessId,
                '-range', [string]$range.xperf_start, [string]$range.xperf_end,
                '-stacks', 'st'
            )
        }
        $analysisExitCode = Invoke-RenderExtractWprNativeTool `
            -FilePath $xperf.Source `
            -Arguments $analysisArguments
        if ($analysisExitCode -ne 0) {
            throw "xperf could not export the product-filtered $profile analysis; exit code $analysisExitCode."
        }
        $analysisEvidence = Get-RenderExtractWprFileEvidence -Path $analysis
    }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = if ($profile -eq 'CPU') {
            'windows_product_sampled_cpu_stacks'
        }
        else {
            'windows_product_heap_allocation_stacks'
        }
        profile = $profile
        started_at_utc = [string]$Capture.started_at_utc
        stopped_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
        trace = $traceEvidence
        analysis = $analysisEvidence
        attribution = [ordered]@{
            scope = if ($null -ne $analysisEvidence) { 'product_process' } else { 'system' }
            process_id = if ($hasProcessId) { $ProcessId } else { $null }
            process_started_at_utc = if ($hasStarted) { $ProcessStartedAtUtc } else { $null }
            process_ended_at_utc = if ($hasEnded) { $ProcessEndedAtUtc } else { $null }
            process_lifetime_range_applied = $null -ne $range
            trace_range = $range
        }
        recorder = [ordered]@{
            path = [string]$Capture.wpr_path
            arguments = @('-start', $profile, '-filemode', '-recordtempto', [string]$Capture.temporary_directory)
        }
        analysis_tool = [ordered]@{
            path = if ($null -ne $analysisArguments) { [string]$xperf.Source } else { $null }
            arguments = $analysisArguments
        }
        storage = [ordered]@{
            recording_temporary_directory = [string]$Capture.temporary_directory
            system_drive_used = $false
        }
    }
    Write-RenderExtractWprReceiptNew `
        -Path $receipt `
        -Content ($result | ConvertTo-Json -Depth 8)
    return $result
}

Export-ModuleMember -Function @(
    'Resolve-RenderExtractWprProfile',
    'Start-RenderExtractWprCapture',
    'Stop-RenderExtractWprCapture'
)
