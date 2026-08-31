$script:ValidateMatrixScript = Join-Path $PSScriptRoot "..\..\.codex\skills\zircon-dev\scripts\validate-matrix.ps1"
$script:OriginalValidateMatrixTestMode = $env:VALIDATE_MATRIX_TEST_MODE

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ValidateMatrixScript -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalValidateMatrixTestMode

Describe "Validate matrix managed directory restoration" {
    It "restores directories removed by cargo clean before the next Cargo stage" {
        $approvedRoot = "E:\cargo-targets\zircon-engine"
        $targetDirectory = Join-Path $approvedRoot (
            "validate-matrix-clean-restore-{0}" -f [guid]::NewGuid().ToString("N")
        )
        $lease = $null

        try {
            $lease = Push-ManagedCargoEnvironment -TargetDirectory $targetDirectory
            $managedDirectories = @(
                $lease.TemporaryOperationalPath,
                $lease.CargoHomeOperationalPath,
                $lease.SccacheOperationalPath
            )

            foreach ($directory in $managedDirectories) {
                Remove-Item -LiteralPath $directory -Recurse -Force
                Test-Path -LiteralPath $directory | Should Be $false
            }

            Restore-ManagedCargoEnvironmentDirectories -Lease $lease

            foreach ($directory in $managedDirectories) {
                Test-Path -LiteralPath $directory -PathType Container | Should Be $true
            }

            $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript
            $cleanupStage = $source.Split('Invoke-Step "Cargo clean"', 2)[1]
            $cleanupStage = $cleanupStage.Split('if (-not $SkipBuild)', 2)[0]
            $cleanupStage | Should Match 'Restore-ManagedCargoEnvironmentDirectories\s+`?\s*-Lease\s+\$cargoEnvironmentLease'
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
