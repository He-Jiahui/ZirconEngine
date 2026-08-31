[CmdletBinding()]
param(
    [string[]]$BaselineProjectRoot,
    [string[]]$ChangedProjectRoot,
    [string]$OutputDirectory,
    [ValidateRange(20, 50)][int]$RepeatCount = 20,
    [ValidateRange(1, 10)][int]$WarmupCount = 3
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementScaleInventory.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementWorkloadRegistry.psm1') -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = New-MvpArtifactStoragePath -NamespaceId 'resource-management-baselines'
}

$script:ResourceManagementBaselineMaximumMetadataBytes = 4MB
$script:ResourceManagementBaselineMaximumSourceBytes = 64KB
$script:ResourceManagementBaselineJsonReadBufferBytes = 81920

function Get-ResourceManagementBaselineFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    return Get-ResourceManagementFileSha256 -Path $Path
}

function Assert-ResourceManagementBaselineProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath -Path $Path -NamespaceId 'resource-management-projects'
    if (-not [IO.Directory]::Exists($storage.operation_path)) {
        throw "Resource-management baseline project does not exist: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Assert-ResourceManagementBaselineOutputDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath -Path $Path -NamespaceId 'resource-management-baselines'
    if ([IO.Directory]::Exists($storage.operation_path) -or [IO.File]::Exists($storage.operation_path)) {
        throw "Resource-management baseline plan output must not already exist: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Read-ResourceManagementBaselineJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumBytes = $script:ResourceManagementBaselineMaximumMetadataBytes
    )

    try {
        $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        try {
            if ($stream.Length -eq 0) {
                throw "$Label is empty: $Path"
            }
            if ($stream.Length -gt $MaximumBytes) {
                throw "$Label exceeds its byte budget of $MaximumBytes bytes: $Path"
            }
            [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
            $offset = 0
            while ($offset -lt $bytes.Length) {
                $read = $stream.Read(
                    $bytes,
                    $offset,
                    [Math]::Min($script:ResourceManagementBaselineJsonReadBufferBytes, $bytes.Length - $offset))
                if ($read -eq 0) {
                    throw "$Label changed while it was being read: $Path"
                }
                $offset += $read
            }
            if ($stream.ReadByte() -ne -1) {
                throw "$Label exceeds its byte budget of $MaximumBytes bytes: $Path"
            }
        }
        finally {
            $stream.Dispose()
        }
        $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
        return $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "$Label is not valid JSON: ${Path}: $($_.Exception.Message)"
    }
}

function Assert-ResourceManagementBaselineDataInventory {
    param(
        [Parameter(Mandatory)]$ProjectResolution,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount
    )

    $dataRoot = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
            -Path $ProjectResolution.OperationalPath `
            -ChildPath 'assets\data')
    if (-not [IO.Directory]::Exists($dataRoot.OperationalPath)) {
        throw "Resource-management baseline data source inventory is missing: $($dataRoot.DisplayPath)"
    }
    $sourceFiles = @([IO.Directory]::EnumerateFiles(
            $dataRoot.OperationalPath,
            '*.json',
            [IO.SearchOption]::TopDirectoryOnly
        ))
    if ($sourceFiles.Count -ne $DataAssetCount) {
        throw "Resource-management baseline data source inventory count $($sourceFiles.Count) does not match data_asset_count $DataAssetCount."
    }
    for ($index = 1; $index -le $DataAssetCount; $index++) {
        $sourcePath = Join-ZirconWindowsPath `
            -Path $dataRoot.OperationalPath `
            -ChildPath ('catalog_{0:D6}.json' -f $index)
        if (-not [IO.File]::Exists($sourcePath)) {
            throw ("Resource-management baseline data source inventory is missing catalog_{0:D6}.json." -f $index)
        }
    }
    return $dataRoot
}

function Read-ResourceManagementBaselineChangeSet {
    param(
        [Parameter(Mandatory)]$ProjectResolution,
        [Parameter(Mandatory)]$ProjectMetadata,
        [Parameter(Mandatory)]$DataRoot,
        [Parameter(Mandatory)][string]$ActualDataInventorySha256
    )

    $manifestPath = Join-ZirconWindowsPath `
        -Path $ProjectResolution.OperationalPath `
        -ChildPath 'resource-management-scale-change-set.json'
    if (-not [IO.File]::Exists($manifestPath)) {
        throw "Resource-management changed baseline project is missing its change-set manifest: $($ProjectResolution.DisplayPath)"
    }
    $changeSet = Read-ResourceManagementBaselineJson `
        -Path $manifestPath `
        -Label 'Resource-management scale change-set metadata'
    if ($null -eq $changeSet -or [int]$changeSet.schema_version -ne 2) {
        throw "Resource-management scale change-set metadata has an unsupported schema_version: $manifestPath"
    }
    $sourceFingerprint = [string]$ProjectMetadata.source_fingerprint
    if (-not ([string]$changeSet.source_fingerprint).Equals($sourceFingerprint, [StringComparison]::Ordinal) -or
        -not ([string]$changeSet.build_set_id).Equals([string]$ProjectMetadata.build_set_id, [StringComparison]::Ordinal) -or
        -not ([string]$changeSet.product_input_manifest_sha256).Equals(
            [string]$ProjectMetadata.product_input_manifest_sha256,
            [StringComparison]::Ordinal)) {
        throw 'Resource-management scale change set belongs to a different source snapshot.'
    }
    $dataAssetCount = [int]$ProjectMetadata.data_asset_count
    if ([int]$changeSet.data_asset_count -ne $dataAssetCount -or
        [string]$changeSet.asset_kind -ne 'Data' -or
        [string]$changeSet.importer_id -ne 'zircon.builtin.data.json' -or
        [string]$changeSet.data_virtual_prefix -ne 'res://data/') {
        throw "Resource-management scale change set does not match its project metadata: $manifestPath"
    }
    $baselineDataInventorySha256 = [string]$changeSet.baseline_data_inventory_sha256
    $changedDataInventorySha256 = [string]$changeSet.changed_data_inventory_sha256
    if ($baselineDataInventorySha256 -notmatch '^[0-9A-F]{64}$' -or
        $changedDataInventorySha256 -notmatch '^[0-9A-F]{64}$' -or
        -not $baselineDataInventorySha256.Equals([string]$ProjectMetadata.data_inventory_sha256, [StringComparison]::Ordinal) -or
        -not $changedDataInventorySha256.Equals($ActualDataInventorySha256, [StringComparison]::Ordinal)) {
        throw "Resource-management scale change set does not bind the current data inventory: $manifestPath"
    }
    if ([int]$changeSet.change_percent -ne 1) {
        throw "Resource-management baseline change set must be exactly one percent: $manifestPath"
    }
    $expectedChangeCount = [Math]::Max(1, [int][Math]::Ceiling($dataAssetCount / 100.0))
    if ([int]$changeSet.changed_asset_count -ne $expectedChangeCount) {
        throw "Resource-management baseline change-set count does not match one percent of $dataAssetCount resources: $manifestPath"
    }
    $changedVirtualPaths = @($changeSet.changed_virtual_paths | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($changedVirtualPaths.Count -ne $expectedChangeCount) {
        throw "Resource-management baseline change set has an invalid changed_virtual_paths count: $manifestPath"
    }
    for ($index = 1; $index -le $expectedChangeCount; $index++) {
        $expectedVirtualPath = 'res://data/catalog_{0:D6}.json' -f $index
        if ([string]$changedVirtualPaths[$index - 1] -ne $expectedVirtualPath) {
            throw "Resource-management baseline change set does not have the deterministic changed virtual path '$expectedVirtualPath': $manifestPath"
        }
        $sourcePath = Join-ZirconWindowsPath `
            -Path $DataRoot.OperationalPath `
            -ChildPath ('catalog_{0:D6}.json' -f $index)
        $source = Read-ResourceManagementBaselineJson `
            -Path $sourcePath `
            -Label 'Resource-management changed source' `
            -MaximumBytes $script:ResourceManagementBaselineMaximumSourceBytes
        if ([int]$source.index -ne $index -or
            [string]$source.payload -ne 'resource-management-scale' -or
            [int]$source.workload_revision -ne 1) {
            throw "Resource-management baseline changed source does not match its declared one-percent workload: $sourcePath"
        }
    }
    return [pscustomobject]@{
        manifest_sha256 = Get-ResourceManagementBaselineFileSha256 -Path $manifestPath
        baseline_data_inventory_sha256 = $baselineDataInventorySha256
        changed_data_inventory_sha256 = $changedDataInventorySha256
        change_percent = [int]$changeSet.change_percent
        changed_asset_count = [int]$changeSet.changed_asset_count
        changed_virtual_paths = @($changedVirtualPaths)
    }
}

function Read-ResourceManagementBaselineProject {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][ValidateSet('baseline', 'changed')][string]$Role
    )

    $projectResolution = Assert-ResourceManagementBaselineProjectDirectory -Path $ProjectRoot
    $metadataPath = Join-ZirconWindowsPath `
        -Path $projectResolution.OperationalPath `
        -ChildPath 'resource-management-scale-project.json'
    if (-not [IO.File]::Exists($metadataPath)) {
        throw "Resource-management baseline project metadata does not exist: $($projectResolution.DisplayPath)"
    }
    $metadata = Read-ResourceManagementBaselineJson `
        -Path $metadataPath `
        -Label 'Resource-management scale project metadata'
    if ($null -eq $metadata -or [int]$metadata.schema_version -ne 2) {
        throw "Resource-management scale project metadata has an unsupported schema_version: $metadataPath"
    }
    $sourceFingerprint = [string]$metadata.source_fingerprint
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$' -or
        -not $sourceFingerprint.Equals([string]$metadata.build_set_id, [StringComparison]::Ordinal) -or
        [string]$metadata.product_input_manifest_sha256 -notmatch '^[0-9A-F]{64}$') {
        throw "Resource-management scale project metadata has an invalid ProductInput BuildSet identity: $metadataPath"
    }
    $dataAssetCount = [int]$metadata.data_asset_count
    if ($dataAssetCount -lt 1 -or $dataAssetCount -gt 100000 -or
        [string]$metadata.data_inventory_sha256 -notmatch '^[0-9A-F]{64}$' -or
        [string]$metadata.asset_kind -ne 'Data' -or
        [string]$metadata.importer_id -ne 'zircon.builtin.data.json' -or
        [string]$metadata.data_virtual_prefix -ne 'res://data/' -or
        [string]$metadata.data_source_pattern -ne 'res://data/catalog_*.json') {
        throw "Resource-management scale project metadata does not describe the supported JSON data workload: $metadataPath"
    }
    $dataRoot = Assert-ResourceManagementBaselineDataInventory `
        -ProjectResolution $projectResolution `
        -DataAssetCount $dataAssetCount
    $actualDataInventorySha256 = Get-ResourceManagementScaleInventorySha256 `
        -DataRoot $dataRoot.OperationalPath `
        -DataAssetCount $dataAssetCount
    $changeManifestPath = Join-ZirconWindowsPath `
        -Path $projectResolution.OperationalPath `
        -ChildPath 'resource-management-scale-change-set.json'
    $changeSet = $null
    if ($Role -eq 'baseline') {
        if ([IO.File]::Exists($changeManifestPath)) {
            throw "Resource-management baseline project must not already have a change set: $($projectResolution.DisplayPath)"
        }
        if (-not $actualDataInventorySha256.Equals([string]$metadata.data_inventory_sha256, [StringComparison]::Ordinal)) {
            throw 'Resource-management baseline project data inventory does not match its immutable metadata fingerprint.'
        }
    }
    else {
        $changeSet = Read-ResourceManagementBaselineChangeSet `
            -ProjectResolution $projectResolution `
            -ProjectMetadata $metadata `
            -DataRoot $dataRoot `
            -ActualDataInventorySha256 $actualDataInventorySha256
    }

    return [pscustomobject]@{
        project_id = ('data-{0:D6}-{1}' -f $dataAssetCount, $Role)
        project_role = $Role
        source_fingerprint = $sourceFingerprint
        build_set_id = [string]$metadata.build_set_id
        product_input_manifest_sha256 = [string]$metadata.product_input_manifest_sha256
        project_manifest_sha256 = Get-ResourceManagementBaselineFileSha256 -Path $metadataPath
        data_inventory_sha256 = $actualDataInventorySha256
        data_asset_count = $dataAssetCount
        data_virtual_prefix = [string]$metadata.data_virtual_prefix
        data_source_pattern = [string]$metadata.data_source_pattern
        change_set = $changeSet
    }
}

function Assert-ResourceManagementBaselineScaleSet {
    param([Parameter(Mandatory)][int[]]$DataAssetCounts)

    $seen = [Collections.Generic.HashSet[int]]::new()
    foreach ($count in $DataAssetCounts) {
        if (-not $seen.Add($count)) {
            throw "Resource-management baseline scale set contains duplicate scale '$count'."
        }
    }
    $expected = @(1, 1000, 100000)
    if ($seen.Count -ne $expected.Count -or @($expected | Where-Object { -not $seen.Contains($_) }).Count -ne 0) {
        throw 'Resource-management baseline scale set must contain exactly 1, 1000, and 100000 resources.'
    }
}

function New-ResourceManagementBaselinePageQueries {
    param([Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount)

    $queries = [Collections.Generic.List[object]]::new()
    $seen = [Collections.Generic.HashSet[string]]::new()
    $normalLimits = @(50, 1000 | Where-Object { $_ -le $DataAssetCount })
    if ($normalLimits.Count -eq 0) {
        $normalLimits = @($DataAssetCount)
    }
    foreach ($limit in $normalLimits) {
        $key = "0:$limit"
        if ($seen.Add($key)) {
            $queries.Add([ordered]@{
                    operation = 'page'
                    query = [ordered]@{ kind = 'Data'; state = 'any' }
                    offset = 0
                    limit = $limit
                    expected_measurements = @(
                        'resource_management.page.instances',
                        'resource_management.page.matching_rows',
                        'resource_management.page.candidate_rows',
                        'resource_management.page.rows_returned',
                        'resource_management.page.shard_candidate_checks',
                        'resource_management.page.filtered_rows_skipped'
                    )
                }) | Out-Null
        }
    }
    $highOffset = $DataAssetCount - 1
    $highLimit = [Math]::Min(50, $DataAssetCount)
    $highKey = "${highOffset}:${highLimit}"
    if ($seen.Add($highKey)) {
        $queries.Add([ordered]@{
                operation = 'page'
                query = [ordered]@{ kind = 'Data'; state = 'any' }
                offset = $highOffset
                limit = $highLimit
                expected_measurements = @(
                    'resource_management.page.instances',
                    'resource_management.page.matching_rows',
                    'resource_management.page.candidate_rows',
                    'resource_management.page.rows_returned',
                    'resource_management.page.shard_candidate_checks',
                    'resource_management.page.filtered_rows_skipped'
                )
            }) | Out-Null
    }
    return $queries.ToArray()
}

function New-ResourceManagementBaselineScenarioMatrix {
    param(
        [Parameter(Mandatory)]$BaselineProject,
        [Parameter(Mandatory)]$ChangedProject,
        [Parameter(Mandatory)][ValidateRange(20, 50)][int]$RepeatCount,
        [Parameter(Mandatory)][ValidateRange(1, 10)][int]$WarmupCount
    )

    if ([string]$BaselineProject.project_role -ne 'baseline' -or
        [string]$ChangedProject.project_role -ne 'changed') {
        throw 'Resource-management baseline scenario matrix requires one baseline project and one changed project.'
    }
    if ([int]$BaselineProject.data_asset_count -ne [int]$ChangedProject.data_asset_count -or
        -not ([string]$BaselineProject.source_fingerprint).Equals([string]$ChangedProject.source_fingerprint, [StringComparison]::Ordinal) -or
        [string]$BaselineProject.data_virtual_prefix -ne 'res://data/' -or
        [string]$ChangedProject.data_virtual_prefix -ne 'res://data/' -or
        [string]$BaselineProject.data_source_pattern -ne 'res://data/catalog_*.json' -or
        [string]$ChangedProject.data_source_pattern -ne 'res://data/catalog_*.json') {
        throw 'Resource-management baseline project pair does not share one immutable Data workload identity.'
    }
    $changeSet = $ChangedProject.change_set
    if ($null -eq $changeSet -or [int]$changeSet.change_percent -ne 1) {
        throw 'Resource-management baseline changed workload must declare exactly one percent change.'
    }
    $dataAssetCount = [int]$BaselineProject.data_asset_count
    $expectedChangeCount = [Math]::Max(1, [int][Math]::Ceiling($dataAssetCount / 100.0))
    if ([int]$changeSet.changed_asset_count -ne $expectedChangeCount -or
        @($changeSet.changed_virtual_paths).Count -ne $expectedChangeCount) {
        throw 'Resource-management baseline changed workload does not have the required deterministic one-percent change set.'
    }
    if ([string]$BaselineProject.data_inventory_sha256 -notmatch '^[0-9A-F]{64}$' -or
        [string]$ChangedProject.data_inventory_sha256 -notmatch '^[0-9A-F]{64}$' -or
        -not ([string]$changeSet.baseline_data_inventory_sha256).Equals([string]$BaselineProject.data_inventory_sha256, [StringComparison]::Ordinal) -or
        -not ([string]$changeSet.changed_data_inventory_sha256).Equals([string]$ChangedProject.data_inventory_sha256, [StringComparison]::Ordinal)) {
        throw 'Resource-management baseline project pair does not bind its exact data inventories.'
    }
    for ($index = 1; $index -le $expectedChangeCount; $index++) {
        if ([string]$changeSet.changed_virtual_paths[$index - 1] -ne ('res://data/catalog_{0:D6}.json' -f $index)) {
            throw 'Resource-management baseline changed workload has a non-deterministic virtual change path.'
        }
    }

    $queries = [Collections.Generic.List[object]]::new()
    $queries.Add([ordered]@{
            operation = 'scan'
            query = [ordered]@{ kind = 'Data'; state = 'any' }
            expected_measurements = @(
                'resource_management.scan.instances',
                'resource_management.scan.matching_rows',
                'resource_management.scan.rows_emitted',
                'resource_management.scan.shard_candidate_checks',
                'resource_management.scan.filtered_rows_skipped'
            )
        }) | Out-Null
    foreach ($pageQuery in @(New-ResourceManagementBaselinePageQueries -DataAssetCount $dataAssetCount)) {
        $queries.Add($pageQuery) | Out-Null
    }
    $queries.Add([ordered]@{
            operation = 'asset-workspace-snapshot'
            query = [ordered]@{ kind = 'Data'; state = 'any' }
            expected_measurements = @(
                'asset_workspace.snapshot.instances',
                'asset_workspace.catalog_asset_count',
                'asset_workspace.visible_asset_count',
                'asset_workspace.row_by_locator.calls',
                'asset_workspace.row_by_locator.shard_probes',
                'asset_workspace.selection_lookup.calls',
                'asset_workspace.surface_clone.instances'
            )
        }) | Out-Null

    $shared = [ordered]@{
        data_asset_count = $dataAssetCount
        resource_kind = 'Data'
        data_virtual_prefix = 'res://data/'
        data_source_pattern = 'res://data/catalog_*.json'
        required_repetitions = $WarmupCount + $RepeatCount
        queries = $queries.ToArray()
    }
    return @(
        [ordered]@{
            logical_id = ('data-{0:D6}-cold-open' -f $dataAssetCount)
            mode = 'cold-open'
            project_role = 'baseline'
            project_id = [string]$BaselineProject.project_id
            project_manifest_sha256 = [string]$BaselineProject.project_manifest_sha256
            data_inventory_sha256 = [string]$BaselineProject.data_inventory_sha256
            process_lifecycle = 'fresh-process'
            required_generation_relation = 'first-published-generation'
            change_mode = 'none'
            data_asset_count = $shared.data_asset_count
            resource_kind = $shared.resource_kind
            data_virtual_prefix = $shared.data_virtual_prefix
            data_source_pattern = $shared.data_source_pattern
            required_repetitions = $shared.required_repetitions
            queries = $shared.queries
        },
        [ordered]@{
            logical_id = ('data-{0:D6}-stable-generation' -f $dataAssetCount)
            mode = 'stable-generation'
            project_role = 'baseline'
            project_id = [string]$BaselineProject.project_id
            project_manifest_sha256 = [string]$BaselineProject.project_manifest_sha256
            data_inventory_sha256 = [string]$BaselineProject.data_inventory_sha256
            process_lifecycle = 'same-process'
            required_generation_relation = 'same-published-generation'
            change_mode = 'none'
            data_asset_count = $shared.data_asset_count
            resource_kind = $shared.resource_kind
            data_virtual_prefix = $shared.data_virtual_prefix
            data_source_pattern = $shared.data_source_pattern
            required_repetitions = $shared.required_repetitions
            queries = $shared.queries
        },
        [ordered]@{
            logical_id = ('data-{0:D6}-one-percent-change' -f $dataAssetCount)
            mode = 'one-percent-change'
            project_role = 'changed'
            project_id = [string]$ChangedProject.project_id
            project_manifest_sha256 = [string]$ChangedProject.project_manifest_sha256
            data_inventory_sha256 = [string]$ChangedProject.data_inventory_sha256
            change_set_manifest_sha256 = [string]$changeSet.manifest_sha256
            process_lifecycle = 'fresh-process'
            required_generation_relation = 'changed-published-generation'
            change_mode = 'one-percent'
            change_percent = [int]$changeSet.change_percent
            changed_asset_count = [int]$changeSet.changed_asset_count
            changed_virtual_paths = @($changeSet.changed_virtual_paths)
            data_asset_count = $shared.data_asset_count
            resource_kind = $shared.resource_kind
            data_virtual_prefix = $shared.data_virtual_prefix
            data_source_pattern = $shared.data_source_pattern
            required_repetitions = $shared.required_repetitions
            queries = $shared.queries
        }
    )
}

function New-ResourceManagementBaselinePlanDocument {
    param(
        [Parameter(Mandatory)][object[]]$BaselineProjects,
        [Parameter(Mandatory)][object[]]$ChangedProjects,
        [Parameter(Mandatory)][ValidateRange(20, 50)][int]$RepeatCount,
        [Parameter(Mandatory)][ValidateRange(1, 10)][int]$WarmupCount
    )

    if ($BaselineProjects.Count -ne $ChangedProjects.Count) {
        throw 'Resource-management baseline plan requires one changed project for each baseline project.'
    }
    $changedByCount = @{}
    foreach ($changed in $ChangedProjects) {
        $count = [int]$changed.data_asset_count
        if ($changedByCount.ContainsKey($count)) {
            throw "Resource-management baseline plan has duplicate changed project scale '$count'."
        }
        $changedByCount[$count] = $changed
    }
    $workloadSnapshot = Get-ResourceManagementWorkloadRegistrySnapshot
    $workloadProfile = Get-ResourceManagementWorkloadProfile -ProfileId 'json-data-flat-v1'
    $scenarios = [Collections.Generic.List[object]]::new()
    foreach ($baseline in @($BaselineProjects | Sort-Object { [int]$_.data_asset_count })) {
        $count = [int]$baseline.data_asset_count
        if (-not $changedByCount.ContainsKey($count)) {
            throw "Resource-management baseline plan is missing changed project scale '$count'."
        }
        foreach ($scenario in @(New-ResourceManagementBaselineScenarioMatrix `
                -BaselineProject $baseline `
                -ChangedProject $changedByCount[$count] `
                -RepeatCount $RepeatCount `
                -WarmupCount $WarmupCount)) {
            $scenarios.Add($scenario) | Out-Null
        }
    }
    $sourceFingerprints = @($BaselineProjects | ForEach-Object { [string]$_.source_fingerprint } | Select-Object -Unique)
    if ($sourceFingerprints.Count -ne 1) {
        throw 'Resource-management baseline plan projects do not share one source fingerprint.'
    }
    return [ordered]@{
        schema_version = 3
        workload_family = 'resource-management-query'
        workload_profile_id = $workloadProfile.profile_id
        workload_registry_receipt = $workloadSnapshot.receipt
        source_fingerprint = $sourceFingerprints[0]
        resource_kind = $workloadProfile.asset_kinds[0]
        statistical_policy = [ordered]@{
            warmup_repetitions = $WarmupCount
            measurement_repetitions = $RepeatCount
            minimum_sample_count = 20
            confidence_level = 0.95
            maximum_coefficient_of_variation = 0.10
            maximum_relative_margin_of_error = 0.10
        }
        scenarios = $scenarios.ToArray()
    }
}

function Write-ResourceManagementBaselinePlan {
    param(
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)]$Document
    )

    $outputResolution = Assert-ResourceManagementBaselineOutputDirectory -Path $OutputDirectory
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($outputResolution.OperationalPath)) | Out-Null
    $stagingPath = "$($outputResolution.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($stagingPath) | Out-Null
        $manifestPath = Join-ZirconWindowsPath -Path $stagingPath -ChildPath 'resource-management-baseline-plan.json'
        $stream = [IO.FileStream]::new(
            $manifestPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
        try {
            $bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Document | ConvertTo-Json -Depth 16))
            $stream.Write($bytes, 0, $bytes.Length)
            $stream.Flush($true)
        }
        finally {
            $stream.Dispose()
        }
        [IO.Directory]::Move($stagingPath, $outputResolution.OperationalPath)
    }
    catch {
        if ([IO.Directory]::Exists($stagingPath)) {
            [IO.Directory]::Delete($stagingPath, $true)
        }
        throw
    }
    return (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
            -Path $outputResolution.OperationalPath `
            -ChildPath 'resource-management-baseline-plan.json')).DisplayPath
}

function New-ResourceManagementBaselinePlan {
    param(
        [Parameter(Mandatory)][string[]]$BaselineProjectRoot,
        [Parameter(Mandatory)][string[]]$ChangedProjectRoot,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)][ValidateRange(20, 50)][int]$RepeatCount,
        [Parameter(Mandatory)][ValidateRange(1, 10)][int]$WarmupCount
    )

    $baselineProjects = @($BaselineProjectRoot | ForEach-Object {
            Read-ResourceManagementBaselineProject -ProjectRoot $_ -Role 'baseline'
        })
    $changedProjects = @($ChangedProjectRoot | ForEach-Object {
            Read-ResourceManagementBaselineProject -ProjectRoot $_ -Role 'changed'
        })
    Assert-ResourceManagementBaselineScaleSet -DataAssetCounts @($baselineProjects | ForEach-Object { [int]$_.data_asset_count })
    Assert-ResourceManagementBaselineScaleSet -DataAssetCounts @($changedProjects | ForEach-Object { [int]$_.data_asset_count })
    $document = New-ResourceManagementBaselinePlanDocument `
        -BaselineProjects $baselineProjects `
        -ChangedProjects $changedProjects `
        -RepeatCount $RepeatCount `
        -WarmupCount $WarmupCount
    $manifestPath = Write-ResourceManagementBaselinePlan `
        -OutputDirectory $OutputDirectory `
        -Document $document
    return [pscustomobject]@{
        manifest_path = $manifestPath
        source_fingerprint = [string]$document.source_fingerprint
        scenario_count = $document.scenarios.Count
    }
}

if ($env:RESOURCE_MANAGEMENT_BASELINE_PLAN_TEST_MODE -ne '1') {
    if ($null -eq $BaselineProjectRoot -or $BaselineProjectRoot.Count -eq 0) {
        throw '-BaselineProjectRoot is required for resource-management baseline plan generation.'
    }
    if ($null -eq $ChangedProjectRoot -or $ChangedProjectRoot.Count -eq 0) {
        throw '-ChangedProjectRoot is required for resource-management baseline plan generation.'
    }
    New-ResourceManagementBaselinePlan `
        -BaselineProjectRoot $BaselineProjectRoot `
        -ChangedProjectRoot $ChangedProjectRoot `
        -OutputDirectory $OutputDirectory `
        -RepeatCount $RepeatCount `
        -WarmupCount $WarmupCount
}
