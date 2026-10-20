# These are real directory-instance contracts; every negative fixture has its own live owner.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$fixturePathsModule = Join-Path $PSScriptRoot '..\mvp\MvpTestFixturePaths.psm1'
$fixtureContextModule = Join-Path $PSScriptRoot '..\mvp\MvpTestFixtureContext.psm1'
Import-Module $fixturePathsModule -Force -ErrorAction Stop
Import-Module $fixtureContextModule -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpTestCleanup.psm1') -Force -ErrorAction Stop

function Assert-True {
    param([Parameter(Mandatory)][bool]$Condition, [Parameter(Mandatory)][string]$Message)
    if (-not $Condition) { throw $Message }
}

function New-CleanupFaultContext {
    param([switch]$FailJournal)
    # This fake exercises only error aggregation. It never enters Get/New or owns files.
    $journal = [pscustomobject]@{ FailAppend = $FailJournal.IsPresent }
    $journal | Add-Member -MemberType ScriptMethod -Name Append -Value {
        param([string]$Json)
        if ($this.FailAppend) { throw [IO.IOException]::new('injected fixture journal flush failure') }
    }
    $context = [pscustomobject]@{ OwnerId = 'cleanup-error-test'; Journal = $journal; ParentHandle = $null; ParentState = 'Absent'; DisposeAttempted = $false }
    $context | Add-Member -MemberType ScriptMethod -Name AssertLive -Value { }
    $context | Add-Member -MemberType ScriptMethod -Name Instances -Value { return ,@() }
    $context | Add-Member -MemberType ScriptMethod -Name HasPendingInstances -Value { return $false }
    $context | Add-Member -MemberType ScriptMethod -Name Dispose -Value {
        $this.DisposeAttempted = $true
        throw [IO.IOException]::new('injected fixture disposal failure')
    }
    return $context
}

$fixtureRoot = $null
$owner = Get-MvpTestFixtureContext
$ownedRoots = [Collections.Generic.List[string]]::new()
$testOwners = [Collections.Generic.List[object]]::new()
$lockedFile = $null
$primaryError = $null
try {
    $fixtureRoot = New-MvpTestFixtureRoot -Prefix 'paths-contract'
    $ownedRoots.Add($fixtureRoot)
    $fixtureParent = Split-Path -Parent $fixtureRoot
    $instance = $owner.Lookup("\\?\$fixtureRoot")
    Assert-True ($fixtureRoot -match '^E:\\cargo-targets\\zircon-local\\mvp-test-fixtures-[1-9][0-9]*\\paths-contract-[0-9a-f]{32}$') 'Fixture root left the default typed namespace.'
    Assert-True ($instance.State -eq 'Active' -and $null -ne $instance.Identity -and -not $instance.Handle.IsClosed) 'A real Active fixture must retain its instance handle and physical identity.'
    Assert-True ([IO.Directory]::Exists($fixtureRoot)) 'Fixture root was not physically created.'

    if ($owner.Keep) {
        Remove-MvpTestFixtureRoot -Path $fixtureRoot
        Assert-True ($instance.State -eq 'Retained' -and [IO.Directory]::Exists($fixtureRoot)) 'KEEP must retain the original physical fixture and journal state.'
        Write-Host 'MVP fixture-root KEEP contract passed; the journal retains this instance.'
        return
    }

    $outsideRejected = $false
    try { Remove-MvpTestFixtureRoot -Path 'D:\ZirconBuilds\outside-mvp-test-fixture' }
    catch { $outsideRejected = $true }
    Assert-True $outsideRejected 'Fixture cleanup accepted a path outside the typed namespace.'

    $originalHandle = $instance.Handle
    Import-Module $fixturePathsModule -Force -ErrorAction Stop
    Import-Module $fixtureContextModule -Force -ErrorAction Stop
    Assert-True ([Object]::ReferenceEquals($owner, (Get-MvpTestFixtureContext))) 'Force reimport lost the wrapper-owned living context.'
    Assert-True ([Object]::ReferenceEquals($originalHandle, $owner.Lookup("\\?\$fixtureRoot").Handle)) 'Force reimport replaced the real instance handle.'
    Remove-MvpTestFixtureRoot -Path $fixtureRoot
    Assert-True ($instance.State -eq 'Released' -and $originalHandle.IsClosed -and -not [IO.Directory]::Exists($fixtureRoot)) 'Released requires real handle disposal and physical absence.'
    Assert-True (-not [IO.Directory]::Exists($fixtureParent)) 'Cleanup left its empty owned parent.'
    Remove-MvpTestFixtureRoot -Path $fixtureRoot

    # A separate owner exclusively recreates the exact pathname. The old owner has no authority over it.
    $replacementOwner = New-MvpTestFixtureContext -ParentContext $owner -ParentName $owner.ParentName
    $testOwners.Add($replacementOwner)
    $replacement = New-MvpOwnedFixture -Context $replacementOwner -Prefix 'paths-contract' -ExactLeaf (Split-Path -Leaf $fixtureRoot)
    $replacementSentinel = Join-Path $replacement 'replacement-must-survive.txt'
    [IO.File]::WriteAllText($replacementSentinel, 'independently owned replacement')
    $replacementRejected = $false
    try { Remove-MvpTestFixtureRoot -Path $fixtureRoot }
    catch { $replacementRejected = $_.Exception.Message -match 'recreated fixture instance' }
    Assert-True ($replacementRejected -and [IO.File]::Exists($replacementSentinel)) 'A Released pathname authorized deletion of a replacement instance.'

    # FILE_CREATE must reject this existing parent. A failed reservation cannot adopt it.
    $failedOwner = New-MvpTestFixtureContext -ParentContext $owner -ParentName $owner.ParentName
    $testOwners.Add($failedOwner)
    $creationError = $null
    try { New-MvpOwnedFixture -Context $failedOwner -Prefix 'paths-contract' -ExactLeaf (Split-Path -Leaf $fixtureRoot) | Out-Null }
    catch { $creationError = $_ }
    Assert-True ($null -ne $creationError -and $creationError.Exception.Message -match 'exclusive create.*NTSTATUS') 'An existing parent was silently adopted or the original native creation error was lost.'
    $failedInstance = $failedOwner.Lookup("\\?\$fixtureRoot")
    Assert-True ($failedInstance.State -eq 'CleanupFailed' -and $null -eq $failedInstance.Handle -and [IO.File]::Exists($replacementSentinel)) 'Failed creation deleted or acquired an independently owned entry.'
    Remove-MvpOwnedFixture -Context $replacementOwner -Path $replacement
    Remove-MvpOwnedFixture -Context $failedOwner -Path $fixtureRoot
    Assert-True ($failedInstance.State -eq 'Released' -and -not [IO.Directory]::Exists($fixtureRoot)) 'A failed reservation could not reconcile real absence after the actual owner released its entry.'

    $foreignOwner = New-MvpTestFixtureContext -ParentContext $owner
    $testOwners.Add($foreignOwner)
    $foreignRoot = New-MvpOwnedFixture -Context $foreignOwner -Prefix 'paths-other-owner'
    $foreignSentinel = Join-Path $foreignRoot 'foreign-must-survive.txt'
    [IO.File]::WriteAllText($foreignSentinel, 'a different living owner')
    $foreignRejected = $false
    try { Remove-MvpTestFixtureRoot -Path $foreignRoot }
    catch { $foreignRejected = $true }
    Assert-True ($foreignRejected -and [IO.File]::Exists($foreignSentinel)) 'The consumer accepted another owner merely because its path was in the namespace.'
    Remove-MvpOwnedFixture -Context $foreignOwner -Path $foreignRoot

    # Close a real owner now; it is deliberately absent from the later cleanup list.
    $closedOwner = New-MvpTestFixtureContext -ParentContext $owner
    Close-MvpTestFixtureContext -Context $closedOwner
    try {
        [AppDomain]::CurrentDomain.SetData('ZirconEngine.MvpFixtureContext.V1', $closedOwner)
        $closedGetError = $null
        $closedNewError = $null
        try { Get-MvpTestFixtureContext | Out-Null }
        catch { $closedGetError = $_ }
        try { New-MvpTestFixtureRoot -Prefix 'paths-closed-owner' | Out-Null }
        catch { $closedNewError = $_ }
        Assert-True ($null -ne $closedGetError -and $null -ne $closedNewError) 'Get and New must both reject a real closed owner.'
        foreach ($rejection in @($closedGetError, $closedNewError)) {
            $exception = $rejection.Exception
            $typedLivenessError = $false
            while ($null -ne $exception) {
                if ($exception -is [InvalidOperationException] -and $exception.Message -match 'original living process owner') { $typedLivenessError = $true; break }
                $exception = $exception.InnerException
            }
            Assert-True $typedLivenessError 'Closed-owner rejection lost its original typed liveness failure.'
        }
    }
    finally { [AppDomain]::CurrentDomain.SetData('ZirconEngine.MvpFixtureContext.V1', $owner) }

    $first = New-MvpTestFixtureRoot -Prefix 'paths-shared-first'
    $ownedRoots.Add($first)
    $second = New-MvpTestFixtureRoot -Prefix 'paths-shared-second'
    $ownedRoots.Add($second)
    Assert-True ((Split-Path -Parent $first) -eq (Split-Path -Parent $second)) 'One owner must reuse its pinned parent.'
    Remove-MvpTestFixtureRoot -Path $first
    Assert-True ([IO.Directory]::Exists($second) -and [IO.Directory]::Exists((Split-Path -Parent $second))) 'Cleanup removed a sibling or its nonempty parent.'
    Remove-MvpTestFixtureRoot -Path $second
    Assert-True (-not [IO.Directory]::Exists((Split-Path -Parent $second))) 'Cleanup left the final empty parent.'

    $operationalRoot = New-MvpTestFixtureRoot -Prefix 'paths-operational-contract'
    $ownedRoots.Add($operationalRoot)
    Remove-MvpTestFixtureRoot -Path "\\?\$operationalRoot"
    Assert-True (-not [IO.Directory]::Exists($operationalRoot)) 'Cleanup rejected the operational path form.'

    $readOnlyRoot = New-MvpTestFixtureRoot -Prefix 'paths-readonly-contract'
    $ownedRoots.Add($readOnlyRoot)
    $nested = Join-Path $readOnlyRoot 'nested'
    [IO.Directory]::CreateDirectory($nested) | Out-Null
    $readOnlyFile = Join-Path $nested 'readonly.txt'
    [IO.File]::WriteAllText($readOnlyFile, 'read-only compatibility')
    [IO.File]::SetAttributes($readOnlyFile, [IO.FileAttributes]::ReadOnly)
    $longChild = "\\?\$nested\" + ('a' * 241) + '.txt'
    $leadingChild = Join-Path $nested ' leading.txt'
    [IO.File]::WriteAllText($longChild, '245-character ordinary child name')
    [IO.File]::WriteAllText($leadingChild, 'leading-space ordinary child name')
    [IO.File]::SetAttributes($longChild, [IO.FileAttributes]::ReadOnly)
    [IO.File]::SetAttributes($leadingChild, [IO.FileAttributes]::ReadOnly)
    $readOnlyInstance = $owner.Lookup("\\?\$readOnlyRoot")
    Remove-MvpTestFixtureRoot -Path $readOnlyRoot
    Assert-True ($readOnlyInstance.State -eq 'Released' -and $readOnlyInstance.Handle.IsClosed -and -not [IO.Directory]::Exists($readOnlyRoot)) 'Handle-relative cleanup lost read-only, long-child or leading-space compatibility.'

    $retryRoot = New-MvpTestFixtureRoot -Prefix 'paths-cleanup-retry'
    $ownedRoots.Add($retryRoot)
    $retryFile = Join-Path $retryRoot 'locked.txt'
    [IO.File]::WriteAllText($retryFile, 'locked cleanup must be retryable')
    $lockedFile = [IO.FileStream]::new($retryFile, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    $deleteFailed = $false
    try { Remove-MvpTestFixtureRoot -Path $retryRoot }
    catch { $deleteFailed = $true }
    $retryInstance = $owner.Lookup("\\?\$retryRoot")
    Assert-True ($deleteFailed -and $retryInstance.State -eq 'CleanupFailed' -and -not $retryInstance.Handle.IsClosed -and [IO.File]::Exists($retryFile)) 'A failed deletion lost its original handle, evidence or retry state.'
    $lockedFile.Dispose()
    $lockedFile = $null
    Remove-MvpTestFixtureRoot -Path $retryRoot
    Assert-True ($retryInstance.State -eq 'Released' -and $retryInstance.Handle.IsClosed -and -not [IO.Directory]::Exists($retryRoot)) 'Cleanup could not retry after the real conflicting file handle closed.'

    $junctionRoot = New-MvpTestFixtureRoot -Prefix 'paths-junction-contract'
    $ownedRoots.Add($junctionRoot)
    $junctionTarget = New-MvpTestFixtureRoot -Prefix 'paths-junction-target'
    $ownedRoots.Add($junctionTarget)
    $junctionSentinel = Join-Path $junctionTarget 'must-survive.txt'
    [IO.File]::WriteAllText($junctionSentinel, 'junction target must survive')
    New-Item -ItemType Junction -Path (Join-Path $junctionRoot 'outside-target') -Target $junctionTarget -ErrorAction Stop | Out-Null
    Remove-MvpTestFixtureRoot -Path $junctionRoot
    Assert-True (-not [IO.Directory]::Exists($junctionRoot) -and [IO.File]::Exists($junctionSentinel)) 'Cleanup followed a junction into a separate fixture root.'
    Remove-MvpTestFixtureRoot -Path $junctionTarget

    try { throw [InvalidOperationException]::new('injected original fixture primary error') }
    catch { $expectedPrimary = $_ }
    $existingFailure = [IO.IOException]::new('previous cleanup evidence')
    $expectedPrimary.Exception.Data['MvpFixtureCleanupFailures'] = @($existingFailure)
    $faultContext = New-CleanupFaultContext -FailJournal
    Close-MvpTestFixtureContext -Context $faultContext -PrimaryError $expectedPrimary
    $recordedFailures = @($expectedPrimary.Exception.Data['MvpFixtureCleanupFailures'])
    Assert-True ($faultContext.DisposeAttempted -and $expectedPrimary.Exception -is [InvalidOperationException] -and $expectedPrimary.Exception.Message -eq 'injected original fixture primary error') 'Cleanup disposal replaced the original typed primary error.'
    Assert-True ($recordedFailures.Count -eq 3 -and [Object]::ReferenceEquals($recordedFailures[0], $existingFailure) -and $recordedFailures[1].Exception.Message -match 'journal flush failure' -and $recordedFailures[2].Exception.Message -match 'disposal failure') 'Cleanup must append journal and disposal failures without discarding prior evidence.'
    $faultContext = New-CleanupFaultContext
    $cleanupError = $null
    try { Close-MvpTestFixtureContext -Context $faultContext }
    catch { $cleanupError = $_ }
    Assert-True ($faultContext.DisposeAttempted -and $null -ne $cleanupError -and $cleanupError.Exception.Message -match 'disposal failure') 'A disposal failure without a primary error was hidden.'
    Assert-True (@($cleanupError.Exception.Data['MvpFixtureCleanupFailures']).Count -eq 1) 'The visible cleanup error omitted its disposal evidence.'

    # A cold helper has no authority and must not manufacture one from metadata.
    [AppDomain]::CurrentDomain.SetData('ZirconEngine.MvpFixtureContext.V1', $null)
    try {
        Import-Module $fixturePathsModule -Force -ErrorAction Stop
        $missingOwnerRejected = $false
        try { New-MvpTestFixtureRoot -Prefix 'paths-owner-missing' | Out-Null }
        catch { $missingOwnerRejected = $_.Exception.Message -match 'living process owner' }
        Assert-True $missingOwnerRejected 'A helper import manufactured fixture ownership without the wrapper.'
    }
    finally { [AppDomain]::CurrentDomain.SetData('ZirconEngine.MvpFixtureContext.V1', $owner) }
    Import-Module $fixtureContextModule -Force -ErrorAction Stop
    Write-Host 'MVP fixture-root path and living-owner contracts passed'
}
catch { $primaryError = $_; throw }
finally {
    $steps = [ordered]@{}
    $steps['locked-file'] = { if ($null -ne $lockedFile) { $lockedFile.Dispose() } }
    for ($index = 0; $index -lt $testOwners.Count; $index++) {
        $testOwnerToClose = $testOwners[$index]
        $steps["test-owner-$index"] = { Close-MvpTestFixtureContext -Context $testOwnerToClose }.GetNewClosure()
    }
    for ($index = 0; $index -lt $ownedRoots.Count; $index++) {
        $ownedPathToRemove = $ownedRoots[$index]
        $owningContext = $owner
        $steps["fixture-$index"] = { Remove-MvpOwnedFixture -Context $owningContext -Path $ownedPathToRemove }.GetNewClosure()
    }
    Invoke-MvpTestCleanup -PrimaryError $primaryError -Steps $steps
}
