$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$builder = Join-Path $repoRoot 'tools\mvp\Build-RenderExtractProfilingInputs.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$originalTestMode = $env:RENDER_EXTRACT_PROFILING_INPUTS_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop

try {
    $env:RENDER_EXTRACT_PROFILING_INPUTS_TEST_MODE = '1'
    . $builder
}
finally {
    $env:RENDER_EXTRACT_PROFILING_INPUTS_TEST_MODE = $originalTestMode
}

Describe 'Render-extract profiling input build plan' {
    BeforeEach {
        Import-Module $resolverModule -Force -ErrorAction Stop
    }

    It 'uses matching runtime and editor executable-library profiling contracts' {
        $requests = @(Get-RenderExtractProfilingBuildRequests)

        $requests.Count | Should Be 4
        $executable = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-executable' })
        $library = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-library' })
        $editorExecutable = @($requests | Where-Object { $_.logical_id -eq 'editor-profile-executable' })
        $editorLibrary = @($requests | Where-Object { $_.logical_id -eq 'editor-profile-library' })

        $executable.Count | Should Be 1
        $executable[0].Package | Should Be 'zircon_app'
        $executable[0].Bin | Should Be 'zircon_runtime'
        $executable[0].Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
        $executable[0].CargoProfile | Should Be 'profiling'
        $executable[0].ArtifactName | Should Be 'zircon_runtime.exe'
        $executable[0].Product | Should Be 'runtime'

        $library.Count | Should Be 1
        $library[0].Package | Should Be 'zircon_runtime'
        $library[0].Bin | Should BeNullOrEmpty
        $library[0].Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
        $library[0].CargoProfile | Should Be 'profiling'
        $library[0].ArtifactName | Should Be 'zircon_runtime.dll'
        $library[0].Product | Should Be 'runtime'

        $editorExecutable.Count | Should Be 1
        $editorExecutable[0].Package | Should Be 'zircon_app'
        $editorExecutable[0].Bin | Should Be 'zircon_editor'
        $editorExecutable[0].Features | Should Be 'target-editor-host,profiling'
        $editorExecutable[0].CargoProfile | Should Be 'profiling'
        $editorExecutable[0].ArtifactName | Should Be 'zircon_editor.exe'
        $editorExecutable[0].Product | Should Be 'editor'

        $editorLibrary.Count | Should Be 1
        $editorLibrary[0].Package | Should Be 'zircon_runtime'
        $editorLibrary[0].Bin | Should BeNullOrEmpty
        $editorLibrary[0].Features | Should Be 'target-editor-host,profiling'
        $editorLibrary[0].CargoProfile | Should Be 'profiling'
        $editorLibrary[0].ArtifactName | Should Be 'zircon_runtime.dll'
        $editorLibrary[0].Product | Should Be 'editor'
    }

    It 'renders managed profiling validator invocations for both runtime artifacts' {
        $validator = 'E:\Git\ZirconEngine\.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
        $snapshotRoot = 'E:\ZirconBuilds\render-extract-build-set\snapshot'
        $outputDirectory = 'E:\ZirconBuilds\mvp-product-inputs-profile-contract'
        $requests = @(Get-RenderExtractProfilingBuildRequests)
        $executable = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-executable' })[0]
        $library = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-library' })[0]
        $editorExecutable = @($requests | Where-Object { $_.logical_id -eq 'editor-profile-executable' })[0]

        $executableArguments = @(Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -RepositoryRoot $snapshotRoot `
                -OutputDirectory $outputDirectory `
                -Request $executable)
        $libraryArguments = @(Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -RepositoryRoot $snapshotRoot `
                -OutputDirectory $outputDirectory `
                -Request $library)

        $executableArguments | Should Be @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $validator,
            '-RepoRoot', $snapshotRoot,
            '-Package', 'zircon_app',
            '-NoDefaultFeatures',
            '-Features', 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling',
            '-Bin', 'zircon_runtime',
            '-CargoProfile', 'profiling',
            '-SkipTest',
            '-MvpProductInputArtifactOutput',
            '-ArtifactOutputDirectory', (Join-Path $outputDirectory 'runtime'),
            '-PublishArtifact', 'zircon_runtime.exe'
        )
        $libraryArguments | Should Be @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $validator,
            '-RepoRoot', $snapshotRoot,
            '-Package', 'zircon_runtime',
            '-NoDefaultFeatures',
            '-Features', 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling',
            '-CargoProfile', 'profiling',
            '-SkipTest',
            '-MvpProductInputArtifactOutput',
            '-ArtifactOutputDirectory', (Join-Path $outputDirectory 'runtime'),
            '-PublishArtifact', 'zircon_runtime.dll'
        )
        @(Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -RepositoryRoot $snapshotRoot `
                -OutputDirectory $outputDirectory `
                -Request $editorExecutable) | Should Be @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $validator,
            '-RepoRoot', $snapshotRoot,
            '-Package', 'zircon_app',
            '-NoDefaultFeatures',
            '-Features', 'target-editor-host,profiling',
            '-Bin', 'zircon_editor',
            '-CargoProfile', 'profiling',
            '-SkipTest',
            '-MvpProductInputArtifactOutput',
            '-ArtifactOutputDirectory', (Join-Path $outputDirectory 'editor'),
            '-PublishArtifact', 'zircon_editor.exe'
        )
    }

    It 'builds every profiling artifact from one integrity-checked BuildSet snapshot' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'Import-Module .*MvpBuildSet\.psm1'
        $builderSource | Should Match '\$buildSet = New-MvpProductBuildSet'
        $builderSource | Should Match '\$validator = Join-Path \$buildSet\.snapshot_root'
        $builderSource | Should Match '-RepositoryRoot \$buildSet\.snapshot_root'
        $builderSource | Should Match 'Assert-MvpProductBuildSet -ManifestPath \$buildSet\.manifest_path'
        $builderSource | Should Not Match 'Get-MvpSourceFingerprint -RepositoryRoot \$repoRoot'
        $builderSource | Should Not Match '\$sourceFingerprint\s*='
    }

    It 'does not export the removed active-checkout source fingerprint API' {
        $manifestModule = Get-Module MvpProductInputManifest

        (@($manifestModule.ExportedFunctions.Keys) -join ',') | Should Not Match 'Get-MvpSourceFingerprint'
    }

    It 'contains no legacy git diff or untracked-file fingerprint implementation' {
        $manifestModuleSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1') -Raw

        $manifestModuleSource | Should Not Match 'function Get-MvpSourceFingerprint'
        $manifestModuleSource | Should Not Match 'Invoke-MvpSourceGit'
        $manifestModuleSource | Should Not Match "'ls-files', '-z', '--others', '--exclude-standard'"
        $manifestModuleSource | Should Not Match 'zircon-mvp-source-fingerprint-v3'
    }

    It 'accepts only a physical profiling input root on an approved artifact drive' {
        $accepted = Assert-RenderExtractProfilingInputDirectory `
            -Path 'E:\ZirconBuilds\mvp-product-inputs-profile-contract'
        $resolution = Resolve-ZirconWindowsPath `
            -Path 'E:\ZirconBuilds\mvp-product-inputs-profile-contract'

        $accepted | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Be 'E:\ZirconBuilds\mvp-product-inputs-profile-contract'

        $failure = $null
        try {
            Assert-RenderExtractProfilingInputDirectory -Path 'C:\ZirconBuilds\mvp-product-inputs-profile-contract'
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'mvp-product-inputs-profile-'
    }

    It 'publishes through a sibling partial root instead of exposing build groups incrementally' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match '\.partial-'
        $builderSource | Should Match 'Move-ZirconWindowsPath -Source \$PublicationDirectory -Destination \$OutputDirectory -ApprovedRoot \$PublicationParent'
        $builderSource | Should Match 'render-extract-profiling-inputs-aborted\.json'
    }

    It 'writes a source-bound standalone profiling measurement manifest with both product pairs' {
        $runtimeDirectory = Join-Path $TestDrive 'runtime'
        $editorDirectory = Join-Path $TestDrive 'editor'
        [IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null
        [IO.Directory]::CreateDirectory($editorDirectory) | Out-Null
        $executablePath = Join-Path $runtimeDirectory 'zircon_runtime.exe'
        $libraryPath = Join-Path $runtimeDirectory 'zircon_runtime.dll'
        $editorExecutablePath = Join-Path $editorDirectory 'zircon_editor.exe'
        $editorLibraryPath = Join-Path $editorDirectory 'zircon_runtime.dll'
        [System.IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [System.IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8, 9))
        [System.IO.File]::WriteAllBytes($editorExecutablePath, [byte[]](10, 11, 12))
        [System.IO.File]::WriteAllBytes($editorLibraryPath, [byte[]](13, 14))
        $manifestPath = Join-Path $TestDrive 'render-extract-profiling-inputs.json'

        Write-RenderExtractProfilingInputManifest `
            -Path $manifestPath `
            -BuildSet ([pscustomobject]@{
                    build_set_id = 'B' * 64
                    git_revision = 'c' * 40
                    dirty_overlay_sha256 = 'D' * 64
                }) `
            -ArtifactOutputDirectory $TestDrive | Out-Null

        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.schema_version | Should Be 3
        $manifest.source_fingerprint | Should Be ('B' * 64)
        $manifest.build_set.build_set_id | Should Be ('B' * 64)
        $manifest.build_set.git_revision | Should Be ('c' * 40)
        $manifest.build_set.dirty_overlay_sha256 | Should Be ('D' * 64)
        $manifest.build_set.manifest_relative_path | Should Be 'build-set/build-set.json'
        $manifest.cargo_profile | Should Be 'profiling'
        $manifest.artifacts.Count | Should Be 4
        $manifest.artifacts[0].logical_id | Should Be 'runtime-profile-executable'
        $manifest.artifacts[0].product | Should Be 'runtime'
        $manifest.artifacts[0].path | Should Be $executablePath
        $manifest.artifacts[0].sha256 | Should Match '^[0-9A-F]{64}$'
        $manifest.artifacts[0].bytes | Should Be 4
        $manifest.artifacts[1].logical_id | Should Be 'runtime-profile-library'
        $manifest.artifacts[1].product | Should Be 'runtime'
        $manifest.artifacts[1].path | Should Be $libraryPath
        $manifest.artifacts[1].sha256 | Should Match '^[0-9A-F]{64}$'
        $manifest.artifacts[1].bytes | Should Be 5
        $manifest.artifacts[2].logical_id | Should Be 'editor-profile-executable'
        $manifest.artifacts[2].product | Should Be 'editor'
        $manifest.artifacts[2].path | Should Be $editorExecutablePath
        $manifest.artifacts[2].bytes | Should Be 3
        $manifest.artifacts[3].logical_id | Should Be 'editor-profile-library'
        $manifest.artifacts[3].product | Should Be 'editor'
        $manifest.artifacts[3].path | Should Be $editorLibraryPath
        $manifest.artifacts[3].bytes | Should Be 2
    }

    It 'refuses to overwrite an existing profiling input manifest' {
        $runtimeDirectory = Join-Path $TestDrive 'runtime'
        $editorDirectory = Join-Path $TestDrive 'editor'
        [IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null
        [IO.Directory]::CreateDirectory($editorDirectory) | Out-Null
        $executablePath = Join-Path $runtimeDirectory 'zircon_runtime.exe'
        $libraryPath = Join-Path $runtimeDirectory 'zircon_runtime.dll'
        $editorExecutablePath = Join-Path $editorDirectory 'zircon_editor.exe'
        $editorLibraryPath = Join-Path $editorDirectory 'zircon_runtime.dll'
        $manifestPath = Join-Path $TestDrive 'existing-render-extract-profiling-inputs.json'
        [System.IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [System.IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8, 9))
        [System.IO.File]::WriteAllBytes($editorExecutablePath, [byte[]](10, 11, 12))
        [System.IO.File]::WriteAllBytes($editorLibraryPath, [byte[]](13, 14))
        [System.IO.File]::WriteAllText($manifestPath, 'foreign-manifest', [Text.UTF8Encoding]::new($false))
        $failure = $null

        try {
            Write-RenderExtractProfilingInputManifest `
                -Path $manifestPath `
                -BuildSet ([pscustomobject]@{
                        build_set_id = 'B' * 64
                        git_revision = 'c' * 40
                        dirty_overlay_sha256 = 'D' * 64
                    }) `
                -ArtifactOutputDirectory $TestDrive | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'Refusing to overwrite existing render-extract profiling input manifest'
        [System.IO.File]::ReadAllText($manifestPath) | Should Be 'foreign-manifest'
    }
}
