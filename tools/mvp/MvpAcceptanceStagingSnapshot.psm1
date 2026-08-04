Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingProjection.psm1') -Force -ErrorAction Stop

if ($null -eq (Get-Variable -Name MvpAcceptanceStagingWriteLeases -Scope Script -ErrorAction SilentlyContinue)) {
    $script:MvpAcceptanceStagingWriteLeases =
        [System.Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
}

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop

function Get-MvpAcceptanceStagingItems {
    return @(
        'staging-manifest.json',
        'startup-summary.json',
        'project',
        'logs',
        'captures',
        'authoring',
        'reopen'
    )
}

function Test-MvpAcceptanceReparsePoint {
    param([Parameter(Mandatory)][System.IO.FileSystemInfo]$Item)

    return [bool]($Item.Attributes -band [System.IO.FileAttributes]::ReparsePoint)
}

function Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints {
    param([Parameter(Mandatory)][string]$StagingRoot)

    $rootItem = Get-Item -LiteralPath $StagingRoot -Force -ErrorAction Stop
    if (-not $rootItem.PSIsContainer) {
        throw "Acceptance staging root '$StagingRoot' is not a directory."
    }
    if (Test-MvpAcceptanceReparsePoint -Item $rootItem) {
        throw "Acceptance staging root '$($rootItem.FullName)' is a reparse point."
    }

    $directories = [System.Collections.Generic.Queue[string]]::new()
    $directories.Enqueue($rootItem.FullName)
    while ($directories.Count -gt 0) {
        $directoryPath = $directories.Dequeue()
        foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
            if (Test-MvpAcceptanceReparsePoint -Item $child) {
                throw "Acceptance staging tree contains reparse point '$($child.FullName)'."
            }
            if ($child.PSIsContainer) {
                $directories.Enqueue($child.FullName)
            }
        }
    }
}

function Assert-MvpAcceptancePublishedTreeFreeOfReparsePoints {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][Microsoft.Win32.SafeHandles.SafeFileHandle]$RootHandle,
        [string[]]$ExcludedPaths = @()
    )

    $rootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($RootHandle)
    Assert-MvpAcceptanceNativeSourceAttributes -Attributes $rootAttributes -Path $StagingRoot
    if (-not (Test-MvpAcceptanceNativeFileAttribute `
        -Attributes $rootAttributes `
        -Expected ([System.IO.FileAttributes]::Directory))) {
        throw "Acceptance publication root '$StagingRoot' is not a directory."
    }

    $childHandles = [System.Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]::new()
    $excluded = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in @($ExcludedPaths)) {
        if (-not [string]::IsNullOrWhiteSpace($path)) {
            $null = $excluded.Add([IO.Path]::GetFullPath($path))
        }
    }
    $directories = [System.Collections.Generic.Queue[string]]::new()
    $directories.Enqueue($StagingRoot)
    try {
        while ($directories.Count -gt 0) {
            $directoryPath = $directories.Dequeue()
            foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
                $childPath = [IO.Path]::GetFullPath($child.FullName)
                if ($excluded.Contains($childPath)) {
                    continue
                }
                $childHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($childPath, $false)
                try {
                    $childAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($childHandle)
                    Assert-MvpAcceptanceNativeSourceAttributes -Attributes $childAttributes -Path $childPath
                    $null = $childHandles.Add($childHandle)
                    $childHandle = $null
                    if (Test-MvpAcceptanceNativeFileAttribute `
                        -Attributes $childAttributes `
                        -Expected ([System.IO.FileAttributes]::Directory)) {
                        $directories.Enqueue($child.FullName)
                    }
                }
                finally {
                    if ($null -ne $childHandle) {
                        $childHandle.Dispose()
                    }
                }
            }
        }
    }
    finally {
        for ($index = $childHandles.Count - 1; $index -ge 0; $index--) {
            $childHandles[$index].Dispose()
        }
    }
}

function Get-MvpAcceptanceNoFollowDirectoryIdentity {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$CompatibleWriteLeaseRoot
    )

    return Get-MvpAcceptanceNativeDirectoryIdentity `
        -Path $Path `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
}

function Open-MvpAcceptanceStagingWriteLease {
    param([Parameter(Mandatory)][string]$SnapshotRoot)

    $rootHandle = $null
    $parentLease = $null
    try {
        $absoluteSnapshotRoot = [IO.Path]::GetFullPath($SnapshotRoot)
        $parent = [IO.Directory]::GetParent($absoluteSnapshotRoot)
        if ($null -eq $parent) {
            throw "Acceptance staging write lease '$absoluteSnapshotRoot' has no parent directory."
        }

        # The root handle owns DELETE access from creation through publication. It shares reads
        # and writes for the builder's child paths, but never delete, so the root cannot be
        # renamed or replaced before the same handle commits the final move.
        $rootHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForStagingWriteLease(
            $absoluteSnapshotRoot)
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($rootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $absoluteSnapshotRoot
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance staging write lease '$absoluteSnapshotRoot' is not a directory."
        }
        $identity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($rootHandle)
        $parentLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $parent.FullName

        $verifiedRoot = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow(
            $absoluteSnapshotRoot,
            $false)
        try {
            $verifiedAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($verifiedRoot)
            Assert-MvpAcceptanceNativeSourceAttributes `
                -Attributes $verifiedAttributes `
                -Path $absoluteSnapshotRoot
            if (-not (Test-MvpAcceptanceNativeFileAttribute `
                -Attributes $verifiedAttributes `
                -Expected ([System.IO.FileAttributes]::Directory)) -or
                [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($verifiedRoot) -ne $identity) {
                throw "Acceptance staging write lease '$absoluteSnapshotRoot' changed while its ancestor lease was being acquired."
            }
        }
        finally {
            $verifiedRoot.Dispose()
        }

        $leaseId = [guid]::NewGuid().ToString('N')
        $lease = [pscustomobject]@{
            lease_id = $leaseId
            root_handle = $rootHandle
            root_path = $absoluteSnapshotRoot
            parent_lease = $parentLease
            root_identity = $identity
        }
        $registeredLease = [pscustomobject]@{
            lease = $lease
            root_handle = $rootHandle
            root_path = $absoluteSnapshotRoot
            root_identity = $identity
        }
        $script:MvpAcceptanceStagingWriteLeases.Add($leaseId, $registeredLease)
        $rootHandle = $null
        $parentLease = $null
        return $lease
    }
    finally {
        if ($null -ne $rootHandle) {
            $rootHandle.Dispose()
        }
        if ($null -ne $parentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $parentLease
        }
    }
}

function Close-MvpAcceptanceStagingWriteLease {
    param($Lease)

    if ($null -eq $Lease) {
        return
    }
    $leaseIdProperty = $Lease.PSObject.Properties['lease_id']
    [object]$registeredLease = $null
    if ($null -ne $leaseIdProperty) {
        if (-not $script:MvpAcceptanceStagingWriteLeases.TryGetValue(
            [string]$leaseIdProperty.Value,
            [ref]$registeredLease) -or
            -not [object]::ReferenceEquals($Lease, $registeredLease.lease)) {
            throw 'Acceptance staging write lease is not the original registered lease.'
        }
        $null = $script:MvpAcceptanceStagingWriteLeases.Remove([string]$leaseIdProperty.Value)
        if ($null -ne $registeredLease.root_handle) {
            $registeredLease.root_handle.Dispose()
            $registeredLease.root_handle = $null
        }
        $Lease.root_handle = $null
    }
    elseif ($null -ne $Lease.root_handle) {
        $Lease.root_handle.Dispose()
        $Lease.root_handle = $null
    }
    if ($null -ne $Lease.parent_lease) {
        Close-MvpAcceptanceNoFollowDirectoryLease -Handles $Lease.parent_lease
        $Lease.parent_lease = $null
    }
}

function Get-MvpAcceptanceRegisteredStagingWriteLease {
    param([Parameter(Mandatory)]$Lease)

    $leaseIdProperty = $Lease.PSObject.Properties['lease_id']
    if ($null -eq $leaseIdProperty -or [string]::IsNullOrWhiteSpace([string]$leaseIdProperty.Value)) {
        throw 'Acceptance publication requires the original registered staging write lease.'
    }
    [object]$registeredLease = $null
    if (-not $script:MvpAcceptanceStagingWriteLeases.TryGetValue(
        [string]$leaseIdProperty.Value,
        [ref]$registeredLease) -or
        -not [object]::ReferenceEquals($Lease, $registeredLease.lease)) {
        throw 'Acceptance publication requires the original registered staging write lease.'
    }
    return $registeredLease
}

function Take-MvpAcceptanceStagingWriteLeaseRootHandle {
    param([Parameter(Mandatory)]$Lease)

    $registeredLease = Get-MvpAcceptanceRegisteredStagingWriteLease -Lease $Lease
    $rootHandle = $registeredLease.root_handle
    if ($null -eq $rootHandle -or $rootHandle.IsClosed -or $rootHandle.IsInvalid) {
        throw 'Acceptance staging write lease no longer owns a root handle for publication.'
    }
    # Keep the original lease registered until Close releases its parent lease. The root handle
    # itself is now exclusively owned by the move operation and cannot be claimed twice.
    $registeredLease.root_handle = $null
    $Lease.root_handle = $null
    return $rootHandle
}

function New-MvpAcceptanceStagingSnapshotLeaseMarker {
    param([Parameter(Mandatory)][string]$DirectoryPath)

    $path = Join-Path `
        $DirectoryPath `
        ('.zircon-mvp-acceptance-lease-' + [guid]::NewGuid().ToString('N') + '.lock')
    return [pscustomobject]@{
        path = $path
        stream = [System.IO.FileStream]::new(
            $path,
            [System.IO.FileMode]::CreateNew,
            [System.IO.FileAccess]::ReadWrite,
            [System.IO.FileShare]::None,
            1,
            [System.IO.FileOptions]::DeleteOnClose)
    }
}

function Open-MvpAcceptanceStagingSnapshotLease {
    param(
        [Parameter(Mandatory)][string]$SnapshotRoot,
        [Parameter(Mandatory)][string]$ExpectedRootIdentity,
        $StagingWriteLease,
        [scriptblock]$BeforeCreateDirectoryMarkerHook
    )

    $rootHandle = $null
    $ownsRootHandle = $true
    $registeredWriteLease = $null
    $writeRootPath = $null
    $parentLease = $null
    $reopenedRootHandle = $null
    $markerStream = $null
    $entryHandles = [System.Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]::new()
    $markerStreams = [System.Collections.Generic.List[System.IO.FileStream]]::new()
    $markerPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    try {
        $absoluteSnapshotRoot = [IO.Path]::GetFullPath($SnapshotRoot)
        $parent = [IO.Directory]::GetParent($absoluteSnapshotRoot)
        if ($null -eq $parent) {
            throw "Acceptance staging snapshot '$absoluteSnapshotRoot' has no parent directory."
        }

        # When the builder still owns the root through a write lease, reuse that registered
        # handle's identity instead of opening the same directory with incompatible sharing.
        # The write lease and the child marker together pin the root through snapshot setup.
        if ($null -ne $StagingWriteLease) {
            $registeredWriteLease = Get-MvpAcceptanceRegisteredStagingWriteLease -Lease $StagingWriteLease
            $writeRootPath = [IO.Path]::GetFullPath([string]$registeredWriteLease.root_path)
            if (-not $writeRootPath.Equals($absoluteSnapshotRoot, [StringComparison]::OrdinalIgnoreCase) -or
                [string]$registeredWriteLease.root_identity -ne $ExpectedRootIdentity -or
                $null -eq $registeredWriteLease.root_handle -or
                $registeredWriteLease.root_handle.IsClosed -or
                $registeredWriteLease.root_handle.IsInvalid) {
                throw 'Acceptance snapshot lease does not match the original registered staging write lease.'
            }
            $rootHandle = $registeredWriteLease.root_handle
            $ownsRootHandle = $false
        }
        else {
            $rootHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($absoluteSnapshotRoot, $false)
        }
        $rootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($rootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $rootAttributes -Path $absoluteSnapshotRoot
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $rootAttributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance staging snapshot '$absoluteSnapshotRoot' is not a directory."
        }
        if ([ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($rootHandle) -ne $ExpectedRootIdentity) {
            throw "Acceptance staging snapshot '$absoluteSnapshotRoot' no longer identifies the published snapshot."
        }

        $parentLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $parent.FullName `
            -CompatibleWriteLeaseRoot $writeRootPath
        $marker = New-MvpAcceptanceStagingSnapshotLeaseMarker -DirectoryPath $absoluteSnapshotRoot
        $markerStream = $marker.stream
        $null = $markerPaths.Add($marker.path)
        $reopenedRootHandle = if ($null -ne $registeredWriteLease) {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($absoluteSnapshotRoot)
        }
        else {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($absoluteSnapshotRoot, $false)
        }
        $reopenedAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($reopenedRootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $reopenedAttributes -Path $absoluteSnapshotRoot
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $reopenedAttributes `
            -Expected ([System.IO.FileAttributes]::Directory)) -or
            [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($reopenedRootHandle) -ne $ExpectedRootIdentity) {
            throw "Acceptance staging snapshot '$absoluteSnapshotRoot' changed while its parent lease was being acquired."
        }
        $reopenedRootHandle.Dispose()
        $reopenedRootHandle = $null

        $null = $markerStreams.Add($markerStream)
        $markerStream = $null

        # Lock the two metadata roots before walking their descendants. The manifest binds all
        # staged entry hashes, while the startup summary has its own independent contract.
        $prelockedPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($relativePath in @('staging-manifest.json', 'startup-summary.json')) {
            $path = Join-Path $absoluteSnapshotRoot $relativePath
            $entryHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($path, $false)
            try {
                $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($entryHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $path
                if (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $attributes `
                    -Expected ([System.IO.FileAttributes]::Directory)) {
                    throw "Acceptance staging snapshot metadata '$path' is not a file."
                }
                $null = $entryHandles.Add($entryHandle)
                $entryHandle = $null
                $null = $prelockedPaths.Add($path)
            }
            finally {
                if ($null -ne $entryHandle) {
                    $entryHandle.Dispose()
                }
            }
        }

        # Each existing file is held read-only and each existing directory receives a held child
        # marker. A recursive delete can otherwise remove ordinary descendants before reaching
        # one root marker, leaving a damaged but same-identity snapshot.
        $directories = [System.Collections.Generic.Queue[string]]::new()
        $directories.Enqueue($absoluteSnapshotRoot)
        while ($directories.Count -gt 0) {
            $directoryPath = $directories.Dequeue()
            foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
                $childPath = [IO.Path]::GetFullPath($child.FullName)
                if ($markerPaths.Contains($childPath) -or $prelockedPaths.Contains($childPath)) {
                    continue
                }
                $entryHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($childPath, $false)
                $reopenedChildHandle = $null
                $childMarkerStream = $null
                try {
                    $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($entryHandle)
                    Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $childPath
                    $isDirectory = Test-MvpAcceptanceNativeFileAttribute `
                        -Attributes $attributes `
                        -Expected ([System.IO.FileAttributes]::Directory)
                    $childIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($entryHandle)
                    $null = $entryHandles.Add($entryHandle)
                    $entryHandle = $null
                    if ($isDirectory) {
                        if ($null -ne $BeforeCreateDirectoryMarkerHook) {
                            & $BeforeCreateDirectoryMarkerHook $childPath
                        }
                        $childMarker = New-MvpAcceptanceStagingSnapshotLeaseMarker -DirectoryPath $childPath
                        $childMarkerStream = $childMarker.stream
                        $reopenedChildHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($childPath, $false)
                        $reopenedAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($reopenedChildHandle)
                        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $reopenedAttributes -Path $childPath
                        if (-not (Test-MvpAcceptanceNativeFileAttribute `
                            -Attributes $reopenedAttributes `
                            -Expected ([System.IO.FileAttributes]::Directory)) -or
                            [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($reopenedChildHandle) -ne $childIdentity) {
                            throw "Acceptance staging snapshot child directory '$childPath' changed while its child marker was being acquired."
                        }
                        $reopenedChildHandle.Dispose()
                        $reopenedChildHandle = $null
                        $null = $markerPaths.Add($childMarker.path)
                        $null = $markerStreams.Add($childMarkerStream)
                        $childMarkerStream = $null
                        $directories.Enqueue($childPath)
                    }
                }
                finally {
                    if ($null -ne $reopenedChildHandle) {
                        $reopenedChildHandle.Dispose()
                    }
                    if ($null -ne $childMarkerStream) {
                        $childMarkerStream.Dispose()
                    }
                    if ($null -ne $entryHandle) {
                        $entryHandle.Dispose()
                    }
                }
            }
        }

        $lease = [pscustomobject]@{
            root_handle = if ($ownsRootHandle) { $rootHandle } else { $null }
            root_path = $absoluteSnapshotRoot
            root_identity = $ExpectedRootIdentity
            held_staging_write_lease = $StagingWriteLease
            parent_lease = $parentLease
            entry_handles = $entryHandles.ToArray()
            marker_streams = $markerStreams.ToArray()
            marker_paths = [string[]]@($markerPaths)
        }
        $rootHandle = $null
        $parentLease = $null
        $entryHandles.Clear()
        $markerStreams.Clear()
        return $lease
    }
    finally {
        if ($null -ne $markerStream) {
            $markerStream.Dispose()
        }
        if ($null -ne $reopenedRootHandle) {
            $reopenedRootHandle.Dispose()
        }
        for ($index = $markerStreams.Count - 1; $index -ge 0; $index--) {
            $markerStreams[$index].Dispose()
        }
        for ($index = $entryHandles.Count - 1; $index -ge 0; $index--) {
            $entryHandles[$index].Dispose()
        }
        if ($ownsRootHandle -and $null -ne $rootHandle) {
            $rootHandle.Dispose()
        }
        if ($null -ne $parentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $parentLease
        }
    }
}

function Close-MvpAcceptanceStagingSnapshotLease {
    param($Lease)

    if ($null -eq $Lease) {
        return
    }
    for ($index = $Lease.marker_streams.Count - 1; $index -ge 0; $index--) {
        $Lease.marker_streams[$index].Dispose()
    }
    for ($index = $Lease.entry_handles.Count - 1; $index -ge 0; $index--) {
        $Lease.entry_handles[$index].Dispose()
    }
    if ($null -ne $Lease.root_handle) {
        $Lease.root_handle.Dispose()
    }
    if ($null -ne $Lease.parent_lease) {
        Close-MvpAcceptanceNoFollowDirectoryLease -Handles $Lease.parent_lease
    }
}

function Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication {
    param(
        [Parameter(Mandatory)]$Lease,
        [Parameter(Mandatory)]$StagingWriteLease
    )

    if ($null -eq $Lease) {
        throw 'Acceptance publication requires a held snapshot lease.'
    }
    $rootPath = [string]$Lease.root_path
    $rootIdentity = [string]$Lease.root_identity
    $registeredWriteLease = Get-MvpAcceptanceRegisteredStagingWriteLease -Lease $StagingWriteLease
    $writeRootHandle = $registeredWriteLease.root_handle
    $writeRootPath = [string]$registeredWriteLease.root_path
    $writeRootIdentity = [string]$registeredWriteLease.root_identity
    if ([string]::IsNullOrWhiteSpace($rootPath) -or [string]::IsNullOrWhiteSpace($rootIdentity)) {
        throw 'Acceptance publication snapshot lease is missing its root identity.'
    }
    if (-not ($writeRootHandle -is [Microsoft.Win32.SafeHandles.SafeFileHandle]) -or
        $writeRootHandle.IsClosed -or
        $writeRootHandle.IsInvalid -or
        [string]::IsNullOrWhiteSpace($writeRootPath) -or
        [string]::IsNullOrWhiteSpace($writeRootIdentity) -or
        -not [string]::Equals(
            [IO.Path]::GetFullPath($writeRootPath),
            [IO.Path]::GetFullPath($rootPath),
            [StringComparison]::OrdinalIgnoreCase) -or
        $writeRootIdentity -ne $rootIdentity) {
        throw 'Acceptance publication snapshot and staging write leases do not identify the same held root.'
    }
    if ($null -ne $Lease.held_staging_write_lease -and
        -not [object]::ReferenceEquals($Lease.held_staging_write_lease, $StagingWriteLease)) {
        throw 'Acceptance publication snapshot lease is not paired with the original registered staging write lease.'
    }

    $writeAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($writeRootHandle)
    Assert-MvpAcceptanceNativeSourceAttributes -Attributes $writeAttributes -Path $rootPath
    if (-not (Test-MvpAcceptanceNativeFileAttribute `
        -Attributes $writeAttributes `
        -Expected ([System.IO.FileAttributes]::Directory)) -or
        [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($writeRootHandle) -ne $rootIdentity) {
        throw "Acceptance publication staging write lease '$rootPath' no longer identifies the frozen root."
    }

    # The write lease remains the rename authority. Callers close this snapshot lease only after
    # their final projection check, then transfer the original delete-owning handle directly into
    # the move. A publication freeze is applied by the evidence publisher before that handoff.
}

function Move-MvpAcceptanceStagingDirectoryNoFollow {
    param(
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath,
        [scriptblock]$BeforeRenameHook,
        [scriptblock]$AfterRenameHook,
        [string]$ExpectedSourceIdentity,
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$SourceHandle,
        [string[]]$ExcludedSourcePaths = @()
    )

    $absoluteSourcePath = [IO.Path]::GetFullPath($SourcePath)
    $absoluteDestinationPath = [IO.Path]::GetFullPath($DestinationPath)
    $sourcePrefix = $absoluteSourcePath.TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar
    $publishedExcludedPaths = [System.Collections.Generic.List[string]]::new()
    foreach ($path in @($ExcludedSourcePaths)) {
        if ([string]::IsNullOrWhiteSpace($path)) {
            continue
        }
        $absoluteExcludedPath = [IO.Path]::GetFullPath($path)
        if (-not $absoluteExcludedPath.StartsWith($sourcePrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Acceptance publication exclusion '$absoluteExcludedPath' escapes source '$absoluteSourcePath'."
        }
        $relativePath = $absoluteExcludedPath.Substring($sourcePrefix.Length)
        $publishedExcludedPaths.Add((Join-Path $absoluteDestinationPath $relativePath)) | Out-Null
    }
    $destinationParentPath = [IO.Path]::GetDirectoryName($absoluteDestinationPath)
    if ([string]::IsNullOrWhiteSpace($destinationParentPath) -or
        [string]::IsNullOrWhiteSpace([IO.Path]::GetFileName($absoluteDestinationPath))) {
        throw "Acceptance publication destination '$DestinationPath' is invalid."
    }

    $sourceHandle = $SourceHandle
    $destinationParentLease = $null
    $destinationHandle = $null
    $renameSucceeded = $false
    try {
        if ($null -eq $sourceHandle) {
            $sourceHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForDelete(
                $absoluteSourcePath)
        }
        $sourceAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($sourceHandle)
        $sourceIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($sourceHandle)
        if (-not [string]::IsNullOrWhiteSpace($ExpectedSourceIdentity) -and
            $sourceIdentity -ne $ExpectedSourceIdentity) {
            throw "Acceptance publication source '$absoluteSourcePath' no longer identifies the staged tree."
        }
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $sourceAttributes -Path $absoluteSourcePath
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $sourceAttributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance publication source '$absoluteSourcePath' is not a directory."
        }
        Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints -StagingRoot $absoluteSourcePath
        # Hold every ancestor through the absolute rename. RenameTo has no relative-root
        # handle, so retaining this chain prevents an ancestor replacement from redirecting it.
        $destinationParentLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $destinationParentPath
        if ($null -ne $BeforeRenameHook) {
            & $BeforeRenameHook
        }
        if (Test-Path -LiteralPath $absoluteDestinationPath) {
            throw "Acceptance publication destination '$absoluteDestinationPath' already exists."
        }
        [ZirconMvpAcceptanceNativeFileSystem]::RenameTo(
            $sourceHandle,
            $absoluteDestinationPath)
        $renameSucceeded = $true
        if ($null -ne $AfterRenameHook) {
            & $AfterRenameHook
        }
        $destinationHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublishedTree($absoluteDestinationPath)
        $destinationAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($destinationHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $destinationAttributes -Path $absoluteDestinationPath
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $destinationAttributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance publication destination '$absoluteDestinationPath' is not a directory."
        }
        if ([ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($destinationHandle) -ne $sourceIdentity) {
            throw "Acceptance publication destination '$absoluteDestinationPath' does not identify the published source tree."
        }
        $destinationHandle.Dispose()
        $destinationHandle = $null
        Assert-MvpAcceptancePublishedTreeFreeOfReparsePoints `
            -StagingRoot $absoluteDestinationPath `
            -RootHandle $sourceHandle `
            -ExcludedPaths $publishedExcludedPaths.ToArray()
    }
    catch {
        $publicationFailure = $_
        if ($renameSucceeded) {
            try {
                if ($null -ne $destinationHandle) {
                    $destinationHandle.Dispose()
                    $destinationHandle = $null
                }
                if ($null -eq $sourceHandle -or
                    [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($sourceHandle) -ne $sourceIdentity) {
                    throw "Acceptance publication destination '$absoluteDestinationPath' no longer identifies the renamed source tree."
                }
                $null = Remove-MvpAcceptanceStagingTree `
                    -Path $absoluteDestinationPath `
                    -RootHandle $sourceHandle
                $sourceHandle.Dispose()
                $sourceHandle = $null
            }
            catch {
                throw "Acceptance publication failed after rename and safe cleanup of '$absoluteDestinationPath' also failed: $($_.Exception.Message)"
            }
        }
        throw $publicationFailure
    }
    finally {
        if ($null -ne $destinationHandle) {
            $destinationHandle.Dispose()
        }
        if ($null -ne $destinationParentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $destinationParentLease
        }
        if ($null -ne $sourceHandle) {
            $sourceHandle.Dispose()
        }
    }
}

function Remove-MvpAcceptanceStagingTree {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$RootHandle,
        [string]$ExpectedRootIdentity
    )

    $handle = $RootHandle
    $ownsHandle = $null -eq $handle
    if ($ownsHandle) {
        $handle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForDelete($Path)
    }
    $removed = $false
    try {
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($handle)
        if (-not [string]::IsNullOrWhiteSpace($ExpectedRootIdentity) -and
            [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($handle) -ne $ExpectedRootIdentity) {
            return $false
        }
        $isDirectory = Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory)
        $isReparsePoint = Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::ReparsePoint)
        if ($isDirectory -and -not $isReparsePoint) {
            foreach ($child in @(Get-ChildItem -LiteralPath $Path -Force -ErrorAction Stop)) {
                $null = Remove-MvpAcceptanceStagingTree -Path $child.FullName
            }
        }
        [ZirconMvpAcceptanceNativeFileSystem]::MarkForDelete($handle)
        $removed = $true
    }
    finally {
        if ($ownsHandle) {
            $handle.Dispose()
        }
    }
    return $removed
}

function Remove-MvpAcceptanceEmptyDirectoryNoFollow {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedIdentity
    )

    $handle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForDelete($Path)
    try {
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($handle)
        $identity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($handle)
        if (-not [string]::IsNullOrWhiteSpace($ExpectedIdentity) -and
            $identity -ne $ExpectedIdentity) {
            throw "Acceptance directory '$Path' no longer identifies the empty directory selected for cleanup."
        }
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $Path
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance directory '$Path' is not a directory."
        }
        if (@(Get-ChildItem -LiteralPath $Path -Force -ErrorAction Stop).Count -ne 0) {
            throw "Acceptance directory '$Path' is not empty."
        }
        [ZirconMvpAcceptanceNativeFileSystem]::MarkForDelete($handle)
    }
    finally {
        $handle.Dispose()
    }
}

function Copy-MvpAcceptanceStagingTree {
    param(
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath,
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$SourceHandle,
        [scriptblock]$BeforeOpenChildHook,
        [scriptblock]$BeforeOpenDestinationChildHook,
        [AllowNull()][System.Collections.Generic.HashSet[string]]$ExcludedSourcePaths,
        $Projection,
        $DestinationWriteLease
    )

    if ($null -eq $SourceHandle) {
        $SourceHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($SourcePath, $true)
    }
        $destinationParentLease = $null
        $destinationHandle = $null
        $heldWriteRoot = $null
        try {
        $destinationParentPath = [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($DestinationPath))
        if ([string]::IsNullOrWhiteSpace($destinationParentPath)) {
            throw "Acceptance snapshot destination '$DestinationPath' has no parent directory."
        }
        $destinationParentIsHeldWriteRoot = $false
        if ($null -ne $DestinationWriteLease) {
            $registeredWriteLease = Get-MvpAcceptanceRegisteredStagingWriteLease -Lease $DestinationWriteLease
            $heldWriteRoot = [IO.Path]::GetFullPath([string]$registeredWriteLease.root_path)
            if ($destinationParentPath.Equals($heldWriteRoot, [StringComparison]::OrdinalIgnoreCase)) {
                $heldRootHandle = $registeredWriteLease.root_handle
                if ($null -eq $heldRootHandle -or $heldRootHandle.IsClosed -or $heldRootHandle.IsInvalid) {
                    throw "Acceptance snapshot destination write lease '$heldWriteRoot' no longer holds its root."
                }
                $heldRootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($heldRootHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $heldRootAttributes -Path $heldWriteRoot
                if (-not (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $heldRootAttributes `
                    -Expected ([System.IO.FileAttributes]::Directory)) -or
                    [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($heldRootHandle) -ne [string]$registeredWriteLease.root_identity) {
                    throw "Acceptance snapshot destination write lease '$heldWriteRoot' no longer identifies the partial root."
                }
                $destinationParentIsHeldWriteRoot = $true
            }
        }
        if (-not $destinationParentIsHeldWriteRoot) {
            $destinationParentLease = Open-MvpAcceptanceNoFollowDirectoryLease `
                -DirectoryPath $destinationParentPath `
                -CompatibleWriteLeaseRoot $heldWriteRoot
        }
        $sourceAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($SourceHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $sourceAttributes -Path $SourcePath
        $isDirectory = Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $sourceAttributes `
            -Expected ([System.IO.FileAttributes]::Directory)
        if ($null -ne $Projection) {
            Add-MvpAcceptanceStagingProjectionSourceEntry `
                -Projection $Projection `
                -SourcePath $SourcePath `
                -DestinationPath $DestinationPath `
                -IsDirectory $isDirectory
        }
        if ($isDirectory) {
            New-Item -ItemType Directory -Path $DestinationPath -ErrorAction Stop | Out-Null
            $destinationHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($DestinationPath)
            $destinationAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($destinationHandle)
            $destinationIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($destinationHandle)
            Assert-MvpAcceptanceNativeSourceAttributes -Attributes $destinationAttributes -Path $DestinationPath
            if (-not (Test-MvpAcceptanceNativeFileAttribute `
                -Attributes $destinationAttributes `
                -Expected ([System.IO.FileAttributes]::Directory))) {
                throw "Acceptance snapshot destination '$DestinationPath' is not a directory."
            }
            if ($null -ne $BeforeOpenDestinationChildHook) {
                & $BeforeOpenDestinationChildHook $DestinationPath
            }
            $reopenedDestinationHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($DestinationPath)
            try {
                $destinationAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($reopenedDestinationHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $destinationAttributes -Path $DestinationPath
                if ([ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($reopenedDestinationHandle) -ne $destinationIdentity) {
                    throw "Acceptance snapshot destination '$DestinationPath' changed after creation."
                }
            }
            finally {
                $reopenedDestinationHandle.Dispose()
            }
            if (@(Get-ChildItem -LiteralPath $DestinationPath -Force -ErrorAction Stop).Count -ne 0) {
                throw "Acceptance snapshot destination '$DestinationPath' is not empty after creation."
            }
            foreach ($child in @(Get-ChildItem -LiteralPath $SourcePath -Force -ErrorAction Stop)) {
                $childPath = [IO.Path]::GetFullPath($child.FullName)
                if ($null -ne $ExcludedSourcePaths -and $ExcludedSourcePaths.Contains($childPath)) {
                    continue
                }
                # The parent directory handle excludes write/delete sharing. A child is opened
                # no-follow after enumeration and remains open until its copy finishes, so a
                # replacement with a reparse point cannot be followed between these steps.
                if ($null -ne $BeforeOpenChildHook) {
                    & $BeforeOpenChildHook $SourcePath $childPath $DestinationPath
                }
                $childHandle = $null
                try {
                    $childHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($childPath, $true)
                    Copy-MvpAcceptanceStagingTree `
                        -SourcePath $childPath `
                        -DestinationPath (Join-Path $DestinationPath $child.Name) `
                        -SourceHandle $childHandle `
                        -BeforeOpenChildHook $BeforeOpenChildHook `
                        -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
                        -ExcludedSourcePaths $ExcludedSourcePaths `
                        -Projection $Projection `
                        -DestinationWriteLease $DestinationWriteLease
                    $childHandle = $null
                }
                finally {
                    if ($null -ne $childHandle) {
                        $childHandle.Dispose()
                    }
                }
            }
            return
        }

        $sourceStream = [System.IO.FileStream]::new($SourceHandle, [System.IO.FileAccess]::Read)
        $SourceHandle = $null
        try {
            $destinationStream = [System.IO.File]::Open(
                $DestinationPath,
                [System.IO.FileMode]::CreateNew,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::None)
            try {
                $sourceStream.CopyTo($destinationStream)
            }
            finally {
                $destinationStream.Dispose()
            }
        }
        finally {
            $sourceStream.Dispose()
        }
    }
    finally {
        if ($null -ne $destinationHandle) {
            $destinationHandle.Dispose()
        }
        if ($null -ne $destinationParentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $destinationParentLease
        }
        if ($null -ne $SourceHandle) {
            $SourceHandle.Dispose()
        }
    }
}

function Copy-MvpAcceptanceStagingItems {
    param(
        [Parameter(Mandatory)][string]$SourceRoot,
        [Parameter(Mandatory)][string]$DestinationRoot,
        [scriptblock]$BeforeOpenChildHook,
        [scriptblock]$BeforeOpenDestinationChildHook,
        [string[]]$ExcludedSourcePaths,
        [switch]$PassThruProjection,
        $DestinationWriteLease
    )

    $sourceRootLease = $null
    $sourceRootHandle = $null
    $projection = New-MvpAcceptanceStagingProjection -Root $DestinationRoot
    $excludedPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in @($ExcludedSourcePaths)) {
        if (-not [string]::IsNullOrWhiteSpace($path)) {
            $null = $excludedPaths.Add([IO.Path]::GetFullPath($path))
        }
    }
    try {
        $sourceRootLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $SourceRoot
        $sourceRootHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($SourceRoot, $false)
        $sourceRootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($sourceRootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $sourceRootAttributes -Path $SourceRoot
        if (-not (Test-MvpAcceptanceNativeFileAttribute -Attributes $sourceRootAttributes -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance staging root '$SourceRoot' is not a directory."
        }

        $resolvedSourceRoot = (Resolve-Path -LiteralPath $SourceRoot -ErrorAction Stop).Path
        foreach ($relativePath in Get-MvpAcceptanceStagingItems) {
            $sourcePath = Join-Path $resolvedSourceRoot $relativePath
            if (Test-Path -LiteralPath $sourcePath) {
                Copy-MvpAcceptanceStagingTree `
                    -SourcePath $sourcePath `
                    -DestinationPath (Join-Path $DestinationRoot $relativePath) `
                    -BeforeOpenChildHook $BeforeOpenChildHook `
            -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
            -ExcludedSourcePaths $excludedPaths `
            -Projection $projection `
            -DestinationWriteLease $DestinationWriteLease
            }
        }
        if ($PassThruProjection) {
            return [pscustomobject]@{
                source_root = $resolvedSourceRoot
                projection = $projection
            }
        }
        return $resolvedSourceRoot
    }
    finally {
        if ($null -ne $sourceRootHandle) {
            $sourceRootHandle.Dispose()
        }
        if ($null -ne $sourceRootLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $sourceRootLease
        }
    }
}

function New-MvpAcceptanceStagingSnapshot {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [switch]$PassThru,
        [scriptblock]$BeforeOpenChildHook,
        [scriptblock]$BeforeOpenDestinationChildHook,
        [scriptblock]$AfterPublishHook
    )

    $absoluteStagingRoot = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StagingRoot)
    $snapshotRoot = $null
    $snapshotRoot = "$absoluteStagingRoot.acceptance-snapshot-$([guid]::NewGuid().ToString('N'))"
    $partialRoot = "$snapshotRoot.partial"
    $sourceRootLease = $null
    $partialWriteLease = $null
    $partialSnapshotLease = $null
    $partialIdentity = $null
    $snapshotIdentity = $null
    $partialPublished = $false
    try {
        $sourceRootLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $absoluteStagingRoot
        Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints -StagingRoot $absoluteStagingRoot
        New-Item -ItemType Directory -Path $partialRoot -ErrorAction Stop | Out-Null
        $partialWriteLease = Open-MvpAcceptanceStagingWriteLease -SnapshotRoot $partialRoot
        $partialIdentity = $partialWriteLease.root_identity
        if (@(Get-ChildItem -LiteralPath $partialRoot -Force -ErrorAction Stop).Count -ne 0) {
            throw "Acceptance staging snapshot partial root '$partialRoot' is not empty after creation."
        }
        $stagingProjection = Copy-MvpAcceptanceStagingItems `
            -SourceRoot $absoluteStagingRoot `
            -DestinationRoot $partialRoot `
            -BeforeOpenChildHook $BeforeOpenChildHook `
            -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
            -DestinationWriteLease $partialWriteLease `
            -PassThruProjection
        $resolvedStagingRoot = [string]$stagingProjection.source_root
        foreach ($requiredFile in @('staging-manifest.json', 'startup-summary.json')) {
            if (-not (Test-Path -LiteralPath (Join-Path $partialRoot $requiredFile) -PathType Leaf)) {
                throw "Acceptance staging snapshot is missing required '$requiredFile'."
            }
        }
        $partialSnapshotLease = Open-MvpAcceptanceStagingSnapshotLease `
            -SnapshotRoot $partialRoot `
            -ExpectedRootIdentity $partialIdentity `
            -StagingWriteLease $partialWriteLease
        Assert-MvpAcceptanceStagingProjection `
            -Root $partialRoot `
            -Projection $stagingProjection.projection `
            -ExcludedPaths $partialSnapshotLease.marker_paths
        Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication `
            -Lease $partialSnapshotLease `
            -StagingWriteLease $partialWriteLease
        Assert-MvpAcceptanceStagingProjection `
            -Root $partialRoot `
            -Projection $stagingProjection.projection `
            -ExcludedPaths $partialSnapshotLease.marker_paths
        Close-MvpAcceptanceStagingSnapshotLease -Lease $partialSnapshotLease
        $partialSnapshotLease = $null
        $partialSourceHandle = Take-MvpAcceptanceStagingWriteLeaseRootHandle `
            -Lease $partialWriteLease
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $partialRoot `
            -DestinationPath $snapshotRoot `
            -ExpectedSourceIdentity $partialIdentity `
            -SourceHandle $partialSourceHandle
        # The move verifies that the published destination retains this identity. Preserve it
        # with the result so later cleanup cannot remove a substituted snapshot directory.
        $snapshotIdentity = $partialIdentity
        # The rename is the commit point. A later diagnostic/test hook must never make finally
        # treat the already-published snapshot as the old partial path.
        $partialPublished = $true
        if ($null -ne $AfterPublishHook) {
            & $AfterPublishHook $partialRoot
        }
    }
    finally {
        if ($null -ne $partialSnapshotLease) {
            Close-MvpAcceptanceStagingSnapshotLease -Lease $partialSnapshotLease
        }
        if ($null -ne $partialWriteLease) {
            Close-MvpAcceptanceStagingWriteLease -Lease $partialWriteLease
        }
        if (-not $partialPublished -and -not [string]::IsNullOrWhiteSpace($partialIdentity)) {
            $null = Remove-MvpAcceptanceStagingTree `
                -Path $partialRoot `
                -ExpectedRootIdentity $partialIdentity
        }
        if ($null -ne $sourceRootLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $sourceRootLease
        }
    }
    if ($PassThru) {
        return [pscustomobject]@{
            snapshot_root = $snapshotRoot
            snapshot_identity = $snapshotIdentity
            source_root = $resolvedStagingRoot
        }
    }
    return $snapshotRoot
}

function Remove-MvpAcceptanceStagingSnapshot {
    param(
        [string]$SnapshotRoot,
        [string]$ExpectedRootIdentity
    )

    if (-not [string]::IsNullOrWhiteSpace($SnapshotRoot)) {
        try {
            $removed = Remove-MvpAcceptanceStagingTree `
                -Path $SnapshotRoot `
                -ExpectedRootIdentity $ExpectedRootIdentity
            if (-not $removed -and -not [string]::IsNullOrWhiteSpace($ExpectedRootIdentity)) {
                Write-Warning "Acceptance staging snapshot cleanup skipped '$SnapshotRoot' because it no longer identifies the published snapshot."
            }
        }
        catch {
            $exception = $_.Exception
            while ($null -ne $exception.InnerException -and
                -not ($exception -is [System.ComponentModel.Win32Exception])) {
                $exception = $exception.InnerException
            }
            if ($exception -isnot [System.ComponentModel.Win32Exception] -or
                $exception.NativeErrorCode -notin @(2, 3)) {
                throw
            }
        }
    }
}

Export-ModuleMember -Function Get-MvpAcceptanceStagingItems, Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints, Copy-MvpAcceptanceStagingItems, Get-MvpAcceptanceNoFollowDirectoryIdentity, Open-MvpAcceptanceStagingWriteLease, Close-MvpAcceptanceStagingWriteLease, Take-MvpAcceptanceStagingWriteLeaseRootHandle, Open-MvpAcceptanceStagingSnapshotLease, Close-MvpAcceptanceStagingSnapshotLease, Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication, New-MvpAcceptanceStagingSnapshot, Remove-MvpAcceptanceStagingSnapshot, Remove-MvpAcceptanceEmptyDirectoryNoFollow, Move-MvpAcceptanceStagingDirectoryNoFollow
