Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function ConvertFrom-MvpProcessTimestamp {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [DateTimeOffset]) {
        return ([DateTimeOffset]$Value).ToUniversalTime()
    }
    if ($Value -is [DateTime]) {
        return [DateTimeOffset]([DateTime]$Value).ToUniversalTime()
    }

    $text = [string]$Value
    $timestamp = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParseExact(
        $text,
        'o',
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$timestamp
    )) {
        throw "$Label has malformed '$Name' timestamp '$text'; expected an ISO 8601 round-trip timestamp."
    }
    return $timestamp.ToUniversalTime()
}

function Get-MvpProcessTimingWindow {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label
    )

    $properties = $Evidence.PSObject.Properties
    $startedAtProperty = $properties['started_at_utc']
    if ($null -eq $startedAtProperty -or $null -eq $startedAtProperty.Value) {
        throw "$Label is missing 'started_at_utc'."
    }
    $startedAt = ConvertFrom-MvpProcessTimestamp `
        -Value $startedAtProperty.Value `
        -Name 'started_at_utc' `
        -Label $Label

    $endedAtProperty = $properties['ended_at_utc']
    if ($null -eq $endedAtProperty -or $null -eq $endedAtProperty.Value) {
        throw "$Label is missing 'ended_at_utc'."
    }
    $endedAt = ConvertFrom-MvpProcessTimestamp `
        -Value $endedAtProperty.Value `
        -Name 'ended_at_utc' `
        -Label $Label
    if ($endedAt -lt $startedAt) {
        throw "$Label ended_at_utc precedes started_at_utc."
    }

    $exitCode = 0
    $exitCodeProperty = $properties['exit_code']
    if ($null -eq $exitCodeProperty -or $null -eq $exitCodeProperty.Value) {
        throw "$Label is missing 'exit_code'."
    }
    $rawExitCode = $exitCodeProperty.Value
    if (-not [int]::TryParse([string]$rawExitCode, [ref]$exitCode)) {
        throw "$Label has non-integer exit_code '$rawExitCode'."
    }
    if ($exitCode -ne 0) {
        throw "$Label has unsuccessful exit_code '$exitCode'."
    }
    return [pscustomobject]@{
        started_at = $startedAt
        ended_at = $endedAt
    }
}

function Assert-MvpProcessTimingEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label
    )

    $null = Get-MvpProcessTimingWindow -Evidence $Evidence -Label $Label
}

function ConvertTo-MvpCanonicalProcessTimingWindow {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label
    )

    $window = Get-MvpProcessTimingWindow -Evidence $Evidence -Label $Label
    # Windows PowerShell can deserialize ISO timestamps as local DateTime values. Normalize only
    # after validation so the immutable evidence manifest retains the Stage-owned UTC wire form.
    $Evidence.PSObject.Properties['started_at_utc'].Value = $window.started_at.ToUniversalTime().ToString('o')
    $Evidence.PSObject.Properties['ended_at_utc'].Value = $window.ended_at.ToUniversalTime().ToString('o')
    return $window
}

function Get-MvpF5ProductProcessIndex {
    param(
        [Parameter(Mandatory)]$ProductRuns
    )

    $entries = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::OrdinalIgnoreCase)
    $counts = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($process in $ProductRuns) {
        $productName = [string]$process.product
        $attemptNumber = [int]$process.attempt
        $key = $productName + "`0" + $attemptNumber
        [int]$count = 0
        if ($counts.TryGetValue($key, [ref]$count)) {
            $counts[$key] = $count + 1
        }
        else {
            $counts.Add($key, 1)
            $entries.Add($key, $process)
        }
    }
    return [pscustomobject]@{
        entries = $entries
        counts = $counts
    }
}

function Get-MvpF5ProductProcess {
    param(
        [Parameter(Mandatory)]$ProductIndex,
        [Parameter(Mandatory)][string]$Product,
        [Parameter(Mandatory)][int]$Attempt
    )

    $key = "$Product`0$Attempt"
    [int]$count = 0
    $null = $ProductIndex.counts.TryGetValue($key, [ref]$count)
    if ($count -ne 1) {
        throw "F5 process timeline requires exactly one $Product product attempt $Attempt; found $count."
    }
    return $ProductIndex.entries[$key]
}

function Assert-MvpF5ProcessTimingEvidence {
    param(
        [Parameter(Mandatory)]$ProjectCreation,
        [Parameter(Mandatory)]$ProductRuns,
        [Parameter(Mandatory)]$BaselineAutomation,
        [Parameter(Mandatory)]$AuthoringAutomation,
        [Parameter(Mandatory)]$ReopenAutomation
    )

    $products = @($ProductRuns)
    if ($products.Count -ne 5) {
        throw "F5 process timeline requires exactly five product processes; found $($products.Count)."
    }
    $reopens = @($ReopenAutomation)
    if ($reopens.Count -ne 2) {
        throw "F5 process timeline requires exactly two reopen automation processes; found $($reopens.Count)."
    }

    $productIndex = Get-MvpF5ProductProcessIndex -ProductRuns $products
    $timelineEvidence = @(
        $ProjectCreation,
        $BaselineAutomation,
        (Get-MvpF5ProductProcess -ProductIndex $productIndex -Product 'runtime' -Attempt 1),
        (Get-MvpF5ProductProcess -ProductIndex $productIndex -Product 'runtime' -Attempt 2),
        $AuthoringAutomation,
        $reopens[0],
        (Get-MvpF5ProductProcess -ProductIndex $productIndex -Product 'editor' -Attempt 1),
        $reopens[1],
        (Get-MvpF5ProductProcess -ProductIndex $productIndex -Product 'editor' -Attempt 2),
        (Get-MvpF5ProductProcess -ProductIndex $productIndex -Product 'runtime' -Attempt 3)
    )
    $timelineLabels = [string[]]@(
        'F5 project creation process',
        'F5 baseline automation process',
        'F5 runtime product attempt 1',
        'F5 runtime product attempt 2',
        'F5 authoring automation process',
        'F5 reopen automation process 1',
        'F5 editor product attempt 1',
        'F5 reopen automation process 2',
        'F5 editor product attempt 2',
        'F5 runtime product attempt 3'
    )
    $previousWindow = $null
    $previousLabel = $null
    for ($index = 0; $index -lt $timelineEvidence.Count; $index++) {
        $processLabel = $timelineLabels[$index]
        $window = ConvertTo-MvpCanonicalProcessTimingWindow `
            -Evidence $timelineEvidence[$index] `
            -Label $processLabel
        if ($null -ne $previousWindow -and $window.started_at -lt $previousWindow.ended_at) {
            throw "$processLabel overlaps or precedes completed prior process '$previousLabel'."
        }
        $previousWindow = $window
        $previousLabel = $processLabel
    }
}

Export-ModuleMember -Function Assert-MvpProcessTimingEvidence, Assert-MvpF5ProcessTimingEvidence
