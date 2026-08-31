Set-StrictMode -Version Latest

$script:MvpLivenessDefaultMaximumFileCount = 64
$script:MvpLivenessDefaultMaximumDirectoryDepth = 8
$script:MvpLivenessDefaultMaximumBytesPerFile = 1048576
$script:MvpLivenessDefaultMaximumTotalBytesPerPoll = 4194304
$script:MvpLivenessReadBufferBytes = 8192

function Get-MvpProcessLivenessMilestones {
    param(
        [Parameter(Mandatory)]
        [ValidateSet('runtime_first_frame', 'editor_first_frame', 'editor_project_create', 'editor_authoring')]
        [string]$Scenario
    )

    switch ($Scenario) {
        'runtime_first_frame' {
            return @(
                [pscustomobject]@{ marker = 'runtime_first_frame_presented' },
                [pscustomobject]@{ marker = 'runtime_first_frame_presented' },
                [pscustomobject]@{ marker = 'runtime_process_teardown_complete' }
            )
        }
        'editor_first_frame' {
            return @(
                [pscustomobject]@{ marker = 'editor_first_frame_presented' },
                [pscustomobject]@{ marker = 'editor_first_frame_presented' },
                [pscustomobject]@{ marker = 'editor_process_teardown_complete' }
            )
        }
        'editor_project_create' {
            return @(
                [pscustomobject]@{ marker = 'editor_project_open result=completed' },
                [pscustomobject]@{ marker = 'editor_first_frame_presented' },
                [pscustomobject]@{ marker = 'editor_first_frame_presented' },
                [pscustomobject]@{ marker = 'editor_process_teardown_complete' }
            )
        }
        'editor_authoring' {
            return @(
                [pscustomobject]@{ marker = 'editor_project_save result=started' },
                [pscustomobject]@{ marker = 'editor_project_save result=completed' },
                [pscustomobject]@{ marker = 'editor_authoring_trace result=completed' }
            )
        }
    }
}

function New-MvpProcessLivenessProbeState {
    param(
        [Parameter(Mandatory)][string]$DiagnosticRoot,
        [Parameter(Mandatory)]$ScenarioRegistration,
        [ValidateRange(1, 4096)][int]$MaximumFileCount = $script:MvpLivenessDefaultMaximumFileCount,
        [ValidateRange(0, 64)][int]$MaximumDirectoryDepth = $script:MvpLivenessDefaultMaximumDirectoryDepth,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumBytesPerFile = $script:MvpLivenessDefaultMaximumBytesPerFile,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumTotalBytesPerPoll = $script:MvpLivenessDefaultMaximumTotalBytesPerPoll
    )

    $scenarioId = [string]$ScenarioRegistration.scenario_id
    if ($scenarioId -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        throw "Process liveness scenario_id '$scenarioId' is invalid."
    }
    $livenessScenario = [string]$ScenarioRegistration.liveness_scenario
    $milestoneTemplates = @(Get-MvpProcessLivenessMilestones -Scenario $livenessScenario)
    $progressEventIds = @($ScenarioRegistration.progress_event_ids)
    if ($progressEventIds.Count -ne $milestoneTemplates.Count) {
        throw "Process liveness scenario '$scenarioId' progress_event_ids count differs from its marker count."
    }
    $seenProgressEventIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $milestones = [Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $milestoneTemplates.Count; $index++) {
        $progressEventId = [string]$progressEventIds[$index]
        if ($progressEventId -notmatch '^mvp\.[a-z0-9.-]+\.v[1-9][0-9]*$') {
            throw "Process liveness scenario '$scenarioId' progress event ID '$progressEventId' is invalid."
        }
        if (-not $seenProgressEventIds.Add($progressEventId)) {
            throw "Process liveness scenario '$scenarioId' contains duplicate progress event ID '$progressEventId'."
        }
        $milestones.Add([pscustomobject]@{
                marker = [string]$milestoneTemplates[$index].marker
                progress_event_id = $progressEventId
            }) | Out-Null
    }
    $maximumMarkerLength = ($milestones | ForEach-Object { $_.marker.Length } | Measure-Object -Maximum).Maximum
    return [pscustomobject]@{
        schema_version = 1
        diagnostic_root = [IO.Path]::GetFullPath($DiagnosticRoot)
        scenario_id = $scenarioId
        liveness_scenario = $livenessScenario
        maximum_file_count = $MaximumFileCount
        maximum_directory_depth = $MaximumDirectoryDepth
        maximum_bytes_per_file = $MaximumBytesPerFile
        maximum_total_bytes_per_poll = $MaximumTotalBytesPerPoll
        maximum_marker_length = [int]$maximumMarkerLength
        milestones = $milestones.ToArray()
        file_offsets = [Collections.Generic.Dictionary[string, Int64]]::new([StringComparer]::OrdinalIgnoreCase)
        file_carry = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::OrdinalIgnoreCase)
        emitted_progress = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        read_buffer = [byte[]]::new($script:MvpLivenessReadBufferBytes)
        active_paths_scratch = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        detected_markers_scratch = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        snapshot_paths_scratch = [Collections.Generic.List[string]]::new()
        snapshot_offsets_scratch = [Collections.Generic.List[Int64]]::new()
        snapshot_bytes_scratch = [Collections.Generic.List[Int64]]::new()
        snapshot_carry_scratch = [Collections.Generic.List[string]]::new()
        pending_directories_scratch = [Collections.Generic.Stack[IO.DirectoryInfo]]::new()
        pending_depths_scratch = [Collections.Generic.Stack[int]]::new()
        diagnostic_files_scratch = [Collections.Generic.List[IO.FileInfo]]::new()
        progress_scratch = [Collections.Generic.List[string]]::new()
        stale_paths_scratch = [Collections.Generic.List[string]]::new()
    }
}

function Get-MvpProcessLivenessDiagnosticFiles {
    param([Parameter(Mandatory)]$State)

    $pendingDirectories = $State.pending_directories_scratch
    $pendingDepths = $State.pending_depths_scratch
    $files = $State.diagnostic_files_scratch
    $pendingDirectories.Clear()
    $pendingDepths.Clear()
    $files.Clear()
    if (-not [IO.Directory]::Exists($State.diagnostic_root)) {
        Write-Output -NoEnumerate $files
        return
    }
    $root = [IO.DirectoryInfo]::new($State.diagnostic_root)
    if (($root.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Process liveness diagnostic root '$($State.diagnostic_root)' is a reparse point."
    }

    $pendingDirectories.Push($root)
    $pendingDepths.Push(0)
    $fileCount = 0
    while ($pendingDirectories.Count -gt 0) {
        $currentDirectory = $pendingDirectories.Pop()
        $currentDepth = $pendingDepths.Pop()
        foreach ($entry in $currentDirectory.EnumerateFileSystemInfos()) {
            if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Process liveness diagnostic entry '$($entry.FullName)' is a reparse point."
            }
            if (($entry.Attributes -band [IO.FileAttributes]::Directory) -ne 0) {
                $childDepth = $currentDepth + 1
                if ($childDepth -gt $State.maximum_directory_depth) {
                    throw "Process liveness diagnostics exceed the directory-depth budget of $($State.maximum_directory_depth)."
                }
                $pendingDirectories.Push([IO.DirectoryInfo]$entry)
                $pendingDepths.Push($childDepth)
                continue
            }
            $fileCount++
            if ($fileCount -gt $State.maximum_file_count) {
                throw "Process liveness diagnostics exceed the file-count budget of $($State.maximum_file_count)."
            }
            if ($entry.Extension.Equals('.log', [StringComparison]::OrdinalIgnoreCase)) {
                $files.Add([IO.FileInfo]$entry)
            }
        }
    }
    Write-Output -NoEnumerate $files
}

function Get-MvpProcessLivenessCarry {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][ValidateRange(0, [int]::MaxValue)][int]$MaximumLength
    )

    if ($Text.Length -le $MaximumLength) {
        return $Text
    }
    return $Text.Substring($Text.Length - $MaximumLength)
}

function Read-MvpProcessLivenessProgress {
    param([Parameter(Mandatory)]$State)

    if ($State.emitted_progress.Count -eq $State.milestones.Count) {
        return
    }
    $files = Get-MvpProcessLivenessDiagnosticFiles -State $State
    $activePaths = $State.active_paths_scratch
    $snapshotPaths = $State.snapshot_paths_scratch
    $snapshotOffsets = $State.snapshot_offsets_scratch
    $snapshotBytes = $State.snapshot_bytes_scratch
    $snapshotCarry = $State.snapshot_carry_scratch
    $activePaths.Clear()
    $snapshotPaths.Clear()
    $snapshotOffsets.Clear()
    $snapshotBytes.Clear()
    $snapshotCarry.Clear()
    $totalBytesToRead = [Int64]0
    foreach ($file in $files) {
        $activePaths.Add($file.FullName) | Out-Null
        $length = [Int64]$file.Length
        if ($length -gt $State.maximum_bytes_per_file) {
            throw "Process liveness diagnostic '$($file.FullName)' exceeds the per-file byte budget of $($State.maximum_bytes_per_file)."
        }
        $offset = [Int64]0
        $carry = ''
        if ($State.file_offsets.TryGetValue($file.FullName, [ref]$offset)) {
            if ($length -lt $offset) {
                $offset = 0
            }
            else {
                $null = $State.file_carry.TryGetValue($file.FullName, [ref]$carry)
            }
        }
        $bytesToRead = $length - $offset
        $totalBytesToRead += $bytesToRead
        if ($totalBytesToRead -gt $State.maximum_total_bytes_per_poll) {
            throw "Process liveness diagnostics exceed the aggregate poll byte budget of $($State.maximum_total_bytes_per_poll)."
        }
        $snapshotPaths.Add($file.FullName)
        $snapshotOffsets.Add($offset)
        $snapshotBytes.Add($bytesToRead)
        $snapshotCarry.Add($carry)
    }

    $stalePaths = $State.stale_paths_scratch
    $stalePaths.Clear()
    foreach ($path in $State.file_offsets.Keys) {
        if (-not $activePaths.Contains($path)) {
            $stalePaths.Add($path)
        }
    }
    foreach ($path in $stalePaths) {
        $State.file_offsets.Remove($path) | Out-Null
        $State.file_carry.Remove($path) | Out-Null
    }

    $detectedMarkers = $State.detected_markers_scratch
    $detectedMarkers.Clear()
    $maximumCarryLength = [Math]::Max(0, $State.maximum_marker_length - 1)
    $buffer = $State.read_buffer
    if ($buffer -isnot [byte[]] -or $buffer.Length -ne $script:MvpLivenessReadBufferBytes) {
        throw 'Process liveness probe state has an invalid read buffer.'
    }
    for ($snapshotIndex = 0; $snapshotIndex -lt $snapshotPaths.Count; $snapshotIndex++) {
        $snapshotPath = $snapshotPaths[$snapshotIndex]
        $snapshotOffset = [Int64]$snapshotOffsets[$snapshotIndex]
        $bytesToRead = [Int64]$snapshotBytes[$snapshotIndex]
        if ($bytesToRead -eq 0) {
            continue
        }
        $stream = [IO.File]::Open(
            $snapshotPath,
            [IO.FileMode]::Open,
            [IO.FileAccess]::Read,
            [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
        )
        $remaining = $bytesToRead
        $carry = [string]$snapshotCarry[$snapshotIndex]
        try {
            [void]$stream.Seek($snapshotOffset, [IO.SeekOrigin]::Begin)
            while ($remaining -gt 0) {
                $requested = [int][Math]::Min([Int64]$buffer.Length, $remaining)
                $read = $stream.Read($buffer, 0, $requested)
                if ($read -eq 0) {
                    break
                }
                $candidate = $carry + [Text.Encoding]::ASCII.GetString($buffer, 0, $read)
                foreach ($milestone in $State.milestones) {
                    if ($State.emitted_progress.Contains($milestone.progress_event_id)) {
                        continue
                    }
                    if ($candidate.IndexOf($milestone.marker, [StringComparison]::Ordinal) -ge 0) {
                        $detectedMarkers.Add($milestone.marker) | Out-Null
                    }
                }
                $carry = Get-MvpProcessLivenessCarry -Text $candidate -MaximumLength $maximumCarryLength
                $remaining -= $read
            }
            $State.file_offsets[$snapshotPath] = [Int64]$stream.Position
            $State.file_carry[$snapshotPath] = $carry
        }
        finally {
            $stream.Dispose()
        }
    }

    if ($detectedMarkers.Count -eq 0) {
        return
    }
    $progress = $State.progress_scratch
    $progress.Clear()
    foreach ($milestone in $State.milestones) {
        if ($detectedMarkers.Contains($milestone.marker) -and $State.emitted_progress.Add($milestone.progress_event_id)) {
            $progress.Add($milestone.progress_event_id)
        }
    }
    return $progress.ToArray()
}

Export-ModuleMember -Function @(
    'New-MvpProcessLivenessProbeState',
    'Read-MvpProcessLivenessProgress'
)
