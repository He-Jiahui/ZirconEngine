Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$fixturePathsModule = Join-Path $PSScriptRoot '..\mvp\MvpTestFixturePaths.psm1'
$coordinatorScript = Join-Path $PSScriptRoot '..\zircon-session.ps1'
Import-Module $fixturePathsModule -Force -ErrorAction Stop
$fixturePathsSource = Get-Content -LiteralPath $fixturePathsModule -Raw

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$fixtureRoot = New-MvpTestFixtureRoot -Prefix 'paths-contract'
$fixtureParent = Split-Path -Parent $fixtureRoot
$operationalFixtureRoot = $null
$junctionFixtureRoot = $null
$junctionTargetRoot = $null
$sharedFixtureRoot = $null
$sharedFixtureSibling = $null
try {
    Assert-True `
        ($fixturePathsSource -match 'Resolve-ZirconWindowsPath') `
        'MVP fixture paths must resolve physical output paths through the shared Windows resolver.'
    Assert-True `
        ($fixturePathsSource -match 'MvpArtifactStoragePolicy\.psm1' -and $fixturePathsSource -match 'Resolve-MvpArtifactStorageRootPath') `
        'MVP fixture paths must authorize Coordinator-issued roots through the typed storage policy.'
    Assert-True `
        ($fixturePathsSource -match 'Resolve-MvpArtifactStoragePath' -and $fixturePathsSource -match 'mvp-test-fixtures') `
        'MVP fixture paths must remain inside the registered Coordinator fixture namespace.'
    Assert-True `
        ($fixturePathsSource -notmatch '\^\[D-F\]:' -and $fixturePathsSource -notmatch '[D-F]:\\ZirconBuilds') `
        'MVP fixture path production logic must not duplicate physical artifact-root literals.'
    Assert-True `
        ($fixturePathsSource -match 'fixtureParentResolution\s*=\s*Resolve-ZirconWindowsPath') `
        'MVP fixture paths must verify the fixture parent physical path before creating a GUID fixture leaf.'
    Assert-True `
        ($fixturePathsSource -match 'mvp-test-fixtures-\$PID') `
        'MVP fixture paths must use a process-scoped parent so concurrent test sessions do not share cleanup state.'
    Assert-True `
        ($fixturePathsSource -match 'zircon-session\.ps1' -and $fixturePathsSource -match 'fixture-acquire') `
        'MVP fixture paths must acquire a Coordinator-managed artifact fixture lease before directory creation.'
    Assert-True `
        ($fixturePathsSource -match 'zircon-session\.ps1' -and $fixturePathsSource -match 'fixture-release') `
        'MVP fixture paths must release its Coordinator-managed artifact fixture lease after physical cleanup.'
    Assert-True `
        ($fixtureRoot -match '^[D-F]:\\ZirconBuilds\\mvp-test-fixtures-[1-9][0-9]*\\paths-contract-[0-9a-f]{32}$') `
        "MVP fixture root '$fixtureRoot' is outside the approved D/E/F ZirconBuilds test root."
    Assert-True (Test-Path -LiteralPath $fixtureRoot -PathType Container) 'MVP fixture root was not created.'
    $activeAudit = ((@(& $coordinatorScript artifact audit -Json) -join "`n") | ConvertFrom-Json)
    $activeOverlap = @(
        $activeAudit.unmanaged | Where-Object {
            $fixtureRoot.Equals($_, [StringComparison]::OrdinalIgnoreCase) -or
            $fixtureRoot.StartsWith($_ + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
        }
    )
    Assert-True ($activeOverlap.Count -eq 0) 'Artifact governance reported a live leased MVP fixture as unmanaged.'
    $outsideFixtureRootRejected = $false
    try {
        Remove-MvpTestFixtureRoot -Path 'D:\ZirconBuilds\outside-mvp-test-fixture'
    }
    catch {
        $outsideFixtureRootRejected = $_.Exception.Message -match 'approved MVP fixture root'
    }
    Assert-True $outsideFixtureRootRejected 'MVP fixture cleanup accepted a path outside the approved fixture root.'

    Remove-MvpTestFixtureRoot -Path $fixtureRoot
    Assert-True (-not (Test-Path -LiteralPath $fixtureRoot)) 'MVP fixture cleanup did not remove its generated fixture root.'
    Assert-True (-not (Test-Path -LiteralPath $fixtureParent)) 'MVP fixture cleanup left an empty process-scoped fixture parent behind.'

    [IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
    $releasedAudit = ((@(& $coordinatorScript artifact audit -Json) -join "`n") | ConvertFrom-Json)
    $releasedOverlap = @(
        $releasedAudit.unmanaged | Where-Object {
            $fixtureRoot.Equals($_, [StringComparison]::OrdinalIgnoreCase) -or
            $fixtureRoot.StartsWith($_ + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
        }
    )
    Assert-True ($releasedOverlap.Count -gt 0) 'Released fixture path remained permanently exempt from artifact governance.'
    Remove-MvpTestFixtureRoot -Path $fixtureRoot

    $sharedFixtureRoot = New-MvpTestFixtureRoot -Prefix 'paths-shared-first'
    $sharedFixtureSibling = New-MvpTestFixtureRoot -Prefix 'paths-shared-second'
    $sharedFixtureParent = Split-Path -Parent $sharedFixtureRoot
    Assert-True ($sharedFixtureParent -eq (Split-Path -Parent $sharedFixtureSibling)) 'MVP fixture roots from one process must share one process-scoped parent.'
    Remove-MvpTestFixtureRoot -Path $sharedFixtureRoot
    Assert-True (Test-Path -LiteralPath $sharedFixtureParent -PathType Container) 'MVP fixture cleanup removed a parent that still contains another fixture root.'
    Remove-MvpTestFixtureRoot -Path $sharedFixtureSibling
    Assert-True (-not (Test-Path -LiteralPath $sharedFixtureParent)) 'MVP fixture cleanup left a process-scoped parent after its final fixture root was removed.'

    $operationalFixtureRoot = New-MvpTestFixtureRoot -Prefix 'paths-operational-contract'
    $operationalFixturePath = "\\?\$operationalFixtureRoot"
    Remove-MvpTestFixtureRoot -Path $operationalFixturePath
    Assert-True (-not (Test-Path -LiteralPath $operationalFixtureRoot)) 'MVP fixture cleanup did not accept the Windows operational path form.'

    $junctionFixtureRoot = New-MvpTestFixtureRoot -Prefix 'paths-junction-contract'
    $junctionTargetRoot = New-MvpTestFixtureRoot -Prefix 'paths-junction-target'
    $junctionSentinel = Join-Path $junctionTargetRoot 'must-survive.txt'
    [IO.File]::WriteAllText($junctionSentinel, 'fixture cleanup must not follow junctions')
    New-Item -ItemType Junction -Path (Join-Path $junctionFixtureRoot 'outside-target') -Target $junctionTargetRoot -ErrorAction Stop | Out-Null
    Remove-MvpTestFixtureRoot -Path $junctionFixtureRoot
    Assert-True (-not (Test-Path -LiteralPath $junctionFixtureRoot)) 'MVP fixture cleanup did not remove the fixture root containing a junction.'
    Assert-True (Test-Path -LiteralPath $junctionSentinel -PathType Leaf) 'MVP fixture cleanup followed a junction outside its fixture root.'
    Write-Host 'MVP fixture-root path contract passed'
}
finally {
    if (Test-Path -LiteralPath $fixtureRoot -PathType Container) {
        Remove-MvpTestFixtureRoot -Path $fixtureRoot
    }
    if ($null -ne $operationalFixtureRoot -and (Test-Path -LiteralPath $operationalFixtureRoot -PathType Container)) {
        Remove-MvpTestFixtureRoot -Path $operationalFixtureRoot
    }
    if ($null -ne $junctionFixtureRoot -and (Test-Path -LiteralPath $junctionFixtureRoot -PathType Container)) {
        Remove-MvpTestFixtureRoot -Path $junctionFixtureRoot
    }
    if ($null -ne $junctionTargetRoot -and (Test-Path -LiteralPath $junctionTargetRoot -PathType Container)) {
        Remove-MvpTestFixtureRoot -Path $junctionTargetRoot
    }
    if ($null -ne $sharedFixtureRoot -and (Test-Path -LiteralPath $sharedFixtureRoot -PathType Container)) {
        Remove-MvpTestFixtureRoot -Path $sharedFixtureRoot
    }
    if ($null -ne $sharedFixtureSibling -and (Test-Path -LiteralPath $sharedFixtureSibling -PathType Container)) {
        Remove-MvpTestFixtureRoot -Path $sharedFixtureSibling
    }
}
