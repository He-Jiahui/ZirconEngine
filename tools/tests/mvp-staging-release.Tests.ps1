Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$modulePath = Join-Path $PSScriptRoot '..\mvp\MvpStagingRelease.psm1'
if (-not (Test-Path -LiteralPath $modulePath -PathType Leaf)) {
    throw "MVP staging release module is missing: $modulePath"
}
Import-Module $modulePath -Force -ErrorAction Stop

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$stagerPath = Join-Path $PSScriptRoot '..\mvp\Stage-MvpProducts.ps1'
$stagerSource = Get-Content -LiteralPath $stagerPath -Raw
Assert-True ($stagerSource -match "Import-Module .*MvpStagingRelease\.psm1") 'MVP stager does not import the project release probe module.'
Assert-True (
    ([regex]::Matches($stagerSource, 'Test-MvpStagedProjectDirectoryReleased')).Count -eq 3
) 'MVP stager must probe project release after product, automation, and project-creation processes.'

$fixtureRoot = Join-Path ([IO.Path]::GetTempPath()) (
    'zircon_mvp_staging_release_' + [guid]::NewGuid().ToString('N')
)

try {
    $stageRoot = Join-Path $fixtureRoot 'stage'
    $projectRoot = Join-Path $stageRoot 'project\Fixture'
    New-Item -ItemType Directory -Force -Path $projectRoot | Out-Null
    [IO.File]::WriteAllText(
        (Join-Path $projectRoot 'zircon-project.toml'),
        "name = 'Fixture'`n",
        [Text.UTF8Encoding]::new($false)
    )

    Test-MvpStagedProjectDirectoryReleased `
        -StageDirectory $stageRoot `
        -ProjectDirectory $projectRoot

    Assert-True (Test-Path -LiteralPath $projectRoot -PathType Container) 'Project release probe did not restore the project directory.'
    Assert-True (Test-Path -LiteralPath (Join-Path $projectRoot 'zircon-project.toml') -PathType Leaf) 'Project release probe lost project content.'
    Assert-True (-not (Test-Path -LiteralPath "$projectRoot.release-probe")) 'Project release probe left its temporary rename target behind.'

    $outsideProject = Join-Path $fixtureRoot 'outside-project'
    New-Item -ItemType Directory -Force -Path $outsideProject | Out-Null
    $outsideRejected = $false
    try {
        Test-MvpStagedProjectDirectoryReleased `
            -StageDirectory $stageRoot `
            -ProjectDirectory $outsideProject
    }
    catch {
        $outsideRejected = $_.Exception.Message -match 'outside staging root'
    }
    Assert-True $outsideRejected 'Project release probe accepted a directory outside the staging root.'

    $probePath = "$projectRoot.release-probe"
    New-Item -ItemType Directory -Force -Path $probePath | Out-Null
    $conflictRejected = $false
    try {
        Test-MvpStagedProjectDirectoryReleased `
            -StageDirectory $stageRoot `
            -ProjectDirectory $projectRoot
    }
    catch {
        $conflictRejected = $_.Exception.Message -match 'already exists'
    }
    Assert-True $conflictRejected 'Project release probe overwrote an existing probe path.'
    Assert-True (Test-Path -LiteralPath $projectRoot -PathType Container) 'Probe-path conflict changed the project directory.'

    Write-Output 'MVP staged project release contract passed'
}
finally {
    if (Test-Path -LiteralPath $fixtureRoot) {
        $resolvedFixtureRoot = [IO.Path]::GetFullPath($fixtureRoot)
        $resolvedTempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
        if (-not $resolvedFixtureRoot.StartsWith($resolvedTempRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove staging release fixture outside the temp root: $resolvedFixtureRoot"
        }
        Remove-Item -LiteralPath $resolvedFixtureRoot -Recurse -Force
    }
}
