Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementJsonEvidence.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop

$script:ResourceManagementSchemaRegistryMaximumBytes = 65536
$script:ResourceManagementSchemaRegistrySnapshot = $null

function Get-ResourceManagementSchemaRegistrySnapshot {
    if ($null -ne $script:ResourceManagementSchemaRegistrySnapshot) {
        return $script:ResourceManagementSchemaRegistrySnapshot
    }

    $evidence = Get-ResourceManagementJsonEvidence `
        -Path (Join-Path $PSScriptRoot 'resource-management-schema-registry.json') `
        -Label 'Resource-management schema registry' `
        -MaximumBytes 65536
    $registry = $evidence.json
    Assert-ResourceManagementSchemaProperties `
        -Value $registry `
        -RequiredNames @('schema_version', 'registry_kind', 'schemas') `
        -Label 'Resource-management schema registry'
    if ([int]$registry.schema_version -ne 1 -or
        [string]$registry.registry_kind -cne 'zircon.resource-management-schema-registry') {
        throw 'Resource-management schema registry has an unsupported schema identity.'
    }
    $registrations = @($registry.schemas)
    if ($registrations.Count -eq 0 -or $registrations.Count -gt 128) {
        throw 'Resource-management schema registry schema count is outside 1..128.'
    }
    $index = @{}
    foreach ($registration in $registrations) {
        Assert-ResourceManagementSchemaProperties `
            -Value $registration `
            -RequiredNames @(
                'schema_id', 'current_version', 'minimum_reader_version',
                'compatibility', 'identity_property', 'identity_value') `
            -Label 'Resource-management schema registration'
        $schemaId = [string]$registration.schema_id
        if ($schemaId -notmatch '^zircon\.resource-management\.[a-z0-9][a-z0-9-]{0,63}$' -or
            $index.ContainsKey($schemaId)) {
            throw "Resource-management schema registry has invalid or duplicate schema_id '$schemaId'."
        }
        $currentVersion = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $registration.current_version `
            -Label "Resource-management schema '$schemaId' current_version"
        $minimumReaderVersion = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
            -Value $registration.minimum_reader_version `
            -Label "Resource-management schema '$schemaId' minimum_reader_version"
        if ($currentVersion -lt 1 -or $currentVersion -gt [int]::MaxValue -or
            $minimumReaderVersion -lt 1 -or $minimumReaderVersion -gt $currentVersion) {
            throw "Resource-management schema '$schemaId' has invalid version bounds."
        }
        if ([string]$registration.compatibility -cne 'exact' -or
            $minimumReaderVersion -ne $currentVersion) {
            throw "Resource-management schema '$schemaId' has an unsupported compatibility policy."
        }
        $identityProperty = if ($null -eq $registration.identity_property) {
            $null
        }
        else {
            [string]$registration.identity_property
        }
        $identityValue = if ($null -eq $registration.identity_value) {
            $null
        }
        else {
            [string]$registration.identity_value
        }
        if (($null -eq $identityProperty) -ne ($null -eq $identityValue) -or
            ($null -ne $identityProperty -and
                ($identityProperty -notmatch '^[a-z][a-z0-9_]{0,63}$' -or
                 [string]::IsNullOrWhiteSpace($identityValue)))) {
            throw "Resource-management schema '$schemaId' has an invalid identity property/value pair."
        }
        $index[$schemaId] = [pscustomobject][ordered]@{
            schema_id = $schemaId
            current_version = [int]$currentVersion
            minimum_reader_version = [int]$minimumReaderVersion
            compatibility = 'exact'
            identity_property = $identityProperty
            identity_value = $identityValue
        }
    }
    $script:ResourceManagementSchemaRegistrySnapshot = [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            registry_kind = 'zircon.resource-management-schema-registry'
            schema_count = $index.Count
            bytes = $evidence.bytes
            sha256 = $evidence.sha256
        }
        schemas = $index
    }
    return $script:ResourceManagementSchemaRegistrySnapshot
}

function Assert-ResourceManagementRegisteredSchemaIdentity {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$SchemaId,
        [string]$Label = $SchemaId
    )

    $snapshot = Get-ResourceManagementSchemaRegistrySnapshot
    if (-not $snapshot.schemas.ContainsKey($SchemaId)) {
        throw "$Label references unregistered schema_id '$SchemaId'."
    }
    $registration = $snapshot.schemas[$SchemaId]
    $version = ConvertTo-ResourceManagementSchemaNonNegativeInteger `
        -Value (Get-ResourceManagementSchemaProperty -Value $Value -Name 'schema_version' -Label $Label) `
        -Label "$Label schema_version"
    if ($version -lt $registration.minimum_reader_version) {
        throw "$Label schema_version $version is stale; minimum is $($registration.minimum_reader_version)."
    }
    if ($version -gt $registration.current_version) {
        throw "$Label schema_version $version is from the future; current is $($registration.current_version)."
    }
    if ($registration.compatibility -eq 'exact' -and $version -ne $registration.current_version) {
        throw "$Label schema_version $version is incompatible with exact version $($registration.current_version)."
    }
    if ($null -ne $registration.identity_property) {
        $actualIdentity = [string](Get-ResourceManagementSchemaProperty `
                -Value $Value `
                -Name $registration.identity_property `
                -Label $Label)
        if ($actualIdentity -cne $registration.identity_value) {
            throw "$Label $($registration.identity_property) '$actualIdentity' differs from registered '$($registration.identity_value)'."
        }
    }
    return $Value
}

Export-ModuleMember -Function @(
    'Assert-ResourceManagementRegisteredSchemaIdentity',
    'Get-ResourceManagementSchemaRegistrySnapshot'
)
