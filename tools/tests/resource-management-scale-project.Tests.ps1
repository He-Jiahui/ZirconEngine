$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$generator = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementScaleProject.ps1'
$changeSet = Join-Path $repoRoot 'tools\mvp\Set-ResourceManagementScaleProjectChangeSet.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$artifactStorageModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'
$originalTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE
$originalChangeSetTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop
Import-Module $artifactStorageModule -Force -Global -ErrorAction Stop

try {
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE = '1'
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE = '1'
    . $generator
    . $changeSet
}
finally {
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE = $originalTestMode
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE = $originalChangeSetTestMode
}

function New-TestResourceManagementScaleSourceIdentity {
    param([string]$BuildSetId = ('A' * 64))

    $templateRoot = Join-Path $repoRoot 'templates\projects\renderable-empty'
    $templatePrefix = $repoRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $files = @([IO.Directory]::EnumerateFiles($templateRoot, '*', [IO.SearchOption]::AllDirectories) |
        ForEach-Object {
            [pscustomobject]@{
                relative_path = $_.Substring($templatePrefix.Length).Replace('\', '/')
            }
        } |
        Sort-Object { $_.relative_path })
    return [pscustomobject]@{
        manifest_path = 'E:\ZirconBuilds\mvp-product-inputs\fixture\mvp-product-inputs.json'
        manifest_sha256 = ('C' * 64)
        source_fingerprint = $BuildSetId
        build_set_id = $BuildSetId
        build_set = [pscustomobject]@{
            build_set_id = $BuildSetId
            snapshot_root = $repoRoot
            files = $files
        }
    }
}

function New-TestResourceManagementProjectPath {
    param([Parameter(Mandatory)][string]$Prefix)

    return New-MvpArtifactStoragePath `
        -NamespaceId 'resource-management-projects' `
        -InstanceId ($Prefix + '-' + [guid]::NewGuid().ToString('N'))
}

$resourceScaleSourceIdentity = New-TestResourceManagementScaleSourceIdentity

Describe 'Resource-management scale project generator' {
    BeforeEach {
        Import-Module $manifestModule -Force -Global -ErrorAction Stop
        Import-Module $resolverModule -Force -Global -ErrorAction Stop
    }

    It 'rejects output roots outside the registered artifact storage roots' {
        { Assert-ResourceManagementScaleProjectDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-project-scale' } |
            Should Throw 'outside the approved'
    }

    It 'keeps change-set mutations inside the registered artifact storage roots' {
        { Assert-ResourceManagementScaleMutationProjectDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-project-scale' } |
            Should Throw 'outside the approved'
    }

    It 'rejects change-set JSON input above its caller-owned byte budget' {
        $path = Join-Path $TestDrive 'oversized-change-set-input.json'
        [IO.File]::WriteAllText(
            $path,
            ('{"payload":"' + ('x' * 64) + '"}'),
            [Text.UTF8Encoding]::new($false))

        {
            Read-ResourceManagementScaleChangeJson `
                -Path $path `
                -Label 'Oversized change-set input' `
                -MaximumBytes 32
        } | Should Throw 'byte budget of 32 bytes'
        (Get-Content -Raw $changeSet) | Should Not Match '\[IO\.File\]::ReadAllText'
    }

    It 'rounds one-percent changes for every required registry scale' {
        Get-ResourceManagementScaleChangeCount -DataAssetCount 1 -ChangePercent 1 | Should Be 1
        Get-ResourceManagementScaleChangeCount -DataAssetCount 1000 -ChangePercent 1 | Should Be 10
        Get-ResourceManagementScaleChangeCount -DataAssetCount 100000 -ChangePercent 1 | Should Be 1000
    }

    It 'resolves ProductInput source identity from the verified BuildSet receipt' {
        Mock Resolve-MvpProductInputManifest {
            [ordered]@{
                operation_path = 'E:\ZirconBuilds\fixture\mvp-product-inputs.json'
                sha256 = ('C' * 64)
                source_fingerprint = ('A' * 64)
                build_set = [ordered]@{
                    build_set_id = ('A' * 64)
                    git_revision = ('b' * 40)
                    dirty_overlay_sha256 = ('D' * 64)
                    manifest_relative_path = 'build-set/build-set.json'
                }
            }
        } -ModuleName MvpProductSourceIdentity
        Mock Assert-MvpProductBuildSet {
            [pscustomobject]@{
                build_set_id = ('A' * 64)
                git_revision = ('b' * 40)
                dirty_overlay_sha256 = ('D' * 64)
                snapshot_root = 'E:\ZirconBuilds\fixture\build-set\source'
                files = @()
            }
        } -ModuleName MvpProductSourceIdentity

        $identity = Resolve-MvpProductSourceIdentity -ManifestPath 'E:\ZirconBuilds\fixture\mvp-product-inputs.json'

        $identity.source_fingerprint | Should Be ('A' * 64)
        $identity.build_set_id | Should Be ('A' * 64)
        $identity.manifest_sha256 | Should Be ('C' * 64)
        $identity.build_set.snapshot_root | Should Be 'E:\ZirconBuilds\fixture\build-set\source'
        Assert-MockCalled Resolve-MvpProductInputManifest -ModuleName MvpProductSourceIdentity -Times 1
        Assert-MockCalled Assert-MvpProductBuildSet -ModuleName MvpProductSourceIdentity -Times 1
    }

    It 'rejects a ProductInput receipt detached from the verified BuildSet' {
        Mock Resolve-MvpProductInputManifest {
            [ordered]@{
                operation_path = 'E:\ZirconBuilds\fixture\mvp-product-inputs.json'
                sha256 = ('C' * 64)
                source_fingerprint = ('A' * 64)
                build_set = [ordered]@{
                    build_set_id = ('A' * 64)
                    git_revision = ('b' * 40)
                    dirty_overlay_sha256 = ('D' * 64)
                    manifest_relative_path = 'build-set/build-set.json'
                }
            }
        } -ModuleName MvpProductSourceIdentity
        Mock Assert-MvpProductBuildSet {
            [pscustomobject]@{
                build_set_id = ('B' * 64)
                git_revision = ('b' * 40)
                dirty_overlay_sha256 = ('D' * 64)
                snapshot_root = 'E:\ZirconBuilds\fixture\build-set\source'
                files = @()
            }
        } -ModuleName MvpProductSourceIdentity

        { Resolve-MvpProductSourceIdentity -ManifestPath 'E:\ZirconBuilds\fixture\mvp-product-inputs.json' } |
            Should Throw 'does not match its verified manifest'
    }

    It 'creates one independent data source for each requested catalog resource' {
        $inventoryModule = Join-Path $repoRoot 'tools\mvp\ResourceManagementScaleInventory.psm1'
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-test'
        try {
            $created = New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceScaleSourceIdentity
            $dataRoot = Join-Path $projectRoot 'assets\data'
            $sourceFiles = @([IO.Directory]::EnumerateFiles($dataRoot, '*.json', [IO.SearchOption]::TopDirectoryOnly)) |
                Sort-Object
            $manifest = [IO.File]::ReadAllText((Join-Path $projectRoot 'resource-management-scale-project.json')) |
                ConvertFrom-Json

            $created.project_root | Should Be (Resolve-ZirconWindowsPath -Path $projectRoot).DisplayPath
            $created.data_asset_count | Should Be 4
            $created.asset_kind | Should Be 'Data'
            $created.importer_id | Should Be 'zircon.builtin.data.json'
            $created.data_virtual_prefix | Should Be 'res://data/'
            $manifest.schema_version | Should Be 2
            $manifest.source_fingerprint | Should Be $resourceScaleSourceIdentity.build_set_id
            $manifest.build_set_id | Should Be $resourceScaleSourceIdentity.build_set_id
            $manifest.product_input_manifest_sha256 | Should Be $resourceScaleSourceIdentity.manifest_sha256
            $manifest.data_asset_count | Should Be 4
            $manifest.asset_kind | Should Be 'Data'
            $manifest.importer_id | Should Be 'zircon.builtin.data.json'
            $manifest.data_virtual_prefix | Should Be 'res://data/'
            $manifest.data_source_pattern | Should Be 'res://data/catalog_*.json'
            $manifest.data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            $created.data_inventory_sha256 | Should Be $manifest.data_inventory_sha256
            (Get-Content -Raw $inventoryModule) | Should Match '\[IO\.Path\]::Combine\(\$DataRoot, \$fileName\)'
            (Get-Content -Raw $inventoryModule) | Should Not Match 'Join-Path \$DataRoot \$fileName'
            (Get-Content -Raw $inventoryModule) | Should Match '\[char\[\]\]::new\(\$HashBytes.Length \* 2\)'
            (Get-Content -Raw $inventoryModule) | Should Not Match '\[Convert\]::ToHexString'
            ($manifest | ConvertTo-Json -Depth 3) | Should Not Match '[A-Za-z]:\\'
            [IO.File]::Exists((Join-Path $projectRoot 'zircon-project.toml')) | Should Be $true
            $sourceFiles.Count | Should Be 4
            ([IO.Directory]::EnumerateFiles($dataRoot, '*.zmeta', [IO.SearchOption]::TopDirectoryOnly) | Measure-Object).Count | Should Be 0

            for ($index = 0; $index -lt $sourceFiles.Count; $index++) {
                $source = [IO.File]::ReadAllText($sourceFiles[$index]) | ConvertFrom-Json
                [IO.Path]::GetFileName($sourceFiles[$index]) | Should Be ('catalog_{0:D6}.json' -f ($index + 1))
                $source.index | Should Be ($index + 1)
                $source.payload | Should Be 'resource-management-scale'
            }
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'binds generator and change-set entry points to one verified ProductInput BuildSet' {
        $generatorSource = Get-Content -LiteralPath $generator -Raw
        $changeSetSource = Get-Content -LiteralPath $changeSet -Raw

        $generatorSource | Should Match 'MvpProductSourceIdentity\.psm1'
        $changeSetSource | Should Match 'MvpProductSourceIdentity\.psm1'
        $generatorSource | Should Match 'Resolve-MvpProductSourceIdentity'
        $changeSetSource | Should Match 'Resolve-MvpProductSourceIdentity'
        $generatorSource | Should Match 'Copy-ResourceManagementScaleTemplate\s+`?\s*-BuildSet'
        $generatorSource | Should Not Match 'Get-MvpSourceFingerprint -RepositoryRoot \$repoRoot'
        $changeSetSource | Should Not Match 'Get-MvpSourceFingerprint -RepositoryRoot \$repoRoot'
    }

    It 'rejects a source identity detached from a verified BuildSet before it creates the project root' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-invalid-fingerprint'

        $invalidIdentity = New-TestResourceManagementScaleSourceIdentity -BuildSetId 'invalid'

        { New-ResourceManagementScaleProject -ProjectRoot $projectRoot -DataAssetCount 1 -SourceIdentity $invalidIdentity } |
            Should Throw 'BuildSet'
        [IO.Directory]::Exists($projectRoot) | Should Be $false
    }

    It 'rejects an invalid resource count before it creates the project root' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-invalid-count'

        $failure = $null
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 0 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
        }
        catch {
            $failure = $_
        }
        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'DataAssetCount|minimum allowed range'
        [IO.Directory]::Exists($projectRoot) | Should Be $false
    }

    It 'applies a deterministic source change set without creating sidecars' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-change-set'
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $change = Set-ResourceManagementScaleProjectChangeSet `
                -ProjectRoot $projectRoot `
                -ChangePercent 25 `
                -ExpectedSourceFingerprint ('A' * 64) `
                -ExpectedProductInputManifestSha256 ('C' * 64)
            $dataRoot = Join-Path $projectRoot 'assets\data'
            $changedSource = [IO.File]::ReadAllText((Join-Path $dataRoot 'catalog_000001.json')) | ConvertFrom-Json
            $unchangedSource = [IO.File]::ReadAllText((Join-Path $dataRoot 'catalog_000002.json')) | ConvertFrom-Json
            $changeManifest = [IO.File]::ReadAllText((Join-Path $projectRoot 'resource-management-scale-change-set.json')) |
                ConvertFrom-Json

            $change.project_root | Should Be (Resolve-ZirconWindowsPath -Path $projectRoot).DisplayPath
            $change.source_fingerprint | Should Be ('A' * 64)
            $change.data_asset_count | Should Be 4
            $change.asset_kind | Should Be 'Data'
            $change.importer_id | Should Be 'zircon.builtin.data.json'
            $change.change_percent | Should Be 25
            $change.changed_asset_count | Should Be 1
            $change.data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            @($change.changed_virtual_paths) | Should Be @('res://data/catalog_000001.json')
            $changedSource.index | Should Be 1
            $changedSource.payload | Should Be 'resource-management-scale'
            $changedSource.workload_revision | Should Be 1
            $unchangedSource.index | Should Be 2
            $unchangedSource.PSObject.Properties['workload_revision'] | Should BeNullOrEmpty
            $changeManifest.source_fingerprint | Should Be ('A' * 64)
            $changeManifest.schema_version | Should Be 2
            $changeManifest.build_set_id | Should Be ('A' * 64)
            $changeManifest.product_input_manifest_sha256 | Should Be ('C' * 64)
            $changeManifest.changed_asset_count | Should Be 1
            $changeManifest.baseline_data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            $changeManifest.changed_data_inventory_sha256 | Should Be $change.data_inventory_sha256
            $changeManifest.baseline_data_inventory_sha256 | Should Not Be $changeManifest.changed_data_inventory_sha256
            @($changeManifest.changed_virtual_paths) | Should Be @('res://data/catalog_000001.json')
            ($changeManifest | ConvertTo-Json -Depth 3) | Should Not Match '[A-Za-z]:\\'
            ([IO.Directory]::EnumerateFiles($dataRoot, '*.zmeta', [IO.SearchOption]::TopDirectoryOnly) | Measure-Object).Count | Should Be 0
            ([IO.Directory]::EnumerateDirectories((Join-Path $projectRoot '.zircon'), 'resource-management-change-set-*') |
                    Measure-Object).Count | Should Be 0

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 25 `
                    -ExpectedSourceFingerprint ('A' * 64) `
                    -ExpectedProductInputManifestSha256 ('C' * 64) } |
                Should Throw 'already has a change set'
            [IO.File]::ReadAllText((Join-Path $dataRoot 'catalog_000001.json')) | ConvertFrom-Json |
                Select-Object -ExpandProperty workload_revision | Should Be 1
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'rejects a change set for a different source snapshot before it modifies data' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-change-fingerprint'
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 1 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $sourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($sourcePath)

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 1 `
                    -ExpectedSourceFingerprint ('B' * 64) `
                    -ExpectedProductInputManifestSha256 ('C' * 64) } |
                Should Throw 'different source snapshot'
            [IO.File]::ReadAllText($sourcePath) | Should Be $before
            [IO.File]::Exists((Join-Path $projectRoot 'resource-management-scale-change-set.json')) | Should Be $false
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'rejects a change set from a different ProductInputManifest before it modifies data' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-change-product-input'
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 1 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $sourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($sourcePath)

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 1 `
                    -ExpectedSourceFingerprint ('A' * 64) `
                    -ExpectedProductInputManifestSha256 ('D' * 64) } |
                Should Throw 'different ProductInputManifest'
            [IO.File]::ReadAllText($sourcePath) | Should Be $before
            [IO.File]::Exists((Join-Path $projectRoot 'resource-management-scale-change-set.json')) | Should Be $false
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'rejects an incomplete data inventory before it mutates a scale project' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-incomplete-inventory'
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $firstSourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($firstSourcePath)
            [IO.File]::Delete((Join-Path $projectRoot 'assets\data\catalog_000004.json'))

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 25 `
                    -ExpectedSourceFingerprint ('A' * 64) `
                    -ExpectedProductInputManifestSha256 ('C' * 64) } |
                Should Throw 'data source inventory'
            [IO.File]::ReadAllText($firstSourcePath) | Should Be $before
            [IO.File]::Exists((Join-Path $projectRoot 'resource-management-scale-change-set.json')) | Should Be $false
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'rejects an undeclared data mutation before it creates a one-percent change set' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-undeclared-mutation'
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $firstSourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($firstSourcePath)
            [IO.File]::WriteAllText(
                (Join-Path $projectRoot 'assets\data\catalog_000004.json'),
                '{"index":4,"payload":"resource-management-scale","undeclared":true}' + [Environment]::NewLine,
                [Text.UTF8Encoding]::new($false)
            )

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 1 `
                    -ExpectedSourceFingerprint ('A' * 64) `
                    -ExpectedProductInputManifestSha256 ('C' * 64) } |
                Should Throw 'does not match its immutable metadata fingerprint'
            [IO.File]::ReadAllText($firstSourcePath) | Should Be $before
            [IO.File]::Exists((Join-Path $projectRoot 'resource-management-scale-change-set.json')) | Should Be $false
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }

    It 'rejects a concurrent change-set lease before it modifies data' {
        $projectRoot = New-TestResourceManagementProjectPath -Prefix 'resource-management-scale-change-lease'
        $lease = $null
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 1 `
                -SourceIdentity $resourceScaleSourceIdentity | Out-Null
            $sourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($sourcePath)
            $leasePath = Join-Path $projectRoot '.zircon\resource-management-scale-change-set.active'
            $lease = [IO.FileStream]::new(
                $leasePath,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::ReadWrite,
                [IO.FileShare]::None,
                1,
                [IO.FileOptions]::DeleteOnClose
            )

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 1 `
                    -ExpectedSourceFingerprint ('A' * 64) `
                    -ExpectedProductInputManifestSha256 ('C' * 64) } |
                Should Throw 'already active'
            [IO.File]::ReadAllText($sourcePath) | Should Be $before
            [IO.File]::Exists((Join-Path $projectRoot 'resource-management-scale-change-set.json')) | Should Be $false
        }
        finally {
            if ($null -ne $lease) {
                $lease.Dispose()
            }
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }
}
