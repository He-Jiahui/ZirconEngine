Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-MvpProcessTimingProperty {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Evidence.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

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

    $startedAt = ConvertFrom-MvpProcessTimestamp `
        -Value (Get-MvpProcessTimingProperty -Evidence $Evidence -Name 'started_at_utc' -Label $Label) `
        -Name 'started_at_utc' `
        -Label $Label
    $endedAt = ConvertFrom-MvpProcessTimestamp `
        -Value (Get-MvpProcessTimingProperty -Evidence $Evidence -Name 'ended_at_utc' -Label $Label) `
        -Name 'ended_at_utc' `
        -Label $Label
    if ($endedAt -lt $startedAt) {
        throw "$Label ended_at_utc precedes started_at_utc."
    }

    $exitCode = 0
    $rawExitCode = Get-MvpProcessTimingProperty -Evidence $Evidence -Name 'exit_code' -Label $Label
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

function Get-MvpF5ProductProcess {
    param(
        [Parameter(Mandatory)]$ProductRuns,
        [Parameter(Mandatory)][string]$Product,
        [Parameter(Mandatory)][int]$Attempt
    )

    $matches = @($ProductRuns | Where-Object {
        [string]$_.product -eq $Product -and [int]$_.attempt -eq $Attempt
    })
    if ($matches.Count -ne 1) {
        throw "F5 process timeline requires exactly one $Product product attempt $Attempt; found $($matches.Count)."
    }
    return $matches[0]
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

    $timeline = @(
        [pscustomobject]@{ label = 'F5 project creation process'; evidence = $ProjectCreation },
        [pscustomobject]@{ label = 'F5 baseline automation process'; evidence = $BaselineAutomation },
        [pscustomobject]@{ label = 'F5 runtime product attempt 1'; evidence = Get-MvpF5ProductProcess -ProductRuns $products -Product 'runtime' -Attempt 1 },
        [pscustomobject]@{ label = 'F5 runtime product attempt 2'; evidence = Get-MvpF5ProductProcess -ProductRuns $products -Product 'runtime' -Attempt 2 },
        [pscustomobject]@{ label = 'F5 authoring automation process'; evidence = $AuthoringAutomation },
        [pscustomobject]@{ label = 'F5 reopen automation process 1'; evidence = $reopens[0] },
        [pscustomobject]@{ label = 'F5 editor product attempt 1'; evidence = Get-MvpF5ProductProcess -ProductRuns $products -Product 'editor' -Attempt 1 },
        [pscustomobject]@{ label = 'F5 reopen automation process 2'; evidence = $reopens[1] },
        [pscustomobject]@{ label = 'F5 editor product attempt 2'; evidence = Get-MvpF5ProductProcess -ProductRuns $products -Product 'editor' -Attempt 2 },
        [pscustomobject]@{ label = 'F5 runtime product attempt 3'; evidence = Get-MvpF5ProductProcess -ProductRuns $products -Product 'runtime' -Attempt 3 }
    )
    $previousWindow = $null
    $previousLabel = $null
    foreach ($process in $timeline) {
        $window = ConvertTo-MvpCanonicalProcessTimingWindow `
            -Evidence $process.evidence `
            -Label $process.label
        if ($null -ne $previousWindow -and $window.started_at -lt $previousWindow.ended_at) {
            throw "$($process.label) overlaps or precedes completed prior process '$previousLabel'."
        }
        $previousWindow = $window
        $previousLabel = $process.label
    }
}

Export-ModuleMember -Function Assert-MvpProcessTimingEvidence, Assert-MvpF5ProcessTimingEvidence
