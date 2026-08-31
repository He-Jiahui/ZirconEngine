Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementJsonEvidence.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop

$script:ResourceManagementWorkloadRegistrySnapshot = $null

function Get-ResourceManagementWorkloadRegistrySnapshot {
    if ($null -ne $script:ResourceManagementWorkloadRegistrySnapshot) {
        return $script:ResourceManagementWorkloadRegistrySnapshot
    }

    $evidence = Get-ResourceManagementJsonEvidence `
        -Path (Join-Path $PSScriptRoot 'resource-management-workload-registry.json') `
        -Label 'Resource-management workload registry' `
        -MaximumBytes 65536
    $registry = $evidence.json
    Assert-ResourceManagementSchemaProperties `
        -Value $registry `
        -RequiredNames @('schema_version', 'registry_kind', 'profiles') `
        -Label 'Resource-management workload registry'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $registry `
        -SchemaId 'zircon.resource-management.workload-registry' `
        -Label 'Resource-management workload registry' | Out-Null
    $profiles = @($registry.profiles)
    if ($profiles.Count -eq 0 -or $profiles.Count -gt 128) {
        throw 'Resource-management workload registry profile count is outside 1..128.'
    }
    $index = @{}
    foreach ($profile in $profiles) {
        Assert-ResourceManagementSchemaProperties `
            -Value $profile `
            -RequiredNames @(
                'schema_version', 'profile_kind', 'profile_id', 'asset_kinds',
                'dependency_graph_shape', 'tag_cardinality', 'query_mix',
                'minimum_asset_count', 'maximum_asset_count', 'change_percent') `
            -Label 'Resource-management workload profile'
        Assert-ResourceManagementRegisteredSchemaIdentity `
            -Value $profile `
            -SchemaId 'zircon.resource-management.workload-profile' `
            -Label 'Resource-management workload profile' | Out-Null
        $profileId = [string]$profile.profile_id
        if ($profileId -notmatch '^[a-z0-9][a-z0-9-]{0,63}$' -or $index.ContainsKey($profileId)) {
            throw "Resource-management workload registry has invalid or duplicate profile_id '$profileId'."
        }
        $assetKinds = @($profile.asset_kinds | ForEach-Object { [string]$_ })
        if ($assetKinds.Count -eq 0 -or
            @($assetKinds | Select-Object -Unique).Count -ne $assetKinds.Count -or
            @($assetKinds | Where-Object { $_ -notmatch '^[A-Z][A-Za-z0-9]{0,63}$' }).Count -ne 0) {
            throw "Resource-management workload profile '$profileId' has invalid asset_kinds."
        }
        $dependencyGraphShape = [string]$profile.dependency_graph_shape
        if ($dependencyGraphShape -cnotin @('none', 'dag')) {
            throw "Resource-management workload profile '$profileId' has unsupported dependency_graph_shape."
        }
        $tagCardinality = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $profile.tag_cardinality `
            -Label "Resource-management workload profile '$profileId' tag_cardinality"
        $queryMix = @($profile.query_mix | ForEach-Object { [string]$_ })
        if ($queryMix.Count -eq 0 -or
            @($queryMix | Select-Object -Unique).Count -ne $queryMix.Count -or
            @($queryMix | Where-Object { $_ -notin @('scan', 'page', 'asset-workspace-snapshot') }).Count -ne 0) {
            throw "Resource-management workload profile '$profileId' has invalid query_mix."
        }
        $minimumAssetCount = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $profile.minimum_asset_count `
            -Label "Resource-management workload profile '$profileId' minimum_asset_count"
        $maximumAssetCount = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $profile.maximum_asset_count `
            -Label "Resource-management workload profile '$profileId' maximum_asset_count"
        $changePercent = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $profile.change_percent `
            -Label "Resource-management workload profile '$profileId' change_percent"
        if ($minimumAssetCount -lt 1 -or $maximumAssetCount -lt $minimumAssetCount -or
            $maximumAssetCount -gt [int]::MaxValue -or $tagCardinality -gt [int]::MaxValue -or
            $changePercent -lt 1 -or $changePercent -gt 100) {
            throw "Resource-management workload profile '$profileId' has invalid numeric bounds."
        }
        $index[$profileId] = [pscustomobject][ordered]@{
            schema_version = 1
            profile_kind = 'zircon.resource-management-workload-profile'
            profile_id = $profileId
            asset_kinds = $assetKinds
            dependency_graph_shape = $dependencyGraphShape
            tag_cardinality = [int]$tagCardinality
            query_mix = $queryMix
            minimum_asset_count = [int]$minimumAssetCount
            maximum_asset_count = [int]$maximumAssetCount
            change_percent = [int]$changePercent
        }
    }
    $script:ResourceManagementWorkloadRegistrySnapshot = [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            registry_kind = 'zircon.resource-management-workload-registry'
            profile_count = $index.Count
            bytes = $evidence.bytes
            sha256 = $evidence.sha256
        }
        profiles = $index
    }
    return $script:ResourceManagementWorkloadRegistrySnapshot
}

function Get-ResourceManagementWorkloadProfile {
    param([Parameter(Mandatory)][string]$ProfileId)

    $snapshot = Get-ResourceManagementWorkloadRegistrySnapshot
    if (-not $snapshot.profiles.ContainsKey($ProfileId)) {
        throw "Resource-management workload profile '$ProfileId' is not registered."
    }
    return $snapshot.profiles[$ProfileId]
}

function Assert-ResourceManagementWorkloadRegistryReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [string]$Label = 'Resource-management workload registry receipt'
    )

    Assert-ResourceManagementSchemaProperties `
        -Value $Receipt `
        -RequiredNames @('schema_version', 'registry_kind', 'profile_count', 'bytes', 'sha256') `
        -Label $Label
    $snapshot = Get-ResourceManagementWorkloadRegistrySnapshot
    foreach ($name in @('schema_version', 'registry_kind', 'profile_count', 'bytes', 'sha256')) {
        if ([string]$Receipt.$name -cne [string]$snapshot.receipt.$name) {
            throw "$Label '$name' differs from the current workload registry snapshot."
        }
    }
    return $snapshot.receipt
}

Export-ModuleMember -Function @(
    'Assert-ResourceManagementWorkloadRegistryReceipt',
    'Get-ResourceManagementWorkloadProfile',
    'Get-ResourceManagementWorkloadRegistrySnapshot'
)
