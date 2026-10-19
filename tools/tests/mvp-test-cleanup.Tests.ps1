param([switch]$DiagnoseInvocationInfo)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$cleanupModule = Join-Path $PSScriptRoot 'MvpTestCleanup.psm1'
Import-Module $cleanupModule -Force -ErrorAction Stop

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Get-MvpInvocationDiagnosticSnapshot {
    param(
        [AllowNull()][System.Management.Automation.ErrorRecord]$ErrorRecord,
        [AllowNull()][System.Management.Automation.InvocationInfo]$Invocation
    )
    $command = if ($null -ne $Invocation) { $Invocation.MyCommand } else { $null }
    [pscustomobject][ordered]@{
        ErrorRecordPresent = $null -ne $ErrorRecord
        InvocationInfoPresent = $null -ne $Invocation
        ExceptionType = if ($null -ne $ErrorRecord) { $ErrorRecord.Exception.GetType().FullName } else { $null }
        ExceptionMessage = if ($null -ne $ErrorRecord) { $ErrorRecord.Exception.Message } else { $null }
        FullyQualifiedErrorId = if ($null -ne $ErrorRecord) { $ErrorRecord.FullyQualifiedErrorId } else { $null }
        Category = if ($null -ne $ErrorRecord) { $ErrorRecord.CategoryInfo.Category.ToString() } else { $null }
        ScriptName = if ($null -ne $Invocation) { $Invocation.ScriptName } else { $null }
        PSCommandPath = if ($null -ne $Invocation) { $Invocation.PSCommandPath } else { $null }
        ScriptLineNumber = if ($null -ne $Invocation) { $Invocation.ScriptLineNumber } else { $null }
        OffsetInLine = if ($null -ne $Invocation) { $Invocation.OffsetInLine } else { $null }
        Line = if ($null -ne $Invocation) { $Invocation.Line } else { $null }
        PositionMessage = if ($null -ne $Invocation) { $Invocation.PositionMessage } else { $null }
        InvocationName = if ($null -ne $Invocation) { $Invocation.InvocationName } else { $null }
        MyCommandName = if ($null -ne $command) { $command.Name } else { $null }
        MyCommandType = if ($null -ne $command) { $command.CommandType.ToString() } else { $null }
        ScriptStackTrace = if ($null -ne $ErrorRecord) { $ErrorRecord.ScriptStackTrace } else { $null }
    }
}

function Get-MvpInvocationDiagnosticChanges {
    param($Before, $After)
    if ($null -eq $Before -or $null -eq $After) { return 'SnapshotMissing' }
    $fields = @(
        'ErrorRecordPresent', 'InvocationInfoPresent', 'ExceptionType', 'ExceptionMessage',
        'FullyQualifiedErrorId', 'Category', 'ScriptName', 'PSCommandPath', 'ScriptLineNumber',
        'OffsetInLine', 'Line', 'PositionMessage', 'InvocationName', 'MyCommandName',
        'MyCommandType', 'ScriptStackTrace'
    )
    foreach ($snapshot in @($Before, $After)) {
        $names = [Collections.Generic.HashSet[string]]::new(
            [string[]]@($snapshot.PSObject.Properties.Name), [StringComparer]::Ordinal)
        if (-not $names.SetEquals([string[]]$fields)) { 'SnapshotSchema' }
    }
    foreach ($name in $fields) {
        $beforeProperty = $Before.PSObject.Properties[$name]
        $afterProperty = $After.PSObject.Properties[$name]
        if ($null -eq $beforeProperty -or $null -eq $afterProperty) { continue }
        $beforeValue = $beforeProperty.Value
        $afterValue = $afterProperty.Value
        $bothPresent = $null -ne $beforeValue -and $null -ne $afterValue
        if (($null -eq $beforeValue) -ne ($null -eq $afterValue) -or
            ($bothPresent -and ($beforeValue.GetType() -ne $afterValue.GetType() -or
                -not [object]::Equals($beforeValue, $afterValue)))) { $name }
    }
}

function Invoke-CleanupScenario {
    param(
        [AllowNull()][System.Management.Automation.ErrorRecord]$Failure,
        [Collections.IDictionary]$Steps,
        [pscustomobject]$Evidence
    )

    $primaryError = $null
    try {
        if ($null -ne $Failure) { throw $Failure }
    }
    catch {
        $primaryError = $_
        $Evidence.PrimaryError = $_
        $Evidence.PrimaryException = $_.Exception
        $Evidence.PrimarySnapshot = Get-MvpInvocationDiagnosticSnapshot $_ $_.InvocationInfo
        throw
    }
    finally {
        Invoke-MvpTestCleanup -PrimaryError $primaryError -Steps $Steps
    }
    Write-Output 'passed'
}

if ($DiagnoseInvocationInfo) {
    & {
        function Invoke-MvpInvocationDiagnosticScenario {
            param(
                [AllowNull()][System.Management.Automation.ErrorRecord]$Failure,
                [Collections.IDictionary]$Steps,
                [bool]$UseHelper,
                [pscustomobject]$Evidence
            )
            $primaryError = $null
            try {
                if ($null -ne $Failure) { throw $Failure }
            }
            catch {
                $primaryError = $_
                $Evidence.InnerError = $primaryError
                $Evidence.InnerInvocation = $primaryError.InvocationInfo
                $Evidence.InnerSnapshot = Get-MvpInvocationDiagnosticSnapshot $primaryError $Evidence.InnerInvocation
                $Evidence.InnerGetterStable = [object]::ReferenceEquals($Evidence.InnerInvocation, $primaryError.InvocationInfo)
                throw
            }
            finally {
                $Evidence.BeforeInvocation = $primaryError.InvocationInfo
                $Evidence.BeforeSnapshot = Get-MvpInvocationDiagnosticSnapshot $primaryError $Evidence.BeforeInvocation
                $Evidence.BeforeGetterStable = [object]::ReferenceEquals($Evidence.BeforeInvocation, $primaryError.InvocationInfo)
                try {
                    if ($UseHelper) {
                        $Evidence.HelperInvoked = $true
                        Invoke-MvpTestCleanup -PrimaryError $primaryError -Steps $Steps
                        $Evidence.HelperReturned = $true
                    }
                }
                finally {
                    $Evidence.AfterInvocation = $primaryError.InvocationInfo
                    $Evidence.AfterSnapshot = Get-MvpInvocationDiagnosticSnapshot $primaryError $Evidence.AfterInvocation
                    $Evidence.AfterGetterStable = [object]::ReferenceEquals($Evidence.AfterInvocation, $primaryError.InvocationInfo)
                }
            }
            Write-Output 'passed'
        }

        $results = [Collections.Generic.List[object]]::new()
        foreach ($caseName in @('no-helper', 'helper-empty', 'helper-success', 'helper-failures')) {
            $events = [Collections.Generic.List[string]]::new()
            $warnings = [Collections.Generic.List[string]]::new()
            $output = [Collections.Generic.List[object]]::new()
            $evidence = [pscustomobject]@{
                InnerError = $null; InnerInvocation = $null; InnerSnapshot = $null; InnerGetterStable = $null
                BeforeInvocation = $null; BeforeSnapshot = $null; BeforeGetterStable = $null
                AfterInvocation = $null; AfterSnapshot = $null; AfterGetterStable = $null
                HelperInvoked = $false; HelperReturned = $null
            }
            $target = [pscustomobject]@{ Name = 'diagnostic primary target' }
            $primary = [System.Management.Automation.ErrorRecord]::new(
                [InvalidOperationException]::new('diagnostic primary failure'), 'MvpCleanupDiagnosticPrimary',
                [System.Management.Automation.ErrorCategory]::InvalidOperation, $target)
            $firstCleanup = [System.Management.Automation.ErrorRecord]::new(
                [IO.IOException]::new('diagnostic first cleanup failure'), 'MvpCleanupDiagnosticFirst',
                [System.Management.Automation.ErrorCategory]::CloseError, 'diagnostic first resource')
            $secondCleanup = [System.Management.Automation.ErrorRecord]::new(
                [UnauthorizedAccessException]::new('diagnostic second cleanup failure'), 'MvpCleanupDiagnosticSecond',
                [System.Management.Automation.ErrorCategory]::PermissionDenied, 'diagnostic second resource')
            $steps = [ordered]@{}
            if ($caseName -eq 'helper-success') {
                $steps.Add('last', { $events.Add('last') })
            }
            elseif ($caseName -eq 'helper-failures') {
                $steps.Add('first', { $events.Add('first'); throw $firstCleanup })
                $steps.Add('second', { $events.Add('second'); throw $secondCleanup })
                $steps.Add('last', { $events.Add('last') })
            }
            $caught = $null
            $outerInvocation = $null
            $outerSnapshot = $null
            $outerGetterStable = $null
            try {
                Invoke-MvpInvocationDiagnosticScenario -Failure $primary -Steps $steps -UseHelper ($caseName -ne 'no-helper') -Evidence $evidence 3>&1 | ForEach-Object {
                    if ($_ -is [System.Management.Automation.WarningRecord]) { $warnings.Add([string]$_.Message) }
                    else { $output.Add($_) }
                }
            }
            catch {
                $caught = $_
                $outerInvocation = $caught.InvocationInfo
                $outerSnapshot = Get-MvpInvocationDiagnosticSnapshot $caught $outerInvocation
                $outerGetterStable = [object]::ReferenceEquals($outerInvocation, $caught.InvocationInfo)
            }
            $retainedInvocation = if ($null -ne $evidence.InnerError) { $evidence.InnerError.InvocationInfo } else { $null }
            $retainedRecordSnapshot = Get-MvpInvocationDiagnosticSnapshot $evidence.InnerError $retainedInvocation
            $retainedInvocationSnapshot = Get-MvpInvocationDiagnosticSnapshot $evidence.InnerError $evidence.InnerInvocation
            $cleanupFailureSteps = @(
                if ($null -ne $evidence.InnerError) {
                    foreach ($failure in $evidence.InnerError.Exception.Data['MvpTestCleanupFailures']) { [string]$failure.Step }
                }
            )
            $hasBothErrors = $null -ne $evidence.InnerError -and $null -ne $caught
            $results.Add([pscustomobject][ordered]@{
                Case = $caseName
                HelperInvoked = $evidence.HelperInvoked
                HelperReturned = $evidence.HelperReturned
                Events = $events.ToArray()
                Warnings = $warnings.ToArray()
                SuccessOutputCount = $output.Count
                CleanupFailureSteps = $cleanupFailureSteps
                References = [pscustomobject][ordered]@{
                    InputToInnerErrorRecord = [object]::ReferenceEquals($primary, $evidence.InnerError)
                    InnerToOuterErrorRecord = [object]::ReferenceEquals($evidence.InnerError, $caught)
                    InnerToOuterException = $hasBothErrors -and [object]::ReferenceEquals($evidence.InnerError.Exception, $caught.Exception)
                    InnerToOuterTarget = $hasBothErrors -and [object]::ReferenceEquals($evidence.InnerError.TargetObject, $caught.TargetObject)
                    InnerToBeforeInvocation = [object]::ReferenceEquals($evidence.InnerInvocation, $evidence.BeforeInvocation)
                    BeforeToAfterInvocation = [object]::ReferenceEquals($evidence.BeforeInvocation, $evidence.AfterInvocation)
                    InnerToOuterInvocation = [object]::ReferenceEquals($evidence.InnerInvocation, $outerInvocation)
                    InnerToRetainedRecordInvocation = [object]::ReferenceEquals($evidence.InnerInvocation, $retainedInvocation)
                    OuterToRetainedRecordInvocation = [object]::ReferenceEquals($outerInvocation, $retainedInvocation)
                    InnerGetterStable = $evidence.InnerGetterStable
                    BeforeGetterStable = $evidence.BeforeGetterStable
                    AfterGetterStable = $evidence.AfterGetterStable
                    OuterGetterStable = $outerGetterStable
                }
                Snapshots = [pscustomobject][ordered]@{
                    InnerCatch = $evidence.InnerSnapshot
                    BeforeCleanup = $evidence.BeforeSnapshot
                    AfterCleanup = $evidence.AfterSnapshot
                    OuterCatch = $outerSnapshot
                    RetainedInnerRecord = $retainedRecordSnapshot
                    RetainedInnerInvocation = $retainedInvocationSnapshot
                }
                ChangedFields = [pscustomobject][ordered]@{
                    InnerToBefore = @(Get-MvpInvocationDiagnosticChanges $evidence.InnerSnapshot $evidence.BeforeSnapshot)
                    BeforeToAfter = @(Get-MvpInvocationDiagnosticChanges $evidence.BeforeSnapshot $evidence.AfterSnapshot)
                    InnerToOuter = @(Get-MvpInvocationDiagnosticChanges $evidence.InnerSnapshot $outerSnapshot)
                    InnerToRetainedRecord = @(Get-MvpInvocationDiagnosticChanges $evidence.InnerSnapshot $retainedRecordSnapshot)
                    InnerToRetainedInvocation = @(Get-MvpInvocationDiagnosticChanges $evidence.InnerSnapshot $retainedInvocationSnapshot)
                }
            })
        }
        $report = [pscustomobject][ordered]@{
            Kind = 'MvpTestCleanupInvocationInfoDiagnostic'
            SchemaVersion = 2
            PowerShellVersion = $PSVersionTable.PSVersion.ToString()
            Cases = $results.ToArray()
        }
        Write-Output ('MVP_CLEANUP_INVOCATION_DIAGNOSTIC ' + ($report | ConvertTo-Json -Depth 8 -Compress))
        foreach ($result in $results) {
            $caseName = $result.Case
            foreach ($snapshot in $result.Snapshots.PSObject.Properties) {
                Assert-True ($snapshot.Value.ErrorRecordPresent -and $snapshot.Value.InvocationInfoPresent) "Diagnostic '$caseName' lost error or invocation evidence at '$($snapshot.Name)'."
            }
            foreach ($comparison in $result.ChangedFields.PSObject.Properties) {
                Assert-True (@($comparison.Value).Count -eq 0) "Diagnostic '$caseName' changed invocation fields at '$($comparison.Name)': $($comparison.Value -join ',')."
            }
            Assert-True ($result.References.InnerToOuterException -and $result.References.InnerToOuterTarget) "Diagnostic '$caseName' replaced the primary exception or target."
            Assert-True ($result.SuccessOutputCount -eq 0) "Diagnostic '$caseName' emitted success output for a primary failure."
            $usesHelper = $caseName -ne 'no-helper'
            Assert-True ($result.HelperInvoked -eq $usesHelper) "Diagnostic '$caseName' did not follow its helper path."
            if ($usesHelper) {
                Assert-True ($result.HelperReturned -eq $true) "Diagnostic '$caseName' did not return from cleanup."
            }
            else {
                Assert-True ($null -eq $result.HelperReturned) "Diagnostic '$caseName' unexpectedly returned from cleanup."
            }
            if ($caseName -eq 'helper-failures') {
                Assert-True (($result.Events -join ',') -ceq 'first,second,last') 'Diagnostic cleanup failures stopped later resources or changed step order.'
                Assert-True (($result.CleanupFailureSteps -join ',') -ceq 'first,second') 'Diagnostic cleanup failures lost their ordered records.'
                Assert-True ($result.Warnings.Count -eq 2 -and
                    $result.Warnings[0] -ceq "MVP test cleanup 'first' failed: diagnostic first cleanup failure" -and
                    $result.Warnings[1] -ceq "MVP test cleanup 'second' failed: diagnostic second cleanup failure") 'Diagnostic cleanup failures lost their warning records.'
            }
            else {
                $expectedEvents = if ($caseName -eq 'helper-success') { 'last' } else { '' }
                Assert-True (($result.Events -join ',') -ceq $expectedEvents) "Diagnostic '$caseName' changed its cleanup steps."
                Assert-True ($result.CleanupFailureSteps.Count -eq 0 -and $result.Warnings.Count -eq 0) "Diagnostic '$caseName' invented cleanup failures."
            }
        }
    }
}

$localScopeModule = New-Module -ArgumentList $cleanupModule -ScriptBlock {
    param($CleanupModule)
    Import-Module $CleanupModule -ErrorAction Stop

    function Add-PrivateCleanupEvent {
        param($Events, [string]$Name)
        $Events.Add($Name)
    }

    function Invoke-LocalCleanupScenario {
        $localEvents = [Collections.Generic.List[string]]::new()
        $localResource = [IO.MemoryStream]::new([byte[]]@(1, 2, 3))
        Invoke-MvpTestCleanup -Steps ([ordered]@{
            'local stream' = {
                Add-PrivateCleanupEvent -Events $localEvents -Name 'close'
                $localResource.Dispose()
            }
            'private helper' = { Add-PrivateCleanupEvent -Events $localEvents -Name 'after-close' }
        })
        [pscustomobject]@{ Closed = -not $localResource.CanRead; Events = $localEvents.ToArray() }
    }
    Export-ModuleMember -Function Invoke-LocalCleanupScenario
}
$localResult = & $localScopeModule { Invoke-LocalCleanupScenario }
Assert-True ($localResult.Closed) 'Cleanup callbacks did not close a function-local resource across the module boundary.'
Assert-True (($localResult.Events -join ',') -eq 'close,after-close') 'Cleanup callbacks lost the private helper or their declared order.'

$events = [Collections.Generic.List[string]]::new()
$warnings = [Collections.Generic.List[System.Management.Automation.WarningRecord]]::new()
$output = [Collections.Generic.List[object]]::new()
$evidence = [pscustomobject]@{ PrimaryError = $null; PrimaryException = $null; PrimarySnapshot = $null }
$target = [pscustomobject]@{ Name = 'primary target' }
$primary = [System.Management.Automation.ErrorRecord]::new(
    [InvalidOperationException]::new('primary failure'), 'MvpCleanupPrimary',
    [System.Management.Automation.ErrorCategory]::InvalidOperation, $target)
$firstCleanup = [System.Management.Automation.ErrorRecord]::new(
    [IO.IOException]::new('first cleanup failure'), 'MvpCleanupFirst',
    [System.Management.Automation.ErrorCategory]::CloseError, 'first resource')
$secondCleanup = [System.Management.Automation.ErrorRecord]::new(
    [UnauthorizedAccessException]::new('second cleanup failure'), 'MvpCleanupSecond',
    [System.Management.Automation.ErrorCategory]::PermissionDenied, 'second resource')
$caught = $null
try {
    Invoke-CleanupScenario -Failure $primary -Evidence $evidence -Steps ([ordered]@{
        first = { $events.Add('first'); throw $firstCleanup }
        second = { $events.Add('second'); throw $secondCleanup }
        last = { $events.Add('last') }
    }) 3>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.WarningRecord]) { $warnings.Add($_) }
        else { $output.Add($_) }
    }
}
catch { $caught = $_ }
Assert-True ($null -ne $caught) 'A primary failure was swallowed by cleanup.'
Assert-True ([object]::ReferenceEquals($caught.Exception, $evidence.PrimaryException)) 'Cleanup replaced the primary exception.'
Assert-True ([object]::ReferenceEquals($caught.TargetObject, $target)) 'Cleanup replaced the primary error target.'
Assert-True ($caught.FullyQualifiedErrorId -ceq $evidence.PrimarySnapshot.FullyQualifiedErrorId) 'Cleanup changed the primary error identity.'
Assert-True ($caught.CategoryInfo.Category.ToString() -ceq $evidence.PrimarySnapshot.Category) 'Cleanup changed the primary error category.'
$caughtSnapshot = Get-MvpInvocationDiagnosticSnapshot $caught $caught.InvocationInfo
$changedFields = @(Get-MvpInvocationDiagnosticChanges $evidence.PrimarySnapshot $caughtSnapshot)
Assert-True ($evidence.PrimarySnapshot.InvocationInfoPresent -and $changedFields.Count -eq 0) "Cleanup changed the primary invocation evidence: $($changedFields -join ',')."
Assert-True ($caught.ScriptStackTrace -ceq $evidence.PrimarySnapshot.ScriptStackTrace) 'Cleanup changed the primary stack trace.'
$diagnostics = @($caught.Exception.Data['MvpTestCleanupFailures'])
Assert-True ($diagnostics.Count -eq 2) 'Cleanup did not retain every cleanup error.'
Assert-True (($diagnostics.Step -join ',') -eq 'first,second') 'Cleanup diagnostics lost their step order.'
Assert-True ([object]::ReferenceEquals($diagnostics[0].ErrorRecord.Exception, $firstCleanup.Exception)) 'Cleanup diagnostics replaced the first typed error.'
Assert-True ([object]::ReferenceEquals($diagnostics[1].ErrorRecord.Exception, $secondCleanup.Exception)) 'Cleanup diagnostics replaced the second typed error.'
Assert-True (($events -join ',') -eq 'first,second,last') 'Cleanup stopped before releasing later resources.'
Assert-True ($warnings.Count -eq 2 -and $warnings[0].Message -match 'first.*first cleanup failure' -and $warnings[1].Message -match 'second.*second cleanup failure') 'Cleanup diagnostics did not reach the warning stream.'
Assert-True ($output.Count -eq 0) 'A failed scenario emitted its passed message.'

$events.Clear()
$output.Clear()
$warnings.Clear()
$caught = $null
try {
    Invoke-CleanupScenario -Failure $null -Evidence $evidence -Steps ([ordered]@{
        first = { $events.Add('first'); throw $firstCleanup }
        last = { $events.Add('last') }
    }) 3>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.WarningRecord]) { $warnings.Add($_) }
        else { $output.Add($_) }
    }
}
catch { $caught = $_ }
Assert-True ($null -ne $caught -and [object]::ReferenceEquals($caught.Exception, $firstCleanup.Exception)) 'Cleanup-only failure did not report the original typed cleanup error.'
Assert-True (($events -join ',') -eq 'first,last') 'Cleanup-only failure prevented the later resource release.'
Assert-True ($output.Count -eq 0 -and $warnings.Count -eq 1) 'Cleanup-only failure emitted passed or lost its diagnostic.'

$nestedPrimary = [System.Management.Automation.ErrorRecord]::new(
    [InvalidOperationException]::new('nested primary failure'), 'MvpCleanupNested',
    [System.Management.Automation.ErrorCategory]::InvalidOperation, 'nested target')
$nestedWarnings = [Collections.Generic.List[System.Management.Automation.WarningRecord]]::new()
$caught = $null
try {
    & {
        $outerError = $null
        try {
            $innerError = $null
            try { throw $nestedPrimary }
            catch { $innerError = $_; throw }
            finally {
                Invoke-MvpTestCleanup -PrimaryError $innerError -Steps ([ordered]@{
                    inner = { throw [IO.IOException]::new('inner cleanup failure') }
                })
            }
        }
        catch { $outerError = $_; throw }
        finally {
            Invoke-MvpTestCleanup -PrimaryError $outerError -Steps ([ordered]@{
                outer = { throw [IO.IOException]::new('outer cleanup failure') }
            })
        }
    } 3>&1 | ForEach-Object { $nestedWarnings.Add($_) }
}
catch { $caught = $_ }
Assert-True ($null -ne $caught -and [object]::ReferenceEquals($caught.Exception, $nestedPrimary.Exception)) 'Nested cleanup replaced the primary exception.'
Assert-True ((@($caught.Exception.Data['MvpTestCleanupFailures']).Step -join ',') -eq 'inner,outer') 'Outer cleanup overwrote inner cleanup diagnostics.'
Assert-True ($nestedWarnings.Count -eq 2) 'Nested cleanup did not emit both failure diagnostics.'

$success = @(Invoke-CleanupScenario -Failure $null -Evidence $evidence -Steps ([ordered]@{
    release = { $events.Add('success release'); Write-Output 'cleanup implementation output' }
}))
Assert-True ($success.Count -eq 1 -and $success[0] -eq 'passed') 'A clean scenario lost passed or leaked cleanup output.'

Write-Output 'MVP test cleanup contract passed'
