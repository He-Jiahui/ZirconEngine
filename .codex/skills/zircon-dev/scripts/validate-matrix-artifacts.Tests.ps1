$validatorPath = Join-Path $PSScriptRoot 'validate-matrix.ps1'
$previousTestMode = $env:VALIDATE_MATRIX_TEST_MODE
try {
    $env:VALIDATE_MATRIX_TEST_MODE = '1'
    . $validatorPath -Package zircon_app -DryRun
} finally {
    $env:VALIDATE_MATRIX_TEST_MODE = $previousTestMode
}

Describe 'Cargo target artifact publication' {
    It 'publishes explicit target artifacts without selecting host artifacts' {
        foreach ($profile in @('development', 'release', 'profiling')) {
            $directoryName = if ($profile -eq 'development') { 'debug' } else { $profile }
            $targetDirectory = Join-Path $TestDrive "target-$profile"
            $hostDirectory = Join-Path $targetDirectory $directoryName
            $cargoTarget = 'x86_64-pc-windows-msvc'
            $targetProfile = Join-Path (Join-Path $targetDirectory $cargoTarget) $directoryName
            [System.IO.Directory]::CreateDirectory($hostDirectory) | Out-Null
            [System.IO.Directory]::CreateDirectory($targetProfile) | Out-Null
            [System.IO.File]::WriteAllBytes((Join-Path $hostDirectory 'zircon_app.exe'), [byte[]](0))
            [System.IO.File]::WriteAllBytes((Join-Path $targetProfile 'zircon_app.exe'), [byte[]](1, 2, 3))

            $published = @(Publish-BuildArtifacts -TargetDirectory $targetDirectory `
                -ArtifactOutputDirectory (Join-Path $TestDrive "published-$profile") `
                -ArtifactName zircon_app.exe -CargoProfile $profile -CargoTarget $cargoTarget)

            $published.Count | Should Be 1
            [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($published[0].Path)) |
                Should Be 'AQID'
        }
    }

    It 'uses the custom target file stem as the output directory' {
        $targetDirectory = Join-Path $TestDrive 'custom-target'
        $targetProfile = Join-Path $targetDirectory 'custom target\debug'
        [System.IO.Directory]::CreateDirectory($targetProfile) | Out-Null
        [System.IO.File]::WriteAllBytes((Join-Path $targetProfile 'zircon_app.exe'), [byte[]](1))

        $published = @(Publish-BuildArtifacts -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory (Join-Path $TestDrive 'custom-published') `
            -ArtifactName zircon_app.exe -CargoTarget 'specs/custom target.json')

        $published.Count | Should Be 1
        $published[0].Bytes | Should Be 1
    }

    It 'passes the configured target to development DLL staging' {
        $targetDirectory = Join-Path $TestDrive 'dynamic-target'
        $cargoTarget = 'x86_64-pc-windows-msvc'
        $targetProfile = Join-Path (Join-Path $targetDirectory $cargoTarget) 'debug'
        $outputDirectory = Join-Path $TestDrive 'dynamic-published'
        [System.IO.Directory]::CreateDirectory((Join-Path $targetProfile 'deps')) | Out-Null
        [System.IO.File]::WriteAllBytes((Join-Path $targetProfile 'zircon_app.exe'), [byte[]](1))
        foreach ($carrier in @('zr_dev_deps_dylib.dll', 'zr_runtime_dev_dylib.dll')) {
            [System.IO.File]::WriteAllBytes((Join-Path $targetProfile "deps/$carrier"), [byte[]](1))
        }

        Publish-BuildArtifacts -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $outputDirectory `
            -ArtifactName zircon_app.exe -CargoTarget $cargoTarget -DevelopmentDlls | Out-Null

        $receipt = Get-Content (Join-Path $outputDirectory 'development-dlls.receipt.json') -Raw | ConvertFrom-Json
        $receipt.target | Should Be $cargoTarget
        $receipt.profileDirectory | Should Be (Resolve-ZirconWindowsPath -Path $targetProfile).OperationalPath
        @($receipt.artifacts | Where-Object { $_.fileName -like 'std-*.dll' }).Count | Should BeGreaterThan 0
    }

    It 'keeps implicit host DLL staging in the profile directory' {
        $targetDirectory = Join-Path $TestDrive 'implicit-target'
        $targetProfile = Join-Path $targetDirectory 'debug'
        $outputDirectory = Join-Path $TestDrive 'implicit-published'
        [System.IO.Directory]::CreateDirectory((Join-Path $targetProfile 'deps')) | Out-Null
        [System.IO.File]::WriteAllBytes((Join-Path $targetProfile 'zircon_app.exe'), [byte[]](1))
        foreach ($carrier in @('zr_dev_deps_dylib.dll', 'zr_runtime_dev_dylib.dll')) {
            [System.IO.File]::WriteAllBytes((Join-Path $targetProfile "deps/$carrier"), [byte[]](1))
        }

        Publish-BuildArtifacts -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $outputDirectory `
            -ArtifactName zircon_app.exe -DevelopmentDlls | Out-Null

        $receipt = Get-Content (Join-Path $outputDirectory 'development-dlls.receipt.json') -Raw | ConvertFrom-Json
        $receipt.target | Should BeNullOrEmpty
        $receipt.profileDirectory | Should Be (Resolve-ZirconWindowsPath -Path $targetProfile).OperationalPath
    }
}
