$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$builder = Join-Path $repoRoot 'tools\mvp\Build-MvpProductInputs.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$originalTestMode = $env:MVP_PRODUCT_INPUTS_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop

try {
    $env:MVP_PRODUCT_INPUTS_TEST_MODE = '1'
    . $builder
}
finally {
    $env:MVP_PRODUCT_INPUTS_TEST_MODE = $originalTestMode
}

Describe 'MVP product input build plan' {
    BeforeEach {
        # Pester v4 executes each example in its own scope, so make the shared resolver
        # available to both fixture construction and the deferred build-driver functions.
        Import-Module $resolverModule -Force -ErrorAction Stop
    }

    It 'keeps its Windows path resolver visible after importing manifest helpers' {
        $isolatedBuilderInvocation = @'
$ErrorActionPreference = 'Stop'
$env:MVP_PRODUCT_INPUTS_TEST_MODE = '1'
. '__BUILDER__'
$target = Join-Path 'D:\ZirconBuilds' ('mvp-product-inputs-resolver-' + [guid]::NewGuid().ToString('N'))
$resolved = Assert-MvpProductInputDirectory -Path $target
if ([string]::IsNullOrWhiteSpace($resolved)) {
    throw 'Build product input resolver returned a blank operation path.'
}
'@.Replace('__BUILDER__', $builder.Replace("'", "''"))

        $null = & powershell.exe -NoProfile -ExecutionPolicy Bypass -Command $isolatedBuilderInvocation 2>&1
        $LASTEXITCODE | Should Be 0
    }

    It 'keeps client and editor-host artifacts in separate feature-scoped directories' {
        $requests = @(Get-MvpProductBuildRequests)
        $runtimeExecutable = $requests | Where-Object { $_.ArtifactName -eq 'zircon_runtime.exe' }
        $runtimeLibrary = $requests | Where-Object {
            $_.ArtifactName -eq 'zircon_runtime.dll' -and $_.OutputGroup -eq 'runtime'
        }
        $editorExecutable = $requests | Where-Object { $_.ArtifactName -eq 'zircon_editor.exe' }
        $editorLibrary = $requests | Where-Object {
            $_.ArtifactName -eq 'zircon_runtime.dll' -and $_.OutputGroup -eq 'editor'
        }

        $requests.Count | Should Be 4
        $runtimeExecutable.logical_id | Should Be 'runtime-executable'
        $runtimeExecutable.Package | Should Be 'zircon_app'
        $runtimeExecutable.Bin | Should Be 'zircon_runtime'
        $runtimeExecutable.Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        $runtimeExecutable.OutputGroup | Should Be 'runtime'
        $runtimeLibrary.Package | Should Be 'zircon_runtime'
        $runtimeLibrary.logical_id | Should Be 'runtime-library/runtime'
        $runtimeLibrary.Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        $editorExecutable.Package | Should Be 'zircon_app'
        $editorExecutable.logical_id | Should Be 'editor-executable'
        $editorExecutable.Bin | Should Be 'zircon_editor'
        $editorExecutable.Features | Should Be 'target-editor-host'
        $editorExecutable.OutputGroup | Should Be 'editor'
        $editorLibrary.Package | Should Be 'zircon_runtime'
        $editorLibrary.logical_id | Should Be 'runtime-library/editor'
        $editorLibrary.Features | Should Be 'target-editor-host'
    }

    It 'writes its product manifest through the resolver operational path' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'Publish-MvpProductInputManifest `'
        $builderSource | Should Match '\[IO\.FileMode\]::CreateNew'
        $builderSource | Should Not Match 'Set-Content -LiteralPath \$summaryPath'
        $builderSource | Should Not Match 'Remove-Item -LiteralPath \$summaryPath'
    }

    It 'publishes one BOM-less product input manifest' {
        $manifestPath = Join-Path $TestDrive 'mvp-product-inputs.json'
        $summary = [ordered]@{
            schema_version = 1
            source_fingerprint = ('A' * 64)
            artifacts = @()
        }

        Write-MvpProductInputManifest -Path $manifestPath -Summary $summary

        $bytes = [IO.File]::ReadAllBytes($manifestPath)
        $bytes.Length | Should BeGreaterThan 0
        $bytes[0] | Should Be ([byte][char]'{')
        $published = [Text.UTF8Encoding]::new($false).GetString($bytes) | ConvertFrom-Json
        $published.schema_version | Should Be 1
        $published.source_fingerprint | Should Be ('A' * 64)
    }

    It 'refuses to overwrite an existing product input manifest' {
        $manifestPath = Join-Path $TestDrive 'mvp-product-inputs.json'
        $existingBytes = [byte[]]@(0x66, 0x69, 0x78, 0x74, 0x75, 0x72, 0x65)
        [IO.File]::WriteAllBytes($manifestPath, $existingBytes)

        $failure = $null
        try {
            Write-MvpProductInputManifest -Path $manifestPath -Summary ([ordered]@{
                    schema_version = 1
                    source_fingerprint = ('A' * 64)
                    artifacts = @()
                })
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'Refusing to overwrite existing MVP product input manifest'
        [IO.File]::ReadAllBytes($manifestPath) | Should Be $existingBytes
    }

    It 'preserves a non-collision manifest I/O failure' {
        $manifestPath = Join-Path (Join-Path $TestDrive 'missing-parent') 'mvp-product-inputs.json'

        $failure = $null
        try {
            Write-MvpProductInputManifest -Path $manifestPath -Summary ([ordered]@{
                    schema_version = 1
                    source_fingerprint = ('A' * 64)
                    artifacts = @()
                })
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Not Match 'Refusing to overwrite'
        $failure.Exception.InnerException.GetType().FullName | Should Be 'System.IO.DirectoryNotFoundException'
    }

    It 'preserves a replacement manifest when the post-publication fingerprint changes' {
        $manifestPath = Join-Path $TestDrive 'post-publication-fingerprint.json'
        $replacementBytes = [byte[]]@(0x72, 0x65, 0x70, 0x6C, 0x61, 0x63, 0x65, 0x64)
        Mock Get-MvpSourceFingerprint {
            [IO.File]::WriteAllBytes($manifestPath, $replacementBytes)
            return ('B' * 64)
        }

        $failure = $null
        try {
            Publish-MvpProductInputManifest `
                -Path $manifestPath `
                -Summary ([ordered]@{
                        schema_version = 1
                        source_fingerprint = ('A' * 64)
                        artifacts = @()
                    }) `
                -RepositoryRoot $repoRoot `
                -ExpectedFingerprint ('A' * 64)
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'source fingerprint changed during after product input manifest publication'
        [IO.File]::ReadAllBytes($manifestPath) | Should Be $replacementBytes
    }

    It 'publishes display paths while retaining operational paths for artifact I/O' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match '\$artifactDisplayPath = \(Resolve-ZirconWindowsPath -Path \$artifactPath\)\.DisplayPath'
        $builderSource | Should Match 'Path\s+= \$artifactDisplayPath'
        $builderSource | Should Match 'artifact_output_directory = \(Resolve-ZirconWindowsPath -Path \$resolvedOutputDirectory\)\.DisplayPath'
        $builderSource | Should Not Match 'artifact_output_directory = \$resolvedOutputDirectory'
    }

    It 'binds its artifact manifest to the source before launching managed builds' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'Import-Module .*MvpProductInputManifest\.psm1'
        $builderSource | Should Match '\$sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot \$repoRoot'
        $builderSource | Should Match 'source_fingerprint\s+= \$sourceFingerprint'
        $builderSource | Should Match 'LogicalId\s+= \$request\.logical_id'
    }

    It 'rejects a source fingerprint that changes while product inputs are being built' {
        Mock Get-MvpSourceFingerprint { 'B' * 64 }

        $failure = $null
        try {
            Assert-MvpProductInputSourceFingerprint `
                -RepositoryRoot $repoRoot `
                -ExpectedFingerprint ('A' * 64) `
                -Phase 'after runtime artifact build'
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'source fingerprint changed during after runtime artifact build'
    }

    It 'hashes manifest artifacts through the managed SHA-256 provider' {
        $artifact = Join-Path $TestDrive 'empty-artifact.bin'
        [System.IO.File]::WriteAllBytes($artifact, [byte[]]@())

        Get-MvpProductInputFileSha256 -Path $artifact | Should Be 'E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855'
    }

    It 'returns the exact raw manifest byte evidence consumed during resolution' {
        $artifactRoot = Join-Path $TestDrive 'manifest-artifacts'
        [System.IO.Directory]::CreateDirectory($artifactRoot) | Out-Null
        $artifacts = foreach ($specification in Get-MvpProductInputSpecifications) {
            $artifactPath = Join-Path $artifactRoot ($specification.logical_id.Replace('/', '-'))
            [System.IO.File]::WriteAllText($artifactPath, $specification.logical_id, [Text.UTF8Encoding]::new($false))
            [ordered]@{
                LogicalId = $specification.logical_id
                Package = $specification.package
                Bin = $specification.bin
                Features = $specification.features
                OutputGroup = $specification.output_group
                ArtifactName = $specification.artifact_name
                Path = (Resolve-ZirconWindowsPath -Path $artifactPath).DisplayPath
                Bytes = [IO.FileInfo]::new($artifactPath).Length
                Sha256 = Get-MvpProductInputFileSha256 -Path $artifactPath
            }
        }
        $manifestPath = Join-Path $TestDrive 'mvp-product-inputs.json'
        $manifest = [ordered]@{
            schema_version = 1
            source_fingerprint = ('A' * 64)
            artifacts = @($artifacts)
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        $resolved = Resolve-MvpProductInputManifest -Path $manifestPath

        $resolved.bytes | Should Be ([IO.FileInfo]::new($manifestPath).Length)
        $resolved.sha256 | Should Be (Get-MvpProductInputFileSha256 -Path $manifestPath)
    }

    It 'does not request ephemeral lanes for sequential product artifacts' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Not Match '"-Ephemeral"'
    }

    It 'accepts only the dedicated physical MVP product-input root' {
        $requestedPath = "D:\ZirconBuilds\mvp-product-inputs-contract-$([guid]::NewGuid().ToString('N'))"

        $resolved = Assert-MvpProductInputDirectory -Path $requestedPath
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Match '^D:\\ZirconBuilds\\mvp-product-inputs-'
    }

    It 'rejects output paths outside the dedicated physical MVP product-input root' {
        $messages = @(
            'C:\zircon-mvp-product-inputs',
            'D:\ZirconBuilds\unscoped-product-inputs',
            'E:\ZirconBuilds\mvp-product-inputs'
        ) | ForEach-Object {
            try {
                Assert-MvpProductInputDirectory -Path $_
                $null
            }
            catch {
                $_.Exception.Message
            }
        }

        $messages.Count | Should Be 3
        $messages | ForEach-Object {
            $_ | Should Match 'MVP product input artifact output must resolve under'
        }
    }

    It 'rejects drive-relative product input paths before resolving their per-drive working directory' {
        $rejected = $false
        try {
            Assert-MvpProductInputDirectory -Path 'C:ambiguous-product-inputs'
        }
        catch {
            $rejected = $_.Exception.Message -match 'drive-rooted'
        }

        $rejected | Should Be $true
    }

    It 'keeps the resolver physical path for the dedicated product-input directory' {
        $requestedPath = "D:\ZirconBuilds\mvp-product-inputs-physical-$([guid]::NewGuid().ToString('N'))"
        $resolvedPath = Assert-MvpProductInputDirectory -Path $requestedPath
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolvedPath | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Match '^D:\\ZirconBuilds\\mvp-product-inputs-physical-'
    }
}
