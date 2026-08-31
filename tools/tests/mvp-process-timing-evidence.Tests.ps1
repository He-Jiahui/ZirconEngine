Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpProcessTimingEvidence.psm1'
$moduleSource = Get-Content -LiteralPath $modulePath -Raw
Import-Module $modulePath -Force -ErrorAction Stop
$baseTimestamp = [DateTimeOffset]::Parse('2026-08-27T00:00:00.0000000+00:00')

function New-MvpProcessTiming {
    param(
        [Parameter(Mandatory)][int]$Second,
        [string]$Product,
        [int]$Attempt = 0
    )

    $timing = [pscustomobject]@{
        started_at_utc = $baseTimestamp.AddSeconds($Second).ToString('o')
        ended_at_utc = $baseTimestamp.AddSeconds($Second + 1).ToString('o')
        exit_code = 0
    }
    if (-not [string]::IsNullOrEmpty($Product)) {
        $timing | Add-Member -NotePropertyName product -NotePropertyValue $Product
        $timing | Add-Member -NotePropertyName attempt -NotePropertyValue $Attempt
    }
    return $timing
}

function New-MvpF5ProcessTimelineFixture {
    $runtime1 = New-MvpProcessTiming -Second 2 -Product 'runtime' -Attempt 1
    $runtime2 = New-MvpProcessTiming -Second 3 -Product 'runtime' -Attempt 2
    $editor1 = New-MvpProcessTiming -Second 6 -Product 'editor' -Attempt 1
    $editor2 = New-MvpProcessTiming -Second 8 -Product 'editor' -Attempt 2
    $runtime3 = New-MvpProcessTiming -Second 9 -Product 'runtime' -Attempt 3
    $reopen1 = New-MvpProcessTiming -Second 5
    $reopen2 = New-MvpProcessTiming -Second 7
    return [pscustomobject]@{
        creation = New-MvpProcessTiming -Second 0
        baseline = New-MvpProcessTiming -Second 1
        products = @($runtime1, $runtime2, $editor1, $editor2, $runtime3)
        authoring = New-MvpProcessTiming -Second 4
        reopens = @($reopen1, $reopen2)
    }
}

Describe 'MVP process-timing evidence' {
    It 'resolves fixed timing fields from one cached property collection' {
        $moduleSource | Should Match '\$properties\s*=\s*\$Evidence\.PSObject\.Properties'
        $moduleSource | Should Not Match 'function Get-MvpProcessTimingProperty'
    }

    It 'indexes product attempts once without repeated pipeline filtering' {
        $moduleSource | Should Match 'function Get-MvpF5ProductProcessIndex'
        $moduleSource | Should Match 'Dictionary\[string, object\]\]::new\(\[StringComparer\]::OrdinalIgnoreCase\)'
        $moduleSource | Should Match 'Dictionary\[string, int\]\]::new\(\[StringComparer\]::OrdinalIgnoreCase\)'
        ([regex]::Matches($moduleSource, 'Get-MvpF5ProductProcessIndex\s+-ProductRuns')).Count | Should Be 1
        $moduleSource | Should Match 'Get-MvpF5ProductProcess\s+-ProductIndex'
        $moduleSource | Should Not Match '\$ProductRuns\s+\|\s+Where-Object'
    }

    It 'builds product-attempt keys from typed scalars without subexpressions' {
        $moduleSource | Should Match '\$productName\s*=\s*\[string\]\$process\.product'
        $moduleSource | Should Match '\$attemptNumber\s*=\s*\[int\]\$process\.attempt'
        $moduleSource | Should Match '\$key\s*=\s*\$productName\s*\+\s*"`0"\s*\+\s*\$attemptNumber'
        $moduleSource | Should Not Match '\$\(\s*\[string\]\$process\.product\)'
    }

    It 'uses parallel timeline arrays without allocating one object per step' {
        $moduleSource | Should Match '\$timelineEvidence\s*=\s*@\('
        $moduleSource | Should Match '\$timelineLabels\s*=\s*\[string\[\]\]@\('
        $moduleSource | Should Not Match '\[pscustomobject\]@\{ label ='
    }

    It 'accepts the ordered ten-step F5 process timeline' {
        $fixture = New-MvpF5ProcessTimelineFixture

        {
            Assert-MvpF5ProcessTimingEvidence `
                -ProjectCreation $fixture.creation `
                -ProductRuns $fixture.products `
                -BaselineAutomation $fixture.baseline `
                -AuthoringAutomation $fixture.authoring `
                -ReopenAutomation $fixture.reopens
        } | Should Not Throw

        $fixture.products[0].started_at_utc | Should Be '2026-08-27T00:00:02.0000000+00:00'
    }

    It 'rejects a duplicate product attempt' {
        $fixture = New-MvpF5ProcessTimelineFixture
        $fixture.products[4].product = 'runtime'
        $fixture.products[4].attempt = 1

        {
            Assert-MvpF5ProcessTimingEvidence `
                -ProjectCreation $fixture.creation `
                -ProductRuns $fixture.products `
                -BaselineAutomation $fixture.baseline `
                -AuthoringAutomation $fixture.authoring `
                -ReopenAutomation $fixture.reopens
        } | Should Throw 'exactly one runtime product attempt 1; found 2'
    }

    It 'rejects a process that overlaps its completed predecessor' {
        $fixture = New-MvpF5ProcessTimelineFixture
        $fixture.products[0].started_at_utc = $baseTimestamp.AddMilliseconds(1500).ToString('o')

        {
            Assert-MvpF5ProcessTimingEvidence `
                -ProjectCreation $fixture.creation `
                -ProductRuns $fixture.products `
                -BaselineAutomation $fixture.baseline `
                -AuthoringAutomation $fixture.authoring `
                -ReopenAutomation $fixture.reopens
        } | Should Throw 'overlaps or precedes completed prior process'
    }

    It 'rejects a timing record without an exit code' {
        $timing = New-MvpProcessTiming -Second 0
        $timing.PSObject.Properties.Remove('exit_code')

        {
            Assert-MvpProcessTimingEvidence -Evidence $timing -Label 'fixture process'
        } | Should Throw "fixture process is missing 'exit_code'"
    }
}
