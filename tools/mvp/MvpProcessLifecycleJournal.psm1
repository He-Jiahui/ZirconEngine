$ErrorActionPreference = 'Stop'

$script:MvpProcessJournalLowerHexDigits = [char[]]'0123456789abcdef'
$script:MvpProcessJournalEventSchemaVersion = 1
$script:MvpProcessJournalEventStreamKind = 'zircon.mvp-process-lifecycle-event'
$script:MvpProcessJournalUtf8 = [Text.UTF8Encoding]::new($false)
$script:MvpProcessJournalStrictUtf8 = [Text.UTF8Encoding]::new($false, $true)
$script:MvpProcessJournalByteArrayPool = $null
$mvpProcessJournalArrayPoolType = 'System.Buffers.ArrayPool`1[System.Byte]' -as [type]
if ($null -ne $mvpProcessJournalArrayPoolType) {
    $script:MvpProcessJournalByteArrayPool =
        $mvpProcessJournalArrayPoolType.GetProperty('Shared').GetValue($null)
}
Remove-Variable -Name mvpProcessJournalArrayPoolType

function ConvertTo-MvpProcessJournalLowerHex {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpProcessJournalLowerHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpProcessJournalLowerHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpProcessJournalSha256 {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProcessJournalLowerHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpProcessJournalStringSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $byteCount = $script:MvpProcessJournalUtf8.GetByteCount($Text)
    $bufferLength = [Math]::Max(1, $byteCount)
    [byte[]]$buffer = $null
    if ($null -ne $script:MvpProcessJournalByteArrayPool) {
        $buffer = $script:MvpProcessJournalByteArrayPool.Rent($bufferLength)
    }
    else {
        $buffer = [byte[]]::new($bufferLength)
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $written = $script:MvpProcessJournalUtf8.GetBytes(
            $Text,
            0,
            $Text.Length,
            $buffer,
            0)
        return ConvertTo-MvpProcessJournalLowerHex -Bytes $hasher.ComputeHash($buffer, 0, $written)
    }
    finally {
        $hasher.Dispose()
        if ($null -ne $script:MvpProcessJournalByteArrayPool) {
            $script:MvpProcessJournalByteArrayPool.Return($buffer, $false)
        }
    }
}

function Get-MvpProcessJournalFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProcessJournalLowerHex -Bytes $hasher.ComputeHash($stream)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Join-MvpProcessJournalPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$ChildPath
    )

    return [IO.Path]::GetFullPath([IO.Path]::Combine($Root, $ChildPath))
}

function Assert-MvpProcessJournalResumeEventSchema {
    param([Parameter(Mandatory)]$Event)

    $schemaVersionProperty = $Event.PSObject.Properties['schema_version']
    $eventStreamKindProperty = $Event.PSObject.Properties['event_stream_kind']
    if ($null -eq $schemaVersionProperty -or $null -eq $eventStreamKindProperty) {
        throw 'Active process journal terminal event is missing its event schema identity.'
    }
    try {
        $schemaVersion = [int]$schemaVersionProperty.Value
    }
    catch {
        throw 'Active process journal terminal event has an invalid schema version.'
    }
    if ($schemaVersion -ne $script:MvpProcessJournalEventSchemaVersion) {
        throw "Active process journal has unsupported schema version '$schemaVersion'."
    }
    $eventStreamKind = [string]$eventStreamKindProperty.Value
    if (-not [string]::Equals(
            $eventStreamKind,
            $script:MvpProcessJournalEventStreamKind,
            [StringComparison]::Ordinal)) {
        throw "Active process journal has unsupported event stream kind '$eventStreamKind'."
    }
}

function Get-MvpProcessJournalResumeCursor {
    param(
        [Parameter(Mandatory)][string]$JournalPath,
        [Parameter(Mandatory)][ValidateRange(1024, [Int64]::MaxValue)][Int64]$MaximumJournalBytes
    )

    if (-not [IO.File]::Exists($JournalPath)) {
        return [pscustomobject]@{
            next_sequence = 1
            previous_event_sha256 = $null
            journal_offset_bytes = [Int64]0
        }
    }
    $length = [Int64]([IO.FileInfo]::new($JournalPath).Length)
    if ($length -eq 0) {
        return [pscustomobject]@{
            next_sequence = 1
            previous_event_sha256 = $null
            journal_offset_bytes = [Int64]0
        }
    }
    if ($length -gt $MaximumJournalBytes) {
        throw "Active process journal exceeds its byte budget of $MaximumJournalBytes bytes."
    }
    $stream = [IO.File]::Open(
        $JournalPath,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::ReadWrite)
    $reader = $null
    try {
        [void]$stream.Seek(-1, [IO.SeekOrigin]::End)
        if ($stream.ReadByte() -ne 0x0a) {
            throw 'Active process journal has bytes but no complete event line.'
        }
        [void]$stream.Seek(0, [IO.SeekOrigin]::Begin)
            $reader = [IO.StreamReader]::new(
                $stream,
                $script:MvpProcessJournalStrictUtf8,
            $true,
            8192,
            $true)
        $lastLine = $null
        try {
            while ($true) {
                $line = $reader.ReadLine()
                if ($null -eq $line) {
                    break
                }
                $lastLine = $line
            }
        }
        catch {
            $decoderFailure = $_.Exception -is [Text.DecoderFallbackException] -or
                $_.Exception.InnerException -is [Text.DecoderFallbackException]
            if ($decoderFailure) {
                throw 'Active process journal contains invalid UTF-8.'
            }
            throw
        }
    }
    finally {
        if ($null -ne $reader) {
            $reader.Dispose()
        }
        $stream.Dispose()
    }
    if ([string]::IsNullOrWhiteSpace($lastLine)) {
        throw 'Active process journal has bytes but no complete event.'
    }
    try {
        $lastEvent = $lastLine | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw 'Active process journal has an invalid terminal event.'
    }
    Assert-MvpProcessJournalResumeEventSchema -Event $lastEvent
    $sequenceProperty = $lastEvent.PSObject.Properties['sequence']
    $hashProperty = $lastEvent.PSObject.Properties['event_sha256']
    if ($null -eq $sequenceProperty -or $null -eq $hashProperty) {
        throw 'Active process journal terminal event is missing its sequence or hash.'
    }
    try {
        $sequence = [Int64]$sequenceProperty.Value
    }
    catch {
        throw 'Active process journal terminal event has an invalid sequence.'
    }
    $eventSha256 = [string]$hashProperty.Value
    if ($sequence -lt 1 -or $sequence -ge [Int32]::MaxValue) {
        throw 'Active process journal terminal event has an out-of-range sequence.'
    }
    if ($eventSha256 -notmatch '^[0-9a-f]{64}$') {
        throw 'Active process journal terminal event has an invalid event hash.'
    }
    return [pscustomobject]@{
        next_sequence = [int]($sequence + 1)
        previous_event_sha256 = $eventSha256
        journal_offset_bytes = $length
    }
}

function Get-MvpProcessJournalCurrentSegment {
    param(
        [Parameter(Mandatory)][string]$LogRoot,
        [Parameter(Mandatory)][ValidateRange(1, 64)][int]$MaximumArchivedSegments
    )

    $highestArchivedSegment = -1
    $archiveCount = 0
    foreach ($path in Get-ChildItem -LiteralPath $LogRoot -Filter 'process-execution-journal.*.jsonl' -File) {
        $match = [Text.RegularExpressions.Regex]::Match($path.Name, '^process-execution-journal\.(\d{6})\.jsonl$')
        if (-not $match.Success) {
            continue
        }
        $archiveCount++
        if ($archiveCount -gt $MaximumArchivedSegments) {
            throw "Process journal archive count exceeds its retention limit of $MaximumArchivedSegments."
        }
        $highestArchivedSegment = [Math]::Max($highestArchivedSegment, [int]$match.Groups[1].Value)
    }
    return $highestArchivedSegment + 1
}

function New-MvpProcessJournalState {
    param(
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][ValidateRange(1024, [Int64]::MaxValue)][Int64]$MaximumJournalBytes,
        [Parameter(Mandatory)][ValidateRange(1, 64)][int]$MaximumArchivedSegments
    )

    $logRoot = Join-MvpProcessJournalPath -Root $StageRoot -ChildPath 'logs'
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
    $journalSegment = Get-MvpProcessJournalCurrentSegment `
        -LogRoot $logRoot `
        -MaximumArchivedSegments $MaximumArchivedSegments
    $journalPath = Join-MvpProcessJournalPath -Root $logRoot -ChildPath 'process-execution-journal.jsonl'
    $resumeCursor = Get-MvpProcessJournalResumeCursor `
        -JournalPath $journalPath `
        -MaximumJournalBytes $MaximumJournalBytes
    return [pscustomobject]@{
        next_sequence = $resumeCursor.next_sequence
        previous_event_sha256 = $resumeCursor.previous_event_sha256
        journal_segment = $journalSegment
        journal_offset_bytes = $resumeCursor.journal_offset_bytes
        maximum_journal_bytes = $MaximumJournalBytes
        maximum_archived_segments = $MaximumArchivedSegments
    }
}

function Get-MvpProcessJournalTail {
    param(
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][ValidateRange(0, [Int32]::MaxValue)][int]$JournalSegment,
        [Parameter(Mandatory)][ValidateRange(0, [Int64]::MaxValue)][Int64]$JournalOffsetBytes,
        [Parameter(Mandatory)][ValidateRange(1024, [Int64]::MaxValue)][Int64]$MaximumJournalBytes,
        [Parameter(Mandatory)][ValidateRange(1, 64)][int]$MaximumArchivedSegments
    )

    $logRoot = Join-MvpProcessJournalPath -Root $StageRoot -ChildPath 'logs'
    $currentSegment = Get-MvpProcessJournalCurrentSegment `
        -LogRoot $logRoot `
        -MaximumArchivedSegments $MaximumArchivedSegments
    $path = if ($JournalSegment -lt $currentSegment) {
        Join-MvpProcessJournalPath -Root $logRoot -ChildPath ('process-execution-journal.{0:D6}.jsonl' -f $JournalSegment)
    }
    elseif ($JournalSegment -eq $currentSegment) {
        Join-MvpProcessJournalPath -Root $logRoot -ChildPath 'process-execution-journal.jsonl'
    }
    else {
        throw "Process journal segment '$JournalSegment' has not been created."
    }
    if (-not [IO.File]::Exists($path)) {
        throw "Process journal segment '$JournalSegment' is unavailable."
    }
    $stream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
        if ($JournalOffsetBytes -gt $stream.Length) {
            throw "Process journal offset '$JournalOffsetBytes' exceeds the '$JournalSegment' segment length."
        }
        $remaining = [Int64]($stream.Length - $JournalOffsetBytes)
        if ($remaining -gt $MaximumJournalBytes) {
            throw "Process journal segment '$JournalSegment' exceeds its byte budget."
        }
        if ($remaining -gt [Int32]::MaxValue) {
            throw "Process journal segment '$JournalSegment' exceeds the tail materialization limit."
        }
        [void]$stream.Seek($JournalOffsetBytes, [IO.SeekOrigin]::Begin)
        $contentBytes = [byte[]]::new([int]$remaining)
        $bytesRead = 0
        while ($bytesRead -lt $contentBytes.Length) {
            $read = $stream.Read($contentBytes, $bytesRead, $contentBytes.Length - $bytesRead)
            if ($read -eq 0) {
                throw "Process journal segment '$JournalSegment' changed while its tail was being read."
            }
            $bytesRead += $read
        }
        return [pscustomobject]@{
            journal_segment = $JournalSegment
            journal_offset_bytes = $JournalOffsetBytes
            next_journal_offset_bytes = [Int64]($JournalOffsetBytes + $contentBytes.LongLength)
            content = $script:MvpProcessJournalStrictUtf8.GetString($contentBytes)
        }
    }
    finally {
        $stream.Dispose()
    }
}

function Write-MvpProcessJournalEntry {
    param(
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][ValidateSet('started', 'heartbeat', 'progress', 'cancellation_requested', 'supervisor_failure', 'exit', 'crash', 'cleanup', 'terminal')][string]$EventKind,
        [Parameter(Mandatory)]$LaunchIdentity,
        [Parameter(Mandatory)]$JournalState,
        [Parameter(Mandatory)][int]$ProcessId,
        [Parameter(Mandatory)][string]$ProcessStartedAtUtc,
        [AllowNull()][Nullable[Int64]]$ElapsedMilliseconds,
        [AllowNull()][string]$ProgressRecordedAtUtc,
        [AllowNull()][string]$ProgressName,
        [AllowNull()][string]$StartedAtUtc,
        [AllowNull()][string]$EndedAtUtc,
        [AllowNull()][Nullable[int]]$ExitCode,
        [AllowNull()][Nullable[bool]]$RootProcessExited,
        [AllowNull()][Nullable[bool]]$JobEmpty,
        [AllowNull()][string]$CleanedAtUtc,
        [AllowNull()][ValidateSet('exited', 'timed_out', 'cancelled', 'crashed', 'supervisor_failed', 'cleanup_failed')][string]$Outcome,
        [AllowNull()][string]$CancellationRequestedAtUtc,
        [AllowNull()][string]$CancellationReason,
        [AllowNull()][string]$FailureKind,
        [AllowNull()][string]$FailureMessageSha256,
        [AllowNull()][string]$CrashKind,
        [AllowNull()]$StdoutCapture,
        [AllowNull()]$StderrCapture,
        [AllowNull()][string]$StdoutTailPath,
        [AllowNull()][string]$StderrTailPath,
        [AllowNull()]$RetainedOutputBudget,
        [AllowNull()]$ArtifactBudgetMeasurement
    )

    $logRoot = Join-MvpProcessJournalPath -Root $StageRoot -ChildPath 'logs'
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
    $journalPath = Join-MvpProcessJournalPath -Root $logRoot -ChildPath 'process-execution-journal.jsonl'
    $journalSegment = [int]$JournalState.journal_segment
    $journalOffsetBytes = [Int64]$JournalState.journal_offset_bytes
    $maximumJournalBytes = [Int64]$JournalState.maximum_journal_bytes
    $maximumArchivedSegments = [int]$JournalState.maximum_archived_segments
    $sequence = [int]$JournalState.next_sequence
    if ($sequence -lt 1) {
        throw "Process journal event '$Phase' has an invalid next sequence '$sequence'."
    }
    $entry = [ordered]@{
        schema_version = $script:MvpProcessJournalEventSchemaVersion
        event_stream_kind = $script:MvpProcessJournalEventStreamKind
        event_kind = $EventKind
        run_id = $RunId
        phase = $Phase
        sequence = $sequence
        previous_event_sha256 = $JournalState.previous_event_sha256
        recorded_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
        executable_sha256 = $LaunchIdentity.executable_sha256
        working_directory = $LaunchIdentity.working_directory
        arguments_sha256 = $LaunchIdentity.arguments_sha256
        environment_sha256 = $LaunchIdentity.environment_sha256
        environment_policy_schema_version = $LaunchIdentity.environment_policy_schema_version
        environment_policy_kind = $LaunchIdentity.environment_policy_kind
        environment_policy_id = $LaunchIdentity.environment_policy_id
            environment_variables = $LaunchIdentity.environment_variables
        resource_limits = $LaunchIdentity.resource_limits
        artifact_budget_schema_version = $LaunchIdentity.artifact_budget_schema_version
        artifact_budget_kind = $LaunchIdentity.artifact_budget_kind
        artifact_budget_id = $LaunchIdentity.artifact_budget_id
        artifact_budget_baseline_sha256 = $LaunchIdentity.artifact_budget_baseline_sha256
        artifact_budget_maximum_additional_bytes = $LaunchIdentity.artifact_budget_maximum_additional_bytes
        artifact_budget_maximum_additional_file_count = $LaunchIdentity.artifact_budget_maximum_additional_file_count
        qualification_context_id = $LaunchIdentity.qualification_context_id
        process_id = $ProcessId
        process_started_at_utc = $ProcessStartedAtUtc
        journal_segment = $journalSegment
        journal_offset_bytes = $journalOffsetBytes
    }
    if ($EventKind -eq 'started') {
        $entry.arguments = $LaunchIdentity.arguments
        $entry.qualification_context = $LaunchIdentity.qualification_context
    }
    elseif ($EventKind -eq 'heartbeat') {
        if ($null -eq $ElapsedMilliseconds -or $ElapsedMilliseconds -le 0) {
            throw "Heartbeat process journal event '$Phase' is missing elapsed time."
        }
        $entry.elapsed_milliseconds = [Int64]$ElapsedMilliseconds
    }
    elseif ($EventKind -eq 'progress') {
        if ($null -eq $ElapsedMilliseconds -or $ElapsedMilliseconds -le 0 -or
            [string]::IsNullOrWhiteSpace($ProgressRecordedAtUtc) -or
            [string]::IsNullOrWhiteSpace($ProgressName)) {
            throw "Progress process journal event '$Phase' is missing progress evidence."
        }
        $entry.elapsed_milliseconds = [Int64]$ElapsedMilliseconds
        $entry.progress_recorded_at_utc = $ProgressRecordedAtUtc
        $entry.progress_name = $ProgressName
    }
    elseif ($EventKind -eq 'cancellation_requested') {
        if ([string]::IsNullOrWhiteSpace($CancellationRequestedAtUtc) -or
            [string]::IsNullOrWhiteSpace($CancellationReason)) {
            throw "Cancellation process journal event '$Phase' is missing cancellation evidence."
        }
        $entry.cancellation_requested_at_utc = $CancellationRequestedAtUtc
        $entry.cancellation_reason = $CancellationReason
    }
    elseif ($EventKind -eq 'supervisor_failure') {
        if ([string]::IsNullOrWhiteSpace($FailureKind) -or
            [string]::IsNullOrWhiteSpace($FailureMessageSha256)) {
            throw "Supervisor failure journal event '$Phase' is missing failure evidence."
        }
        $entry.failure_kind = $FailureKind
        $entry.failure_message_sha256 = $FailureMessageSha256
    }
    elseif ($EventKind -eq 'exit') {
        if ([string]::IsNullOrWhiteSpace($EndedAtUtc) -or $null -eq $RootProcessExited) {
            throw "Exit process journal event '$Phase' is missing root-process exit evidence."
        }
        $entry.ended_at_utc = $EndedAtUtc
        $entry.root_process_exited = [bool]$RootProcessExited
        $entry.exit_code = $ExitCode
    }
    elseif ($EventKind -eq 'crash') {
        if ([string]::IsNullOrWhiteSpace($CrashKind) -or $null -eq $ExitCode) {
            throw "Crash process journal event '$Phase' is missing crash evidence."
        }
        $entry.crash_kind = $CrashKind
        $entry.exit_code = [int]$ExitCode
    }
    elseif ($EventKind -eq 'cleanup') {
        if ([string]::IsNullOrWhiteSpace($CleanedAtUtc) -or $null -eq $JobEmpty -or
            [string]::IsNullOrWhiteSpace($Outcome)) {
            throw "Cleanup process journal event '$Phase' is missing cleanup evidence."
        }
        $entry.cleaned_at_utc = $CleanedAtUtc
        $entry.job_empty = [bool]$JobEmpty
        $entry.outcome = $Outcome
    }
    elseif ($EventKind -eq 'terminal') {
        if ([string]::IsNullOrWhiteSpace($StartedAtUtc) -or [string]::IsNullOrWhiteSpace($EndedAtUtc) -or
            [string]::IsNullOrWhiteSpace($Outcome) -or $null -eq $StdoutCapture -or $null -eq $StderrCapture -or
            [string]::IsNullOrWhiteSpace($StdoutTailPath) -or [string]::IsNullOrWhiteSpace($StderrTailPath) -or
            $null -eq $RetainedOutputBudget -or $null -eq $ArtifactBudgetMeasurement) {
            throw "Terminal process journal event '$Phase' is missing required terminal evidence."
        }
        $entry.started_at_utc = $StartedAtUtc
        $entry.ended_at_utc = $EndedAtUtc
        $entry.exit_code = $ExitCode
        $entry.outcome = $Outcome
        $entry.stdout = [ordered]@{
            total_bytes = [Int64]$StdoutCapture.TotalBytes
            retained_bytes = [Int64]$StdoutCapture.RetainedBytes
            dropped_bytes = [Int64]$StdoutCapture.DroppedBytes
            tail_file_name = [IO.Path]::GetFileName($StdoutTailPath)
            tail_capacity_bytes = [Int64]$StdoutCapture.MaximumTailBytes
            tail_retained_bytes = [Int64]$StdoutCapture.TailRetainedBytes
        }
        $entry.stderr = [ordered]@{
            total_bytes = [Int64]$StderrCapture.TotalBytes
            retained_bytes = [Int64]$StderrCapture.RetainedBytes
            dropped_bytes = [Int64]$StderrCapture.DroppedBytes
            tail_file_name = [IO.Path]::GetFileName($StderrTailPath)
            tail_capacity_bytes = [Int64]$StderrCapture.MaximumTailBytes
            tail_retained_bytes = [Int64]$StderrCapture.TailRetainedBytes
        }
        $entry.retained_output_budget = [ordered]@{
            maximum_bytes = [Int64]$RetainedOutputBudget.MaximumBytes
            retained_bytes = [Int64]$StdoutCapture.RetainedBytes + [Int64]$StderrCapture.RetainedBytes
            dropped_bytes = [Int64]$StdoutCapture.DroppedBytes + [Int64]$StderrCapture.DroppedBytes
            remaining_bytes = [Int64]$RetainedOutputBudget.RemainingBytes
        }
        $entry.artifact_budget = [ordered]@{
            schema_version = [int]$ArtifactBudgetMeasurement.schema_version
            measurement_kind = [string]$ArtifactBudgetMeasurement.measurement_kind
            policy_id = [string]$ArtifactBudgetMeasurement.policy_id
            measured_at_utc = [string]$ArtifactBudgetMeasurement.measured_at_utc
            additional_bytes = [Int64]$ArtifactBudgetMeasurement.additional_bytes
            additional_file_count = [int]$ArtifactBudgetMeasurement.additional_file_count
            current_bytes = [Int64]$ArtifactBudgetMeasurement.current_bytes
            current_file_count = [int]$ArtifactBudgetMeasurement.current_file_count
            remaining_bytes = [Int64]$ArtifactBudgetMeasurement.remaining_bytes
            remaining_file_count = [int]$ArtifactBudgetMeasurement.remaining_file_count
            within_budget = [bool]$ArtifactBudgetMeasurement.within_budget
        }
        if (-not [string]::IsNullOrWhiteSpace($ProgressName)) {
            if ([string]::IsNullOrWhiteSpace($ProgressRecordedAtUtc)) {
                throw "Terminal process journal event '$Phase' is missing final progress time."
            }
            $entry.phase_progress = [ordered]@{
                last_name = $ProgressName
                recorded_at_utc = $ProgressRecordedAtUtc
            }
        }
        if ($Outcome -eq 'cancelled') {
            if ([string]::IsNullOrWhiteSpace($CancellationRequestedAtUtc) -or
                [string]::IsNullOrWhiteSpace($CancellationReason)) {
                throw "Cancelled terminal process journal event '$Phase' is missing cancellation evidence."
            }
            $entry.cancellation = [ordered]@{
                requested_at_utc = $CancellationRequestedAtUtc
                reason = $CancellationReason
            }
        }
        if ($Outcome -eq 'supervisor_failed') {
            if ([string]::IsNullOrWhiteSpace($FailureKind) -or
                [string]::IsNullOrWhiteSpace($FailureMessageSha256)) {
                throw "Failed terminal process journal event '$Phase' is missing supervisor failure evidence."
            }
            $entry.supervisor_failure = [ordered]@{
                kind = $FailureKind
                message_sha256 = $FailureMessageSha256
            }
        }
        if ($Outcome -eq 'crashed') {
            if ([string]::IsNullOrWhiteSpace($CrashKind) -or $null -eq $ExitCode) {
                throw "Crashed terminal process journal event '$Phase' is missing crash evidence."
            }
            $entry.crash = [ordered]@{
                kind = $CrashKind
                exit_code = [int]$ExitCode
            }
        }
    }
    $payloadJson = $entry | ConvertTo-Json -Compress
    $eventSha256 = Get-MvpProcessJournalStringSha256 -Text $payloadJson
    $entry.event_sha256 = $eventSha256
    $entryLine = ($entry | ConvertTo-Json -Compress) + [Environment]::NewLine
    $entryByteCount = $script:MvpProcessJournalUtf8.GetByteCount($entryLine)
    if ($entryByteCount -gt $maximumJournalBytes) {
        throw "Process journal event '$Phase' exceeds its byte budget of $maximumJournalBytes bytes."
    }
    $prunedArchivePath = $null
    if ($journalOffsetBytes -gt 0 -and $journalOffsetBytes + $entryByteCount -gt $maximumJournalBytes) {
        $archiveName = 'process-execution-journal.{0:D6}.jsonl' -f $journalSegment
        $archivePath = Join-MvpProcessJournalPath -Root $logRoot -ChildPath $archiveName
        if ([IO.File]::Exists($archivePath)) {
            throw "Process journal segment '$archiveName' already exists."
        }
        [IO.File]::Move($journalPath, $archivePath)
        $journalSegment++
        $journalOffsetBytes = 0
        $entry.journal_segment = $journalSegment
        $entry.journal_offset_bytes = $journalOffsetBytes
        $archiveCount = 0
        $oldestArchivedSegment = [int]::MaxValue
        $oldestArchivedPath = $null
        $logDirectory = [IO.DirectoryInfo]::new($logRoot)
        foreach ($archiveFile in $logDirectory.EnumerateFiles(
                'process-execution-journal.*.jsonl',
                [IO.SearchOption]::TopDirectoryOnly)) {
            $match = [Text.RegularExpressions.Regex]::Match(
                $archiveFile.Name,
                '^process-execution-journal\.(\d{6})\.jsonl$')
            if (-not $match.Success) {
                continue
            }
            $archiveCount++
            $archivedSegment = [int]$match.Groups[1].Value
            if ($archivedSegment -lt $oldestArchivedSegment) {
                $oldestArchivedSegment = $archivedSegment
                $oldestArchivedPath = $archiveFile.FullName
            }
        }
        if ($archiveCount -gt ($maximumArchivedSegments + 1)) {
            throw "Process journal archive count exceeds the single-rotation bound of $($maximumArchivedSegments + 1)."
        }
        if ($archiveCount -gt $maximumArchivedSegments) {
            if ([string]::IsNullOrWhiteSpace($oldestArchivedPath)) {
                throw 'Process journal retention could not resolve the oldest archived segment.'
            }
            $archiveSha256 = Get-MvpProcessJournalFileSha256 -Path $oldestArchivedPath
            $retentionManifest = ('{0}:{1}' -f $oldestArchivedSegment, $archiveSha256) + "`n"
            $entry.retention = [ordered]@{
                maximum_archived_segments = $maximumArchivedSegments
                pruned_from_segment = $oldestArchivedSegment
                pruned_through_segment = $oldestArchivedSegment
                pruned_segment_count = 1
                pruned_segments_sha256 = Get-MvpProcessJournalStringSha256 `
                    -Text $retentionManifest
            }
            $prunedArchivePath = $oldestArchivedPath
        }
        $entry.Remove('event_sha256') | Out-Null
        $payloadJson = $entry | ConvertTo-Json -Compress
        $eventSha256 = Get-MvpProcessJournalStringSha256 -Text $payloadJson
        $entry.event_sha256 = $eventSha256
        $entryLine = ($entry | ConvertTo-Json -Compress) + [Environment]::NewLine
        $entryByteCount = $script:MvpProcessJournalUtf8.GetByteCount($entryLine)
        if ($entryByteCount -gt $maximumJournalBytes) {
            throw "Process journal event '$Phase' exceeds its byte budget of $maximumJournalBytes bytes."
        }
    }
    [IO.File]::AppendAllText(
        $journalPath,
        $entryLine,
        $script:MvpProcessJournalUtf8
    )
    $JournalState.next_sequence = $sequence + 1
    $JournalState.previous_event_sha256 = $eventSha256
    $JournalState.journal_segment = $journalSegment
    $JournalState.journal_offset_bytes = $journalOffsetBytes + $entryByteCount
    if ($null -ne $prunedArchivePath) {
        [IO.File]::Delete($prunedArchivePath)
    }
}

Export-ModuleMember -Function @(
    'ConvertTo-MvpProcessJournalLowerHex',
    'Get-MvpProcessJournalSha256',
    'New-MvpProcessJournalState',
    'Get-MvpProcessJournalTail',
    'Write-MvpProcessJournalEntry'
)
