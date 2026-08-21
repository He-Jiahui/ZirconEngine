Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingProjection.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingTreeManifest.psm1') -Force -ErrorAction Stop

if ($null -eq (Get-Variable -Name MvpAcceptanceStagingWriteLeases -Scope Script -ErrorAction SilentlyContinue)) {
    $script:MvpAcceptanceStagingWriteLeases =
        [System.Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
}

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop

function Get-MvpAcceptanceStagingItems {
    return @(
        'staging-manifest.json',
        'startup-summary.json',
        'staging-tree-manifest.json',
        'runtime',
        'editor',
        'templates',
        'project',
        'build',
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

function Open-MvpAcceptanceStagingTreeManifestEntryLeases {
    param(
        [Parameter(Mandatory)][string]$SnapshotRoot,
        [Parameter(Mandatory)][System.Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]$EntryHandles,
        [Parameter(Mandatory)][System.Collections.Generic.HashSet[string]]$PrelockedPaths,
        [scriptblock]$BeforeOpenTreeManifestEntryHook
    )

    $treeManifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $SnapshotRoot
    $treeManifestHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($treeManifestPath, $true)
    try {
        $treeManifestAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($treeManifestHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $treeManifestAttributes -Path $treeManifestPath
        if (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $treeManifestAttributes `
            -Expected ([System.IO.FileAttributes]::Directory)) {
            throw "Acceptance staging tree manifest '$treeManifestPath' is not a file."
        }
        $null = $EntryHandles.Add($treeManifestHandle)
        $treeManifestHandle = $null
        $null = $PrelockedPaths.Add($treeManifestPath)
    }
    finally {
        if ($null -ne $treeManifestHandle) {
            $treeManifestHandle.Dispose()
        }
    }

    $leasedEntries = [System.Collections.Generic.List[object]]::new()
    foreach ($entry in Read-MvpAcceptanceStagingTreeManifest -StagingRoot $SnapshotRoot) {
        if ($null -ne $BeforeOpenTreeManifestEntryHook) {
            & $BeforeOpenTreeManifestEntryHook $entry.path
        }
        $entryHandle = $null
        try {
            try {
                $entryHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($entry.path, $true)
            }
            catch {
                $exception = $_.Exception
                while ($null -ne $exception.InnerException -and
                    -not ($exception -is [System.ComponentModel.Win32Exception])) {
                    $exception = $exception.InnerException
                }
                if ($exception -is [System.ComponentModel.Win32Exception] -and
                    $exception.NativeErrorCode -in @(2, 3)) {
                    throw "Acceptance staging tree manifest entry '$($entry.relative_path)' does not exist in the staging root."
                }
                throw
            }
            $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($entryHandle)
            Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $entry.path
            $isDirectory = Test-MvpAcceptanceNativeFileAttribute `
                -Attributes $attributes `
                -Expected ([System.IO.FileAttributes]::Directory)
            if (($entry.kind -eq 'directory') -ne $isDirectory) {
                throw "Acceptance staging tree manifest entry '$($entry.relative_path)' changed its kind."
            }
            if (-not $isDirectory) {
                $length = [Int64]([ZirconMvpAcceptanceNativeFileSystem]::GetLength($entryHandle))
                if ($length -ne $entry.size_bytes) {
                    throw "Acceptance staging tree manifest file '$($entry.relative_path)' changed its length."
                }
                $actualSha256 = Get-MvpAcceptanceStagingTreeManifestSha256 -Path $entry.path
                if ($actualSha256 -ne $entry.sha256) {
                    throw "Acceptance staging tree manifest file '$($entry.relative_path)' changed its content."
                }
            }
            $leasedEntries.Add([pscustomobject]@{
                    path = $entry.path
                    relative_path = $entry.relative_path
                    identity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($entryHandle)
                    is_directory = $isDirectory
                    handle = $entryHandle
                }) | Out-Null
            $null = $EntryHandles.Add($entryHandle)
            $entryHandle = $null
            $null = $PrelockedPaths.Add($entry.path)
        }
        finally {
            if ($null -ne $entryHandle) {
                $entryHandle.Dispose()
            }
        }
    }
    return $leasedEntries.ToArray()
}

function Assert-MvpAcceptanceStagingTreeManifestCopyContract {
    param(
        [Parameter(Mandatory)][object[]]$Entries,
        [switch]$AllowEvidencePackageEntries
    )

    $allowedTopLevelItems = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase)
    foreach ($item in Get-MvpAcceptanceStagingItems) {
        $null = $allowedTopLevelItems.Add($item)
    }
    if ($AllowEvidencePackageEntries) {
        # The evidence publisher writes these final outputs after copying the fixed Stage roots.
        # They are valid only when leasing an evidence partial tree, never a Stage source copy root.
        $null = $allowedTopLevelItems.Add('manifest.json')
        $null = $allowedTopLevelItems.Add('comparison')
    }
    foreach ($entry in $Entries) {
        $topLevelItem = $entry.relative_path.Split('/')[0]
        if (-not $allowedTopLevelItems.Contains($topLevelItem)) {
            throw "Acceptance staging tree manifest entry '$($entry.relative_path)' is outside the accepted staging top-level contract."
        }
    }
}

function Assert-MvpAcceptanceStagingSnapshotLeaseTreeManifestMembership {
    param([Parameter(Mandatory)]$Lease)

    if ($null -eq $Lease.tree_manifest_expected_children -or $null -eq $Lease.marker_paths) {
        throw 'Acceptance staging snapshot lease is missing its tree-manifest membership contract.'
    }
    $markerPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($markerPath in @($Lease.marker_paths)) {
        $null = $markerPaths.Add([IO.Path]::GetFullPath($markerPath))
    }
    foreach ($directoryPath in $Lease.tree_manifest_expected_children.Keys) {
        $actualChildren = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
            $childPath = [IO.Path]::GetFullPath($child.FullName)
            if (-not $markerPaths.Contains($childPath)) {
                $null = $actualChildren.Add($childPath)
            }
        }
        if (-not $actualChildren.SetEquals($Lease.tree_manifest_expected_children[$directoryPath])) {
            throw "Acceptance staging snapshot source directory '$directoryPath' changed during snapshot copy."
        }
    }
}

function Open-MvpAcceptanceStagingSnapshotLease {
    param(
        [Parameter(Mandatory)][string]$SnapshotRoot,
        [Parameter(Mandatory)][string]$ExpectedRootIdentity,
        $StagingWriteLease,
        [scriptblock]$BeforeCreateDirectoryMarkerHook,
        [scriptblock]$BeforeOpenTreeManifestEntryHook,
        [switch]$AllowEvidencePackageEntries
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

        # The producer's immutable relative-path tree manifest lets this consumer pin every
        # expected node before it starts a recursive census. A walk of an arbitrary mutable tree
        # could otherwise lose an unknown descendant before that descendant is ever opened.
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
        $manifestEntries = @(Open-MvpAcceptanceStagingTreeManifestEntryLeases `
            -SnapshotRoot $absoluteSnapshotRoot `
            -EntryHandles $entryHandles `
            -PrelockedPaths $prelockedPaths `
            -BeforeOpenTreeManifestEntryHook $BeforeOpenTreeManifestEntryHook)
        Assert-MvpAcceptanceStagingTreeManifestCopyContract `
            -Entries $manifestEntries `
            -AllowEvidencePackageEntries:$AllowEvidencePackageEntries

        # The manifest leases pin the complete published tree before this census verifies its
        # membership. The census remains the defense-in-depth check against unlisted additions.
        $expectedChildrenByDirectory = [System.Collections.Generic.Dictionary[string, System.Collections.Generic.HashSet[string]]]::new([StringComparer]::OrdinalIgnoreCase)
        $censusDirectories = [System.Collections.Generic.List[object]]::new()
        $rootDirectoryEntry = [pscustomobject]@{
                path = $absoluteSnapshotRoot
                identity = $ExpectedRootIdentity
                is_root = $true
            }
        $censusDirectories.Add($rootDirectoryEntry) | Out-Null
        $expectedChildrenByDirectory.Add(
            $absoluteSnapshotRoot,
            [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase))
        $treeManifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteSnapshotRoot
        $null = $expectedChildrenByDirectory[$absoluteSnapshotRoot].Add($treeManifestPath)
        foreach ($entry in $manifestEntries) {
            $parentPath = [IO.Directory]::GetParent($entry.path).FullName
            if (-not $expectedChildrenByDirectory.ContainsKey($parentPath)) {
                $expectedChildrenByDirectory.Add(
                    $parentPath,
                    [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase))
            }
            $null = $expectedChildrenByDirectory[$parentPath].Add($entry.path)
            if ($entry.is_directory) {
                $censusDirectories.Add($entry) | Out-Null
                if (-not $expectedChildrenByDirectory.ContainsKey($entry.path)) {
                    $expectedChildrenByDirectory.Add(
                        $entry.path,
                        [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase))
                }
            }
        }

        foreach ($directoryEntry in $censusDirectories) {
            $ownsDirectoryHandle = $true
            $directoryHandle = if ($null -ne $registeredWriteLease -and
                $directoryEntry.path.Equals($absoluteSnapshotRoot, [StringComparison]::OrdinalIgnoreCase)) {
                $ownsDirectoryHandle = $false
                $rootHandle
            }
            else {
                [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($directoryEntry.path, $true)
            }
            try {
                $directoryAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($directoryHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $directoryAttributes -Path $directoryEntry.path
                if (-not (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $directoryAttributes `
                    -Expected ([System.IO.FileAttributes]::Directory)) -or
                    [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($directoryHandle) -ne $directoryEntry.identity) {
                    throw "Acceptance staging snapshot source directory '$($directoryEntry.path)' changed during its lease census."
                }

                if (-not $expectedChildrenByDirectory.ContainsKey($directoryEntry.path)) {
                    throw "Acceptance staging tree manifest does not declare directory '$($directoryEntry.path)'."
                }
                $actualChildren = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
                foreach ($child in @(Get-ChildItem -LiteralPath $directoryEntry.path -Force -ErrorAction Stop)) {
                    $childPath = [IO.Path]::GetFullPath($child.FullName)
                    if ($markerPaths.Contains($childPath)) {
                        continue
                    }
                    $null = $actualChildren.Add($childPath)
                }
                if (-not $actualChildren.SetEquals($expectedChildrenByDirectory[$directoryEntry.path])) {
                    throw "Acceptance staging snapshot source directory '$($directoryEntry.path)' differs from its published tree manifest."
                }
            }
            finally {
                if ($ownsDirectoryHandle) {
                    $directoryHandle.Dispose()
                }
            }
        }

        # Every manifest entry remains held from discovery through copying. Verify its identity
        # before marking; because the handle withholds delete sharing, the namespace cannot be
        # rebound between census and marker creation.
        foreach ($expectedEntry in $manifestEntries) {
            $childMarkerStream = $null
            try {
                $entryHandle = $expectedEntry.handle
                $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($entryHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $expectedEntry.path
                $isDirectory = Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $attributes `
                    -Expected ([System.IO.FileAttributes]::Directory)
                if ($isDirectory -ne $expectedEntry.is_directory -or
                    [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($entryHandle) -ne $expectedEntry.identity) {
                    throw "Acceptance staging snapshot source entry '$($expectedEntry.path)' changed while its census handle was held."
                }

                if ($isDirectory) {
                    if ($null -ne $BeforeCreateDirectoryMarkerHook) {
                        & $BeforeCreateDirectoryMarkerHook $expectedEntry.path
                    }
                    $childMarker = New-MvpAcceptanceStagingSnapshotLeaseMarker -DirectoryPath $expectedEntry.path
                    $childMarkerStream = $childMarker.stream
                    $null = $markerPaths.Add($childMarker.path)
                    $null = $markerStreams.Add($childMarkerStream)
                    $childMarkerStream = $null
                }
            }
            finally {
                if ($null -ne $childMarkerStream) {
                    $childMarkerStream.Dispose()
                }
            }
        }

        # A census alone would still accept an entry created after its parent was scanned. Once
        # all original paths are pinned, require every directory's immediate membership to match
        # the recorded source tree before returning a usable snapshot lease.
        foreach ($directoryPath in $expectedChildrenByDirectory.Keys) {
            $actualChildren = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
            foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
                $childPath = [IO.Path]::GetFullPath($child.FullName)
                if (-not $markerPaths.Contains($childPath)) {
                    $null = $actualChildren.Add($childPath)
                }
            }
            if (-not $actualChildren.SetEquals($expectedChildrenByDirectory[$directoryPath])) {
                throw "Acceptance staging snapshot source directory '$directoryPath' changed while its entry leases were being acquired."
            }
        }

        $lease = [pscustomobject]@{
            root_handle = if ($ownsRootHandle) { $rootHandle } else { $null }
            root_path = $absoluteSnapshotRoot
            root_identity = $ExpectedRootIdentity
            held_staging_write_lease = $StagingWriteLease
            parent_lease = $parentLease
            tree_manifest_expected_children = $expectedChildrenByDirectory
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
        [scriptblock]$AfterCopyChildHook,
        [scriptblock]$BeforeOpenDestinationChildHook,
        [Parameter(Mandatory)]$ExpectedChildrenByDirectory,
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
            if (-not $ExpectedChildrenByDirectory.ContainsKey($SourcePath)) {
                throw "Acceptance staging tree manifest does not declare source directory '$SourcePath'."
            }
            foreach ($childPath in @($ExpectedChildrenByDirectory[$SourcePath] | Sort-Object)) {
                if ($null -ne $ExcludedSourcePaths -and $ExcludedSourcePaths.Contains($childPath)) {
                    continue
                }
                $childName = [IO.Path]::GetFileName($childPath)
                # The parent directory handle excludes write/delete sharing. A child is opened
                # no-follow from the immutable manifest and remains open until its copy
                # finishes, so a replacement with a reparse point cannot be followed.
                if ($null -ne $BeforeOpenChildHook) {
                    & $BeforeOpenChildHook $SourcePath $childPath $DestinationPath
                }
                $childHandle = $null
                try {
                    $childHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($childPath, $true)
                    Copy-MvpAcceptanceStagingTree `
                        -SourcePath $childPath `
                        -DestinationPath (Join-Path $DestinationPath $childName) `
                        -SourceHandle $childHandle `
                        -BeforeOpenChildHook $BeforeOpenChildHook `
                        -AfterCopyChildHook $AfterCopyChildHook `
                        -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
                        -ExpectedChildrenByDirectory $ExpectedChildrenByDirectory `
                        -ExcludedSourcePaths $ExcludedSourcePaths `
                        -Projection $Projection `
                        -DestinationWriteLease $DestinationWriteLease
                    $childHandle = $null
                    if ($null -ne $AfterCopyChildHook) {
                        & $AfterCopyChildHook $childPath (Join-Path $DestinationPath $childName)
                    }
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
        [Parameter(Mandatory)]$SourceSnapshotLease,
        [scriptblock]$BeforeOpenChildHook,
        [scriptblock]$AfterCopyChildHook,
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
        $leaseRootPath = [IO.Path]::GetFullPath([string]$SourceSnapshotLease.root_path)
        $sourceRootPath = [IO.Path]::GetFullPath($SourceRoot)
        if (-not $leaseRootPath.Equals($sourceRootPath, [StringComparison]::OrdinalIgnoreCase) -or
            $null -eq $SourceSnapshotLease.tree_manifest_expected_children) {
            throw 'Acceptance staging copy requires the matching source snapshot lease and tree-manifest membership.'
        }
        $expectedChildrenByDirectory = $SourceSnapshotLease.tree_manifest_expected_children
        $sourceRootLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $SourceRoot
        $sourceRootHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($SourceRoot, $false)
        $sourceRootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($sourceRootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $sourceRootAttributes -Path $SourceRoot
        if (-not (Test-MvpAcceptanceNativeFileAttribute -Attributes $sourceRootAttributes -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance staging root '$SourceRoot' is not a directory."
        }

        $resolvedSourceRoot = (Resolve-Path -LiteralPath $SourceRoot -ErrorAction Stop).Path
        if (-not $expectedChildrenByDirectory.ContainsKey($resolvedSourceRoot)) {
            throw "Acceptance staging tree manifest does not declare source root '$resolvedSourceRoot'."
        }
        foreach ($relativePath in Get-MvpAcceptanceStagingItems) {
            $sourcePath = Join-Path $resolvedSourceRoot $relativePath
            if ($excludedPaths.Contains([IO.Path]::GetFullPath($sourcePath))) {
                continue
            }
            if (-not $expectedChildrenByDirectory[$resolvedSourceRoot].Contains($sourcePath)) {
                continue
            }
            if ($null -ne $BeforeOpenChildHook) {
                & $BeforeOpenChildHook $resolvedSourceRoot $sourcePath $DestinationRoot
            }
            Copy-MvpAcceptanceStagingTree `
                -SourcePath $sourcePath `
                -DestinationPath (Join-Path $DestinationRoot $relativePath) `
                -BeforeOpenChildHook $BeforeOpenChildHook `
                -AfterCopyChildHook $AfterCopyChildHook `
                -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
                -ExpectedChildrenByDirectory $expectedChildrenByDirectory `
                -ExcludedSourcePaths $excludedPaths `
                -Projection $projection `
                -DestinationWriteLease $DestinationWriteLease
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
        [scriptblock]$BeforeCreateSourceDirectoryMarkerHook,
        [scriptblock]$BeforeOpenSourceTreeManifestEntryHook,
        [scriptblock]$AfterCopyChildHook,
        [scriptblock]$AfterPublishHook
    )

    # Preserve a fully qualified caller entry so the no-follow native checks can reject a root junction.
    # Relative roots intentionally retain PowerShell's current-location semantics.
    $absoluteStagingRoot = if ([System.IO.Path]::IsPathFullyQualified($StagingRoot)) {
        [System.IO.Path]::GetFullPath($StagingRoot)
    }
    else {
        $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StagingRoot)
    }
    $snapshotRoot = $null
    $snapshotRoot = "$absoluteStagingRoot.acceptance-snapshot-$([guid]::NewGuid().ToString('N'))"
    $partialRoot = "$snapshotRoot.partial"
    $sourceRootLease = $null
    $sourceSnapshotLease = $null
    $sourceIdentity = $null
    $partialWriteLease = $null
    $partialSnapshotLease = $null
    $partialIdentity = $null
    $snapshotIdentity = $null
    $partialPublished = $false
    try {
        $sourceRootLease = Open-MvpAcceptanceNoFollowDirectoryLease -DirectoryPath $absoluteStagingRoot
        $sourceIdentity = Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $absoluteStagingRoot
        # Freeze the producer-published source before any copy traversal. The full tree manifest
        # is a producer handoff contract; it lets source leases cover unknown descendants before
        # a consumer walk can observe only a reduced tree.
        $sourceSnapshotLease = Open-MvpAcceptanceStagingSnapshotLease `
            -SnapshotRoot $absoluteStagingRoot `
            -ExpectedRootIdentity $sourceIdentity `
            -BeforeCreateDirectoryMarkerHook $BeforeCreateSourceDirectoryMarkerHook `
            -BeforeOpenTreeManifestEntryHook $BeforeOpenSourceTreeManifestEntryHook
        New-Item -ItemType Directory -Path $partialRoot -ErrorAction Stop | Out-Null
        $partialWriteLease = Open-MvpAcceptanceStagingWriteLease -SnapshotRoot $partialRoot
        $partialIdentity = $partialWriteLease.root_identity
        if (@(Get-ChildItem -LiteralPath $partialRoot -Force -ErrorAction Stop).Count -ne 0) {
            throw "Acceptance staging snapshot partial root '$partialRoot' is not empty after creation."
        }
        # A nested snapshot owns a fresh inventory for its partial tree. Do not copy the
        # producer inventory and then overwrite the same path, because that would invalidate
        # the projection that is about to bind the partial publication.
        $sourceCopyExclusions = @($sourceSnapshotLease.marker_paths) + @(
            Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteStagingRoot)
        $stagingProjection = Copy-MvpAcceptanceStagingItems `
            -SourceRoot $absoluteStagingRoot `
            -DestinationRoot $partialRoot `
            -SourceSnapshotLease $sourceSnapshotLease `
            -BeforeOpenChildHook $BeforeOpenChildHook `
            -AfterCopyChildHook $AfterCopyChildHook `
            -BeforeOpenDestinationChildHook $BeforeOpenDestinationChildHook `
            -ExcludedSourcePaths $sourceCopyExclusions `
            -DestinationWriteLease $partialWriteLease `
            -PassThruProjection
        # Directory handles prevent deletion/rebinding, but they do not forbid a writer from
        # adding a child. Recheck the producer's fixed membership after recursive copy so such
        # an addition cannot be silently promoted into a later partial-tree manifest.
        Assert-MvpAcceptanceStagingSnapshotLeaseTreeManifestMembership -Lease $sourceSnapshotLease
        $resolvedStagingRoot = [string]$stagingProjection.source_root
        foreach ($requiredFile in @('staging-manifest.json', 'startup-summary.json')) {
            if (-not (Test-Path -LiteralPath (Join-Path $partialRoot $requiredFile) -PathType Leaf)) {
                throw "Acceptance staging snapshot is missing required '$requiredFile'."
            }
        }
        # Bind the raw copy before creating this partial tree's own manifest. Otherwise an
        # ordinary destination-side injection could be swept into that generated inventory.
        Assert-MvpAcceptanceStagingProjection `
            -Root $partialRoot `
            -Projection $stagingProjection.projection
        # This partial tree becomes a source for the next snapshot lease. Publish its complete
        # relative-path inventory only after every copied file has landed.
        $partialTreeManifestPath = Write-MvpAcceptanceStagingTreeManifest -StagingRoot $partialRoot
        Add-MvpAcceptanceStagingProjectionOwnedFile `
            -Projection $stagingProjection.projection `
            -Path $partialTreeManifestPath `
            -ContentBytes ([IO.File]::ReadAllBytes($partialTreeManifestPath))
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
        if ($null -ne $sourceSnapshotLease) {
            Close-MvpAcceptanceStagingSnapshotLease -Lease $sourceSnapshotLease
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

Export-ModuleMember -Function Get-MvpAcceptanceStagingItems, Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints, Copy-MvpAcceptanceStagingItems, Get-MvpAcceptanceNoFollowDirectoryIdentity, Open-MvpAcceptanceStagingWriteLease, Close-MvpAcceptanceStagingWriteLease, Take-MvpAcceptanceStagingWriteLeaseRootHandle, Open-MvpAcceptanceStagingSnapshotLease, Close-MvpAcceptanceStagingSnapshotLease, Assert-MvpAcceptanceStagingSnapshotLeaseTreeManifestMembership, Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication, New-MvpAcceptanceStagingSnapshot, Remove-MvpAcceptanceStagingSnapshot, Remove-MvpAcceptanceEmptyDirectoryNoFollow, Move-MvpAcceptanceStagingDirectoryNoFollow
