function Get-ValidationStagePlan {
    param(
        [switch]$SkipBuild,
        [switch]$SkipTest,
        [switch]$CheckOnly,
        [switch]$BuildBeforeTest,
        [switch]$LibTests,
        [string]$TestTarget,
        [string]$Bin,
        [int]$PublishArtifactCount = 0
    )

    if ($BuildBeforeTest -and $SkipBuild) {
        throw "-BuildBeforeTest cannot be combined with -SkipBuild."
    }
    if ($CheckOnly -and ($SkipBuild -or $BuildBeforeTest -or $PublishArtifactCount -gt 0)) {
        throw "-CheckOnly cannot be combined with build/test stages or artifact publishing."
    }

    $focusedTest = $LibTests -or -not [string]::IsNullOrWhiteSpace($TestTarget)
    $build = -not $SkipBuild -and -not $CheckOnly -and (
        $SkipTest -or -not $focusedTest -or $BuildBeforeTest -or
        $PublishArtifactCount -gt 0 -or -not [string]::IsNullOrWhiteSpace($Bin)
    )
    return [pscustomobject]@{
        Build = [bool]$build
        Test = [bool](-not $SkipTest -and -not $CheckOnly)
        Check = [bool]($CheckOnly -or -not $SkipTest)
    }
}
