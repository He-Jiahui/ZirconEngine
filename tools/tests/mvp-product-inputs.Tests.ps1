$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$builder = Join-Path $repoRoot 'tools\mvp\Build-MvpProductInputs.ps1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$productProfileRegistryModule = Join-Path $repoRoot 'tools\mvp\MvpProductProfileRegistry.psm1'
$productProfileRegistryPath = Join-Path $repoRoot 'tools\mvp\mvp-product-profile-registry.json'
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

    It 'loads TargetProfile Role and Configuration product specifications from one versioned registry' {
        (Test-Path -LiteralPath $productProfileRegistryModule -PathType Leaf) | Should Be $true
        (Test-Path -LiteralPath $productProfileRegistryPath -PathType Leaf) | Should Be $true
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop

        $snapshot = Get-MvpProductProfileRegistrySnapshot
        $specifications = @(Get-MvpProductInputSpecifications -RegistrySnapshot $snapshot)

        @($snapshot.profiles).Count | Should Be 2
        $specifications.Count | Should Be 4
        (@($snapshot.profiles.profile_id) -join ',') | Should Be 'runtime-windows-development,editor-windows-development'
        (@($snapshot.profiles.target_profile) -join ',') | Should Be 'target-client-platform,target-editor-host'
        (@($snapshot.profiles.role) -join ',') | Should Be 'runtime,editor'
        @($snapshot.profiles | Where-Object { $_.configuration -ne 'development' }).Count | Should Be 0
        @($snapshot.profiles | Where-Object { $_.platform -ne 'windows' }).Count | Should Be 0
        $specifications[0].features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
    }

    It 'freezes the exact product profile registry bytes into one reusable receipt' {
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop

        $snapshot = Get-MvpProductProfileRegistrySnapshot

        @($snapshot.receipt.PSObject.Properties).Count | Should Be 4
        $snapshot.receipt.schema_version | Should Be 1
        $snapshot.receipt.registry_kind | Should Be 'zircon.mvp-product-profile-registry'
        $snapshot.receipt.sha256 | Should Be (Get-FileHash -LiteralPath $productProfileRegistryPath -Algorithm SHA256).Hash
        $snapshot.receipt.size_bytes | Should Be ([IO.FileInfo]::new($productProfileRegistryPath).Length)
    }

    It 'rejects a product profile receipt detached from the current registry snapshot' {
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
        $snapshot = Get-MvpProductProfileRegistrySnapshot
        $receipt = $snapshot.receipt | ConvertTo-Json | ConvertFrom-Json
        $receipt.sha256 = ('0' * 64)

        { Assert-MvpProductProfileRegistryReceipt -Receipt $receipt -ExpectedSnapshot $snapshot } |
            Should Throw 'receipt sha256 differs'
    }

    It 'rejects an unknown product profile registry property' {
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'unknown-product-profile-property.json'
        $fixture = Get-Content -LiteralPath $productProfileRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture | Add-Member -NotePropertyName unexpected_property -NotePropertyValue 'must-fail'
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpProductProfileRegistrySnapshot -RegistryPath $fixturePath } |
            Should Throw "unknown property 'unexpected_property'"
    }

    It 'rejects duplicate product logical IDs across profiles' {
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'duplicate-product-logical-id.json'
        $fixture = Get-Content -LiteralPath $productProfileRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture.profiles[1].products[0].logical_id = $fixture.profiles[0].products[0].logical_id
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpProductProfileRegistrySnapshot -RegistryPath $fixturePath } |
            Should Throw 'duplicate product logical_id'
    }

    It 'rejects a feature token that could collapse the validator argv boundary' {
        Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'unsafe-product-profile-feature.json'
        $fixture = Get-Content -LiteralPath $productProfileRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture.profiles[0].features[0] = 'target-client --release'
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpProductProfileRegistrySnapshot -RegistryPath $fixturePath } |
            Should Throw 'feature token'
    }

    It 'generates builder requests from one frozen profile snapshot and binds its receipt' {
        $builderSource = Get-Content -LiteralPath $builder -Raw
        $manifestSource = Get-Content -LiteralPath $manifestModule -Raw

        $builderSource | Should Match '\$productProfileRegistrySnapshot = Get-MvpProductProfileRegistrySnapshot'
        $builderSource | Should Match 'Get-MvpProductBuildRequests -RegistrySnapshot \$productProfileRegistrySnapshot'
        $builderSource | Should Match 'product_profile_registry\s+= \$productProfileRegistrySnapshot\.receipt'
        $builderSource | Should Match 'schema_version\s+= 2'
        $manifestSource | Should Match 'Assert-MvpProductProfileRegistryReceipt'
        $manifestSource | Should Match '\$script:MvpProductInputSchemaVersion = 2'
        $manifestSource | Should Not Match '\$script:MvpProductInputSpecifications\s*='
    }

    It 'encodes product input SHA-256 values through one fixed-size uppercase buffer' {
        $bytes = [byte[]]@(0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF)
        $artifactPath = Join-Path $TestDrive 'sha256-boundaries.bin'
        $manifestSource = Get-Content -LiteralPath $manifestModule -Raw

        [IO.File]::WriteAllBytes($artifactPath, $bytes)
        $hasher = [Security.Cryptography.SHA256]::Create()
        try {
            $expected = ([BitConverter]::ToString($hasher.ComputeHash($bytes))).Replace('-', '')
        }
        finally {
            $hasher.Dispose()
        }
        Get-MvpProductInputFileSha256 -Path $artifactPath | Should Be $expected
        $manifestSource | Should Match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)'
        $manifestSource | Should Not Match "ToString\('X2'\)"
    }

    It 'writes its product manifest through the resolver operational path' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'Publish-MvpProductInputManifest `'
        $builderSource | Should Match '\[IO\.FileMode\]::CreateNew'
        $builderSource | Should Not Match 'Set-Content -LiteralPath \$summaryPath'
        $builderSource | Should Not Match 'Remove-Item -LiteralPath \$summaryPath'
    }

    It 'writes a durable abort receipt without overwriting earlier failure evidence' {
        $abortPath = Join-Path $TestDrive 'mvp-product-inputs-aborted.json'

        Write-MvpProductInputAbortReceipt `
            -Path $abortPath `
            -ArtifactOutputName 'mvp-product-inputs-aborted-fixture' `
            -FailureKind 'build_failed' `
            -FailureMessage 'fixture build failure'

        $receipt = [IO.File]::ReadAllText($abortPath, [Text.UTF8Encoding]::new($false)) | ConvertFrom-Json
        $receipt.schema_version | Should Be 1
        $receipt.receipt_kind | Should Be 'zircon.mvp-product-input-abort'
        $receipt.artifact_output_name | Should Be 'mvp-product-inputs-aborted-fixture'
        $receipt.failure_kind | Should Be 'build_failed'
        $receipt.failure_message_length | Should Be 21
        $receipt.failure_message_prefix_length | Should Be 21
        $receipt.failure_message_prefix_sha256 | Should Match '^[0-9A-F]{64}$'
        $receipt.failure_message_truncated | Should Be $false
        @($receipt.PSObject.Properties).Count | Should Be 9
        ([IO.File]::ReadAllText($abortPath) -match 'fixture build failure') | Should Be $false

        {
            Write-MvpProductInputAbortReceipt `
                -Path $abortPath `
                -ArtifactOutputName 'mvp-product-inputs-aborted-fixture' `
                -FailureKind 'build_failed' `
                -FailureMessage 'different fixture build failure'
        } | Should Throw 'Refusing to overwrite existing MVP product input manifest'
    }

    It 'atomically publishes one sibling abort receipt without occupying the artifact root' {
        $publicationParent = (Resolve-ZirconWindowsPath -Path (Join-Path $TestDrive 'abort-publication')).OperationalPath
        [IO.Directory]::CreateDirectory($publicationParent) | Out-Null

        $abortPath = Publish-MvpProductInputAbortReceipt `
            -PublicationParent $publicationParent `
            -PublicationLeaf 'mvp-product-inputs-aborted-fixture' `
            -FailureKind 'io_failure' `
            -FailureMessage ('sensitive-' + ('x' * 5000))

        [IO.File]::Exists($abortPath) | Should Be $true
        [IO.Directory]::Exists((Join-ZirconWindowsPath -Path $publicationParent -ChildPath 'mvp-product-inputs-aborted-fixture')) | Should Be $false
        @(Get-ChildItem -LiteralPath $publicationParent -Filter '*.partial-*').Count | Should Be 0
        $receipt = [IO.File]::ReadAllText($abortPath) | ConvertFrom-Json
        $receipt.failure_message_prefix_length | Should Be 4096
        $receipt.failure_message_truncated | Should Be $true
        ([IO.File]::ReadAllText($abortPath) -match 'sensitive-') | Should Be $false

        {
            Publish-MvpProductInputAbortReceipt `
                -PublicationParent $publicationParent `
                -PublicationLeaf 'mvp-product-inputs-aborted-fixture' `
                -FailureKind 'build_failed' `
                -FailureMessage 'later failure'
        } | Should Throw 'already exists'
    }

    It 'stages every product artifact before one atomic publication move' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match '\$publicationDirectory = Join-ZirconWindowsPath -Path \$publicationParent -ChildPath \(\$publicationLeaf \+ "\.partial-" \+ \[guid\]::NewGuid\(\)\.ToString\("N"\)\)'
        $builderSource | Should Match '\$stagedGroupDirectory = Join-ZirconWindowsPath -Path \$publicationDirectory -ChildPath \$request\.OutputGroup'
        $builderSource | Should Match '"-ArtifactOutputDirectory", \$stagedGroupDirectory'
        $builderSource | Should Match 'Publish-MvpProductInputPublicationRoot `'
        $builderSource | Should Match 'Move-ZirconWindowsPath -Source \$PublicationDirectory -Destination \$OutputDirectory -ApprovedRoot \$PublicationParent'
        $builderSource | Should Match 'Publish-MvpProductInputAbortReceipt'
        $builderSource | Should Not Match '\$publicationDirectory.+mvp-product-inputs-aborted\.json'
        $builderSource | Should Not Match '\[System\.IO\.Directory\]::CreateDirectory\(\$resolvedOutputDirectory\)'
    }

    It 'moves one completed staged publication root into an empty target' {
        $publicationParent = (Resolve-ZirconWindowsPath -Path (Join-Path $TestDrive 'atomic-publication')).OperationalPath
        $publicationDirectory = Join-ZirconWindowsPath -Path $publicationParent -ChildPath 'candidate.partial-fixture'
        $outputDirectory = Join-ZirconWindowsPath -Path $publicationParent -ChildPath 'published'
        [IO.Directory]::CreateDirectory($publicationDirectory) | Out-Null
        $stagedManifest = Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath 'mvp-product-inputs.json'
        [IO.File]::WriteAllText($stagedManifest, '{"state":"complete"}', [Text.UTF8Encoding]::new($false))

        Publish-MvpProductInputPublicationRoot `
            -PublicationDirectory $publicationDirectory `
            -OutputDirectory $outputDirectory `
            -PublicationParent $publicationParent

        [IO.Directory]::Exists($publicationDirectory) | Should Be $false
        [IO.File]::Exists((Join-ZirconWindowsPath -Path $outputDirectory -ChildPath 'mvp-product-inputs.json')) | Should Be $true

        $rejectedStagingDirectory = Join-ZirconWindowsPath -Path $publicationParent -ChildPath 'rejected.partial-fixture'
        $occupiedOutputDirectory = Join-ZirconWindowsPath -Path $publicationParent -ChildPath 'occupied'
        [IO.Directory]::CreateDirectory($rejectedStagingDirectory) | Out-Null
        [IO.File]::WriteAllText((Join-ZirconWindowsPath -Path $rejectedStagingDirectory -ChildPath 'mvp-product-inputs.json'), '{"state":"complete"}', [Text.UTF8Encoding]::new($false))
        [IO.Directory]::CreateDirectory($occupiedOutputDirectory) | Out-Null
        [IO.File]::WriteAllText((Join-ZirconWindowsPath -Path $occupiedOutputDirectory -ChildPath 'earlier-evidence.json'), '{"state":"preserved"}', [Text.UTF8Encoding]::new($false))

        {
            Publish-MvpProductInputPublicationRoot `
                -PublicationDirectory $rejectedStagingDirectory `
                -OutputDirectory $occupiedOutputDirectory `
                -PublicationParent $publicationParent
        } | Should Throw 'publication target must remain empty'

        [IO.Directory]::Exists($rejectedStagingDirectory) | Should Be $true
        [IO.File]::Exists((Join-ZirconWindowsPath -Path $occupiedOutputDirectory -ChildPath 'earlier-evidence.json')) | Should Be $true
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

    It 'does not re-read the active checkout after publishing a BuildSet-backed manifest' {
        $manifestPath = Join-Path $TestDrive 'build-set-backed-manifest.json'
        Mock Assert-MvpProductBuildSet {}

        Publish-MvpProductInputManifest `
            -Path $manifestPath `
            -Summary ([ordered]@{
                    schema_version = 1
                    source_fingerprint = ('A' * 64)
                    build_set = [ordered]@{
                        build_set_id = ('B' * 64)
                        git_revision = ('c' * 40)
                        dirty_overlay_sha256 = ('D' * 64)
                    }
                    artifacts = @()
                }) `
            -BuildSet ([pscustomobject]@{ manifest_path = (Join-Path $TestDrive 'build-set.json') })

        $published = [IO.File]::ReadAllText($manifestPath, [Text.UTF8Encoding]::new($false)) | ConvertFrom-Json
        $published.build_set.build_set_id | Should Be ('B' * 64)
    }

    It 'publishes display paths while retaining operational paths for artifact I/O' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match '\$publishedArtifactPath = Join-ZirconWindowsPath'
        $builderSource | Should Match '\$artifactDisplayPath = \(Resolve-ZirconWindowsPath -Path \$publishedArtifactPath\)\.DisplayPath'
        $builderSource | Should Match 'Path\s+= \$artifactDisplayPath'
        $builderSource | Should Match 'artifact_output_directory = \(Resolve-ZirconWindowsPath -Path \$resolvedOutputDirectory\)\.DisplayPath'
        $builderSource | Should Not Match 'artifact_output_directory = \$resolvedOutputDirectory'
    }

    It 'runs managed builds from an immutable BuildSet snapshot' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'Import-Module .*MvpProductInputManifest\.psm1'
        $builderSource | Should Match 'Import-Module .*MvpBuildSet\.psm1'
        $builderSource | Should Match '\$buildSet = New-MvpProductBuildSet'
        $builderSource | Should Match 'Assert-MvpProductBuildSet'
        $builderSource | Should Match '"-RepoRoot", \$buildSet\.snapshot_root'
        $builderSource | Should Not Match 'Assert-MvpProductInputSourceFingerprint'
        $builderSource | Should Match 'source_fingerprint\s+= \$buildSet\.build_set_id'
        $builderSource | Should Match 'build_set\s+= \[ordered\]@\{'
        $builderSource | Should Match 'LogicalId\s+= \$request\.logical_id'

        ([regex]::Matches($builderSource, '\bGet-MvpSourceFingerprint\b')).Count | Should Be 0
    }

    It 'uses the immutable BuildSet identity as its source fingerprint' {
        $builderSource = Get-Content -LiteralPath $builder -Raw

        $builderSource | Should Match 'build_set_id\s+= \$buildSet\.build_set_id'
        $builderSource | Should Match 'git_revision\s+= \$buildSet\.git_revision'
        $builderSource | Should Match 'dirty_overlay_sha256\s+= \$buildSet\.dirty_overlay_sha256'
        $builderSource | Should Match 'manifest_relative_path\s+= ''build-set/build-set\.json'''
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
        $productProfileRegistrySnapshot = Get-MvpProductProfileRegistrySnapshot
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('B' * 64)
            product_profile_registry = $productProfileRegistrySnapshot.receipt
            build_set = [ordered]@{
                build_set_id = ('B' * 64)
                git_revision = ('c' * 40)
                dirty_overlay_sha256 = ('D' * 64)
                manifest_relative_path = 'build-set/build-set.json'
            }
            artifacts = @($artifacts)
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        $resolved = Resolve-MvpProductInputManifest -Path $manifestPath

        $resolved.bytes | Should Be ([IO.FileInfo]::new($manifestPath).Length)
        $resolved.sha256 | Should Be (Get-MvpProductInputFileSha256 -Path $manifestPath)
        $resolved.build_set.build_set_id | Should Be ('B' * 64)
        $resolved.build_set.git_revision | Should Be ('c' * 40)
        $resolved.build_set.manifest_relative_path | Should Be 'build-set/build-set.json'
        $resolved.product_profile_registry.sha256 | Should Be $productProfileRegistrySnapshot.receipt.sha256
    }

    It 'rejects a source fingerprint detached from its BuildSet identity' {
        $manifestPath = Join-Path $TestDrive 'detached-source-fingerprint.json'
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('A' * 64)
            product_profile_registry = (Get-MvpProductProfileRegistrySnapshot).receipt
            build_set = [ordered]@{
                build_set_id = ('B' * 64)
                git_revision = ('c' * 40)
                dirty_overlay_sha256 = ('D' * 64)
                manifest_relative_path = 'build-set/build-set.json'
            }
            artifacts = @()
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        { Resolve-MvpProductInputManifest -Path $manifestPath } |
            Should Throw 'source_fingerprint must equal its BuildSetId'
    }

    It 'rejects malformed optional BuildSet identity before consuming product artifacts' {
        $manifestPath = Join-Path $TestDrive 'invalid-build-set.json'
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('A' * 64)
            product_profile_registry = (Get-MvpProductProfileRegistrySnapshot).receipt
            build_set = [ordered]@{
                build_set_id = 'invalid'
                git_revision = ('c' * 40)
                dirty_overlay_sha256 = ('D' * 64)
            }
            artifacts = @()
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        $failure = $null
        try {
            Resolve-MvpProductInputManifest -Path $manifestPath | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'build_set_id'
    }

    It 'rejects an unsafe BuildSet receipt path before consuming product artifacts' {
        $manifestPath = Join-Path $TestDrive 'unsafe-build-set-path.json'
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('A' * 64)
            product_profile_registry = (Get-MvpProductProfileRegistrySnapshot).receipt
            build_set = [ordered]@{
                build_set_id = ('B' * 64)
                git_revision = ('c' * 40)
                dirty_overlay_sha256 = ('D' * 64)
                manifest_relative_path = '../outside/build-set.json'
            }
            artifacts = @()
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        {
            Resolve-MvpProductInputManifest -Path $manifestPath
        } | Should Throw 'manifest_relative_path'
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
