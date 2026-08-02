Set-StrictMode -Version Latest

$MvpStagingEvidenceReserveBytes = [Int64](512MB)

function Add-MvpStagingByteCount {
    param(
        [Parameter(Mandatory)][Int64]$Total,
        [Parameter(Mandatory)][Int64]$FileBytes,
        [ValidateRange(1, 2)][int]$CopyCount = 1
    )

    if ($FileBytes -lt 0) {
        throw "MVP staging cannot budget a negative file size '$FileBytes'."
    }
    $next = [decimal]$Total + ([decimal]$FileBytes * $CopyCount)
    if ($next -gt [Int64]::MaxValue) {
        throw 'MVP staging input size exceeds the supported 64-bit byte budget.'
    }
    return [Int64]$next
}

function Get-MvpStagingRequiredBytes {
    param([Parameter(Mandatory)][object[]]$InputCopies)

    [Int64]$inputCopyBytes = 0
    foreach ($inputCopy in $InputCopies) {
        $path = [string]$inputCopy.path
        if ([string]::IsNullOrWhiteSpace($path)) {
            throw 'MVP staging disk budget contains an input without a path.'
        }
        $copyCount = [int]$inputCopy.copy_count
        if ($copyCount -lt 1 -or $copyCount -gt 2) {
            throw "MVP staging disk budget input '$path' has invalid copy count '$copyCount'."
        }
        $file = Get-Item -LiteralPath $path -ErrorAction Stop
        if ($file.PSIsContainer) {
            throw "MVP staging disk budget input '$path' is not a file."
        }
        $inputCopyBytes = Add-MvpStagingByteCount `
            -Total $inputCopyBytes `
            -FileBytes $file.Length `
            -CopyCount $copyCount
    }
    $requiredFreeSpaceBytes = Add-MvpStagingByteCount `
        -Total $inputCopyBytes `
        -FileBytes $MvpStagingEvidenceReserveBytes

    return [ordered]@{
        input_copy_bytes = $inputCopyBytes
        evidence_reserve_bytes = $MvpStagingEvidenceReserveBytes
        required_free_space_bytes = $requiredFreeSpaceBytes
    }
}

function Assert-MvpStagingCapacityValues {
    param(
        [Parameter(Mandatory)][string]$StagingRootPath,
        [Parameter(Mandatory)][Int64]$RequiredFreeSpaceBytes,
        [Parameter(Mandatory)][Int64]$AvailableFreeSpaceBytes
    )

    if ($AvailableFreeSpaceBytes -lt $RequiredFreeSpaceBytes) {
        throw "Staging root '$StagingRootPath' requires at least $RequiredFreeSpaceBytes free bytes for source inputs and evidence but only $AvailableFreeSpaceBytes bytes are available."
    }
}

function Assert-MvpStagingDiskCapacity {
    param(
        [Parameter(Mandatory)][string]$StagingRootPath,
        [Parameter(Mandatory)][Int64]$RequiredFreeSpaceBytes
    )

    $driveRoot = [IO.Path]::GetPathRoot($StagingRootPath)
    if ([string]::IsNullOrWhiteSpace($driveRoot)) {
        throw "Could not resolve the staging drive for '$StagingRootPath'."
    }
    try {
        $drive = [IO.DriveInfo]::new($driveRoot)
        if (-not $drive.IsReady) {
            throw "staging drive '$driveRoot' is not ready"
        }
        [Int64]$availableFreeSpaceBytes = $drive.AvailableFreeSpace
    }
    catch {
        throw "Could not inspect free space for staging root '$StagingRootPath': $($_.Exception.Message)"
    }
    Assert-MvpStagingCapacityValues `
        -StagingRootPath $StagingRootPath `
        -RequiredFreeSpaceBytes $RequiredFreeSpaceBytes `
        -AvailableFreeSpaceBytes $availableFreeSpaceBytes
    return [ordered]@{
        drive_root = $driveRoot
        available_free_space_bytes = $availableFreeSpaceBytes
    }
}

function Assert-MvpInteractiveSessionValues {
    param(
        [Parameter(Mandatory)][bool]$UserInteractive,
        [Parameter(Mandatory)][int]$SessionId
    )

    if (-not $UserInteractive) {
        throw 'MVP windowed product staging requires an interactive Windows user session.'
    }
    if ($SessionId -le 0) {
        throw "MVP windowed product staging cannot run from non-interactive Windows session $SessionId."
    }
}

function Assert-MvpAttachedDisplayCount {
    param([Parameter(Mandatory)][int]$MonitorCount)

    if ($MonitorCount -le 0) {
        throw 'MVP windowed product staging requires at least one attached display.'
    }
}

function Get-MvpInteractiveDesktopPreflight {
    param([Parameter(Mandatory)][bool]$Required)

    if (-not $Required) {
        return [ordered]@{
            required = $false
            user_interactive = [Environment]::UserInteractive
            session_id = $null
            monitor_count = $null
        }
    }
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'MVP windowed product staging requires Windows.'
    }
    $userInteractive = [Environment]::UserInteractive
    $sessionId = [Diagnostics.Process]::GetCurrentProcess().SessionId
    Assert-MvpInteractiveSessionValues `
        -UserInteractive $userInteractive `
        -SessionId $sessionId
    try {
        Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
        $monitorCount = [System.Windows.Forms.SystemInformation]::MonitorCount
    }
    catch {
        throw "Could not inspect the Windows display session for MVP product staging: $($_.Exception.Message)"
    }
    Assert-MvpAttachedDisplayCount -MonitorCount $monitorCount
    return [ordered]@{
        required = $true
        user_interactive = $userInteractive
        session_id = $sessionId
        monitor_count = $monitorCount
    }
}

function Get-MvpStagingPreflight {
    param(
        [Parameter(Mandatory)][string]$StagingRootPath,
        [Parameter(Mandatory)][object[]]$InputCopies,
        [Parameter(Mandatory)][bool]$InteractiveDesktopRequired
    )

    $budget = Get-MvpStagingRequiredBytes -InputCopies $InputCopies
    $disk = Assert-MvpStagingDiskCapacity `
        -StagingRootPath $StagingRootPath `
        -RequiredFreeSpaceBytes $budget.required_free_space_bytes

    return [ordered]@{
        input_copy_bytes = $budget.input_copy_bytes
        evidence_reserve_bytes = $budget.evidence_reserve_bytes
        required_free_space_bytes = $budget.required_free_space_bytes
        available_free_space_bytes = $disk.available_free_space_bytes
        staging_drive_root = $disk.drive_root
        interactive_desktop = Get-MvpInteractiveDesktopPreflight -Required $InteractiveDesktopRequired
    }
}

function Assert-MvpStagingEntryBudget {
    param(
        [Parameter(Mandatory)][object[]]$Entries,
        [Parameter(Mandatory)][Int64]$ExpectedInputCopyBytes
    )

    [Int64]$entryBytes = 0
    foreach ($entry in $Entries) {
        if ($entry -is [Collections.IDictionary]) {
            if (-not $entry.Contains('size_bytes')) {
                throw 'MVP staging final entry is missing size_bytes.'
            }
            $sizeValue = $entry['size_bytes']
        }
        else {
            $sizeProperty = $entry.PSObject.Properties['size_bytes']
            if ($null -eq $sizeProperty) {
                throw 'MVP staging final entry is missing size_bytes.'
            }
            $sizeValue = $sizeProperty.Value
        }
        [Int64]$sizeBytes = 0
        if (-not [Int64]::TryParse([string]$sizeValue, [ref]$sizeBytes) -or $sizeBytes -lt 0) {
            throw "MVP staging final entry has invalid size_bytes '$sizeValue'."
        }
        $entryBytes = Add-MvpStagingByteCount -Total $entryBytes -FileBytes $sizeBytes
    }
    if ($entryBytes -ne $ExpectedInputCopyBytes) {
        throw "MVP staging final entry bytes '$entryBytes' differ from preflight input_copy_bytes '$ExpectedInputCopyBytes'."
    }
    return $entryBytes
}

Export-ModuleMember -Function @('Get-MvpStagingPreflight', 'Assert-MvpStagingEntryBudget')
