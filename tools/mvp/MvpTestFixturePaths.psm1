Set-StrictMode -Version Latest

Import-Module (Join-Path $PSScriptRoot 'MvpTestFixtureContext.psm1') -Force -ErrorAction Stop

function New-MvpTestFixtureRoot {
    param(
        [Parameter(Mandatory)]
        [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,119}$')]
        [string]$Prefix
    )

    $context = Get-MvpTestFixtureContext
    return New-MvpOwnedFixture -Context $context -Prefix $Prefix
}

function Remove-MvpTestFixtureRoot {
    param([Parameter(Mandatory)][string]$Path)

    $context = Get-MvpTestFixtureContext
    Remove-MvpOwnedFixture -Context $context -Path $Path
}

function New-ZirconMvpTestFixtureDirectory {
    param([Parameter(Mandatory)][string]$Prefix)
    return New-MvpTestFixtureRoot -Prefix $Prefix
}

function Remove-ZirconMvpTestFixtureDirectory {
    param([Parameter(Mandatory)][string]$Path)
    Remove-MvpTestFixtureRoot -Path $Path
}

Export-ModuleMember -Function @('New-MvpTestFixtureRoot', 'Remove-MvpTestFixtureRoot', 'New-ZirconMvpTestFixtureDirectory', 'Remove-ZirconMvpTestFixtureDirectory')
