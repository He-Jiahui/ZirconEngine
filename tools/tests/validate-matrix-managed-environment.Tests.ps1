$script:ValidateMatrixScript = Join-Path $PSScriptRoot "..\..\.codex\skills\zircon-dev\scripts\validate-matrix.ps1"
$script:OriginalValidateMatrixTestMode = $env:VALIDATE_MATRIX_TEST_MODE

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ValidateMatrixScript -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalValidateMatrixTestMode

Describe "Validate matrix managed directory restoration" {
    It "keeps the pool Cargo home after retiring only this job scratch" {
        $approvedRoot = "E:\cargo-targets\zircon-engine"
        $targetDirectory = Join-Path $approvedRoot (
            "validate-matrix-clean-restore-{0}" -f [guid]::NewGuid().ToString("N")
        )
        $lease = $null

        try {
            $lease = Push-ManagedCargoEnvironment -TargetDirectory $targetDirectory -JobId ('app08-' + [guid]::NewGuid().ToString('N')) -StorageMode diagnostic
            $managedDirectories = @(
                $lease.TemporaryOperationalPath,
                $lease.CargoHomeOperationalPath
            )

            foreach ($directory in $managedDirectories) {
                Test-Path -LiteralPath $directory -PathType Container | Should Be $true
            }

            $cargoHome = $lease.CargoHomeOperationalPath
            $scratch = $lease.ScratchOperationalPath
            Pop-ManagedCargoEnvironment -Lease $lease
            $lease = $null
            Test-Path -LiteralPath $cargoHome -PathType Container | Should Be $true
            Test-Path -LiteralPath $scratch | Should Be $false
            $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript
            $source | Should Not Match 'Invoke-Step "Cargo clean"'
        }
        finally {
            if ($null -ne $lease) {
                Pop-ManagedCargoEnvironment -Lease $lease
            }

            $resolvedTarget = [System.IO.Path]::GetFullPath($targetDirectory)
            $resolvedApprovedRoot = [System.IO.Path]::GetFullPath($approvedRoot).TrimEnd("\") + "\"
            if ($resolvedTarget.StartsWith(
                    $resolvedApprovedRoot,
                    [System.StringComparison]::OrdinalIgnoreCase
                ) -and (Test-Path -LiteralPath $resolvedTarget)) {
                Remove-Item -LiteralPath $resolvedTarget -Recurse -Force
            }
        }
    }
}
