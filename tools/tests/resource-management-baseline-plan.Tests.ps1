$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$generator = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementScaleProject.ps1'
$changeSet = Join-Path $repoRoot 'tools\mvp\Set-ResourceManagementScaleProjectChangeSet.ps1'
$baselinePlan = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementBaselinePlan.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$artifactStorageModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'
$originalProjectTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE
$originalChangeSetTestMode = $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE
$originalBaselinePlanTestMode = $env:RESOURCE_MANAGEMENT_BASELINE_PLAN_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop
Import-Module $artifactStorageModule -Force -Global -ErrorAction Stop

try {
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE = '1'
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE = '1'
    $env:RESOURCE_MANAGEMENT_BASELINE_PLAN_TEST_MODE = '1'
    . $generator
    . $changeSet
    . $baselinePlan
}
finally {
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE = $originalProjectTestMode
    $env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE = $originalChangeSetTestMode
    $env:RESOURCE_MANAGEMENT_BASELINE_PLAN_TEST_MODE = $originalBaselinePlanTestMode
}

function New-TestResourceManagementBaselineSourceIdentity {
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
        manifest_sha256 = ('C' * 64)
        source_fingerprint = ('A' * 64)
        build_set_id = ('A' * 64)
        build_set = [pscustomobject]@{
            build_set_id = ('A' * 64)
            snapshot_root = $repoRoot
            files = $files
        }
    }
}

$resourceBaselineSourceIdentity = New-TestResourceManagementBaselineSourceIdentity

Describe 'Resource-management baseline plan' {
    BeforeEach {
        Import-Module $manifestModule -Force -Global -ErrorAction Stop
        Import-Module $resolverModule -Force -Global -ErrorAction Stop
    }

    It 'requires the complete 1, 1k, and 100k registry scale set' {
        { Assert-ResourceManagementBaselineScaleSet -DataAssetCounts @(1, 1000, 100000) } |
            Should Not Throw
        { Assert-ResourceManagementBaselineScaleSet -DataAssetCounts @(1, 1000) } |
            Should Throw '1, 1000, and 100000'
        { Assert-ResourceManagementBaselineScaleSet -DataAssetCounts @(1, 1000, 1000) } |
            Should Throw 'duplicate'
    }

    It 'rejects baseline JSON input above its caller-owned byte budget' {
        $path = Join-Path $TestDrive 'oversized-baseline-input.json'
        [IO.File]::WriteAllText(
            $path,
            ('{"payload":"' + ('x' * 64) + '"}'),
            [Text.UTF8Encoding]::new($false))

        {
            Read-ResourceManagementBaselineJson `
                -Path $path `
                -Label 'Oversized baseline input' `
                -MaximumBytes 32
        } | Should Throw 'byte budget of 32 bytes'
        (Get-Content -Raw $baselinePlan) | Should Not Match '\[IO\.File\]::ReadAllText'
    }

    It 'describes cold, stable, and one-percent-change data workloads without physical resource paths' {
        $baselineRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'resource-management-projects' `
            -InstanceId ('resource-management-baseline-plan-baseline-' + [guid]::NewGuid().ToString('N'))
        $changedRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'resource-management-projects' `
            -InstanceId ('resource-management-baseline-plan-changed-' + [guid]::NewGuid().ToString('N'))
        try {
            New-ResourceManagementScaleProject `
                -ProjectRoot $baselineRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceBaselineSourceIdentity | Out-Null
            New-ResourceManagementScaleProject `
                -ProjectRoot $changedRoot `
                -DataAssetCount 4 `
                -SourceIdentity $resourceBaselineSourceIdentity | Out-Null
            Set-ResourceManagementScaleProjectChangeSet `
                -ProjectRoot $changedRoot `
                -ChangePercent 1 `
                -ExpectedSourceFingerprint ('A' * 64) `
                -ExpectedProductInputManifestSha256 ('C' * 64) | Out-Null

            $baseline = Read-ResourceManagementBaselineProject `
                -ProjectRoot $baselineRoot `
                -Role 'baseline'
            $changed = Read-ResourceManagementBaselineProject `
                -ProjectRoot $changedRoot `
                -Role 'changed'
            $baselineMetadataPath = Join-Path $baselineRoot 'resource-management-scale-project.json'
            $hasher = [Security.Cryptography.SHA256]::Create()
            try {
                $expectedBaselineManifestSha256 = ([BitConverter]::ToString(
                        $hasher.ComputeHash([IO.File]::ReadAllBytes($baselineMetadataPath))
                    )).Replace('-', '')
            }
            finally {
                $hasher.Dispose()
            }
            $document = New-ResourceManagementBaselinePlanDocument `
                -BaselineProjects @($baseline) `
                -ChangedProjects @($changed) `
                -RepeatCount 20 `
                -WarmupCount 3
            $serialized = $document | ConvertTo-Json -Depth 16
            $stable = @($document.scenarios | Where-Object { $_.mode -eq 'stable-generation' })[0]
            $changedScenario = @($document.scenarios | Where-Object { $_.mode -eq 'one-percent-change' })[0]

            $document.schema_version | Should Be 3
            $document.workload_family | Should Be 'resource-management-query'
            $document.workload_profile_id | Should Be 'json-data-flat-v1'
            $document.workload_registry_receipt.registry_kind | Should Be 'zircon.resource-management-workload-registry'
            $document.workload_registry_receipt.sha256 | Should Match '^[0-9A-F]{64}$'
            $document.resource_kind | Should Be 'Data'
            $document.statistical_policy.warmup_repetitions | Should Be 3
            $document.statistical_policy.measurement_repetitions | Should Be 20
            $document.statistical_policy.minimum_sample_count | Should Be 20
            $document.statistical_policy.confidence_level | Should Be 0.95
            $document.statistical_policy.maximum_coefficient_of_variation | Should Be 0.10
            $document.statistical_policy.maximum_relative_margin_of_error | Should Be 0.10
            $document.scenarios.Count | Should Be 3
            $stable.required_repetitions | Should Be 23
            $stable.process_lifecycle | Should Be 'same-process'
            $stable.required_generation_relation | Should Be 'same-published-generation'
            $changedScenario.project_role | Should Be 'changed'
            $changedScenario.changed_asset_count | Should Be 1
            $changedScenario.change_percent | Should Be 1
            (@($changedScenario.changed_virtual_paths) -join ',') | Should Be 'res://data/catalog_000001.json'
            $baseline.data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            $changed.data_inventory_sha256 | Should Match '^[0-9A-F]{64}$'
            $baseline.project_manifest_sha256 | Should Be $expectedBaselineManifestSha256
            (Get-Content -Raw $baselinePlan) | Should Match 'Get-ResourceManagementFileSha256 -Path \$Path'
            (Get-Content -Raw $baselinePlan) | Should Not Match 'Get-FileHash'
            $changed.change_set.baseline_data_inventory_sha256 | Should Be $baseline.data_inventory_sha256
            $changed.change_set.changed_data_inventory_sha256 | Should Be $changed.data_inventory_sha256
            $changedScenario.data_inventory_sha256 | Should Be $changed.data_inventory_sha256
            @($changedScenario.queries | Where-Object { $_.operation -eq 'page' }).Count | Should Be 2
            $serialized | Should Match 'res://data/'
            $serialized | Should Not Match '[A-Za-z]:\\'
            $serialized | Should Not Match 'primitive_count'
            $serialized | Should Not Match 'render-extract'
        }
        finally {
            if ([IO.Directory]::Exists($baselineRoot)) {
                [IO.Directory]::Delete($baselineRoot, $true)
            }
            if ([IO.Directory]::Exists($changedRoot)) {
                [IO.Directory]::Delete($changedRoot, $true)
            }
        }
    }

    It 'rejects a changed workload whose declared mutation is not exactly one percent' {
        $project = [pscustomobject]@{
            project_id = 'data-000004-baseline'
            project_role = 'baseline'
            source_fingerprint = 'A' * 64
            project_manifest_sha256 = 'B' * 64
            data_inventory_sha256 = 'E' * 64
            data_asset_count = 4
            data_virtual_prefix = 'res://data/'
            data_source_pattern = 'res://data/catalog_*.json'
            change_set = $null
        }
        $changed = [pscustomobject]@{
            project_id = 'data-000004-changed'
            project_role = 'changed'
            source_fingerprint = 'A' * 64
            project_manifest_sha256 = 'C' * 64
            data_inventory_sha256 = 'F' * 64
            data_asset_count = 4
            data_virtual_prefix = 'res://data/'
            data_source_pattern = 'res://data/catalog_*.json'
            change_set = [pscustomobject]@{
                manifest_sha256 = 'D' * 64
                baseline_data_inventory_sha256 = 'E' * 64
                changed_data_inventory_sha256 = 'F' * 64
                change_percent = 2
                changed_asset_count = 1
                changed_virtual_paths = @('res://data/catalog_000001.json')
            }
        }

        {
            New-ResourceManagementBaselineScenarioMatrix `
                -BaselineProject $project `
                -ChangedProject $changed `
                -RepeatCount 20 `
                -WarmupCount 3
        } | Should Throw 'exactly one percent'
    }
}
