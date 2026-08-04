Set-StrictMode -Version Latest

$stagingReleaseRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $stagingReleaseRepoRoot 'tools\WindowsPathResolver.psm1') -ErrorAction Stop

function Test-MvpStagedProjectDirectoryReleased {
    param(
        [Parameter(Mandatory)][string]$StageDirectory,
        [Parameter(Mandatory)][string]$ProjectDirectory
    )

    if (-not (Test-Path -LiteralPath $StageDirectory -PathType Container)) {
        throw "Staging directory '$StageDirectory' does not exist."
    }
    if (-not (Test-Path -LiteralPath $ProjectDirectory -PathType Container)) {
        throw "Staged project directory '$ProjectDirectory' does not exist."
    }

    # The release probe performs a real filesystem rename, so retain the resolver's physical
    # path. Display paths are only for diagnostics and must not weaken verbatim-path semantics.
    $resolvedStage = (Resolve-ZirconWindowsPath -Path $StageDirectory).OperationalPath.TrimEnd([char[]]@('\', '/'))
    $resolvedProject = (Resolve-ZirconWindowsPath -Path $ProjectDirectory).OperationalPath.TrimEnd([char[]]@('\', '/'))
    $stagePrefix = $resolvedStage + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedProject.StartsWith($stagePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Staged project directory '$resolvedProject' is outside staging root '$resolvedStage'."
    }

    $probe = "$resolvedProject.release-probe"
    if ([IO.Directory]::Exists($probe) -or [IO.File]::Exists($probe)) {
        throw "Staged project release probe '$probe' already exists."
    }

    Move-ZirconWindowsPath -Source $resolvedProject -Destination $probe
    try {
        Move-ZirconWindowsPath -Source $probe -Destination $resolvedProject
    }
    catch {
        $restoreError = $_.Exception
        $recovery = 'automatic recovery was not required'
        if ([IO.Directory]::Exists($probe) -and -not [IO.Directory]::Exists($resolvedProject)) {
            try {
                Move-ZirconWindowsPath -Source $probe -Destination $resolvedProject
                $recovery = 'the project directory was restored on retry'
            }
            catch {
                $recovery = "the project remains at '$probe': $($_.Exception.Message)"
            }
        }
        throw "Staged project release probe could not restore '$resolvedProject': $($restoreError.Message); $recovery."
    }
}

Export-ModuleMember -Function Test-MvpStagedProjectDirectoryReleased
