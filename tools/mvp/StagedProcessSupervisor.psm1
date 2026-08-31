$ErrorActionPreference = 'Stop'

$outputCaptureModule = Join-Path $PSScriptRoot 'MvpProcessOutputCapture.psm1'
Import-Module $outputCaptureModule -Force -ErrorAction Stop
$processJournalModule = Join-Path $PSScriptRoot 'MvpProcessLifecycleJournal.psm1'
Import-Module $processJournalModule -Force -ErrorAction Stop
$environmentPolicyModule = Join-Path $PSScriptRoot 'MvpProcessEnvironmentPolicy.psm1'
Import-Module $environmentPolicyModule -ErrorAction Stop
$artifactBudgetModule = Join-Path $PSScriptRoot 'MvpRunArtifactBudget.psm1'
Import-Module $artifactBudgetModule -ErrorAction Stop
$qualificationContextModule = Join-Path $PSScriptRoot 'MvpProcessQualificationContext.psm1'
Import-Module $qualificationContextModule -ErrorAction Stop
$renderExtractJobModule = Join-Path $PSScriptRoot 'RenderExtractProcessJob.psm1'
Import-Module $renderExtractJobModule -Force -ErrorAction Stop

$script:MvpSupervisorMaximumTailOutputBytes = 65536
$script:MvpSupervisorMaximumDiagnosticBytesPerFile = 1048576
$script:MvpSupervisorMaximumDiagnosticAggregateBytes = 4194304
$script:MvpSupervisorMaximumJournalBytes = 1048576
$script:MvpSupervisorMaximumArchivedJournalSegments = 64
$script:MvpSupervisorMaximumActiveProcessCount = 8
$script:MvpSupervisorMaximumJobMemoryBytes = 4GB
$script:MvpSupervisorMaximumCpuRatePerTenThousand = 7500
$script:MvpSupervisorMaximumProgressMilestonesPerProbe = 16
$script:MvpSupervisorMaximumProgressMilestonesPerProcess = 256
$script:MvpSupervisorMaximumAdditionalArtifactBytes = 512MB
$script:MvpSupervisorMaximumAdditionalArtifactFileCount = 4096
$script:MvpSupervisorDefaultArtifactBudgetPolicyId = 'mvp.supervised-process-artifacts.v1'
$script:MvpSupervisorMaximumArgumentBytes = 32768

function ConvertTo-MvpSupervisorLowerHex {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    return ConvertTo-MvpProcessJournalLowerHex -Bytes $Bytes
}

function Get-MvpSupervisorSha256 {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    return Get-MvpProcessJournalSha256 -Bytes $Bytes
}

function Get-MvpSupervisorFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpSupervisorLowerHex -Bytes $hasher.ComputeHash($stream)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Join-MvpSupervisorPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$ChildPath
    )

    return [IO.Path]::GetFullPath([IO.Path]::Combine($Root, $ChildPath))
}

function New-MvpSupervisorLaunchIdentity {
    param(
        [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$StartInfo,
        [Parameter(Mandatory)][int]$EnvironmentPolicySchemaVersion,
        [Parameter(Mandatory)][string]$EnvironmentPolicyKind,
        [Parameter(Mandatory)][string]$EnvironmentPolicyId,
        [Parameter(Mandatory)][object[]]$EnvironmentVariables,
        [Parameter(Mandatory)]$ResourceLimits,
        [Parameter(Mandatory)]$ArtifactBudget,
        [Parameter(Mandatory)]$QualificationContext
    )

    if ([string]::IsNullOrWhiteSpace($StartInfo.FileName) -or -not [IO.File]::Exists($StartInfo.FileName)) {
        throw 'The supervised process executable must exist before its launch identity can be recorded.'
    }
    if ([string]::IsNullOrWhiteSpace($StartInfo.WorkingDirectory)) {
        throw 'The supervised process working directory is required for its launch identity.'
    }
    $environmentEntries = [System.Collections.Generic.List[string]]::new()
    foreach ($name in $StartInfo.EnvironmentVariables.Keys) {
        $environmentEntries.Add(([string]$name + '=' + [string]$StartInfo.EnvironmentVariables[$name])) | Out-Null
    }
    $canonicalEnvironment = (@($environmentEntries.ToArray() | Sort-Object) -join "`n")
    $arguments = [string]$StartInfo.Arguments
    $argumentBytes = [Text.UTF8Encoding]::new($false, $true).GetBytes($arguments)
    if ($argumentBytes.LongLength -gt $script:MvpSupervisorMaximumArgumentBytes) {
        throw "The supervised process arguments exceed the $($script:MvpSupervisorMaximumArgumentBytes)-byte journal limit."
    }
    return [pscustomobject]@{
        executable_sha256 = Get-MvpSupervisorFileSha256 -Path $StartInfo.FileName
        working_directory = $StartInfo.WorkingDirectory
        arguments = $arguments
        arguments_sha256 = Get-MvpSupervisorSha256 -Bytes $argumentBytes
        environment_sha256 = Get-MvpSupervisorSha256 -Bytes ([Text.Encoding]::UTF8.GetBytes($canonicalEnvironment))
        environment_policy_schema_version = $EnvironmentPolicySchemaVersion
        environment_policy_kind = $EnvironmentPolicyKind
        environment_policy_id = $EnvironmentPolicyId
        environment_variables = $EnvironmentVariables
        resource_limits = $ResourceLimits
        artifact_budget_schema_version = [int]$ArtifactBudget.schema_version
        artifact_budget_kind = [string]$ArtifactBudget.policy_kind
        artifact_budget_id = [string]$ArtifactBudget.policy_id
        artifact_budget_baseline_sha256 = [string]$ArtifactBudget.baseline_sha256
        artifact_budget_maximum_additional_bytes = [Int64]$ArtifactBudget.maximum_additional_bytes
        artifact_budget_maximum_additional_file_count = [int]$ArtifactBudget.maximum_additional_file_count
        qualification_context = $QualificationContext
        qualification_context_id = [string]$QualificationContext.context_id
    }
}

function Get-MvpSupervisedBoundedTailText {
    param(
        [Parameter(Mandatory)][string]$Path,
        [int]$MaximumCharacters = 2048
    )

    if (-not [IO.File]::Exists($Path)) {
        return '<unavailable>'
    }
    $maximumBytes = ([Int64]$MaximumCharacters) * 4
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
        $bytesToRead = [Math]::Min($stream.Length, $maximumBytes)
        if ($bytesToRead -eq 0) {
            return ''
        }
        [void]$stream.Seek(-$bytesToRead, [IO.SeekOrigin]::End)
        $buffer = New-Object byte[] ([int]$bytesToRead)
        $read = 0
        while ($read -lt $buffer.Length) {
            $current = $stream.Read($buffer, $read, $buffer.Length - $read)
            if ($current -eq 0) {
                break
            }
            $read += $current
        }
        $content = [Text.UTF8Encoding]::new($false, $false).GetString($buffer, 0, $read).Trim()
    }
    finally {
        $stream.Dispose()
    }
    if ($content.Length -le $MaximumCharacters) {
        return $content
    }
    return $content.Substring($content.Length - $MaximumCharacters)
}

function Read-MvpSupervisedBoundedDiagnosticFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumBytes
    )

    if (-not [IO.File]::Exists($Path)) {
        throw "Diagnostic log '$Path' does not exist."
    }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
        $length = $stream.Length
        if ($length -gt $MaximumBytes) {
            throw "Diagnostic log '$Path' exceeds its byte budget of $MaximumBytes bytes."
        }
        $bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw "Diagnostic log '$Path' changed while its bounded snapshot was read."
            }
            $offset += $read
        }
        if ($stream.ReadByte() -ne -1) {
            throw "Diagnostic log '$Path' changed while its bounded snapshot was read."
        }
        return [pscustomobject]@{
            bytes = [Int64]$bytes.Length
            text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
        }
    }
    finally {
        $stream.Dispose()
    }
}

function Get-MvpSupervisedBoundedDiagnosticText {
    param(
        [Parameter(Mandatory)][string[]]$Paths,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumBytesPerFile = $script:MvpSupervisorMaximumDiagnosticBytesPerFile,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumTotalBytes = $script:MvpSupervisorMaximumDiagnosticAggregateBytes
    )

    $text = [Text.StringBuilder]::new()
    $totalBytes = [Int64]0
    $separatorBytes = [Text.UTF8Encoding]::new($false).GetByteCount([Environment]::NewLine)
    foreach ($path in $Paths) {
        $additionalBytes = if ($text.Length -eq 0) { [Int64]0 } else { [Int64]$separatorBytes }
        if ($totalBytes + $additionalBytes -ge $MaximumTotalBytes) {
            throw "Diagnostic logs exceed their aggregate byte budget of $MaximumTotalBytes bytes."
        }
        $availableBytes = $MaximumTotalBytes - $totalBytes - $additionalBytes
        $record = Read-MvpSupervisedBoundedDiagnosticFile `
            -Path $path `
            -MaximumBytes ([Math]::Min($MaximumBytesPerFile, $availableBytes))
        if ($additionalBytes -gt 0) {
            $text.Append([Environment]::NewLine) | Out-Null
        }
        $text.Append($record.text) | Out-Null
        $totalBytes += $additionalBytes + [Int64]$record.bytes
    }
    return $text.ToString()
}

function Get-MvpSupervisedJournalTail {
    param(
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][ValidateRange(0, [Int32]::MaxValue)][int]$JournalSegment,
        [Parameter(Mandatory)][ValidateRange(0, [Int64]::MaxValue)][Int64]$JournalOffsetBytes
    )

    return Get-MvpProcessJournalTail `
        -StageRoot $StageRoot `
        -JournalSegment $JournalSegment `
        -JournalOffsetBytes $JournalOffsetBytes `
        -MaximumJournalBytes $script:MvpSupervisorMaximumJournalBytes `
        -MaximumArchivedSegments $script:MvpSupervisorMaximumArchivedJournalSegments
}
function Start-MvpSupervisedProcess {
    param(
        [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$StartInfo,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$StdoutPath,
        [Parameter(Mandatory)][string]$StderrPath,
        [Parameter(Mandatory)][Int64]$MaximumRetainedLogBytes,
        [ValidateRange(1024, [Int64]::MaxValue)][Int64]$MaximumJournalBytes = $script:MvpSupervisorMaximumJournalBytes,
        [ValidateRange(1, 64)][int]$MaximumArchivedJournalSegments = $script:MvpSupervisorMaximumArchivedJournalSegments,
        [Parameter(Mandatory)]$EnvironmentPolicy,
        [Parameter(Mandatory)]$QualificationContext,
        [AllowNull()]$ArtifactBudget,
        [hashtable]$DeclaredEnvironment = @{},
        [ValidateRange(50, 60000)][int]$HeartbeatIntervalMilliseconds = 5000
    )

    $startedAt = [DateTimeOffset]::UtcNow
    $validatedQualificationContext = Assert-MvpProcessQualificationContext `
        -Context $QualificationContext `
        -ExpectedRunId $RunId
    $appliedEnvironment = Set-MvpProcessEnvironmentPolicy `
        -StartInfo $StartInfo `
        -Policy $EnvironmentPolicy `
        -DeclaredEnvironment $DeclaredEnvironment
    $resourceLimits = [ordered]@{
        maximum_active_process_count = $script:MvpSupervisorMaximumActiveProcessCount
        maximum_job_memory_bytes = [Int64]$script:MvpSupervisorMaximumJobMemoryBytes
        maximum_cpu_rate_per_ten_thousand = $script:MvpSupervisorMaximumCpuRatePerTenThousand
    }
    if ($null -eq $ArtifactBudget) {
        $ArtifactBudget = New-MvpRunArtifactBudget `
            -Root $StageRoot `
            -PolicyId $script:MvpSupervisorDefaultArtifactBudgetPolicyId `
            -MaximumAdditionalBytes $script:MvpSupervisorMaximumAdditionalArtifactBytes `
            -MaximumAdditionalFileCount $script:MvpSupervisorMaximumAdditionalArtifactFileCount
    }
    $artifactBudgetMeasurement = Assert-MvpRunArtifactBudget -Budget $ArtifactBudget
    $launchIdentity = New-MvpSupervisorLaunchIdentity `
        -StartInfo $StartInfo `
        -EnvironmentPolicySchemaVersion $appliedEnvironment.schema_version `
        -EnvironmentPolicyKind $appliedEnvironment.policy_kind `
        -EnvironmentPolicyId $appliedEnvironment.policy_id `
        -EnvironmentVariables $appliedEnvironment.variables `
        -ResourceLimits $resourceLimits `
        -ArtifactBudget $ArtifactBudget `
        -QualificationContext $validatedQualificationContext
    $journalState = New-MvpProcessJournalState `
        -StageRoot $StageRoot `
        -MaximumJournalBytes $MaximumJournalBytes `
        -MaximumArchivedSegments $MaximumArchivedJournalSegments
    $maximumTailLogBytes = [Math]::Min(
        $MaximumRetainedLogBytes,
        [Int64]($script:MvpSupervisorMaximumTailOutputBytes / 2))
    $stdoutTailPath = [IO.Path]::ChangeExtension($StdoutPath, 'tail.log')
    $stderrTailPath = [IO.Path]::ChangeExtension($StderrPath, 'tail.log')
    $retainedOutputBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes $MaximumRetainedLogBytes
    $tailOutputBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes $script:MvpSupervisorMaximumTailOutputBytes
    $processJob = New-RenderExtractBaselineProcessJob `
        -MaximumActiveProcessCount $resourceLimits.maximum_active_process_count `
        -MaximumJobMemoryBytes $resourceLimits.maximum_job_memory_bytes `
        -MaximumCpuRatePerTenThousand $resourceLimits.maximum_cpu_rate_per_ten_thousand
    $assignedProcess = $null
    try {
        $assignedProcess = Start-RenderExtractBaselineSuspendedProcess -Job $processJob -StartInfo $StartInfo
        $stdoutCapture = Start-RenderExtractBaselineBoundedOutputCapture `
            -Reader $assignedProcess.StandardOutput `
            -OutputPath $StdoutPath `
            -MaximumRetainedBytes $MaximumRetainedLogBytes `
            -TailOutputPath $stdoutTailPath `
            -MaximumTailBytes $maximumTailLogBytes `
            -RetainedBudget $retainedOutputBudget `
            -TailBudget $tailOutputBudget
        $stderrCapture = Start-RenderExtractBaselineBoundedOutputCapture `
            -Reader $assignedProcess.StandardError `
            -OutputPath $StderrPath `
            -MaximumRetainedBytes $MaximumRetainedLogBytes `
            -TailOutputPath $stderrTailPath `
            -MaximumTailBytes $maximumTailLogBytes `
            -RetainedBudget $retainedOutputBudget `
            -TailBudget $tailOutputBudget
        $processStartedAtUtc = $assignedProcess.Process.StartTime.ToUniversalTime().ToString('o')
        Write-MvpProcessJournalEntry `
            -StageRoot $StageRoot `
            -RunId $RunId `
            -Phase $Phase `
            -EventKind 'started' `
            -LaunchIdentity $launchIdentity `
            -JournalState $journalState `
            -ProcessId $assignedProcess.Process.Id `
            -ProcessStartedAtUtc $processStartedAtUtc
        Resume-RenderExtractBaselineProcess -Process $assignedProcess
    }
    catch {
        if ($null -ne $assignedProcess) {
            $assignedProcess.Dispose()
        }
        $processJob.Dispose()
        throw
    }
    return [pscustomobject]@{
        process = $assignedProcess.Process
        assigned_process = $assignedProcess
        process_job = $processJob
        stdout_task = $stdoutCapture
        stderr_task = $stderrCapture
        stdout_tail_path = $stdoutTailPath
        stderr_tail_path = $stderrTailPath
        retained_output_budget = $retainedOutputBudget
        tail_output_budget = $tailOutputBudget
        artifact_budget = $ArtifactBudget
        artifact_budget_measurement = $artifactBudgetMeasurement
        maximum_tail_log_bytes = $maximumTailLogBytes
        maximum_tail_output_bytes = [Int64]$script:MvpSupervisorMaximumTailOutputBytes
        resource_limits = $resourceLimits
        staged_product_root = $StageRoot
        run_id = $RunId
        launch_identity = $launchIdentity
        journal_state = $journalState
        phase = $Phase
        started_at = $startedAt
        started_at_utc = $startedAt.ToString('o')
        process_id = $assignedProcess.Process.Id
        process_started_at_utc = $processStartedAtUtc
        heartbeat_interval_milliseconds = $HeartbeatIntervalMilliseconds
        progress_probe_result_scratch = [pscustomobject]@{
            failure = $null
            last_progress_name = $null
            last_progress_recorded_at_utc = $null
        }
        ended_at_utc = $null
    }
}

function Receive-MvpSupervisedProcessStream {
    param(
        [Parameter(Mandatory)]$Task,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        if (-not $Task.Wait(5000)) {
            throw "MVP staging could not drain $Label within 5 seconds after process cleanup."
        }
        return $Task.GetAwaiter().GetResult()
    }
    catch {
        throw "MVP staging could not drain ${Label}: $($_.Exception.Message)"
    }
}

function Write-MvpSupervisedProcessHeartbeat {
    param([Parameter(Mandatory)]$ProcessState)

    $elapsedMilliseconds = [Math]::Max(
        [Int64]1,
        [Int64]([DateTimeOffset]::UtcNow - $ProcessState.started_at).TotalMilliseconds)
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'heartbeat' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -ElapsedMilliseconds $elapsedMilliseconds
}

function Write-MvpSupervisedProcessProgress {
    param(
        [Parameter(Mandatory)]$ProcessState,
        [Parameter(Mandatory)][string]$ProgressName
    )

    $elapsedMilliseconds = [Math]::Max(
        [Int64]1,
        [Int64]([DateTimeOffset]::UtcNow - $ProcessState.started_at).TotalMilliseconds)
    $recordedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'progress' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -ElapsedMilliseconds $elapsedMilliseconds `
        -ProgressRecordedAtUtc $recordedAtUtc `
        -ProgressName $ProgressName
    return $recordedAtUtc
}

function Write-MvpSupervisedProcessFailure {
    param(
        [Parameter(Mandatory)]$ProcessState,
        [Parameter(Mandatory)][ValidateSet('artifact_budget_probe_failed', 'artifact_quota_exceeded', 'cancellation_probe_failed', 'progress_probe_failed', 'progress_stalled')][string]$FailureKind,
        [Parameter(Mandatory)][Exception]$Exception
    )

    $messageSha256 = Get-MvpSupervisorSha256 -Bytes ([Text.Encoding]::UTF8.GetBytes([string]$Exception.Message))
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'supervisor_failure' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -FailureKind $FailureKind `
        -FailureMessageSha256 $messageSha256
    return [pscustomobject]@{
        kind = $FailureKind
        message_sha256 = $messageSha256
    }
}

function Invoke-MvpSupervisedArtifactBudgetProbe {
    param([Parameter(Mandatory)]$ProcessState)

    try {
        $measurement = Measure-MvpRunArtifactBudget `
            -Budget $ProcessState.artifact_budget `
            -ResultScratch $ProcessState.artifact_budget_measurement
        $ProcessState.artifact_budget_measurement = $measurement
        if (-not [bool]$measurement.within_budget) {
            $message = if ([Int64]$measurement.additional_bytes -gt
                [Int64]$ProcessState.artifact_budget.maximum_additional_bytes) {
                "MVP run artifact byte quota exceeded: $($measurement.additional_bytes) > $($ProcessState.artifact_budget.maximum_additional_bytes)."
            }
            else {
                "MVP run artifact file-count quota exceeded: $($measurement.additional_file_count) > $($ProcessState.artifact_budget.maximum_additional_file_count)."
            }
            return Write-MvpSupervisedProcessFailure `
                -ProcessState $ProcessState `
                -FailureKind 'artifact_quota_exceeded' `
                -Exception ([InvalidOperationException]::new($message))
        }
        return $null
    }
    catch {
        return Write-MvpSupervisedProcessFailure `
            -ProcessState $ProcessState `
            -FailureKind 'artifact_budget_probe_failed' `
            -Exception $_.Exception
    }
}

function Invoke-MvpSupervisedProcessProgressProbe {
    param(
        [Parameter(Mandatory)]$ProcessState,
        [Parameter(Mandatory)][scriptblock]$ProgressProbe,
        [AllowNull()][string]$PreviousProgressName,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.HashSet[string]]$EmittedProgressNames
    )

    $result = $ProcessState.progress_probe_result_scratch
    $result.failure = $null
    $result.last_progress_name = $PreviousProgressName
    $result.last_progress_recorded_at_utc = $null
    try {
        $reportedProgress = & $ProgressProbe $ProcessState
        $reportedProgressCount = if ($null -eq $reportedProgress) {
            0
        }
        elseif ($reportedProgress -is [array]) {
            $reportedProgress.Length
        }
        else {
            1
        }
        if ($reportedProgressCount -gt $script:MvpSupervisorMaximumProgressMilestonesPerProbe) {
            throw "The staged process progress probe emitted more than $($script:MvpSupervisorMaximumProgressMilestonesPerProbe) milestones in one poll."
        }
        $lastProgressName = $PreviousProgressName
        $lastProgressRecordedAtUtc = $null
        foreach ($reportedProgressName in $reportedProgress) {
            if ($null -eq $reportedProgressName) {
                continue
            }
            $progressName = ([string]$reportedProgressName).Trim()
            if ($progressName.Length -gt 128) {
                throw 'The staged process progress name exceeds its 128-character budget.'
            }
            if ([string]::IsNullOrWhiteSpace($progressName) -or $EmittedProgressNames.Contains($progressName)) {
                continue
            }
            if ($EmittedProgressNames.Count -ge $script:MvpSupervisorMaximumProgressMilestonesPerProcess) {
                throw "The staged process emitted more than $($script:MvpSupervisorMaximumProgressMilestonesPerProcess) distinct progress milestones."
            }
            $recordedAtUtc = Write-MvpSupervisedProcessProgress `
                -ProcessState $ProcessState `
                -ProgressName $progressName
            $EmittedProgressNames.Add($progressName) | Out-Null
            $lastProgressName = $progressName
            $lastProgressRecordedAtUtc = $recordedAtUtc
        }
        $result.last_progress_name = $lastProgressName
        $result.last_progress_recorded_at_utc = $lastProgressRecordedAtUtc
        return $result
    }
    catch {
        $result.failure = Write-MvpSupervisedProcessFailure `
            -ProcessState $ProcessState `
            -FailureKind 'progress_probe_failed' `
            -Exception $_.Exception
        $result.last_progress_name = $PreviousProgressName
        $result.last_progress_recorded_at_utc = $null
        return $result
    }
}

function Complete-MvpSupervisedProcess {
    param(
        [Parameter(Mandatory)]$ProcessState,
        [Parameter(Mandatory)][int]$TimeoutSeconds,
        [scriptblock]$CancellationProbe,
        [ValidateLength(1, 128)][string]$CancellationReason = 'caller_requested',
        [scriptblock]$ProgressProbe,
        [ValidateRange(0, 600)][int]$ProgressInactivityTimeoutSeconds = 0
    )

    $process = $ProcessState.process
    $processJob = $ProcessState.process_job
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds($TimeoutSeconds)
    $nextHeartbeatAt = [DateTimeOffset]::UtcNow.AddMilliseconds($ProcessState.heartbeat_interval_milliseconds)
    $timedOut = $false
    $cancelled = $false
    $cancellationRequestedAtUtc = $null
    $lastProgressName = $null
    $lastProgressRecordedAtUtc = $null
    $emittedProgressNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $supervisorFailure = $null
    $progressInactivityDeadline = if ($ProgressInactivityTimeoutSeconds -gt 0) {
        [DateTimeOffset]::UtcNow.AddSeconds($ProgressInactivityTimeoutSeconds)
    }
    else {
        $null
    }
    if ($null -ne $progressInactivityDeadline -and $null -eq $ProgressProbe) {
        $supervisorFailure = Write-MvpSupervisedProcessFailure `
            -ProcessState $ProcessState `
            -FailureKind 'progress_probe_failed' `
            -Exception ([ArgumentException]::new('A progress inactivity timeout requires a progress probe.'))
    }
    while (-not $process.HasExited) {
        $now = [DateTimeOffset]::UtcNow
        if ($null -ne $supervisorFailure) {
            break
        }
        if ($now -ge $deadline) {
            $timedOut = $true
            break
        }
        if ($null -ne $progressInactivityDeadline -and $now -ge $progressInactivityDeadline) {
            $supervisorFailure = Write-MvpSupervisedProcessFailure `
                -ProcessState $ProcessState `
                -FailureKind 'progress_stalled' `
                -Exception ([TimeoutException]::new("Product semantic progress did not advance within $ProgressInactivityTimeoutSeconds seconds."))
            break
        }
        $cancellationRequested = $false
        if ($null -ne $CancellationProbe) {
            try {
                $cancellationRequested = [bool](& $CancellationProbe $ProcessState)
            }
            catch {
                $supervisorFailure = Write-MvpSupervisedProcessFailure `
                    -ProcessState $ProcessState `
                    -FailureKind 'cancellation_probe_failed' `
                    -Exception $_.Exception
                break
            }
        }
        if ($cancellationRequested) {
            $cancelled = $true
            $cancellationRequestedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
            Write-MvpProcessJournalEntry `
                -StageRoot $ProcessState.staged_product_root `
                -RunId $ProcessState.run_id `
                -Phase $ProcessState.phase `
                -EventKind 'cancellation_requested' `
                -LaunchIdentity $ProcessState.launch_identity `
                -JournalState $ProcessState.journal_state `
                -ProcessId $ProcessState.process_id `
                -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
                -CancellationRequestedAtUtc $cancellationRequestedAtUtc `
                -CancellationReason $CancellationReason
            break
        }
        $waitMilliseconds = [Math]::Max(
            1,
            [Math]::Min(
                $ProcessState.heartbeat_interval_milliseconds,
                [Int32][Math]::Ceiling(($deadline - $now).TotalMilliseconds)))
        if ($null -ne $progressInactivityDeadline) {
            $waitMilliseconds = [Math]::Max(
                1,
                [Math]::Min(
                    $waitMilliseconds,
                    [Int32][Math]::Ceiling(($progressInactivityDeadline - $now).TotalMilliseconds)))
        }
        if ($process.WaitForExit($waitMilliseconds)) {
            break
        }
        if ([DateTimeOffset]::UtcNow -ge $nextHeartbeatAt) {
            $supervisorFailure = Invoke-MvpSupervisedArtifactBudgetProbe -ProcessState $ProcessState
            if ($null -ne $supervisorFailure) {
                break
            }
            Write-MvpSupervisedProcessHeartbeat -ProcessState $ProcessState
            if ($null -ne $ProgressProbe) {
                $progressProbeResult = Invoke-MvpSupervisedProcessProgressProbe `
                    -ProcessState $ProcessState `
                    -ProgressProbe $ProgressProbe `
                    -PreviousProgressName $lastProgressName `
                    -EmittedProgressNames $emittedProgressNames
                if ($null -ne $progressProbeResult.failure) {
                    $supervisorFailure = $progressProbeResult.failure
                    break
                }
                $lastProgressName = $progressProbeResult.last_progress_name
                if ($null -ne $progressProbeResult.last_progress_recorded_at_utc) {
                    $lastProgressRecordedAtUtc = $progressProbeResult.last_progress_recorded_at_utc
                    if ($ProgressInactivityTimeoutSeconds -gt 0) {
                        $progressInactivityDeadline = [DateTimeOffset]::UtcNow.AddSeconds($ProgressInactivityTimeoutSeconds)
                    }
                }
            }
            $nextHeartbeatAt = [DateTimeOffset]::UtcNow.AddMilliseconds($ProcessState.heartbeat_interval_milliseconds)
        }
    }

    if ($process.HasExited -and $null -eq $supervisorFailure -and $null -ne $ProgressProbe) {
        $progressProbeResult = Invoke-MvpSupervisedProcessProgressProbe `
            -ProcessState $ProcessState `
            -ProgressProbe $ProgressProbe `
            -PreviousProgressName $lastProgressName `
            -EmittedProgressNames $emittedProgressNames
        if ($null -ne $progressProbeResult.failure) {
            $supervisorFailure = $progressProbeResult.failure
        }
        else {
            $lastProgressName = $progressProbeResult.last_progress_name
            if ($null -ne $progressProbeResult.last_progress_recorded_at_utc) {
                $lastProgressRecordedAtUtc = $progressProbeResult.last_progress_recorded_at_utc
            }
        }
    }

    $terminationCleanupErrors = [System.Collections.Generic.List[string]]::new()
    $requiresTermination = $timedOut -or $cancelled -or $null -ne $supervisorFailure
    if ($requiresTermination) {
        try {
            Stop-RenderExtractBaselineProcessJob -Job $processJob -SessionId $ProcessState.phase
        }
        catch {
            $terminationCleanupErrors.Add($_.Exception.Message)
            try {
                if (-not $process.HasExited) {
                    $process.Kill()
                }
            }
            catch {
                $terminationCleanupErrors.Add("Fallback root-process termination failed: $($_.Exception.Message)")
            }
        }
    }
    $processExited = if ($requiresTermination) {
        $process.WaitForExit(5000)
    }
    else {
        $process.WaitForExit()
        $true
    }
    if (-not $processExited) {
        $terminationCleanupErrors.Add('Root process did not exit within 5 seconds after process cleanup.')
    }
    $ProcessState.ended_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    $exitCode = if ($processExited) {
        # CreateProcess supplies the authoritative handle; Process.GetProcessById can lose ExitCode
        # after a short-lived suspended child has already been reaped.
        $nativeExitCode = $ProcessState.assigned_process.TryGetExitCode()
        if ($null -eq $nativeExitCode) {
            throw 'The exited staged product has no readable native process exit code.'
        }
        [int]$nativeExitCode
    }
    else {
        -1
    }
    $journalExitCode = if ($requiresTermination) { $null } else { $exitCode }
    $crashed = -not $timedOut -and -not $cancelled -and $null -eq $supervisorFailure -and $processExited -and $exitCode -ne 0
    $crashKind = if ($crashed) { 'nonzero_exit' } else { $null }
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'exit' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -EndedAtUtc $ProcessState.ended_at_utc `
        -ExitCode $journalExitCode `
        -RootProcessExited $processExited
    if ($crashed) {
        Write-MvpProcessJournalEntry `
            -StageRoot $ProcessState.staged_product_root `
            -RunId $ProcessState.run_id `
            -Phase $ProcessState.phase `
            -EventKind 'crash' `
            -LaunchIdentity $ProcessState.launch_identity `
            -JournalState $ProcessState.journal_state `
            -ProcessId $ProcessState.process_id `
            -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
            -ExitCode $exitCode `
            -CrashKind $crashKind
    }
    $releaseError = $null
    if ($crashed) {
        try {
            Stop-RenderExtractBaselineProcessJob -Job $processJob -SessionId $ProcessState.phase
        }
        catch {
            $releaseError = $_.Exception
        }
    }
    $jobEmpty = $false
    if (-not $requiresTermination -and -not $crashed) {
        try {
            $jobBecameEmpty = Wait-RenderExtractBaselineProcessJobEmpty `
                -Job $processJob `
                -SessionId $ProcessState.phase `
                -TimeoutMilliseconds 5000
            $jobEmpty = $jobBecameEmpty -and (Test-RenderExtractBaselineProcessJobEmpty -Job $processJob -SessionId $ProcessState.phase)
            if (-not $jobEmpty) {
                Stop-RenderExtractBaselineProcessJob -Job $processJob -SessionId $ProcessState.phase
                throw 'Staged process job retained a descendant after its root product exited.'
            }
        }
        catch {
            $releaseError = $_.Exception
        }
    }
    else {
        try {
            $jobEmpty = Wait-RenderExtractBaselineProcessJobEmpty `
                -Job $processJob `
                -SessionId $ProcessState.phase `
                -TimeoutMilliseconds 5000
            if ($jobEmpty) {
                $jobEmpty = Test-RenderExtractBaselineProcessJobEmpty -Job $processJob -SessionId $ProcessState.phase
            }
        }
        catch {
            if ($crashed) {
                if ($null -eq $releaseError) {
                    $releaseError = $_.Exception
                }
            }
            else {
                $terminationCleanupErrors.Add("Process job cleanup verification failed: $($_.Exception.Message)")
            }
            $jobEmpty = $false
        }
        if ($crashed -and -not $jobEmpty -and $null -eq $releaseError) {
            $releaseError = [InvalidOperationException]::new(
                'Staged process job retained a descendant after its root product crashed.'
            )
        }
    }
    $stdout = Receive-MvpSupervisedProcessStream -Task $ProcessState.stdout_task -Label 'stdout'
    $stderr = Receive-MvpSupervisedProcessStream -Task $ProcessState.stderr_task -Label 'stderr'
    if ($null -eq $supervisorFailure) {
        $supervisorFailure = Invoke-MvpSupervisedArtifactBudgetProbe -ProcessState $ProcessState
    }
    $outcome = if ($timedOut) {
        'timed_out'
    }
    elseif ($cancelled) {
        'cancelled'
    }
    elseif ($null -ne $supervisorFailure) {
        'supervisor_failed'
    }
    elseif ($crashed) {
        'crashed'
    }
    elseif ($null -ne $releaseError) {
        'cleanup_failed'
    }
    else {
        'exited'
    }
    $cleanupEndedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'cleanup' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -CleanedAtUtc $cleanupEndedAtUtc `
        -JobEmpty $jobEmpty `
        -Outcome $outcome
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -RunId $ProcessState.run_id `
        -Phase $ProcessState.phase `
        -EventKind 'terminal' `
        -LaunchIdentity $ProcessState.launch_identity `
        -JournalState $ProcessState.journal_state `
        -ProcessId $ProcessState.process_id `
        -ProcessStartedAtUtc $ProcessState.process_started_at_utc `
        -StartedAtUtc $ProcessState.started_at_utc `
        -EndedAtUtc $ProcessState.ended_at_utc `
        -ExitCode $journalExitCode `
        -Outcome $outcome `
        -StdoutCapture $stdout `
        -StderrCapture $stderr `
        -StdoutTailPath $ProcessState.stdout_tail_path `
        -StderrTailPath $ProcessState.stderr_tail_path `
        -RetainedOutputBudget $ProcessState.retained_output_budget `
        -ArtifactBudgetMeasurement $ProcessState.artifact_budget_measurement `
        -CancellationRequestedAtUtc $cancellationRequestedAtUtc `
        -CancellationReason $(if ($cancelled) { $CancellationReason } else { $null }) `
        -FailureKind $(if ($null -ne $supervisorFailure) { $supervisorFailure.kind } else { $null }) `
        -FailureMessageSha256 $(if ($null -ne $supervisorFailure) { $supervisorFailure.message_sha256 } else { $null }) `
        -ProgressRecordedAtUtc $lastProgressRecordedAtUtc `
        -ProgressName $lastProgressName `
        -CrashKind $crashKind
    if ($timedOut) {
        $cleanupDetail = if ($terminationCleanupErrors.Count -eq 0) {
            ''
        }
        else {
            " Cleanup: $($terminationCleanupErrors -join '; ')"
        }
        throw [TimeoutException]::new("Process did not exit within $TimeoutSeconds seconds.$cleanupDetail")
    }
    if ($cancelled) {
        $cleanupDetail = if ($terminationCleanupErrors.Count -eq 0) {
            ''
        }
        else {
            " Cleanup: $($terminationCleanupErrors -join '; ')"
        }
        throw [OperationCanceledException]::new("Process cancellation requested: '$CancellationReason'.$cleanupDetail")
    }
    if ($null -ne $supervisorFailure) {
        $cleanupDetail = if ($terminationCleanupErrors.Count -eq 0) {
            ''
        }
        else {
            " Cleanup: $($terminationCleanupErrors -join '; ')"
        }
        throw [InvalidOperationException]::new("Staged process supervisor failed: '$($supervisorFailure.kind)'.$cleanupDetail")
    }
    if ($null -ne $releaseError) {
        throw [InvalidOperationException]::new(
            "Process exited with code $exitCode. Cleanup: $($releaseError.Message)"
        )
    }
    return $exitCode
}

function Close-MvpSupervisedProcessState {
    param([Parameter(Mandatory)]$ProcessState)

    try {
        if ($null -ne $ProcessState.assigned_process) {
            $ProcessState.assigned_process.Dispose()
        }
        elseif ($null -ne $ProcessState.process) {
            $ProcessState.process.Dispose()
        }
    }
    finally {
        if ($null -ne $ProcessState.process_job) {
            $ProcessState.process_job.Dispose()
        }
    }
}

Export-ModuleMember -Function @(
    'Start-MvpSupervisedProcess',
    'Complete-MvpSupervisedProcess',
    'Close-MvpSupervisedProcessState',
    'Get-MvpSupervisedBoundedTailText',
    'Get-MvpSupervisedBoundedDiagnosticText',
    'Get-MvpSupervisedJournalTail'
)
