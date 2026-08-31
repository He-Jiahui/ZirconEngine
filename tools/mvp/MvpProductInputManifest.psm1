Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpProductInputSchemaVersion = 2
$script:MvpProductInputUpperHexDigits = [char[]]'0123456789ABCDEF'

$moduleRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductProfileRegistry.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $moduleRepoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Get-MvpProductInputSpecifications {
    param([AllowNull()]$RegistrySnapshot)

    return @(Get-MvpProductProfileSpecifications -RegistrySnapshot $RegistrySnapshot)
}

function ConvertTo-MvpProductInputUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    for ($index = 0; $index -lt $Bytes.Length; $index++) {
        $value = $Bytes[$index]
        $characters[$index * 2] = $script:MvpProductInputUpperHexDigits[$value -shr 4]
        $characters[$index * 2 + 1] = $script:MvpProductInputUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-MvpProductInputFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProductInputUpperHex -Bytes $hasher.ComputeHash($stream)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-MvpProductInputBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProductInputUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpProductInputManifestProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        throw "$Label is missing required property '$Name'."
    }
    return $property.Value
}

function Resolve-MvpProductInputBuildSet {
    param(
        [Parameter(Mandatory)]$Manifest,
        [Parameter(Mandatory)][string]$Path
    )

    $property = $Manifest.PSObject.Properties['build_set']
    if ($null -eq $property) {
        return $null
    }
    $buildSet = $property.Value
    if ($null -eq $buildSet -or $buildSet -is [Array]) {
        throw "ProductInputManifest '$Path' build_set must contain one JSON object."
    }
    $buildSetId = [string](Get-MvpProductInputManifestProperty `
            -Value $buildSet `
            -Name 'build_set_id' `
            -Label 'ProductInputManifest build_set')
    if ($buildSetId -notmatch '^[0-9A-F]{64}$') {
        throw "ProductInputManifest '$Path' build_set_id must be an uppercase SHA-256."
    }
    $gitRevision = [string](Get-MvpProductInputManifestProperty `
            -Value $buildSet `
            -Name 'git_revision' `
            -Label 'ProductInputManifest build_set')
    if ($gitRevision -notmatch '^[0-9a-f]{40}$') {
        throw "ProductInputManifest '$Path' build_set git_revision must be a lowercase Git object ID."
    }
    $dirtyOverlaySha256 = [string](Get-MvpProductInputManifestProperty `
            -Value $buildSet `
            -Name 'dirty_overlay_sha256' `
            -Label 'ProductInputManifest build_set')
    if ($dirtyOverlaySha256 -notmatch '^[0-9A-F]{64}$') {
        throw "ProductInputManifest '$Path' build_set dirty_overlay_sha256 must be an uppercase SHA-256."
    }
    $manifestRelativePath = [string](Get-MvpProductInputManifestProperty `
            -Value $buildSet `
            -Name 'manifest_relative_path' `
            -Label 'ProductInputManifest build_set')
    $pathSegments = @($manifestRelativePath.Split('/'))
    $invalidPathSegments = @($pathSegments | Where-Object {
            [string]::IsNullOrWhiteSpace($_) -or $_ -in @('.', '..')
        })
    if ([string]::IsNullOrWhiteSpace($manifestRelativePath) -or
        [IO.Path]::IsPathRooted($manifestRelativePath) -or
        $manifestRelativePath.IndexOf([char]92) -ge 0 -or
        $manifestRelativePath.IndexOf([char]58) -ge 0 -or
        $invalidPathSegments.Count -ne 0) {
        throw "ProductInputManifest '$Path' build_set manifest_relative_path must be a non-rooted, slash-delimited path below the product-input manifest."
    }
    return [ordered]@{
        build_set_id = $buildSetId
        git_revision = $gitRevision
        dirty_overlay_sha256 = $dirtyOverlaySha256
        manifest_relative_path = $manifestRelativePath
    }
}

function Resolve-MvpProductInputManifest {
    param([Parameter(Mandatory)][string]$Path)

    $manifestPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($manifestPath)) {
        throw "ProductInputManifest '$Path' does not exist or is not a file."
    }
    try {
        # Stage copies must be compared with the exact bytes parsed here, not a later re-read.
        $manifestBytes = [IO.File]::ReadAllBytes($manifestPath)
        $manifest = ([Text.UTF8Encoding]::new($false)).GetString($manifestBytes) | ConvertFrom-Json
    }
    catch {
        throw "ProductInputManifest '$Path' is not valid JSON: $($_.Exception.Message)"
    }
    if ($null -eq $manifest -or $manifest -is [Array]) {
        throw "ProductInputManifest '$Path' must contain one JSON object."
    }

    $schemaVersion = Get-MvpProductInputManifestProperty -Value $manifest -Name 'schema_version' -Label 'ProductInputManifest'
    if ([int]$schemaVersion -ne $script:MvpProductInputSchemaVersion) {
        throw "ProductInputManifest '$Path' has unsupported schema_version '$schemaVersion'."
    }
    $sourceFingerprint = [string](Get-MvpProductInputManifestProperty -Value $manifest -Name 'source_fingerprint' -Label 'ProductInputManifest')
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw "ProductInputManifest '$Path' must contain an uppercase SHA-256 source_fingerprint."
    }
    $productProfileRegistrySnapshot = Get-MvpProductProfileRegistrySnapshot
    $productProfileRegistryReceipt = Assert-MvpProductProfileRegistryReceipt `
        -Receipt (Get-MvpProductInputManifestProperty -Value $manifest -Name 'product_profile_registry' -Label 'ProductInputManifest') `
        -ExpectedSnapshot $productProfileRegistrySnapshot
    $productInputSpecifications = @(Get-MvpProductInputSpecifications -RegistrySnapshot $productProfileRegistrySnapshot)
    $buildSet = Resolve-MvpProductInputBuildSet -Manifest $manifest -Path $Path
    if ($null -ne $buildSet -and
        -not $sourceFingerprint.Equals([string]$buildSet.build_set_id, [StringComparison]::Ordinal)) {
        throw "ProductInputManifest '$Path' source_fingerprint must equal its BuildSetId."
    }
    $artifacts = @(Get-MvpProductInputManifestProperty -Value $manifest -Name 'artifacts' -Label 'ProductInputManifest')
    if ($artifacts.Count -ne $productInputSpecifications.Count) {
        throw "ProductInputManifest '$Path' must contain exactly $($productInputSpecifications.Count) product artifacts."
    }

    $resolvedArtifacts = [ordered]@{}
    foreach ($specification in $productInputSpecifications) {
        $matches = @($artifacts | Where-Object {
            ([string](Get-MvpProductInputManifestProperty -Value $_ -Name 'LogicalId' -Label 'Product artifact')) -eq $specification.logical_id
        })
        if ($matches.Count -ne 1) {
            throw "ProductInputManifest '$Path' must contain exactly one '$($specification.logical_id)' artifact."
        }
        $artifact = $matches[0]
        foreach ($propertyName in @('Package', 'Bin', 'Features', 'OutputGroup', 'ArtifactName')) {
            $actual = Get-MvpProductInputManifestProperty -Value $artifact -Name $propertyName -Label "Product artifact '$($specification.logical_id)'"
            $expectedName = switch ($propertyName) {
                'Package' { 'package' }
                'Bin' { 'bin' }
                'Features' { 'features' }
                'OutputGroup' { 'output_group' }
                'ArtifactName' { 'artifact_name' }
            }
            $expected = $specification.PSObject.Properties[$expectedName].Value
            if ($null -eq $expected) {
                if ($null -ne $actual -and -not [string]::IsNullOrWhiteSpace([string]$actual)) {
                    throw "Product artifact '$($specification.logical_id)' has unexpected $propertyName '$actual'."
                }
            }
            elseif ([string]$actual -ne [string]$expected) {
                throw "Product artifact '$($specification.logical_id)' has $propertyName '$actual'; expected '$expected'."
            }
        }

        $artifactPath = [string](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Path' -Label "Product artifact '$($specification.logical_id)'")
        $artifactResolution = Resolve-ZirconWindowsPath -Path $artifactPath
        if (-not [IO.File]::Exists($artifactResolution.OperationalPath)) {
            throw "Product artifact '$($specification.logical_id)' does not exist: $artifactPath"
        }
        $expectedBytes = [Int64](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Bytes' -Label "Product artifact '$($specification.logical_id)'")
        $actualBytes = [IO.FileInfo]::new($artifactResolution.OperationalPath).Length
        if ($expectedBytes -ne $actualBytes) {
            throw "Product artifact '$($specification.logical_id)' byte length differs from ProductInputManifest."
        }
        $expectedHash = [string](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Sha256' -Label "Product artifact '$($specification.logical_id)'")
        if ($expectedHash -notmatch '^[0-9A-F]{64}$') {
            throw "Product artifact '$($specification.logical_id)' must contain an uppercase SHA-256."
        }
        $actualHash = Get-MvpProductInputFileSha256 -Path $artifactResolution.OperationalPath
        if ($expectedHash -ne $actualHash) {
            throw "Product artifact '$($specification.logical_id)' SHA-256 differs from ProductInputManifest."
        }
        $resolvedArtifacts[$specification.logical_id] = [ordered]@{
            operation_path = $artifactResolution.OperationalPath
            bytes = $actualBytes
            sha256 = $actualHash
        }
    }

    return [ordered]@{
        operation_path = $manifestPath
        bytes = [Int64]$manifestBytes.LongLength
        sha256 = Get-MvpProductInputBytesSha256 -Bytes $manifestBytes
        source_fingerprint = $sourceFingerprint
        product_profile_registry = $productProfileRegistryReceipt
        build_set = $buildSet
        artifacts = $resolvedArtifacts
    }
}

Export-ModuleMember -Function @(
    'Get-MvpProductInputSpecifications',
    'Get-MvpProductInputFileSha256',
    'Get-MvpProductInputBytesSha256',
    'Resolve-MvpProductInputManifest'
)
