Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Assert-SnapshotLeaseBlocksRootReplacement {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidateSet('delete', 'rename')][string]$Attempt
    )

    $details = New-MvpAcceptanceStagingSnapshot -StagingRoot $StagingRoot -PassThru
    $snapshotRoot = [string]$details.snapshot_root
    $snapshotIdentity = [string]$details.snapshot_identity
    $movedRoot = "$snapshotRoot.moved"
    $archiveRoot = "$snapshotRoot.archive"
    $lease = Open-MvpAcceptanceStagingSnapshotLease `
        -SnapshotRoot $snapshotRoot `
        -ExpectedRootIdentity $snapshotIdentity
    try {
        $operationSucceeded = $false
        try {
            if ($Attempt -eq 'delete') {
                Remove-Item -LiteralPath $snapshotRoot -Recurse -Force -ErrorAction Stop
            }
            else {
                Move-Item -LiteralPath $snapshotRoot -Destination $movedRoot -ErrorAction Stop
            }
            $operationSucceeded = $true
        }
        catch {
            # The held snapshot entries and per-directory markers must reject this destructive operation.
        }
        Assert-True (-not $operationSucceeded) "Acceptance staging snapshot lease allowed $Attempt of its root during use."
        Assert-True (Test-Path -LiteralPath $snapshotRoot -PathType Container) "Acceptance staging snapshot lease lost its root after rejected $Attempt."
        Assert-True (
            (Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $snapshotRoot) -eq $snapshotIdentity
        ) "Acceptance staging snapshot lease changed root identity after rejected $Attempt."
        foreach ($relativePath in @('staging-manifest.json', 'startup-summary.json', 'logs/source.log')) {
            Assert-True (
                Test-Path -LiteralPath (Join-Path $snapshotRoot $relativePath) -PathType Leaf
            ) "Acceptance staging snapshot lease lost '$relativePath' after rejected $Attempt."
        }
        New-Item -ItemType Directory -Force -Path $archiveRoot | Out-Null
        Copy-MvpAcceptanceStagingItems `
            -SourceRoot $snapshotRoot `
            -DestinationRoot $archiveRoot `
            -ExcludedSourcePaths $lease.marker_paths | Out-Null
        Assert-True (
            Test-Path -LiteralPath (Join-Path $archiveRoot 'logs/source.log') -PathType Leaf
        ) 'Acceptance staging snapshot lease prevented archival of a held snapshot.'
        $copiedMarkerPaths = @(
            Get-ChildItem -LiteralPath $archiveRoot -Recurse -File -Force |
                Where-Object { $_.Name -like '.zircon-mvp-acceptance-lease-*.lock' }
        )
        Assert-True ($copiedMarkerPaths.Count -eq 0) 'Acceptance snapshot archival copied a lease marker.'
    }
    finally {
        Close-MvpAcceptanceStagingSnapshotLease -Lease $lease
        if (Test-Path -LiteralPath $snapshotRoot) {
            Remove-MvpAcceptanceStagingSnapshot `
                -SnapshotRoot $snapshotRoot `
                -ExpectedRootIdentity $snapshotIdentity
        }
        if (Test-Path -LiteralPath $movedRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $movedRoot
        }
        if (Test-Path -LiteralPath $archiveRoot) {
            Remove-Item -LiteralPath $archiveRoot -Recurse -Force
        }
    }
}

$stagingSnapshotModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingSnapshot.psm1'
Import-Module $stagingSnapshotModule -Force
$nativeFileSystemModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceNativeFileSystem.psm1'
Import-Module $nativeFileSystemModule -Force
$stagingProjectionModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingProjection.psm1'
Import-Module $stagingProjectionModule -Force
$buildSummaryEvidenceModule = Join-Path $PSScriptRoot '..\mvp\MvpBuildSummaryEvidence.psm1'
Import-Module $buildSummaryEvidenceModule -Force
$persistenceComparisonModule = Join-Path $PSScriptRoot '..\mvp\MvpPersistenceComparison.psm1'
Import-Module $persistenceComparisonModule -Force

$snapshotFixtureRoot = Join-Path ([IO.Path]::GetTempPath()) ('zircon-mvp-acceptance-snapshot-' + [guid]::NewGuid().ToString('N'))
$snapshotSourceRoot = Join-Path $snapshotFixtureRoot 'stage'
$snapshotLogsRoot = Join-Path $snapshotSourceRoot 'logs'
New-Item -ItemType Directory -Force -Path $snapshotLogsRoot | Out-Null
[IO.File]::WriteAllText((Join-Path $snapshotSourceRoot 'staging-manifest.json'), '{}', [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText((Join-Path $snapshotSourceRoot 'startup-summary.json'), '{}', [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText((Join-Path $snapshotLogsRoot 'source.log'), 'before-snapshot', [Text.UTF8Encoding]::new($false))
$projectionRoot = Join-Path $snapshotFixtureRoot 'projection'
$projectionOutput = Join-Path $projectionRoot 'generated.json'
$projectionExpectedBytes = [Text.UTF8Encoding]::new($false).GetBytes('{"result":"expected"}')
New-Item -ItemType Directory -Force -Path $projectionRoot | Out-Null
[IO.File]::WriteAllBytes($projectionOutput, $projectionExpectedBytes)
$projection = New-MvpAcceptanceStagingProjection -Root $projectionRoot
Add-MvpAcceptanceStagingProjectionOwnedFile `
    -Projection $projection `
    -Path $projectionOutput `
    -ContentBytes $projectionExpectedBytes
[IO.File]::WriteAllText($projectionOutput, '{"result":"tampered"}', [Text.UTF8Encoding]::new($false))
$projectionTamperRejected = $false
try {
    Assert-MvpAcceptanceStagingProjection -Root $projectionRoot -Projection $projection
}
catch {
    $projectionTamperRejected = $_.Exception.Message -match 'differs from its expected'
}
Assert-True $projectionTamperRejected 'Acceptance projection accepted a generated output overwritten after its expected bytes were captured.'
$snapshotRoot = $null
$snapshotIdentity = $null
try {
    $snapshotDetails = New-MvpAcceptanceStagingSnapshot -StagingRoot $snapshotSourceRoot -PassThru
    $snapshotRoot = [string]$snapshotDetails.snapshot_root
    $snapshotIdentity = [string]$snapshotDetails.snapshot_identity
    Assert-True (-not [string]::IsNullOrWhiteSpace($snapshotIdentity)) 'Acceptance staging snapshot did not return its published directory identity.'
    Assert-SnapshotLeaseBlocksRootReplacement -StagingRoot $snapshotSourceRoot -Attempt 'delete'
    Assert-SnapshotLeaseBlocksRootReplacement -StagingRoot $snapshotSourceRoot -Attempt 'rename'

    $markerRaceDetails = New-MvpAcceptanceStagingSnapshot -StagingRoot $snapshotSourceRoot -PassThru
    $markerRaceRoot = [string]$markerRaceDetails.snapshot_root
    $markerRaceIdentity = [string]$markerRaceDetails.snapshot_identity
    $markerRaceLogsRoot = Join-Path $markerRaceRoot 'logs'
    $markerRaceInjection = [pscustomobject]@{ attempted = $false }
    $markerRaceLease = $null
    $markerRaceRejected = $false
    try {
        $markerRaceLease = Open-MvpAcceptanceStagingSnapshotLease `
            -SnapshotRoot $markerRaceRoot `
            -ExpectedRootIdentity $markerRaceIdentity `
            -BeforeCreateDirectoryMarkerHook {
                param($childPath)
                if ($markerRaceInjection.attempted -or
                    -not $childPath.Equals($markerRaceLogsRoot, [StringComparison]::OrdinalIgnoreCase)) {
                    return
                }
                $markerRaceInjection.attempted = $true
                Remove-Item -LiteralPath $markerRaceLogsRoot -Recurse -Force -ErrorAction Stop
                New-Item -ItemType Directory -Force -Path $markerRaceLogsRoot | Out-Null
                [IO.File]::WriteAllText(
                    (Join-Path $markerRaceLogsRoot 'source.log'),
                    'replacement-must-not-be-leased',
                    [Text.UTF8Encoding]::new($false))
            }
    }
    catch {
        $markerRaceRejected = $_.Exception.Message -match 'changed while its child marker was being acquired'
    }
    finally {
        Close-MvpAcceptanceStagingSnapshotLease -Lease $markerRaceLease
    }
    Assert-True $markerRaceInjection.attempted 'Acceptance snapshot lease marker race hook did not reach a child directory.'
    Assert-True $markerRaceRejected 'Acceptance snapshot lease accepted a child directory replaced before its marker was acquired.'
    Remove-MvpAcceptanceStagingSnapshot `
        -SnapshotRoot $markerRaceRoot `
        -ExpectedRootIdentity $markerRaceIdentity

    Assert-True (
        ([string]$snapshotDetails.source_root).Equals(
            (Resolve-Path -LiteralPath $snapshotSourceRoot).Path,
            [StringComparison]::OrdinalIgnoreCase
        )
    ) 'Acceptance staging snapshot did not retain the handle-verified source root.'
    [IO.File]::WriteAllText((Join-Path $snapshotLogsRoot 'source.log'), 'after-snapshot', [Text.UTF8Encoding]::new($false))
    Assert-True ((Get-Content -LiteralPath (Join-Path $snapshotRoot 'logs/source.log') -Raw) -eq 'before-snapshot') 'Acceptance staging snapshot changed when its source was mutated.'
    Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints -StagingRoot $snapshotRoot

    $snapshotSourceDriveRoot = [IO.Path]::GetPathRoot($snapshotSourceRoot)
    $relativeSnapshotSourceRoot = $snapshotSourceRoot.Substring($snapshotSourceDriveRoot.Length)
    Push-Location $snapshotSourceDriveRoot
    try {
        $relativeSnapshotDetails = New-MvpAcceptanceStagingSnapshot -StagingRoot $relativeSnapshotSourceRoot -PassThru
        try {
            Assert-True ([IO.Path]::IsPathRooted([string]$relativeSnapshotDetails.snapshot_root)) 'Acceptance staging snapshot returned a relative snapshot root.'
            Assert-True (
                ([string]$relativeSnapshotDetails.source_root).Equals(
                    (Resolve-Path -LiteralPath $snapshotSourceRoot).Path,
                    [StringComparison]::OrdinalIgnoreCase
                )
            ) 'Acceptance staging snapshot did not normalize a relative source root.'
        }
        finally {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot ([string]$relativeSnapshotDetails.snapshot_root)
        }
    }
    finally {
        Pop-Location
    }

    $snapshotRootJunction = Join-Path $snapshotFixtureRoot 'stage-junction'
    New-Item -ItemType Junction -Path $snapshotRootJunction -Target $snapshotSourceRoot -ErrorAction Stop | Out-Null
    $snapshotRootJunctionRejected = $false
    try {
        New-MvpAcceptanceStagingSnapshot -StagingRoot $snapshotRootJunction | Out-Null
    }
    catch {
        $snapshotRootJunctionRejected = $_.Exception.Message -match 'reparse point'
    }
    finally {
        if (Test-Path -LiteralPath $snapshotRootJunction) {
            [IO.Directory]::Delete($snapshotRootJunction, $false)
        }
    }
    Assert-True $snapshotRootJunctionRejected 'Acceptance staging snapshot accepted a root junction.'

    $snapshotAncestorTargetRoot = Join-Path $snapshotFixtureRoot 'ancestor-junction-target'
    $snapshotAncestorStageRoot = Join-Path $snapshotAncestorTargetRoot 'stage'
    $snapshotAncestorLogsRoot = Join-Path $snapshotAncestorStageRoot 'logs'
    $snapshotAncestorJunctionRoot = Join-Path $snapshotFixtureRoot 'ancestor-junction'
    New-Item -ItemType Directory -Force -Path $snapshotAncestorLogsRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotAncestorStageRoot 'staging-manifest.json'), '{}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $snapshotAncestorStageRoot 'startup-summary.json'), '{}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $snapshotAncestorLogsRoot 'source.log'), 'ancestor-junction', [Text.UTF8Encoding]::new($false))
    New-Item -ItemType Junction -Path $snapshotAncestorJunctionRoot -Target $snapshotAncestorTargetRoot -ErrorAction Stop | Out-Null
    $snapshotAncestorJunctionRejected = $false
    try {
        New-MvpAcceptanceStagingSnapshot -StagingRoot (Join-Path $snapshotAncestorJunctionRoot 'stage') | Out-Null
    }
    catch {
        $snapshotAncestorJunctionRejected = $_.Exception.Message -match 'reparse point'
    }
    finally {
        if (Test-Path -LiteralPath $snapshotAncestorJunctionRoot) {
            [IO.Directory]::Delete($snapshotAncestorJunctionRoot, $false)
        }
    }
    Assert-True $snapshotAncestorJunctionRejected 'Acceptance staging snapshot accepted a staging root beneath a junction ancestor.'

    $snapshotJunctionTargetRoot = Join-Path $snapshotFixtureRoot 'outside-staging-logs'
    New-Item -ItemType Directory -Force -Path $snapshotJunctionTargetRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotJunctionTargetRoot 'source.log'), 'outside-staging', [Text.UTF8Encoding]::new($false))
    Remove-Item -LiteralPath $snapshotLogsRoot -Recurse -Force
    New-Item -ItemType Junction -Path $snapshotLogsRoot -Target $snapshotJunctionTargetRoot -ErrorAction Stop | Out-Null
    $snapshotJunctionRejected = $false
    try {
        New-MvpAcceptanceStagingSnapshot -StagingRoot $snapshotSourceRoot | Out-Null
    }
    catch {
        $snapshotJunctionRejected = $_.Exception.Message -match 'reparse point'
    }
    Assert-True $snapshotJunctionRejected 'Acceptance staging snapshot accepted a junction that escapes the staging tree.'

    $snapshotRaceTargetRoot = Join-Path $snapshotFixtureRoot 'snapshot-race-target'
    New-Item -ItemType Directory -Force -Path $snapshotRaceTargetRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotRaceTargetRoot 'must-survive.log'), 'outside-race-target', [Text.UTF8Encoding]::new($false))
    $snapshotLogsLink = Get-Item -LiteralPath $snapshotLogsRoot -Force
    [IO.Directory]::Delete($snapshotLogsLink.FullName, $false)
    New-Item -ItemType Directory -Force -Path $snapshotLogsRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotLogsRoot 'source.log'), 'race-source', [Text.UTF8Encoding]::new($false))
    $snapshotRaceInjection = [pscustomobject]@{ attempted = $false; replaced = $false; childRemoved = $false }
    $snapshotRaceRejected = $false
    $snapshotRaceDetails = $null
    try {
        $snapshotRaceDetails = New-MvpAcceptanceStagingSnapshot `
            -StagingRoot $snapshotSourceRoot `
            -PassThru `
            -BeforeOpenChildHook {
                param($parentPath, $childPath)
                if ($snapshotRaceInjection.attempted -or
                    -not $parentPath.Equals($snapshotLogsRoot, [StringComparison]::OrdinalIgnoreCase)) {
                    return
                }
                $snapshotRaceInjection.attempted = $true
                try {
                    Remove-Item -LiteralPath $snapshotLogsRoot -Recurse -Force -ErrorAction Stop
                    New-Item -ItemType Junction `
                        -Path $snapshotLogsRoot `
                        -Target $snapshotRaceTargetRoot `
                        -ErrorAction Stop | Out-Null
                    $snapshotRaceInjection.replaced = $true
                }
                catch {
                    # The directory lease can reject removal after a child has already been
                    # deleted. That still yields a safe copy failure: the later no-follow open
                    # cannot resolve the enumerated child to an external target.
                    $snapshotRaceInjection.childRemoved = -not (Test-Path -LiteralPath (Join-Path $snapshotLogsRoot 'source.log'))
                }
            }
    }
    catch {
        $snapshotRaceRejected = $_.Exception.Message -match 'reparse point'
    }
    try {
        Assert-True $snapshotRaceInjection.attempted 'Acceptance snapshot race hook did not reach the enumerated child boundary.'
        if ($snapshotRaceInjection.replaced) {
            Assert-True $snapshotRaceRejected 'Acceptance snapshot followed a junction inserted after child enumeration.'
        }
        else {
            if ($null -ne $snapshotRaceDetails) {
                Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints -StagingRoot ([string]$snapshotRaceDetails.snapshot_root)
            }
        }
        Assert-True (Test-Path -LiteralPath (Join-Path $snapshotRaceTargetRoot 'must-survive.log') -PathType Leaf) 'Acceptance snapshot race handling modified the external junction target.'
    }
    finally {
        if ($null -ne $snapshotRaceDetails) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot ([string]$snapshotRaceDetails.snapshot_root)
        }
        if (Test-Path -LiteralPath $snapshotLogsRoot) {
            $snapshotLogsItem = Get-Item -LiteralPath $snapshotLogsRoot -Force
            if ([bool]($snapshotLogsItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                [IO.Directory]::Delete($snapshotLogsItem.FullName, $false)
            }
        }
        if (-not (Test-Path -LiteralPath $snapshotLogsRoot)) {
            New-Item -ItemType Directory -Force -Path $snapshotLogsRoot | Out-Null
        }
        if (-not (Test-Path -LiteralPath (Join-Path $snapshotLogsRoot 'source.log'))) {
            [IO.File]::WriteAllText((Join-Path $snapshotLogsRoot 'source.log'), 'race-source', [Text.UTF8Encoding]::new($false))
        }
    }

    $snapshotDestinationRaceInjection = [pscustomobject]@{ attempted = $false; replaced = $false; injected = $false }
    $snapshotDestinationRaceRejected = $false
    $snapshotDestinationRaceDetails = $null
    try {
        $snapshotDestinationRaceDetails = New-MvpAcceptanceStagingSnapshot `
            -StagingRoot $snapshotSourceRoot `
            -PassThru `
            -BeforeOpenDestinationChildHook {
                param($destinationPath)
                if ($snapshotDestinationRaceInjection.attempted -or
                    -not [IO.Path]::GetFileName($destinationPath).Equals('logs', [StringComparison]::OrdinalIgnoreCase)) {
                    return
                }
                $snapshotDestinationRaceInjection.attempted = $true
                try {
                    [IO.Directory]::Delete($destinationPath, $false)
                    New-Item -ItemType Directory -Path $destinationPath -ErrorAction Stop | Out-Null
                    [IO.File]::WriteAllText(
                        (Join-Path $destinationPath 'must-not-copy.txt'),
                        'destination-race-injection',
                        [Text.UTF8Encoding]::new($false))
                    $snapshotDestinationRaceInjection.replaced = $true
                    $snapshotDestinationRaceInjection.injected = $true
                }
                catch {
                    # The created directory is already held no-delete before this hook.
                }
            }
    }
    catch {
        $snapshotDestinationRaceRejected = $true
    }
    try {
        Assert-True $snapshotDestinationRaceInjection.attempted 'Acceptance snapshot destination race hook did not reach the created child boundary.'
        if ($snapshotDestinationRaceInjection.replaced) {
            Assert-True $snapshotDestinationRaceInjection.injected 'Acceptance snapshot destination race hook did not inject a replacement entry.'
            Assert-True $snapshotDestinationRaceRejected 'Acceptance snapshot accepted a normal directory substituted after destination creation.'
        }
        else {
            Assert-True ($null -ne $snapshotDestinationRaceDetails) 'Acceptance snapshot failed after rejecting a concurrent destination child replacement.'
            Assert-MvpAcceptanceStagingTreeFreeOfReparsePoints -StagingRoot ([string]$snapshotDestinationRaceDetails.snapshot_root)
        }
    }
    finally {
        if ($null -ne $snapshotDestinationRaceDetails) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot ([string]$snapshotDestinationRaceDetails.snapshot_root)
        }
    }

    $snapshotOrdinaryDestinationInjection = [pscustomobject]@{ attempted = $false; injected = $false }
    $snapshotOrdinaryDestinationRejected = $false
    try {
        New-MvpAcceptanceStagingSnapshot `
            -StagingRoot $snapshotSourceRoot `
            -BeforeOpenChildHook {
                param($parentPath, $childPath, $destinationParentPath)
                if ($snapshotOrdinaryDestinationInjection.attempted -or
                    -not $parentPath.Equals($snapshotLogsRoot, [StringComparison]::OrdinalIgnoreCase)) {
                    return
                }
                $snapshotOrdinaryDestinationInjection.attempted = $true
                [IO.File]::WriteAllText(
                    (Join-Path $destinationParentPath 'unexpected-destination-entry.txt'),
                    'ordinary-destination-injection',
                    [Text.UTF8Encoding]::new($false))
                $snapshotOrdinaryDestinationInjection.injected = $true
            } | Out-Null
    }
    catch {
        $snapshotOrdinaryDestinationRejected = $_.Exception.Message -match 'unexpected entry'
    }
    Assert-True $snapshotOrdinaryDestinationInjection.attempted 'Acceptance snapshot ordinary destination injection hook did not reach a copied directory.'
    Assert-True $snapshotOrdinaryDestinationInjection.injected 'Acceptance snapshot ordinary destination injection hook did not create its extra file.'
    Assert-True $snapshotOrdinaryDestinationRejected 'Acceptance snapshot accepted an ordinary file injected into a copied destination directory.'

    $publishedPartial = [pscustomobject]@{ root = $null }
    $publishedPartialDetails = New-MvpAcceptanceStagingSnapshot `
        -StagingRoot $snapshotSourceRoot `
        -PassThru `
        -AfterPublishHook {
            param($partialPath)
            New-Item -ItemType Directory -Path $partialPath -ErrorAction Stop | Out-Null
            [IO.File]::WriteAllText((Join-Path $partialPath 'must-survive.txt'), 'post-publish-partial', [Text.UTF8Encoding]::new($false))
            $publishedPartial.root = $partialPath
        }
    try {
        Assert-True (Test-Path -LiteralPath (Join-Path $publishedPartial.root 'must-survive.txt') -PathType Leaf) 'Acceptance snapshot cleanup removed a partial path recreated after publication.'
        Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot ([string]$publishedPartialDetails.snapshot_root)
        Assert-True (Test-Path -LiteralPath (Join-Path $publishedPartial.root 'must-survive.txt') -PathType Leaf) 'Acceptance snapshot removal followed the stale partial cleanup path.'
    }
    finally {
        if ($null -ne $publishedPartial.root -and (Test-Path -LiteralPath $publishedPartial.root)) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $publishedPartial.root
        }
    }

    $publishedHookFailure = [pscustomobject]@{ partial = $null }
    $publishedHookError = $false
    try {
        New-MvpAcceptanceStagingSnapshot `
            -StagingRoot $snapshotSourceRoot `
            -AfterPublishHook {
                param($partialPath)
                $publishedHookFailure.partial = $partialPath
                throw 'post-publish hook failed'
            } | Out-Null
    }
    catch {
        $publishedHookError = $_.Exception.Message -match 'post-publish hook failed'
    }
    try {
        Assert-True $publishedHookError 'Acceptance staging snapshot masked a post-publish hook failure with partial cleanup.'
        Assert-True ($null -ne $publishedHookFailure.partial) 'Acceptance staging snapshot did not run the post-publish hook.'
        $publishedHookSnapshot = $publishedHookFailure.partial.Substring(
            0,
            $publishedHookFailure.partial.Length - '.partial'.Length
        )
        Assert-True (Test-Path -LiteralPath $publishedHookSnapshot -PathType Container) 'Acceptance staging snapshot removed an already-published tree after a hook failure.'
    }
    finally {
        if ($null -ne $publishedHookFailure.partial) {
            $publishedHookSnapshot = $publishedHookFailure.partial.Substring(
                0,
                $publishedHookFailure.partial.Length - '.partial'.Length
            )
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $publishedHookSnapshot
        }
    }

    $snapshotCleanupRoot = Join-Path $snapshotFixtureRoot 'snapshot-cleanup'
    $snapshotCleanupLogsRoot = Join-Path $snapshotCleanupRoot 'logs'
    $snapshotCleanupTargetRoot = Join-Path $snapshotFixtureRoot 'snapshot-cleanup-target'
    New-Item -ItemType Directory -Force -Path $snapshotCleanupRoot, $snapshotCleanupTargetRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotCleanupTargetRoot 'source.log'), 'must-survive-cleanup', [Text.UTF8Encoding]::new($false))
    New-Item -ItemType Junction -Path $snapshotCleanupLogsRoot -Target $snapshotCleanupTargetRoot -ErrorAction Stop | Out-Null
    Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $snapshotCleanupRoot
    Assert-True (-not (Test-Path -LiteralPath $snapshotCleanupRoot)) 'Acceptance snapshot cleanup did not remove its root.'
    Assert-True (Test-Path -LiteralPath (Join-Path $snapshotCleanupTargetRoot 'source.log') -PathType Leaf) 'Acceptance snapshot cleanup followed a junction and deleted its target.'

    $emptyEvidenceRoot = Join-Path $snapshotFixtureRoot 'empty-evidence-root'
    New-Item -ItemType Directory -Force -Path $emptyEvidenceRoot | Out-Null
    Remove-MvpAcceptanceEmptyDirectoryNoFollow -Path $emptyEvidenceRoot
    Assert-True (-not (Test-Path -LiteralPath $emptyEvidenceRoot)) 'Acceptance empty-root cleanup did not remove the empty directory.'

    $identityBoundEmptyRoot = Join-Path $snapshotFixtureRoot 'identity-bound-empty-root'
    New-Item -ItemType Directory -Force -Path $identityBoundEmptyRoot | Out-Null
    $identityBoundEmptyRootIdentity = Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $identityBoundEmptyRoot
    Remove-Item -LiteralPath $identityBoundEmptyRoot -Recurse -Force
    New-Item -ItemType Directory -Force -Path $identityBoundEmptyRoot | Out-Null
    $identityBoundEmptyCleanupRejected = $false
    try {
        Remove-MvpAcceptanceEmptyDirectoryNoFollow `
            -Path $identityBoundEmptyRoot `
            -ExpectedIdentity $identityBoundEmptyRootIdentity
    }
    catch {
        $identityBoundEmptyCleanupRejected = $_.Exception.Message -match 'no longer identifies'
    }
    Assert-True $identityBoundEmptyCleanupRejected 'Acceptance empty-root cleanup removed an empty replacement with a different identity.'
    Assert-True (Test-Path -LiteralPath $identityBoundEmptyRoot -PathType Container) 'Acceptance empty-root cleanup deleted an identity-mismatched replacement.'

    $buildSummaryReparseRoot = Join-Path $snapshotFixtureRoot 'build-summary-reparse-root'
    $buildSummaryReparseTarget = Join-Path $snapshotFixtureRoot 'build-summary-reparse-target'
    $buildSummaryReparseDirectory = Join-Path $buildSummaryReparseRoot 'build'
    New-Item -ItemType Directory -Force -Path $buildSummaryReparseRoot, $buildSummaryReparseTarget | Out-Null
    New-Item -ItemType Junction -Path $buildSummaryReparseDirectory -Target $buildSummaryReparseTarget -ErrorAction Stop | Out-Null
    $buildSummaryBytes = [Text.UTF8Encoding]::new($false).GetBytes('{"summary":"validated"}')
    $buildSummaryHasher = [Security.Cryptography.SHA256]::Create()
    try {
        $buildSummaryHash = -join ($buildSummaryHasher.ComputeHash($buildSummaryBytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $buildSummaryHasher.Dispose()
    }
    $buildSummaryReparseRejected = $false
    try {
        Copy-MvpBuildSummaryEvidence -Summary ([pscustomobject]@{
            relative_path = 'build/profile-contract-summary.json'
            content_bytes = $buildSummaryBytes
            sha256 = $buildSummaryHash
            size_bytes = [Int64]$buildSummaryBytes.LongLength
            gate_artifacts = @()
        }) -EvidenceRoot $buildSummaryReparseRoot
    }
    catch {
        $buildSummaryReparseRejected = $_.Exception.Message -match 'reparse point'
    }
    Assert-True $buildSummaryReparseRejected 'F5 build summary writing accepted a reparse-point destination directory.'
    Assert-True (
        -not (Test-Path -LiteralPath (Join-Path $buildSummaryReparseTarget 'profile-contract-summary.json'))
    ) 'F5 build summary writing followed a reparse point and wrote validated bytes outside the partial tree.'

    $comparisonReparseRoot = Join-Path $snapshotFixtureRoot 'comparison-reparse-root'
    $comparisonReparseTarget = Join-Path $snapshotFixtureRoot 'comparison-reparse-target'
    New-Item -ItemType Directory -Force -Path $comparisonReparseRoot, $comparisonReparseTarget | Out-Null
    New-Item -ItemType Junction `
        -Path (Join-Path $comparisonReparseRoot 'comparison') `
        -Target $comparisonReparseTarget `
        -ErrorAction Stop | Out-Null
    $comparisonAutomation = [pscustomobject]@{
        project_identity = 'project-identity'
        manifest_identity = 'manifest-identity'
        scene_uri = 'res://scenes/main.scene.toml'
        selected_model_resource_id = 'model-resource'
        selected_material_resource_id = 'material-resource'
        opened_project_inspection_generation = 1
        snapshot = [ordered]@{ schema_version = 1 }
        project_save_lifecycle = [ordered]@{ schema_version = 1 }
    }
    $comparisonReparseRejected = $false
    try {
        Write-MvpPersistenceComparisonEvidence `
            -EvidenceRoot $comparisonReparseRoot `
            -BaselineAutomation $comparisonAutomation `
            -AuthoringAutomation $comparisonAutomation `
            -ReopenAutomation @($comparisonAutomation, $comparisonAutomation) | Out-Null
    }
    catch {
        $comparisonReparseRejected = $_.Exception.Message -match 'reparse point'
    }
    Assert-True $comparisonReparseRejected 'F5 persistence comparison writing accepted a reparse-point destination directory.'
    Assert-True (
        -not (Test-Path -LiteralPath (Join-Path $comparisonReparseTarget 'persisted-state-before.json'))
    ) 'F5 persistence comparison writing followed a reparse point and wrote evidence outside the partial tree.'

    $nativeLeafRoot = Join-Path $snapshotFixtureRoot 'native-leaf-operations'
    $nativeLeafSource = Join-Path $nativeLeafRoot 'manifest.json.partial'
    $nativeLeafDestination = Join-Path $nativeLeafRoot 'manifest.json'
    New-Item -ItemType Directory -Force -Path $nativeLeafRoot | Out-Null
    [IO.File]::WriteAllText($nativeLeafSource, 'temporary', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($nativeLeafDestination, 'existing', [Text.UTF8Encoding]::new($false))
    $nativeLeafCollisionRejected = $false
    try {
        Move-MvpAcceptanceNewFileNoFollow `
            -SourcePath $nativeLeafSource `
            -DestinationPath $nativeLeafDestination
    }
    catch {
        $nativeLeafCollisionRejected = $true
    }
    Assert-True $nativeLeafCollisionRejected 'F5 native file publication overwrote an existing destination leaf.'
    Assert-True (Test-Path -LiteralPath $nativeLeafSource -PathType Leaf) 'F5 native file publication removed a source after rejecting an existing destination.'
    Assert-True (
        (Get-Content -LiteralPath $nativeLeafDestination -Raw) -eq 'existing'
    ) 'F5 native file publication changed an existing destination after rejection.'
    Remove-MvpAcceptanceFileNoFollow -Path $nativeLeafSource
    Assert-True (-not (Test-Path -LiteralPath $nativeLeafSource)) 'F5 native file cleanup did not delete its owned temporary leaf.'

    $nativeWriteRaceRoot = Join-Path $snapshotFixtureRoot 'native-leaf-write-race'
    $nativeWriteRaceSource = Join-Path $nativeWriteRaceRoot 'manifest.json.partial'
    $nativeWriteRaceExpectedBytes = [Text.UTF8Encoding]::new($false).GetBytes('owned-temporary')
    $nativeWriteRaceInjection = [pscustomobject]@{ attempted = $false }
    New-Item -ItemType Directory -Force -Path $nativeWriteRaceRoot | Out-Null
    $nativeWriteRaceRejected = $false
    try {
        Write-MvpAcceptanceNewFileNoFollow `
            -Path $nativeWriteRaceSource `
            -ContentBytes $nativeWriteRaceExpectedBytes `
            -BeforeReopenHook {
                param($temporaryPath)
                $nativeWriteRaceInjection.attempted = $true
                Remove-Item -LiteralPath $temporaryPath -Force -ErrorAction Stop
                [IO.File]::WriteAllText(
                    $temporaryPath,
                    'replacement-temporary',
                    [Text.UTF8Encoding]::new($false))
            } | Out-Null
    }
    catch {
        $nativeWriteRaceRejected = $_.Exception.Message -match 'content changed before verification'
    }
    Assert-True $nativeWriteRaceInjection.attempted 'F5 native file write race hook did not run before no-follow verification.'
    Assert-True $nativeWriteRaceRejected 'F5 native file creation accepted a regular-file replacement before it established the temporary identity.'
    Assert-True (
        (Get-Content -LiteralPath $nativeWriteRaceSource -Raw) -eq 'replacement-temporary'
    ) 'F5 native file creation changed the replacement temporary leaf after rejecting it.'
    Remove-MvpAcceptanceFileNoFollow -Path $nativeWriteRaceSource

    $nativeIdentityRoot = Join-Path $snapshotFixtureRoot 'native-leaf-identity'
    $nativeIdentitySource = Join-Path $nativeIdentityRoot 'manifest.json.partial'
    $nativeIdentityDestination = Join-Path $nativeIdentityRoot 'manifest.json'
    New-Item -ItemType Directory -Force -Path $nativeIdentityRoot | Out-Null
    $nativeIdentityDetails = Write-MvpAcceptanceNewFileNoFollow `
        -Path $nativeIdentitySource `
        -ContentBytes ([Text.UTF8Encoding]::new($false).GetBytes('owned-temporary')) `
        -PassThruDetails
    Assert-True (
        -not [string]::IsNullOrWhiteSpace([string]$nativeIdentityDetails.identity)
    ) 'F5 native file creation did not return the owned file identity.'
    Remove-MvpAcceptanceFileNoFollow -Path $nativeIdentitySource
    [IO.File]::WriteAllText($nativeIdentitySource, 'replacement-temporary', [Text.UTF8Encoding]::new($false))
    $nativeIdentityMoveRejected = $false
    try {
        Move-MvpAcceptanceNewFileNoFollow `
            -SourcePath $nativeIdentitySource `
            -DestinationPath $nativeIdentityDestination `
            -ExpectedSourceIdentity ([string]$nativeIdentityDetails.identity)
    }
    catch {
        $nativeIdentityMoveRejected = $_.Exception.Message -match 'identity changed'
    }
    Assert-True $nativeIdentityMoveRejected 'F5 native file publication accepted a regular-file replacement for its temporary leaf.'
    Assert-True (
        (Get-Content -LiteralPath $nativeIdentitySource -Raw) -eq 'replacement-temporary'
    ) 'F5 native file publication changed the replacement temporary leaf after rejecting it.'
    Assert-True (
        -not (Test-Path -LiteralPath $nativeIdentityDestination -PathType Leaf)
    ) 'F5 native file publication created a destination from a replacement temporary leaf.'
    $nativeIdentityCleanupRejected = $false
    try {
        Remove-MvpAcceptanceFileNoFollow `
            -Path $nativeIdentitySource `
            -ExpectedIdentity ([string]$nativeIdentityDetails.identity)
    }
    catch {
        $nativeIdentityCleanupRejected = $_.Exception.Message -match 'identity changed'
    }
    Assert-True $nativeIdentityCleanupRejected 'F5 native file cleanup removed a regular-file replacement for its temporary leaf.'
    Assert-True (
        (Get-Content -LiteralPath $nativeIdentitySource -Raw) -eq 'replacement-temporary'
    ) 'F5 native file cleanup changed the replacement temporary leaf after rejecting it.'
    Remove-MvpAcceptanceFileNoFollow -Path $nativeIdentitySource

    $nonEmptyEvidenceRoot = Join-Path $snapshotFixtureRoot 'non-empty-evidence-root'
    New-Item -ItemType Directory -Force -Path $nonEmptyEvidenceRoot | Out-Null
    $nonEmptyEvidenceFile = Join-Path $nonEmptyEvidenceRoot 'must-survive.txt'
    [IO.File]::WriteAllText($nonEmptyEvidenceFile, 'must-survive', [Text.UTF8Encoding]::new($false))
    $nonEmptyEvidenceRejected = $false
    try {
        Remove-MvpAcceptanceEmptyDirectoryNoFollow -Path $nonEmptyEvidenceRoot
    }
    catch {
        $nonEmptyEvidenceRejected = $_.Exception.Message -match 'not empty'
    }
    Assert-True $nonEmptyEvidenceRejected 'Acceptance empty-root cleanup removed a non-empty directory.'
    Assert-True (Test-Path -LiteralPath $nonEmptyEvidenceFile -PathType Leaf) 'Acceptance empty-root cleanup removed a non-empty directory artifact.'

    $identityProtectedRoot = Join-Path $snapshotFixtureRoot 'identity-protected-cleanup'
    New-Item -ItemType Directory -Force -Path $identityProtectedRoot | Out-Null
    $identityProtectedOriginalIdentity = Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $identityProtectedRoot
    Remove-Item -LiteralPath $identityProtectedRoot -Recurse -Force
    New-Item -ItemType Directory -Force -Path $identityProtectedRoot | Out-Null
    $identityProtectedFile = Join-Path $identityProtectedRoot 'replacement-must-survive.txt'
    [IO.File]::WriteAllText($identityProtectedFile, 'replacement', [Text.UTF8Encoding]::new($false))
    $identityProtectedCleanupWarnings = @(
        Remove-MvpAcceptanceStagingSnapshot `
            -SnapshotRoot $identityProtectedRoot `
            -ExpectedRootIdentity $identityProtectedOriginalIdentity 3>&1 |
            Where-Object { $_ -is [System.Management.Automation.WarningRecord] }
    )
    Assert-True (Test-Path -LiteralPath $identityProtectedFile -PathType Leaf) 'Acceptance cleanup removed a replacement root with a different identity.'
    Assert-True (
        $identityProtectedCleanupWarnings.Count -eq 1 -and
        $identityProtectedCleanupWarnings[0].Message -match 'cleanup skipped'
    ) 'Acceptance cleanup did not report that it skipped a replacement root.'

    $publishedSnapshotIdentityDetails = New-MvpAcceptanceStagingSnapshot -StagingRoot $snapshotSourceRoot -PassThru
    $publishedSnapshotIdentityRoot = [string]$publishedSnapshotIdentityDetails.snapshot_root
    $publishedSnapshotIdentity = [string]$publishedSnapshotIdentityDetails.snapshot_identity
    Assert-True (-not [string]::IsNullOrWhiteSpace($publishedSnapshotIdentity)) 'Acceptance snapshot publication did not return its cleanup identity.'
    try {
        Remove-Item -LiteralPath $publishedSnapshotIdentityRoot -Recurse -Force
        New-Item -ItemType Directory -Force -Path $publishedSnapshotIdentityRoot | Out-Null
        $publishedSnapshotReplacementFile = Join-Path $publishedSnapshotIdentityRoot 'replacement-must-survive.txt'
        [IO.File]::WriteAllText($publishedSnapshotReplacementFile, 'replacement', [Text.UTF8Encoding]::new($false))
        $publishedSnapshotCleanupWarnings = @(
            Remove-MvpAcceptanceStagingSnapshot `
                -SnapshotRoot $publishedSnapshotIdentityRoot `
                -ExpectedRootIdentity $publishedSnapshotIdentity 3>&1 |
                Where-Object { $_ -is [System.Management.Automation.WarningRecord] }
        )
        Assert-True (Test-Path -LiteralPath $publishedSnapshotReplacementFile -PathType Leaf) 'Acceptance snapshot cleanup removed a replacement published after the snapshot was created.'
        Assert-True (
            $publishedSnapshotCleanupWarnings.Count -eq 1 -and
            $publishedSnapshotCleanupWarnings[0].Message -match 'cleanup skipped'
        ) 'Acceptance snapshot cleanup did not report that it skipped a replacement published after snapshot creation.'
    }
    finally {
        if (Test-Path -LiteralPath $publishedSnapshotIdentityRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $publishedSnapshotIdentityRoot
        }
    }

    $publicationSourceRoot = Join-Path $snapshotFixtureRoot 'publication-source'
    $publicationDestinationRoot = Join-Path $snapshotFixtureRoot 'publication-destination'
    New-Item -ItemType Directory -Force -Path $publicationSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $publicationSourceRoot 'evidence.txt'), 'published', [Text.UTF8Encoding]::new($false))
    Move-MvpAcceptanceStagingDirectoryNoFollow `
        -SourcePath $publicationSourceRoot `
        -DestinationPath $publicationDestinationRoot
    Assert-True (-not (Test-Path -LiteralPath $publicationSourceRoot)) 'Acceptance publication did not remove its source root.'
    Assert-True (Test-Path -LiteralPath (Join-Path $publicationDestinationRoot 'evidence.txt') -PathType Leaf) 'Acceptance publication did not move the evidence tree.'

    $writeLeaseSourceRoot = Join-Path $snapshotFixtureRoot 'write-lease-source'
    $writeLeaseDestinationRoot = Join-Path $snapshotFixtureRoot 'write-lease-destination'
    New-Item -ItemType Directory -Force -Path $writeLeaseSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $writeLeaseSourceRoot 'evidence.txt'), 'write-lease', [Text.UTF8Encoding]::new($false))
    $writeLease = Open-MvpAcceptanceStagingWriteLease -SnapshotRoot $writeLeaseSourceRoot
    try {
        $writeLeaseReplacementBlocked = $false
        try {
            Remove-Item -LiteralPath $writeLeaseSourceRoot -Recurse -Force -ErrorAction Stop
        }
        catch {
            $writeLeaseReplacementBlocked = $true
        }
        Assert-True $writeLeaseReplacementBlocked 'Acceptance staging write lease allowed its partial root to be removed before publication.'
        Assert-True (
            (Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $writeLeaseSourceRoot) -eq $writeLease.root_identity
        ) 'Acceptance staging write lease changed its partial root identity before publication.'

        $writeLeaseSourceHandle = Take-MvpAcceptanceStagingWriteLeaseRootHandle `
            -Lease $writeLease
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $writeLeaseSourceRoot `
            -DestinationPath $writeLeaseDestinationRoot `
            -ExpectedSourceIdentity $writeLease.root_identity `
            -SourceHandle $writeLeaseSourceHandle
        Assert-True (Test-Path -LiteralPath (Join-Path $writeLeaseDestinationRoot 'evidence.txt') -PathType Leaf) 'Acceptance staging write lease did not transfer its root handle into publication.'
    }
    finally {
        Close-MvpAcceptanceStagingWriteLease -Lease $writeLease
        if (Test-Path -LiteralPath $writeLeaseDestinationRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $writeLeaseDestinationRoot
        }
    }

    $snapshotLeasePublicationSourceRoot = Join-Path $snapshotFixtureRoot 'snapshot-lease-publication-source'
    $snapshotLeasePublicationDestinationRoot = Join-Path $snapshotFixtureRoot 'snapshot-lease-publication-destination'
    New-Item -ItemType Directory -Force -Path $snapshotLeasePublicationSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $snapshotLeasePublicationSourceRoot 'staging-manifest.json'), '{}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $snapshotLeasePublicationSourceRoot 'startup-summary.json'), '{}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $snapshotLeasePublicationSourceRoot 'evidence.txt'), 'snapshot-lease-publication', [Text.UTF8Encoding]::new($false))
    $snapshotLeasePublicationWriteLease = Open-MvpAcceptanceStagingWriteLease `
        -SnapshotRoot $snapshotLeasePublicationSourceRoot
    $snapshotLeasePublicationLease = $null
    try {
        $snapshotLeasePublicationLease = Open-MvpAcceptanceStagingSnapshotLease `
            -SnapshotRoot $snapshotLeasePublicationSourceRoot `
            -ExpectedRootIdentity $snapshotLeasePublicationWriteLease.root_identity
        $publicationWithoutWriteLeaseRejected = $false
        try {
            $forgedPublicationWriteLease = [pscustomobject]@{
                lease_id = $snapshotLeasePublicationWriteLease.lease_id
                root_handle = $snapshotLeasePublicationLease.root_handle
                root_path = $snapshotLeasePublicationLease.root_path
                root_identity = $snapshotLeasePublicationLease.root_identity
            }
            Close-MvpAcceptanceStagingWriteLease -Lease $forgedPublicationWriteLease
        }
        catch {
            $publicationWithoutWriteLeaseRejected = $_.Exception.Message -match 'original registered lease'
        }
        Assert-True $publicationWithoutWriteLeaseRejected 'Acceptance staging write lease close accepted a forged lease object.'

        $publicationWithoutWriteLeaseRejected = $false
        try {
            Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication `
                -Lease $snapshotLeasePublicationLease `
                -StagingWriteLease $forgedPublicationWriteLease
        }
        catch {
            $publicationWithoutWriteLeaseRejected = $_.Exception.Message -match 'original registered staging write lease'
        }
        Assert-True $publicationWithoutWriteLeaseRejected 'Acceptance publication conversion accepted a snapshot lease without its held staging write lease.'
        Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication `
            -Lease $snapshotLeasePublicationLease `
            -StagingWriteLease $snapshotLeasePublicationWriteLease
        $snapshotLeasePublicationInjectionBlocked = $false
        try {
            [IO.File]::WriteAllText(
                (Join-Path $snapshotLeasePublicationSourceRoot 'must-not-publish.txt'),
                'ordinary-publication-injection',
                [Text.UTF8Encoding]::new($false))
        }
        catch {
            $snapshotLeasePublicationInjectionBlocked = $true
        }
        Assert-True $snapshotLeasePublicationInjectionBlocked 'Acceptance publication root lease allowed an ordinary sibling to be injected after final projection.'
        $snapshotLeasePublicationSourceHandle = Take-MvpAcceptanceStagingWriteLeaseRootHandle `
            -Lease $snapshotLeasePublicationWriteLease
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $snapshotLeasePublicationSourceRoot `
            -DestinationPath $snapshotLeasePublicationDestinationRoot `
            -ExpectedSourceIdentity $snapshotLeasePublicationWriteLease.root_identity `
            -SourceHandle $snapshotLeasePublicationSourceHandle `
            -ExcludedSourcePaths $snapshotLeasePublicationLease.marker_paths
        Assert-True (
            Test-Path -LiteralPath (Join-Path $snapshotLeasePublicationDestinationRoot 'evidence.txt') -PathType Leaf
        ) 'Acceptance publication did not transfer a snapshot-leased partial root through its write lease.'
    }
    finally {
        Close-MvpAcceptanceStagingSnapshotLease -Lease $snapshotLeasePublicationLease
        Close-MvpAcceptanceStagingWriteLease -Lease $snapshotLeasePublicationWriteLease
        if (Test-Path -LiteralPath $snapshotLeasePublicationDestinationRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $snapshotLeasePublicationDestinationRoot
        }
        if (Test-Path -LiteralPath $snapshotLeasePublicationSourceRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $snapshotLeasePublicationSourceRoot
        }
    }

    $identityProtectedPublicationSourceRoot = Join-Path $snapshotFixtureRoot 'identity-protected-publication-source'
    $identityProtectedPublicationDestinationRoot = Join-Path $snapshotFixtureRoot 'identity-protected-publication-destination'
    New-Item -ItemType Directory -Force -Path $identityProtectedPublicationSourceRoot | Out-Null
    $identityProtectedPublicationIdentity = Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $identityProtectedPublicationSourceRoot
    Remove-Item -LiteralPath $identityProtectedPublicationSourceRoot -Recurse -Force
    New-Item -ItemType Directory -Force -Path $identityProtectedPublicationSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $identityProtectedPublicationSourceRoot 'replacement.txt'), 'replacement', [Text.UTF8Encoding]::new($false))
    $identityProtectedPublicationRejected = $false
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $identityProtectedPublicationSourceRoot `
            -DestinationPath $identityProtectedPublicationDestinationRoot `
            -ExpectedSourceIdentity $identityProtectedPublicationIdentity
    }
    catch {
        $identityProtectedPublicationRejected = $_.Exception.Message -match 'no longer identifies the staged tree'
    }
    Assert-True $identityProtectedPublicationRejected 'Acceptance publication accepted a replacement source with a different identity.'
    Assert-True (Test-Path -LiteralPath (Join-Path $identityProtectedPublicationSourceRoot 'replacement.txt') -PathType Leaf) 'Acceptance publication removed a replacement source after rejecting its identity.'
    Assert-True (-not (Test-Path -LiteralPath $identityProtectedPublicationDestinationRoot)) 'Acceptance publication created a destination from a replacement source.'

    $existingPublicationSourceRoot = Join-Path $snapshotFixtureRoot 'existing-publication-source'
    New-Item -ItemType Directory -Force -Path $existingPublicationSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $existingPublicationSourceRoot 'evidence.txt'), 'source-must-survive', [Text.UTF8Encoding]::new($false))
    $existingPublicationRejected = $false
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $existingPublicationSourceRoot `
            -DestinationPath $publicationDestinationRoot
    }
    catch {
        $existingPublicationRejected = $_.Exception.Message -match 'already exists'
    }
    Assert-True $existingPublicationRejected 'Acceptance publication replaced an existing destination.'
    Assert-True (Test-Path -LiteralPath (Join-Path $existingPublicationSourceRoot 'evidence.txt') -PathType Leaf) 'Acceptance publication removed a source after rejecting its existing destination.'

    $publicationOutsideRoot = Join-Path $snapshotFixtureRoot 'publication-outside'
    $publicationJunctionRoot = Join-Path $snapshotFixtureRoot 'publication-junction'
    $junctionPublicationSourceRoot = Join-Path $snapshotFixtureRoot 'junction-publication-source'
    New-Item -ItemType Directory -Force -Path $publicationOutsideRoot, $junctionPublicationSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $junctionPublicationSourceRoot 'evidence.txt'), 'must-not-escape', [Text.UTF8Encoding]::new($false))
    New-Item -ItemType Junction -Path $publicationJunctionRoot -Target $publicationOutsideRoot -ErrorAction Stop | Out-Null
    $publicationJunctionRejected = $false
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $junctionPublicationSourceRoot `
            -DestinationPath (Join-Path $publicationJunctionRoot 'escaped')
    }
    catch {
        $publicationJunctionRejected = $_.Exception.Message -match 'reparse point'
    }
    finally {
        if (Test-Path -LiteralPath $publicationJunctionRoot) {
            [IO.Directory]::Delete($publicationJunctionRoot, $false)
        }
    }
    Assert-True $publicationJunctionRejected 'Acceptance publication followed a destination-parent junction.'
    Assert-True (Test-Path -LiteralPath (Join-Path $junctionPublicationSourceRoot 'evidence.txt') -PathType Leaf) 'Acceptance publication removed a source after rejecting a destination-parent junction.'

    $parentLeaseRoot = Join-Path $snapshotFixtureRoot 'publication-parent-lease'
    $parentLeaseMovedRoot = Join-Path $snapshotFixtureRoot 'publication-parent-lease-moved'
    $parentLeaseSourceRoot = Join-Path $parentLeaseRoot 'source'
    $parentLeaseDestinationRoot = Join-Path $parentLeaseRoot 'destination'
    New-Item -ItemType Directory -Force -Path $parentLeaseSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $parentLeaseSourceRoot 'evidence.txt'), 'parent-must-not-move', [Text.UTF8Encoding]::new($false))
    $parentLeaseAttempt = [pscustomobject]@{ blocked = $false }
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $parentLeaseSourceRoot `
            -DestinationPath $parentLeaseDestinationRoot `
            -BeforeRenameHook {
                try {
                    Move-Item -LiteralPath $parentLeaseRoot -Destination $parentLeaseMovedRoot -ErrorAction Stop
                }
                catch {
                    $parentLeaseAttempt.blocked = $true
                }
            }
        Assert-True $parentLeaseAttempt.blocked 'Acceptance publication allowed its destination parent to move during rename.'
        Assert-True (Test-Path -LiteralPath (Join-Path $parentLeaseDestinationRoot 'evidence.txt') -PathType Leaf) 'Acceptance publication did not complete after rejecting its parent move.'
    }
    finally {
        foreach ($parentLeaseCleanupRoot in @($parentLeaseRoot, $parentLeaseMovedRoot)) {
            if (Test-Path -LiteralPath $parentLeaseCleanupRoot) {
                Remove-Item -LiteralPath $parentLeaseCleanupRoot -Recurse -Force
            }
        }
    }

    $ancestorLeaseRoot = Join-Path $snapshotFixtureRoot 'publication-ancestor-lease'
    $ancestorLeaseMovedRoot = Join-Path $snapshotFixtureRoot 'publication-ancestor-lease-moved'
    $ancestorLeaseParentRoot = Join-Path $ancestorLeaseRoot 'parent'
    $ancestorLeaseSourceRoot = Join-Path $ancestorLeaseParentRoot 'source'
    $ancestorLeaseDestinationRoot = Join-Path $ancestorLeaseParentRoot 'destination'
    New-Item -ItemType Directory -Force -Path $ancestorLeaseSourceRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $ancestorLeaseSourceRoot 'evidence.txt'), 'ancestor-must-not-move', [Text.UTF8Encoding]::new($false))
    $ancestorLeaseAttempt = [pscustomobject]@{ blocked = $false }
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $ancestorLeaseSourceRoot `
            -DestinationPath $ancestorLeaseDestinationRoot `
            -BeforeRenameHook {
                try {
                    Move-Item -LiteralPath $ancestorLeaseRoot -Destination $ancestorLeaseMovedRoot -ErrorAction Stop
                }
                catch {
                    $ancestorLeaseAttempt.blocked = $true
                }
            }
        Assert-True $ancestorLeaseAttempt.blocked 'Acceptance publication allowed a destination ancestor to move during rename.'
        Assert-True (Test-Path -LiteralPath (Join-Path $ancestorLeaseDestinationRoot 'evidence.txt') -PathType Leaf) 'Acceptance publication did not complete after rejecting its ancestor move.'
    }
    finally {
        foreach ($ancestorLeaseCleanupRoot in @($ancestorLeaseRoot, $ancestorLeaseMovedRoot)) {
            if (Test-Path -LiteralPath $ancestorLeaseCleanupRoot) {
                Remove-Item -LiteralPath $ancestorLeaseCleanupRoot -Recurse -Force
            }
        }
    }

    $failedPublicationSourceRoot = Join-Path $snapshotFixtureRoot 'failed-publication-source'
    $failedPublicationDestinationRoot = Join-Path $snapshotFixtureRoot 'failed-publication-destination'
    $failedPublicationOutsideRoot = Join-Path $snapshotFixtureRoot 'failed-publication-outside'
    $failedPublicationMovedRoot = Join-Path $snapshotFixtureRoot 'failed-publication-moved'
    New-Item -ItemType Directory -Force -Path $failedPublicationSourceRoot, $failedPublicationOutsideRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $failedPublicationSourceRoot 'evidence.txt'), 'must-not-survive-failed-publication', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $failedPublicationOutsideRoot 'must-survive.txt'), 'external-target', [Text.UTF8Encoding]::new($false))
    $failedPublicationRejected = $false
    $failedPublicationRootReplacement = [pscustomobject]@{ blocked = $false }
    try {
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $failedPublicationSourceRoot `
            -DestinationPath $failedPublicationDestinationRoot `
            -AfterRenameHook {
                try {
                    Move-Item -LiteralPath $failedPublicationDestinationRoot -Destination $failedPublicationMovedRoot -ErrorAction Stop
                }
                catch {
                    $failedPublicationRootReplacement.blocked = $true
                }
                New-Item -ItemType Junction `
                    -Path (Join-Path $failedPublicationDestinationRoot 'injected-junction') `
                    -Target $failedPublicationOutsideRoot `
                    -ErrorAction Stop | Out-Null
            }
    }
    catch {
        $failedPublicationRejected = $_.Exception.Message -match 'reparse point'
    }
    finally {
        if (Test-Path -LiteralPath (Join-Path $failedPublicationDestinationRoot 'injected-junction')) {
            [IO.Directory]::Delete((Join-Path $failedPublicationDestinationRoot 'injected-junction'), $false)
        }
        if (Test-Path -LiteralPath $failedPublicationSourceRoot) {
            Remove-MvpAcceptanceStagingSnapshot -SnapshotRoot $failedPublicationSourceRoot
        }
    }
    Assert-True $failedPublicationRejected 'Acceptance publication accepted a junction injected after rename.'
    Assert-True $failedPublicationRootReplacement.blocked 'Acceptance publication allowed its cleanup root to be replaced after rename.'
    Assert-True (-not (Test-Path -LiteralPath $failedPublicationDestinationRoot)) 'Acceptance publication left a rejected destination tree in place.'
    Assert-True (Test-Path -LiteralPath (Join-Path $failedPublicationOutsideRoot 'must-survive.txt') -PathType Leaf) 'Acceptance publication cleanup followed an injected junction.'
}
finally {
    if (-not [string]::IsNullOrWhiteSpace($snapshotIdentity)) {
        Remove-MvpAcceptanceStagingSnapshot `
            -SnapshotRoot $snapshotRoot `
            -ExpectedRootIdentity $snapshotIdentity
    }
    if (Test-Path -LiteralPath $snapshotLogsRoot) {
        $snapshotLogsItem = Get-Item -LiteralPath $snapshotLogsRoot -Force
        if ([bool]($snapshotLogsItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            [IO.Directory]::Delete($snapshotLogsItem.FullName, $false)
        }
    }
    if (Test-Path -LiteralPath $snapshotFixtureRoot) {
        Remove-Item -LiteralPath $snapshotFixtureRoot -Recurse -Force
    }
}
