Set-StrictMode -Version Latest

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

    $resolvedStage = (Resolve-Path -LiteralPath $StageDirectory).Path.TrimEnd([char[]]@('\', '/'))
    $resolvedProject = (Resolve-Path -LiteralPath $ProjectDirectory).Path.TrimEnd([char[]]@('\', '/'))
    $stagePrefix = $resolvedStage + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedProject.StartsWith($stagePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Staged project directory '$resolvedProject' is outside staging root '$resolvedStage'."
    }

    $probe = "$resolvedProject.release-probe"
    if (Test-Path -LiteralPath $probe) {
        throw "Staged project release probe '$probe' already exists."
    }

    Move-Item -LiteralPath $resolvedProject -Destination $probe -ErrorAction Stop
    try {
        Move-Item -LiteralPath $probe -Destination $resolvedProject -ErrorAction Stop
    }
    catch {
        $restoreError = $_.Exception
        $recovery = 'automatic recovery was not required'
        if ((Test-Path -LiteralPath $probe) -and -not (Test-Path -LiteralPath $resolvedProject)) {
            try {
                Move-Item -LiteralPath $probe -Destination $resolvedProject -ErrorAction Stop
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
