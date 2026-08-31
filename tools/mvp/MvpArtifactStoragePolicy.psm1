Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpArtifactStoragePolicyPath = Join-Path $PSScriptRoot 'mvp-artifact-storage-policy.json'
$script:MvpArtifactStoragePolicyMaximumBytes = 32KB
$script:MvpArtifactStoragePolicyUpperHexDigits = [char[]]'0123456789ABCDEF'
$storagePolicyRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $storagePolicyRepoRoot 'tools\WindowsPathResolver.psm1') -ErrorAction Stop

function ConvertTo-MvpArtifactStoragePolicyUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    for ($index = 0; $index -lt $Bytes.Length; $index++) {
        $value = $Bytes[$index]
        $characters[$index * 2] = $script:MvpArtifactStoragePolicyUpperHexDigits[$value -shr 4]
        $characters[$index * 2 + 1] = $script:MvpArtifactStoragePolicyUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-MvpArtifactStoragePolicyBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpArtifactStoragePolicyUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpArtifactStoragePolicyProperty {
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

function Assert-MvpArtifactStoragePolicyExactProperties {
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

function Read-MvpArtifactStoragePolicyBytes {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP artifact storage policy does not exist or is not a file: $Path"
    }
    $stream = [IO.File]::Open($resolvedPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt $script:MvpArtifactStoragePolicyMaximumBytes) {
            throw "MVP artifact storage policy exceeds its byte budget of $($script:MvpArtifactStoragePolicyMaximumBytes) bytes."
        }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw 'MVP artifact storage policy changed while it was being read.'
            }
            $offset += $read
        }
        Write-Output -NoEnumerate $bytes
    }
    finally {
        $stream.Dispose()
    }
}

function Get-MvpArtifactStoragePolicySnapshot {
    param([string]$PolicyPath = $script:MvpArtifactStoragePolicyPath)

    [byte[]]$bytes = Read-MvpArtifactStoragePolicyBytes -Path $PolicyPath
    try {
        $policy = ([Text.UTF8Encoding]::new($false, $true)).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP artifact storage policy is not valid UTF-8 JSON: $($_.Exception.Message)"
    }
    Assert-MvpArtifactStoragePolicyExactProperties `
        -Value $policy `
        -ExpectedNames @('schema_version', 'policy_kind', 'platform', 'default_root_id', 'roots', 'namespaces') `
        -Label 'MVP artifact storage policy'
    $schemaVersion = Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'schema_version' -Label 'MVP artifact storage policy'
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 1) {
        throw "MVP artifact storage policy schema_version must be the JSON integer 1; found '$schemaVersion'."
    }
    $policyKind = [string](Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'policy_kind' -Label 'MVP artifact storage policy')
    if (-not $policyKind.Equals('zircon.mvp-artifact-storage-policy', [StringComparison]::Ordinal)) {
        throw "MVP artifact storage policy has unsupported policy_kind '$policyKind'."
    }
    $platform = [string](Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'platform' -Label 'MVP artifact storage policy')
    if ($platform -ne 'windows') {
        throw "MVP artifact storage policy has unsupported platform '$platform'."
    }
    $defaultRootId = [string](Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'default_root_id' -Label 'MVP artifact storage policy')
    $rawRoots = Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'roots' -Label 'MVP artifact storage policy'
    $rawNamespaces = Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'namespaces' -Label 'MVP artifact storage policy'
    if ($rawRoots -isnot [array] -or $rawRoots.Count -eq 0) {
        throw 'MVP artifact storage policy roots must be one non-empty JSON array.'
    }
    if ($rawNamespaces -isnot [array] -or $rawNamespaces.Count -eq 0) {
        throw 'MVP artifact storage policy namespaces must be one non-empty JSON array.'
    }

    $roots = [Collections.Generic.List[object]]::new()
    $seenRootIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seenRootPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($root in @($rawRoots)) {
        Assert-MvpArtifactStoragePolicyExactProperties `
            -Value $root `
            -ExpectedNames @('root_id', 'display_path', 'capability_class') `
            -Label 'MVP artifact storage policy root'
        $rootId = [string](Get-MvpArtifactStoragePolicyProperty -Value $root -Name 'root_id' -Label 'MVP artifact storage policy root')
        $displayPath = [string](Get-MvpArtifactStoragePolicyProperty -Value $root -Name 'display_path' -Label "MVP artifact storage root '$rootId'")
        $capabilityClass = [string](Get-MvpArtifactStoragePolicyProperty -Value $root -Name 'capability_class' -Label "MVP artifact storage root '$rootId'")
        if ($rootId -notmatch '^[a-z0-9][a-z0-9-]{0,127}$') {
            throw "MVP artifact storage root_id '$rootId' is invalid."
        }
        if (-not $seenRootIds.Add($rootId)) {
            throw "MVP artifact storage policy contains duplicate root_id '$rootId'."
        }
        if ($displayPath -notmatch '^[D-F]:\\ZirconBuilds$') {
            throw "MVP artifact storage root '$rootId' display_path '$displayPath' is invalid."
        }
        if (-not $seenRootPaths.Add($displayPath)) {
            throw "MVP artifact storage policy contains duplicate approved root path '$displayPath'."
        }
        if ($capabilityClass -ne 'windows-local-artifact') {
            throw "MVP artifact storage root '$rootId' has unsupported capability_class '$capabilityClass'."
        }
        $roots.Add([pscustomobject][ordered]@{
                root_id = $rootId
                display_path = $displayPath
                capability_class = $capabilityClass
            }) | Out-Null
    }
    if (@($roots | Where-Object { $_.root_id -eq $defaultRootId }).Count -ne 1) {
        throw "MVP artifact storage policy default_root_id '$defaultRootId' is not registered exactly once."
    }

    $namespaces = [Collections.Generic.List[object]]::new()
    $seenNamespaceIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seenLeafPrefixes = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($namespace in @($rawNamespaces)) {
        Assert-MvpArtifactStoragePolicyExactProperties `
            -Value $namespace `
            -ExpectedNames @('namespace_id', 'leaf_prefix', 'capability_class') `
            -Label 'MVP artifact storage namespace'
        $namespaceId = [string](Get-MvpArtifactStoragePolicyProperty -Value $namespace -Name 'namespace_id' -Label 'MVP artifact storage namespace')
        $leafPrefix = [string](Get-MvpArtifactStoragePolicyProperty -Value $namespace -Name 'leaf_prefix' -Label "MVP artifact storage namespace '$namespaceId'")
        $capabilityClass = [string](Get-MvpArtifactStoragePolicyProperty -Value $namespace -Name 'capability_class' -Label "MVP artifact storage namespace '$namespaceId'")
        if ($namespaceId -notmatch '^[a-z0-9][a-z0-9-]{0,127}$') {
            throw "MVP artifact storage namespace_id '$namespaceId' is invalid."
        }
        if (-not $seenNamespaceIds.Add($namespaceId)) {
            throw "MVP artifact storage policy contains duplicate namespace_id '$namespaceId'."
        }
        if ($leafPrefix -notmatch '^[a-z0-9][a-z0-9-]{0,127}-$') {
            throw "MVP artifact storage namespace '$namespaceId' leaf_prefix '$leafPrefix' is invalid."
        }
        if (-not $seenLeafPrefixes.Add($leafPrefix)) {
            throw "MVP artifact storage policy contains duplicate leaf_prefix '$leafPrefix'."
        }
        if (-not @($roots | Where-Object { $_.capability_class -eq $capabilityClass })) {
            throw "MVP artifact storage namespace '$namespaceId' capability_class '$capabilityClass' has no approved root."
        }
        $namespaces.Add([pscustomobject][ordered]@{
                namespace_id = $namespaceId
                leaf_prefix = $leafPrefix
                capability_class = $capabilityClass
            }) | Out-Null
    }

    return [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            policy_kind = $policyKind
            sha256 = Get-MvpArtifactStoragePolicyBytesSha256 -Bytes $bytes
            size_bytes = [Int64]$bytes.LongLength
        }
        platform = $platform
        default_root_id = $defaultRootId
        roots = $roots.ToArray()
        namespaces = $namespaces.ToArray()
    }
}

function Get-MvpArtifactStorageNamespace {
    param(
        [Parameter(Mandatory)]$PolicySnapshot,
        [Parameter(Mandatory)][string]$NamespaceId
    )

    $matches = @($PolicySnapshot.namespaces | Where-Object { $_.namespace_id -ceq $NamespaceId })
    if ($matches.Count -ne 1) {
        throw "MVP artifact storage namespace '$NamespaceId' is not registered exactly once."
    }
    return $matches[0]
}

function New-MvpArtifactStoragePath {
    param(
        [Parameter(Mandatory)][string]$NamespaceId,
        [string]$InstanceId = [guid]::NewGuid().ToString('N'),
        [AllowNull()]$PolicySnapshot
    )

    if ($InstanceId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
        throw "MVP artifact storage instance ID '$InstanceId' is invalid."
    }
    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    $namespace = Get-MvpArtifactStorageNamespace -PolicySnapshot $snapshot -NamespaceId $NamespaceId
    $root = @($snapshot.roots | Where-Object { $_.root_id -ceq $snapshot.default_root_id })[0]
    if ($root.capability_class -cne $namespace.capability_class) {
        throw "MVP artifact storage default root cannot satisfy namespace '$NamespaceId'."
    }
    return Join-Path $root.display_path ($namespace.leaf_prefix + $InstanceId)
}

function Get-MvpArtifactStorageDefaultRootPath {
    param(
        [Parameter(Mandatory)][string]$CapabilityClass,
        [AllowNull()]$PolicySnapshot
    )

    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    $roots = @($snapshot.roots | Where-Object {
            $_.root_id -ceq $snapshot.default_root_id -and
            $_.capability_class -ceq $CapabilityClass
        })
    if ($roots.Count -ne 1) {
        throw "MVP artifact storage default root cannot satisfy capability '$CapabilityClass'."
    }
    return $roots[0].display_path
}

function Resolve-MvpArtifactStorageRootPath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$CapabilityClass,
        [AllowNull()]$PolicySnapshot
    )

    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    $resolution = Resolve-ZirconWindowsPath -Path $Path
    foreach ($root in @($snapshot.roots | Where-Object { $_.capability_class -ceq $CapabilityClass })) {
        $rootPath = $root.display_path.TrimEnd('\')
        $rootPrefix = $rootPath + '\'
        if ($resolution.DisplayPath.Equals($rootPath, [StringComparison]::OrdinalIgnoreCase) -or
            $resolution.DisplayPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            return [pscustomobject][ordered]@{
                operation_path = $resolution.OperationalPath
                display_path = $resolution.DisplayPath
                root_id = $root.root_id
                root_display_path = $root.display_path
                capability_class = $root.capability_class
            }
        }
    }
    throw "Artifact path '$($resolution.DisplayPath)' is outside the approved '$CapabilityClass' storage roots."
}

function Get-MvpArtifactStorageCapabilityEvidence {
    param(
        [Parameter(Mandatory)][string]$RootPath,
        [Parameter(Mandatory)][string]$CapabilityClass,
        [ValidateRange(0, [Int64]::MaxValue)][Int64]$RequiredFreeSpaceBytes = 0,
        [AllowNull()]$PolicySnapshot
    )

    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    $rootResolution = Resolve-MvpArtifactStorageRootPath `
        -Path $RootPath `
        -CapabilityClass $CapabilityClass `
        -PolicySnapshot $snapshot
    $approvedRootPath = $rootResolution.root_display_path
    if (-not [IO.Directory]::Exists($approvedRootPath)) {
        throw "Approved artifact storage root '$approvedRootPath' does not exist or is not a directory."
    }
    $driveRoot = [IO.Path]::GetPathRoot($approvedRootPath)
    if ([string]::IsNullOrWhiteSpace($driveRoot)) {
        throw "Could not resolve the drive for approved artifact storage root '$approvedRootPath'."
    }
    try {
        $drive = [IO.DriveInfo]::new($driveRoot)
        if (-not $drive.IsReady) {
            throw "drive '$driveRoot' is not ready"
        }
        $driveType = $drive.DriveType.ToString()
        $fileSystem = $drive.DriveFormat
        [Int64]$availableFreeSpaceBytes = $drive.AvailableFreeSpace
    }
    catch {
        throw "Could not inspect approved artifact storage root '$approvedRootPath': $($_.Exception.Message)"
    }
    if ($drive.DriveType -ne [IO.DriveType]::Fixed) {
        throw "Approved artifact storage root '$approvedRootPath' requires a fixed local drive; found '$driveType'."
    }
    if ($fileSystem -notin @('NTFS', 'ReFS')) {
        throw "Approved artifact storage root '$approvedRootPath' requires NTFS or ReFS; found '$fileSystem'."
    }
    if ($availableFreeSpaceBytes -lt $RequiredFreeSpaceBytes) {
        throw "Approved artifact storage root '$approvedRootPath' requires at least $RequiredFreeSpaceBytes free bytes but only $availableFreeSpaceBytes bytes are available."
    }

    $probeDirectory = Join-ZirconWindowsPath `
        -Path $rootResolution.operation_path `
        -ChildPath ('.zircon-storage-capability-probe-' + [guid]::NewGuid().ToString('N'))
    $sourcePath = Join-ZirconWindowsPath -Path $probeDirectory -ChildPath 'source.tmp'
    $destinationPath = Join-ZirconWindowsPath -Path $probeDirectory -ChildPath 'destination.tmp'
    [byte[]]$payload = [Text.Encoding]::ASCII.GetBytes('zircon-artifact-storage-capability-v1')
    try {
        [IO.Directory]::CreateDirectory($probeDirectory) | Out-Null
        $stream = [IO.FileStream]::new(
            $sourcePath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None,
            4096,
            [IO.FileOptions]::WriteThrough)
        try {
            $stream.Write($payload, 0, $payload.Length)
            $stream.Flush($true)
        }
        finally {
            $stream.Dispose()
        }
        [IO.File]::Move($sourcePath, $destinationPath)
        if ([IO.File]::Exists($sourcePath) -or -not [IO.File]::Exists($destinationPath)) {
            throw 'same-volume atomic move did not publish exactly one destination file'
        }
        [byte[]]$publishedPayload = [IO.File]::ReadAllBytes($destinationPath)
        if ($publishedPayload.Length -ne $payload.Length) {
            throw 'durable capability probe payload length changed after publication'
        }
        for ($index = 0; $index -lt $payload.Length; $index++) {
            if ($publishedPayload[$index] -ne $payload[$index]) {
                throw 'durable capability probe payload changed after publication'
            }
        }
    }
    catch {
        throw "Approved artifact storage capability probe failed for '$approvedRootPath': $($_.Exception.Message)"
    }
    finally {
        if ([IO.File]::Exists($sourcePath)) {
            [IO.File]::Delete($sourcePath)
        }
        if ([IO.File]::Exists($destinationPath)) {
            [IO.File]::Delete($destinationPath)
        }
        if ([IO.Directory]::Exists($probeDirectory)) {
            [IO.Directory]::Delete($probeDirectory, $false)
        }
    }

    return [pscustomobject][ordered]@{
        schema_version = 1
        capability_kind = 'zircon.mvp-artifact-storage-capability'
        policy = $snapshot.receipt
        root_id = $rootResolution.root_id
        capability_class = $rootResolution.capability_class
        drive_root = $driveRoot
        drive_type = $driveType
        file_system = $fileSystem
        required_free_space_bytes = $RequiredFreeSpaceBytes
        available_free_space_bytes = $availableFreeSpaceBytes
        durable_file_flush_supported = $true
        same_volume_atomic_move_supported = $true
        captured_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    }
}

function ConvertTo-MvpArtifactStorageEvidenceInt64 {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    if (-not ($Value -is [int] -or $Value -is [long]) -or [Int64]$Value -lt 0) {
        throw "Artifact storage capability evidence '$Name' must be one non-negative JSON integer."
    }
    return [Int64]$Value
}

function Assert-MvpArtifactStorageCapabilityEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$ExpectedPath,
        [ValidateRange(0, [Int64]::MaxValue)][Int64]$ExpectedRequiredFreeSpaceBytes,
        [AllowNull()]$PolicySnapshot
    )

    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    Assert-MvpArtifactStoragePolicyExactProperties `
        -Value $Evidence `
        -ExpectedNames @(
            'schema_version',
            'capability_kind',
            'policy',
            'root_id',
            'capability_class',
            'drive_root',
            'drive_type',
            'file_system',
            'required_free_space_bytes',
            'available_free_space_bytes',
            'durable_file_flush_supported',
            'same_volume_atomic_move_supported',
            'captured_at_utc'
        ) `
        -Label 'Artifact storage capability evidence'
    $schemaVersion = ConvertTo-MvpArtifactStorageEvidenceInt64 `
        -Value (Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'schema_version' -Label 'Artifact storage capability evidence') `
        -Name 'schema_version'
    if ($schemaVersion -ne 1) {
        throw "Artifact storage capability evidence has unsupported schema_version '$schemaVersion'."
    }
    $capabilityKind = [string](Get-MvpArtifactStoragePolicyProperty `
        -Value $Evidence `
        -Name 'capability_kind' `
        -Label 'Artifact storage capability evidence')
    if ($capabilityKind -cne 'zircon.mvp-artifact-storage-capability') {
        throw "Artifact storage capability evidence has unsupported capability_kind '$capabilityKind'."
    }

    $policy = Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'policy' -Label 'Artifact storage capability evidence'
    Assert-MvpArtifactStoragePolicyExactProperties `
        -Value $policy `
        -ExpectedNames @('schema_version', 'policy_kind', 'sha256', 'size_bytes') `
        -Label 'Artifact storage capability evidence policy receipt'
    $policySchemaVersion = ConvertTo-MvpArtifactStorageEvidenceInt64 `
        -Value (Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'schema_version' -Label 'Artifact storage capability evidence policy receipt') `
        -Name 'policy.schema_version'
    $policySizeBytes = ConvertTo-MvpArtifactStorageEvidenceInt64 `
        -Value (Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'size_bytes' -Label 'Artifact storage capability evidence policy receipt') `
        -Name 'policy.size_bytes'
    $policyKind = [string](Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'policy_kind' -Label 'Artifact storage capability evidence policy receipt')
    $policySha256 = [string](Get-MvpArtifactStoragePolicyProperty -Value $policy -Name 'sha256' -Label 'Artifact storage capability evidence policy receipt')
    if ($policySchemaVersion -ne [Int64]$snapshot.receipt.schema_version -or
        $policyKind -cne [string]$snapshot.receipt.policy_kind -or
        -not $policySha256.Equals([string]$snapshot.receipt.sha256, [StringComparison]::OrdinalIgnoreCase) -or
        $policySizeBytes -ne [Int64]$snapshot.receipt.size_bytes) {
        throw 'Artifact storage capability evidence policy receipt differs from the current storage policy.'
    }

    $capabilityClass = [string](Get-MvpArtifactStoragePolicyProperty `
        -Value $Evidence `
        -Name 'capability_class' `
        -Label 'Artifact storage capability evidence')
    if ($capabilityClass -cne 'windows-local-artifact') {
        throw "Artifact storage capability evidence has unsupported capability_class '$capabilityClass'."
    }
    $rootResolution = Resolve-MvpArtifactStorageRootPath `
        -Path $ExpectedPath `
        -CapabilityClass $capabilityClass `
        -PolicySnapshot $snapshot
    $rootId = [string](Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'root_id' -Label 'Artifact storage capability evidence')
    if ($rootId -cne $rootResolution.root_id) {
        throw "Artifact storage capability evidence root_id '$rootId' differs from expected root '$($rootResolution.root_id)'."
    }
    $driveRoot = [string](Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'drive_root' -Label 'Artifact storage capability evidence')
    $expectedDriveRoot = [IO.Path]::GetPathRoot($rootResolution.root_display_path)
    if (-not $driveRoot.Equals($expectedDriveRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Artifact storage capability evidence drive_root '$driveRoot' differs from expected drive '$expectedDriveRoot'."
    }
    $driveType = [string](Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'drive_type' -Label 'Artifact storage capability evidence')
    if ($driveType -cne 'Fixed') {
        throw "Artifact storage capability evidence drive_type must be 'Fixed'; found '$driveType'."
    }
    $fileSystem = [string](Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'file_system' -Label 'Artifact storage capability evidence')
    if ($fileSystem -notin @('NTFS', 'ReFS')) {
        throw "Artifact storage capability evidence file_system must be NTFS or ReFS; found '$fileSystem'."
    }

    $requiredFreeSpaceBytes = ConvertTo-MvpArtifactStorageEvidenceInt64 `
        -Value (Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'required_free_space_bytes' -Label 'Artifact storage capability evidence') `
        -Name 'required_free_space_bytes'
    $availableFreeSpaceBytes = ConvertTo-MvpArtifactStorageEvidenceInt64 `
        -Value (Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'available_free_space_bytes' -Label 'Artifact storage capability evidence') `
        -Name 'available_free_space_bytes'
    if ($requiredFreeSpaceBytes -ne $ExpectedRequiredFreeSpaceBytes) {
        throw "Artifact storage capability evidence required_free_space_bytes '$requiredFreeSpaceBytes' differs from expected '$ExpectedRequiredFreeSpaceBytes'."
    }
    if ($availableFreeSpaceBytes -lt $requiredFreeSpaceBytes) {
        throw "Artifact storage capability evidence available_free_space_bytes '$availableFreeSpaceBytes' is below required_free_space_bytes '$requiredFreeSpaceBytes'."
    }
    foreach ($name in @('durable_file_flush_supported', 'same_volume_atomic_move_supported')) {
        $value = Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name $name -Label 'Artifact storage capability evidence'
        if ($value -isnot [bool] -or -not $value) {
            throw "Artifact storage capability evidence '$name' must be true."
        }
    }
    $capturedAtUtcText = [string](Get-MvpArtifactStoragePolicyProperty -Value $Evidence -Name 'captured_at_utc' -Label 'Artifact storage capability evidence')
    $capturedAtUtc = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
            $capturedAtUtcText,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$capturedAtUtc) -or
        $capturedAtUtc.Offset -ne [TimeSpan]::Zero) {
        throw "Artifact storage capability evidence captured_at_utc '$capturedAtUtcText' must be one UTC timestamp."
    }
    return $Evidence
}

function Resolve-MvpArtifactStoragePath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$NamespaceId,
        [AllowNull()]$PolicySnapshot
    )

    $snapshot = if ($null -eq $PolicySnapshot) { Get-MvpArtifactStoragePolicySnapshot } else { $PolicySnapshot }
    $namespace = Get-MvpArtifactStorageNamespace -PolicySnapshot $snapshot -NamespaceId $NamespaceId
    $rootResolution = Resolve-MvpArtifactStorageRootPath `
        -Path $Path `
        -CapabilityClass $namespace.capability_class `
        -PolicySnapshot $snapshot
    $matchedRoot = @($snapshot.roots | Where-Object { $_.root_id -ceq $rootResolution.root_id })[0]
    $displayPath = $rootResolution.display_path
    $rootPrefix = $matchedRoot.display_path.TrimEnd('\') + '\'
    $relativePath = $displayPath.Substring($rootPrefix.Length)
    $leaf = @($relativePath.Split([char]92, 2))[0]
    if (-not $leaf.StartsWith($namespace.leaf_prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Artifact path '$displayPath' is outside the registered '$NamespaceId' namespace."
    }
    $instanceId = $leaf.Substring($namespace.leaf_prefix.Length)
    if ($instanceId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
        throw "Artifact path '$displayPath' does not contain one valid '$NamespaceId' instance leaf."
    }
    return [pscustomobject][ordered]@{
        operation_path = $rootResolution.operation_path
        display_path = $displayPath
        root_id = $matchedRoot.root_id
        capability_class = $matchedRoot.capability_class
        namespace_id = $namespace.namespace_id
    }
}

Export-ModuleMember -Function @(
    'Get-MvpArtifactStoragePolicySnapshot',
    'Get-MvpArtifactStorageDefaultRootPath',
    'Get-MvpArtifactStorageCapabilityEvidence',
    'Assert-MvpArtifactStorageCapabilityEvidence',
    'New-MvpArtifactStoragePath',
    'Resolve-MvpArtifactStorageRootPath',
    'Resolve-MvpArtifactStoragePath'
)
