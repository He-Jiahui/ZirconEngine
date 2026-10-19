Set-StrictMode -Version Latest

function Invoke-MvpTestCleanup {
    [CmdletBinding()]
    param(
        [AllowNull()][System.Management.Automation.ErrorRecord]$PrimaryError,
        [Parameter(Mandatory)][Collections.IDictionary]$Steps
    )

    $ErrorActionPreference = 'Stop'
    $failures = [Collections.Generic.List[object]]::new()
    foreach ($step in $Steps.GetEnumerator()) {
        try {
            & $step.Value | Out-Null
        }
        catch {
            $failures.Add([pscustomobject]@{
                Step = [string]$step.Key
                ErrorRecord = $_
            })
            Write-Warning -Message ("MVP test cleanup '{0}' failed: {1}" -f $step.Key, $_.Exception.Message) -WarningAction Continue
        }
    }
    if ($failures.Count -eq 0) {
        return
    }

    $failureToReport = if ($null -ne $PrimaryError) { $PrimaryError } else { $failures[0].ErrorRecord }
    $allFailures = [Collections.Generic.List[object]]::new()
    foreach ($failure in $failureToReport.Exception.Data['MvpTestCleanupFailures']) {
        $allFailures.Add($failure)
    }
    foreach ($failure in $failures) {
        $allFailures.Add($failure)
    }
    $failureToReport.Exception.Data['MvpTestCleanupFailures'] = $allFailures.ToArray()

    # A caller with a primary failure keeps its bare throw in flight through finally.
    if ($null -eq $PrimaryError) {
        $PSCmdlet.ThrowTerminatingError($failureToReport)
    }
}

Export-ModuleMember -Function Invoke-MvpTestCleanup
