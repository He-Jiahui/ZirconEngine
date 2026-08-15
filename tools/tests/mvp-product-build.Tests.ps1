$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$validator = Join-Path $repoRoot '.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
$productInputBuilder = Join-Path $repoRoot 'tools\mvp\Build-MvpProductInputs.ps1'
$originalTestMode = $env:VALIDATE_MATRIX_TEST_MODE

$env:VALIDATE_MATRIX_TEST_MODE = '1'
. $validator -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $originalTestMode

Describe 'MVP product binary build contract' {
    It 'renders the selected runtime binary in the managed build command' {
        $previousPackage = $script:Package
        $previousBin = Get-Variable -Name Bin -Scope Script -ErrorAction SilentlyContinue
        try {
            $script:Package = 'zircon_app'
            $script:Bin = 'zircon_runtime'

            $arguments = @(Get-CargoArgs `
                -Subcommand 'build' `
                -ResolvedTargetDir 'D:\cargo-targets\zircon-engine\pool\product-bin' `
                -WorkspaceManifest 'Cargo.toml')

            ($arguments -join ' ') | Should Be 'build -p zircon_app --bin zircon_runtime --locked --target-dir D:\cargo-targets\zircon-engine\pool\product-bin'
        }
        finally {
            $script:Package = $previousPackage
            if ($null -eq $previousBin) {
                Remove-Variable -Name Bin -Scope Script -ErrorAction SilentlyContinue
            }
            else {
                $script:Bin = $previousBin.Value
            }
        }
    }

    It 'publishes a declared runtime artifact before the target directory is released' {
        $targetDirectory = Join-Path $TestDrive 'cargo-target'
        $debugDirectory = Join-Path $targetDirectory 'debug'
        $artifactDirectory = Join-Path $TestDrive 'product-inputs'
        $sourceArtifact = Join-Path $debugDirectory 'zircon_runtime.exe'

        [System.IO.Directory]::CreateDirectory($debugDirectory) | Out-Null
        [System.IO.File]::WriteAllBytes($sourceArtifact, [byte[]](1, 2, 3, 4))

        $published = @(Publish-BuildArtifacts `
            -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $artifactDirectory `
            -ArtifactName 'zircon_runtime.exe')

        $published.Count | Should Be 1
        $published[0].Name | Should Be 'zircon_runtime.exe'
        $published[0].Sha256 | Should Match '^[0-9A-F]{64}$'
        [System.IO.File]::ReadAllBytes((Join-Path $artifactDirectory 'zircon_runtime.exe')) | Should Be @(1, 2, 3, 4)
    }

    It 'rejects a coordinator-managed drive as a build artifact output directory' {
        $message = $null
        try {
            Assert-ArtifactOutputDirectory -Path 'D:\ZirconBuilds\mvp-product-inputs'
        }
        catch {
            $message = $_.Exception.Message
        }

        $message | Should Match 'outside coordinator-governed D/E/F roots'
    }

    It 'permits the dedicated MVP product-input root through the explicit validator mode' {
        $requestedPath = "D:\ZirconBuilds\mvp-product-inputs-contract-$([guid]::NewGuid().ToString('N'))"

        $resolved = Assert-ArtifactOutputDirectory -Path $requestedPath -MvpProductInputArtifactOutput
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Match '^D:\\ZirconBuilds\\mvp-product-inputs-'
    }

    It 'permits a resolver-normalized device path for the dedicated MVP product-input root' {
        $requestedPath = "\\?\D:\ZirconBuilds\mvp-product-inputs-device-$([guid]::NewGuid().ToString('N'))"

        $resolved = Assert-ArtifactOutputDirectory -Path $requestedPath -MvpProductInputArtifactOutput
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Match '^D:\\ZirconBuilds\\mvp-product-inputs-device-'
    }

    It 'keeps the product-input default and managed validator invocation under ZirconBuilds' {
        $builderSource = Get-Content -LiteralPath $productInputBuilder -Raw

        $builderSource | Should Match 'Join-Path ''E:\\ZirconBuilds'' \("mvp-product-inputs-"'
        $builderSource | Should Match '"-MvpProductInputArtifactOutput"'
        $builderSource | Should Not Match '\$env:LOCALAPPDATA'
    }

    It 'resolves device and junction paths before evaluating build artifact output directories' {
        $deviceMessage = $null
        try {
            Assert-ArtifactOutputDirectory -Path '\\?\D:\ZirconBuilds\mvp-product-inputs'
        }
        catch {
            $deviceMessage = $_.Exception.Message
        }

        $targetDirectory = Join-Path $TestDrive 'reparse-target'
        $junctionDirectory = Join-Path $TestDrive 'reparse-link'
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        New-Item -ItemType Junction -Path $junctionDirectory -Target $targetDirectory | Out-Null
        $requestedPath = Join-Path $junctionDirectory 'product-inputs'
        $junctionResolvedPath = Assert-ArtifactOutputDirectory -Path $requestedPath
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $deviceMessage | Should Match 'outside coordinator-governed D/E/F roots'
        $junctionResolvedPath | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Be (Join-Path $targetDirectory 'product-inputs')
    }
}
