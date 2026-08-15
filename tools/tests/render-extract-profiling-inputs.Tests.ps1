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
        $outputDirectory = 'E:\ZirconBuilds\mvp-product-inputs-profile-contract'
        $requests = @(Get-RenderExtractProfilingBuildRequests)
        $executable = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-executable' })[0]
        $library = @($requests | Where-Object { $_.logical_id -eq 'runtime-profile-library' })[0]
        $editorExecutable = @($requests | Where-Object { $_.logical_id -eq 'editor-profile-executable' })[0]

        $executableArguments = @(Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -OutputDirectory $outputDirectory `
                -Request $executable)
        $libraryArguments = @(Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -OutputDirectory $outputDirectory `
                -Request $library)

        $executableArguments | Should Be @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $validator,
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
                -OutputDirectory $outputDirectory `
                -Request $editorExecutable) | Should Be @(
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-File', $validator,
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

    It 'binds the profiling build to the source snapshot before launching the validator' {
        $builderSource = Get-Content -LiteralPath $builder -Raw
        $beforePhasePattern = [regex]::Escape('-Phase "before $($request.logical_id) build"')
        $afterPhasePattern = [regex]::Escape('-Phase "after $($request.logical_id) build"')

        $builderSource | Should Match $beforePhasePattern
        $builderSource | Should Match $afterPhasePattern
    }

    It 'changes the complete source fingerprint when tracked or untracked input bytes change' {
        $sourceRoot = Join-Path $TestDrive ('source-fingerprint-' + [guid]::NewGuid().ToString('N'))
        [IO.Directory]::CreateDirectory($sourceRoot) | Out-Null
        $trackedPaths = @(0..319 | ForEach-Object {
                $name = ('fixture-{0:D4}-' -f $_) + ('x' * 72) + '.rs'
                Join-Path $sourceRoot $name
            })
        foreach ($trackedPath in $trackedPaths) {
            [IO.File]::WriteAllText($trackedPath, 'baseline', [Text.UTF8Encoding]::new($false))
        }

        & git -C $sourceRoot init --quiet
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot config user.email 'mvp-fingerprint@example.invalid'
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot config user.name 'MVP Fingerprint Test'
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot add -- .
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot commit --quiet -m 'baseline'
        $LASTEXITCODE | Should Be 0

        foreach ($trackedPath in $trackedPaths) {
            [IO.File]::WriteAllText($trackedPath, 'tracked-one', [Text.UTF8Encoding]::new($false))
        }
        $trackedOne = Get-MvpSourceFingerprint -RepositoryRoot $sourceRoot
        foreach ($trackedPath in $trackedPaths) {
            [IO.File]::WriteAllText($trackedPath, 'tracked-two', [Text.UTF8Encoding]::new($false))
        }
        $trackedTwo = Get-MvpSourceFingerprint -RepositoryRoot $sourceRoot

        $trackedOne | Should Match '^[0-9A-F]{64}$'
        $trackedTwo | Should Match '^[0-9A-F]{64}$'
        $trackedTwo | Should Not Be $trackedOne

        $untrackedPath = Join-Path $sourceRoot 'generated-input.txt'
        [IO.File]::WriteAllText($untrackedPath, 'untracked-one', [Text.UTF8Encoding]::new($false))
        $untrackedOne = Get-MvpSourceFingerprint -RepositoryRoot $sourceRoot
        [IO.File]::WriteAllText($untrackedPath, 'untracked-two', [Text.UTF8Encoding]::new($false))
        $untrackedTwo = Get-MvpSourceFingerprint -RepositoryRoot $sourceRoot

        $untrackedTwo | Should Not Be $untrackedOne
    }

    It 'accepts a first changed dot-path through Windows PowerShell without a UTF-8 preamble' {
        $sourceRoot = Join-Path $TestDrive ('source-fingerprint-dot-path-' + [guid]::NewGuid().ToString('N'))
        $dotDirectory = Join-Path $sourceRoot '.codex'
        [IO.Directory]::CreateDirectory($dotDirectory) | Out-Null
        $trackedPath = Join-Path $dotDirectory 'config with spaces.toml'
        [IO.File]::WriteAllText($trackedPath, 'baseline', [Text.UTF8Encoding]::new($false))

        & git -C $sourceRoot init --quiet
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot config user.email 'mvp-fingerprint@example.invalid'
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot config user.name 'MVP Fingerprint Test'
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot add -- .
        $LASTEXITCODE | Should Be 0
        & git -C $sourceRoot commit --quiet -m 'baseline'
        $LASTEXITCODE | Should Be 0
        [IO.File]::WriteAllText($trackedPath, 'changed', [Text.UTF8Encoding]::new($false))

        $escapedModule = (Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1').Replace("'", "''")
        $escapedSourceRoot = $sourceRoot.Replace("'", "''")
        $command = "Import-Module '$escapedModule' -Force; Get-MvpSourceFingerprint -RepositoryRoot '$escapedSourceRoot'"
        $fingerprint = @(& powershell.exe -NoProfile -ExecutionPolicy Bypass -Command $command)

        $LASTEXITCODE | Should Be 0
        $fingerprint.Count | Should Be 1
        $fingerprint[0].Trim() | Should Match '^[0-9A-F]{64}$'
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

    It 'does not create a profiling output root when source changes before the first managed build' {
        $outputDirectory = Join-Path $TestDrive 'profiling-build-source-change-output'
        $script:fingerprintCallCount = 0
        Mock Assert-RenderExtractProfilingInputDirectory {
            param($Path)
            $outputDirectory
        }
        Mock Get-MvpSourceFingerprint {
            $script:fingerprintCallCount++
            if ($script:fingerprintCallCount -le 2) {
                return 'A' * 64
            }
            return 'B' * 64
        }
        $failure = $null

        try {
            Invoke-RenderExtractProfilingInputBuild -OutputDirectory 'E:\ZirconBuilds\mvp-product-inputs-profile-contract' | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'source fingerprint changed during before runtime-profile-executable build'
        [System.IO.Directory]::Exists($outputDirectory) | Should Be $false
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
            -SourceFingerprint ('A' * 64) `
            -ArtifactOutputDirectory $TestDrive | Out-Null

        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $manifest.schema_version | Should Be 2
        $manifest.source_fingerprint | Should Be ('A' * 64)
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
                -SourceFingerprint ('A' * 64) `
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
