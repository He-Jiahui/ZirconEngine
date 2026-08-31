Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpProductProfileRegistryPath = Join-Path $PSScriptRoot 'mvp-product-profile-registry.json'
$script:MvpProductProfileRegistryMaximumBytes = 64KB
$script:MvpProductProfileRegistryUpperHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-MvpProductProfileRegistryUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    for ($index = 0; $index -lt $Bytes.Length; $index++) {
        $value = $Bytes[$index]
        $characters[$index * 2] = $script:MvpProductProfileRegistryUpperHexDigits[$value -shr 4]
        $characters[$index * 2 + 1] = $script:MvpProductProfileRegistryUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-MvpProductProfileRegistryBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProductProfileRegistryUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpProductProfileRegistryProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Assert-MvpProductProfileRegistryExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value -or $Value -is [array] -or $Value -is [string] -or $Value -is [ValueType]) {
        throw "$Label must be one JSON object."
    }
    foreach ($name in $ExpectedNames) {
        if ($null -eq $Value.PSObject.Properties[$name]) {
            throw "$Label is missing '$name'."
        }
    }
    foreach ($property in $Value.PSObject.Properties) {
        if ($ExpectedNames -notcontains $property.Name) {
            throw "$Label contains unknown property '$($property.Name)'."
        }
    }
}

function Read-MvpProductProfileRegistryBytes {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP product profile registry does not exist or is not a file: $Path"
    }
    $stream = [IO.File]::Open($resolvedPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt $script:MvpProductProfileRegistryMaximumBytes) {
            throw "MVP product profile registry exceeds its byte budget of $($script:MvpProductProfileRegistryMaximumBytes) bytes."
        }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw 'MVP product profile registry changed while it was being read.'
            }
            $offset += $read
        }
        Write-Output -NoEnumerate $bytes
    }
    finally {
        $stream.Dispose()
    }
}

function Get-MvpProductProfileRegistrySnapshot {
    param([string]$RegistryPath = $script:MvpProductProfileRegistryPath)

    [byte[]]$bytes = Read-MvpProductProfileRegistryBytes -Path $RegistryPath
    try {
        $registry = ([Text.UTF8Encoding]::new($false, $true)).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP product profile registry is not valid UTF-8 JSON: $($_.Exception.Message)"
    }

    Assert-MvpProductProfileRegistryExactProperties `
        -Value $registry `
        -ExpectedNames @('schema_version', 'registry_kind', 'profiles') `
        -Label 'MVP product profile registry'
    $schemaVersion = Get-MvpProductProfileRegistryProperty -Value $registry -Name 'schema_version' -Label 'MVP product profile registry'
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 1) {
        throw "MVP product profile registry schema_version must be the JSON integer 1; found '$schemaVersion'."
    }
    $registryKind = [string](Get-MvpProductProfileRegistryProperty -Value $registry -Name 'registry_kind' -Label 'MVP product profile registry')
    if (-not $registryKind.Equals('zircon.mvp-product-profile-registry', [StringComparison]::Ordinal)) {
        throw "MVP product profile registry has unsupported registry_kind '$registryKind'."
    }
    $rawProfiles = Get-MvpProductProfileRegistryProperty -Value $registry -Name 'profiles' -Label 'MVP product profile registry'
    if ($rawProfiles -isnot [array] -or $rawProfiles.Count -eq 0) {
        throw 'MVP product profile registry profiles must be one non-empty JSON array.'
    }

    $profiles = [Collections.Generic.List[object]]::new()
    $seenProfileIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seenLogicalIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seenPublicationIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($profile in @($rawProfiles)) {
        Assert-MvpProductProfileRegistryExactProperties `
            -Value $profile `
            -ExpectedNames @('profile_id', 'target_profile', 'role', 'configuration', 'platform', 'features', 'products') `
            -Label 'MVP product profile'
        $profileId = [string](Get-MvpProductProfileRegistryProperty -Value $profile -Name 'profile_id' -Label 'MVP product profile')
        if ($profileId -notmatch '^[a-z0-9][a-z0-9-]{0,127}$') {
            throw "MVP product profile profile_id '$profileId' is invalid."
        }
        if (-not $seenProfileIds.Add($profileId)) {
            throw "MVP product profile registry contains duplicate profile_id '$profileId'."
        }
        $targetProfile = [string](Get-MvpProductProfileRegistryProperty -Value $profile -Name 'target_profile' -Label "MVP product profile '$profileId'")
        $role = [string](Get-MvpProductProfileRegistryProperty -Value $profile -Name 'role' -Label "MVP product profile '$profileId'")
        $configuration = [string](Get-MvpProductProfileRegistryProperty -Value $profile -Name 'configuration' -Label "MVP product profile '$profileId'")
        $platform = [string](Get-MvpProductProfileRegistryProperty -Value $profile -Name 'platform' -Label "MVP product profile '$profileId'")
        foreach ($identity in @(
                @{ Label = 'target_profile'; Value = $targetProfile },
                @{ Label = 'role'; Value = $role },
                @{ Label = 'platform'; Value = $platform }
            )) {
            if ($identity.Value -notmatch '^[a-z0-9][a-z0-9-]{0,127}$') {
                throw "MVP product profile '$profileId' $($identity.Label) '$($identity.Value)' is invalid."
            }
        }
        if ($configuration -notin @('development', 'release', 'profiling')) {
            throw "MVP product profile '$profileId' has unsupported configuration '$configuration'."
        }
        if ($platform -ne 'windows') {
            throw "MVP product profile '$profileId' has unsupported platform '$platform'."
        }

        $rawFeatures = Get-MvpProductProfileRegistryProperty -Value $profile -Name 'features' -Label "MVP product profile '$profileId'"
        if ($rawFeatures -isnot [string] -and $rawFeatures -isnot [array]) {
            throw "MVP product profile '$profileId' features must be one non-empty JSON array."
        }
        [object[]]$normalizedFeatures = @($rawFeatures)
        if ($normalizedFeatures.Count -eq 0) {
            throw "MVP product profile '$profileId' features must be one non-empty JSON array."
        }
        $featureTokens = [Collections.Generic.List[string]]::new()
        $seenFeatures = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($feature in $normalizedFeatures) {
            if ($feature -isnot [string] -or $feature -notmatch '^[A-Za-z0-9][A-Za-z0-9._+-]{0,127}$') {
                throw "MVP product profile '$profileId' feature token '$feature' is invalid."
            }
            if (-not $seenFeatures.Add($feature)) {
                throw "MVP product profile '$profileId' contains duplicate feature token '$feature'."
            }
            $featureTokens.Add($feature) | Out-Null
        }

        $rawProducts = Get-MvpProductProfileRegistryProperty -Value $profile -Name 'products' -Label "MVP product profile '$profileId'"
        if ($rawProducts -isnot [array] -or $rawProducts.Count -eq 0) {
            throw "MVP product profile '$profileId' products must be one non-empty JSON array."
        }
        $products = [Collections.Generic.List[object]]::new()
        foreach ($product in @($rawProducts)) {
            Assert-MvpProductProfileRegistryExactProperties `
                -Value $product `
                -ExpectedNames @('logical_id', 'package', 'bin', 'artifact_name') `
                -Label "MVP product profile '$profileId' product"
            $logicalId = [string](Get-MvpProductProfileRegistryProperty -Value $product -Name 'logical_id' -Label "MVP product profile '$profileId' product")
            $package = [string](Get-MvpProductProfileRegistryProperty -Value $product -Name 'package' -Label "MVP product '$logicalId'")
            $binValue = Get-MvpProductProfileRegistryProperty -Value $product -Name 'bin' -Label "MVP product '$logicalId'"
            $bin = if ($null -eq $binValue) { $null } else { [string]$binValue }
            $artifactName = [string](Get-MvpProductProfileRegistryProperty -Value $product -Name 'artifact_name' -Label "MVP product '$logicalId'")
            if ($logicalId -notmatch '^[a-z0-9][a-z0-9._/-]{0,127}$' -or $logicalId.Contains('..')) {
                throw "MVP product logical_id '$logicalId' is invalid."
            }
            if (-not $seenLogicalIds.Add($logicalId)) {
                throw "MVP product profile registry contains duplicate product logical_id '$logicalId'."
            }
            if ($package -notmatch '^[A-Za-z_][A-Za-z0-9_]{0,127}$') {
                throw "MVP product '$logicalId' package '$package' is invalid."
            }
            if ($null -ne $bin -and $bin -notmatch '^[A-Za-z_][A-Za-z0-9_]{0,127}$') {
                throw "MVP product '$logicalId' bin '$bin' is invalid."
            }
            if ($artifactName -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}\.(?:exe|dll)$') {
                throw "MVP product '$logicalId' artifact_name '$artifactName' is not one Windows product leaf."
            }
            $publicationId = "$role/$artifactName"
            if (-not $seenPublicationIds.Add($publicationId)) {
                throw "MVP product profile registry contains duplicate publication '$publicationId'."
            }
            $products.Add([pscustomobject][ordered]@{
                    logical_id = $logicalId
                    package = $package
                    bin = $bin
                    artifact_name = $artifactName
                }) | Out-Null
        }
        $profiles.Add([pscustomobject][ordered]@{
                profile_id = $profileId
                target_profile = $targetProfile
                role = $role
                configuration = $configuration
                platform = $platform
                features = $featureTokens.ToArray()
                products = $products.ToArray()
            }) | Out-Null
    }

    return [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            registry_kind = $registryKind
            sha256 = Get-MvpProductProfileRegistryBytesSha256 -Bytes $bytes
            size_bytes = [Int64]$bytes.LongLength
        }
        profiles = $profiles.ToArray()
    }
}

function Get-MvpProductProfileSpecifications {
    param([AllowNull()]$RegistrySnapshot)

    $snapshot = if ($null -eq $RegistrySnapshot) {
        Get-MvpProductProfileRegistrySnapshot
    }
    else {
        $RegistrySnapshot
    }
    Assert-MvpProductProfileRegistryExactProperties `
        -Value $snapshot `
        -ExpectedNames @('receipt', 'profiles') `
        -Label 'MVP product profile registry snapshot'
    $specifications = [Collections.Generic.List[object]]::new()
    foreach ($profile in @($snapshot.profiles)) {
        $features = @($profile.features) -join ','
        foreach ($product in @($profile.products)) {
            $specifications.Add([pscustomobject][ordered]@{
                    logical_id = $product.logical_id
                    package = $product.package
                    bin = $product.bin
                    features = $features
                    output_group = $profile.role
                    artifact_name = $product.artifact_name
                    profile_id = $profile.profile_id
                    target_profile = $profile.target_profile
                    role = $profile.role
                    configuration = $profile.configuration
                    platform = $profile.platform
                }) | Out-Null
        }
    }
    return $specifications.ToArray()
}

function Assert-MvpProductProfileRegistryReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$ExpectedSnapshot
    )

    Assert-MvpProductProfileRegistryExactProperties `
        -Value $Receipt `
        -ExpectedNames @('schema_version', 'registry_kind', 'sha256', 'size_bytes') `
        -Label 'MVP product profile registry receipt'
    foreach ($propertyName in @('schema_version', 'registry_kind', 'sha256', 'size_bytes')) {
        if ([string]$Receipt.$propertyName -cne [string]$ExpectedSnapshot.receipt.$propertyName) {
            throw "MVP product profile registry receipt $propertyName differs from the current registry snapshot."
        }
    }
    return $ExpectedSnapshot.receipt
}

Export-ModuleMember -Function @(
    'Get-MvpProductProfileRegistrySnapshot',
    'Get-MvpProductProfileSpecifications',
    'Assert-MvpProductProfileRegistryReceipt'
)
