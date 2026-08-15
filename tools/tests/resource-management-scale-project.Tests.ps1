$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$generator = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementScaleProject.ps1'
$changeSet = Join-Path $repoRoot 'tools\mvp\Set-ResourceManagementScaleProjectChangeSet.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$originalTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE
$originalChangeSetTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop

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

Describe 'Resource-management scale project generator' {
    BeforeEach {
        Import-Module $manifestModule -Force -Global -ErrorAction Stop
        Import-Module $resolverModule -Force -Global -ErrorAction Stop
    }

    It 'rejects output roots outside the approved E drive fixture root' {
        { Assert-ResourceManagementScaleProjectDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-projects\scale' } |
            Should Throw 'mvp-resource-management-projects'
    }

    It 'keeps change-set mutations inside the approved E drive fixture root' {
        { Assert-ResourceManagementScaleMutationProjectDirectory -Path 'C:\ZirconBuilds\mvp-resource-management-projects\scale' } |
            Should Throw 'mvp-resource-management-projects'
    }

    It 'rounds one-percent changes for every required registry scale' {
        Get-ResourceManagementScaleChangeCount -DataAssetCount 1 -ChangePercent 1 | Should Be 1
        Get-ResourceManagementScaleChangeCount -DataAssetCount 1000 -ChangePercent 1 | Should Be 10
        Get-ResourceManagementScaleChangeCount -DataAssetCount 100000 -ChangePercent 1 | Should Be 1000
    }

    It 'creates one independent data source for each requested catalog resource' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-test-' + [guid]::NewGuid().ToString('N')
        )
        try {
            $created = New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceFingerprint ('A' * 64)
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
            $manifest.data_asset_count | Should Be 4
            $manifest.asset_kind | Should Be 'Data'
            $manifest.importer_id | Should Be 'zircon.builtin.data.json'
            $manifest.data_virtual_prefix | Should Be 'res://data/'
            $manifest.data_source_pattern | Should Be 'res://data/catalog_*.json'
            $manifest.data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            $created.data_inventory_sha256 | Should Be $manifest.data_inventory_sha256
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

    It 'rejects malformed source fingerprints before it creates the project root' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-invalid-fingerprint-' + [guid]::NewGuid().ToString('N')
        )

        { New-ResourceManagementScaleProject -ProjectRoot $projectRoot -DataAssetCount 1 -SourceFingerprint 'invalid' } |
            Should Throw 'source fingerprint'
        [IO.Directory]::Exists($projectRoot) | Should Be $false
    }

    It 'rejects an invalid resource count before it creates the project root' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-invalid-count-' + [guid]::NewGuid().ToString('N')
        )

        { New-ResourceManagementScaleProject -ProjectRoot $projectRoot -DataAssetCount 0 -SourceFingerprint ('A' * 64) } |
            Should Throw
        [IO.Directory]::Exists($projectRoot) | Should Be $false
    }

    It 'applies a deterministic source change set without creating sidecars' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-change-set-' + [guid]::NewGuid().ToString('N')
        )
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceFingerprint ('A' * 64) | Out-Null
            $change = Set-ResourceManagementScaleProjectChangeSet `
                -ProjectRoot $projectRoot `
                -ChangePercent 25 `
                -ExpectedSourceFingerprint ('A' * 64)
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
            $unchangedSource.workload_revision | Should BeNullOrEmpty
            $changeManifest.source_fingerprint | Should Be ('A' * 64)
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
                    -ExpectedSourceFingerprint ('A' * 64) } |
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
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-change-fingerprint-' + [guid]::NewGuid().ToString('N')
        )
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 1 `
                -SourceFingerprint ('A' * 64) | Out-Null
            $sourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($sourcePath)

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 1 `
                    -ExpectedSourceFingerprint ('B' * 64) } |
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

    It 'rejects an incomplete data inventory before it mutates a scale project' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-incomplete-inventory-' + [guid]::NewGuid().ToString('N')
        )
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceFingerprint ('A' * 64) | Out-Null
            $firstSourcePath = Join-Path $projectRoot 'assets\data\catalog_000001.json'
            $before = [IO.File]::ReadAllText($firstSourcePath)
            [IO.File]::Delete((Join-Path $projectRoot 'assets\data\catalog_000004.json'))

            { Set-ResourceManagementScaleProjectChangeSet `
                    -ProjectRoot $projectRoot `
                    -ChangePercent 25 `
                    -ExpectedSourceFingerprint ('A' * 64) } |
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
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-undeclared-mutation-' + [guid]::NewGuid().ToString('N')
        )
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 4 `
                -SourceFingerprint ('A' * 64) | Out-Null
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
                    -ExpectedSourceFingerprint ('A' * 64) } |
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
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' (
            'resource-management-scale-change-lease-' + [guid]::NewGuid().ToString('N')
        )
        $lease = $null
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $projectRoot `
                -DataAssetCount 1 `
                -SourceFingerprint ('A' * 64) | Out-Null
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
                    -ExpectedSourceFingerprint ('A' * 64) } |
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
