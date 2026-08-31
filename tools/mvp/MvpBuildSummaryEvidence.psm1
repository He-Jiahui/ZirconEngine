Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:MvpBuildSummaryUpperHexDigits = [char[]]'0123456789ABCDEF'
$script:MvpBuildSummaryMaximumSummaryBytes = 1MB
$script:MvpBuildSummaryMaximumGateEvidenceBytes = 64MB
$script:MvpBuildSummaryReadBufferBytes = 81920

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpBuildGateRegistry.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1') -Force -DisableNameChecking -ErrorAction Stop

function Get-MvpBuildSummaryProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Assert-MvpBuildSummaryExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [array] -or $Value -is [string] -or $Value -is [ValueType]) {
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

function ConvertTo-MvpBuildSummaryUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpBuildSummaryUpperHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpBuildSummaryUpperHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpBuildSummaryUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Read-MvpBuildSummaryBoundedBytes {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateRange(1, [Int32]::MaxValue)][int]$MaximumBytes,
        [Parameter(Mandatory)][string]$Label
    )

    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt $MaximumBytes) {
            throw "$Label exceeds its byte budget of $MaximumBytes bytes: $Path"
        }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read(
                $bytes,
                $offset,
                [Math]::Min($script:MvpBuildSummaryReadBufferBytes, $bytes.Length - $offset))
            if ($read -eq 0) {
                throw "$Label changed while it was being read: $Path"
            }
            $offset += $read
        }
        if ($stream.ReadByte() -ne -1) {
            throw "$Label exceeds its byte budget of $MaximumBytes bytes: $Path"
        }
        return ,$bytes
    }
    finally {
        $stream.Dispose()
    }
}

function ConvertFrom-MvpBuildTimestamp {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [DateTimeOffset]) {
        return ([DateTimeOffset]$Value).ToUniversalTime()
    }
    if ($Value -is [DateTime]) {
        return [DateTimeOffset]([DateTime]$Value).ToUniversalTime()
    }
    $timestamp = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParseExact(
        [string]$Value,
        'o',
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$timestamp
    )) {
        throw "$Label has malformed '$Name' timestamp '$Value'."
    }
    return $timestamp.ToUniversalTime()
}

function Assert-MvpBuildSummaryOperationalChildPath {
    param(
        [Parameter(Mandatory)][string]$RootPath,
        [Parameter(Mandatory)][string]$CandidatePath,
        [Parameter(Mandatory)][string]$Label
    )

    $rootPrefix = $RootPath.TrimEnd('\') + '\'
    if (-not $CandidatePath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label '$CandidatePath' resolves outside build summary directory '$RootPath'."
    }
}

function Assert-MvpBuildGateRegistryReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$ExpectedSnapshot
    )

    Assert-MvpBuildSummaryExactProperties `
        -Value $Receipt `
        -ExpectedNames @('schema_version', 'registry_kind', 'sha256', 'size_bytes') `
        -Label 'F5 build summary gate registry receipt'
    Assert-MvpBuildSummaryExactProperties `
        -Value $ExpectedSnapshot `
        -ExpectedNames @('receipt', 'summaries') `
        -Label 'Current MVP build gate registry snapshot'
    $expected = $ExpectedSnapshot.receipt
    $schemaVersion = Get-MvpBuildSummaryProperty `
        -Value $Receipt `
        -Name 'schema_version' `
        -Label 'F5 build summary gate registry receipt'
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 1) {
        throw "F5 build summary gate registry receipt schema_version must be the JSON integer 1; found '$schemaVersion'."
    }
    $registryKind = [string](Get-MvpBuildSummaryProperty `
        -Value $Receipt `
        -Name 'registry_kind' `
        -Label 'F5 build summary gate registry receipt')
    if (-not $registryKind.Equals([string]$expected.registry_kind, [StringComparison]::Ordinal)) {
        throw "F5 build summary gate registry receipt registry_kind differs from the current registry snapshot."
    }
    $sha256 = [string](Get-MvpBuildSummaryProperty `
        -Value $Receipt `
        -Name 'sha256' `
        -Label 'F5 build summary gate registry receipt')
    if ($sha256 -notmatch '^[0-9A-F]{64}$' -or
        -not $sha256.Equals([string]$expected.sha256, [StringComparison]::Ordinal)) {
        throw 'F5 build summary gate registry receipt sha256 differs from the current registry snapshot.'
    }
    $sizeBytes = Get-MvpBuildSummaryProperty `
        -Value $Receipt `
        -Name 'size_bytes' `
        -Label 'F5 build summary gate registry receipt'
    if (-not ($sizeBytes -is [int] -or $sizeBytes -is [long]) -or
        [Int64]$sizeBytes -ne [Int64]$expected.size_bytes) {
        throw 'F5 build summary gate registry receipt size_bytes differs from the current registry snapshot.'
    }
}

function Assert-MvpBuildSummaryEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateSet('profile-contract', 'workspace')][string]$ExpectedKind,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
    )

    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "F5 $ExpectedKind build summary '$Path' does not exist or is not a file."
    }
    $summaryBytes = Read-MvpBuildSummaryBoundedBytes `
        -Path $resolvedPath `
        -MaximumBytes $script:MvpBuildSummaryMaximumSummaryBytes `
        -Label "F5 $ExpectedKind build summary"
    try {
        $summaryText = [Text.UTF8Encoding]::new($false, $true).GetString($summaryBytes)
        $summary = $summaryText | ConvertFrom-Json
    }
    catch {
        throw "F5 $ExpectedKind build summary '$resolvedPath' is not valid UTF-8 JSON: $($_.Exception.Message)"
    }
    if ($null -eq $summary -or $summary -is [array]) {
        throw "F5 $ExpectedKind build summary must contain one JSON object."
    }
    $gateRegistrySnapshot = Get-MvpBuildGateRegistrySnapshot
    Assert-MvpBuildSummaryExactProperties `
        -Value $summary `
        -ExpectedNames @('schema_version', 'summary_kind', 'source_fingerprint', 'status', 'gate_registry', 'gates') `
        -Label "F5 $ExpectedKind build summary"

    $schemaVersion = Get-MvpBuildSummaryProperty -Value $summary -Name 'schema_version' -Label "F5 $ExpectedKind build summary"
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 2) {
        throw "F5 $ExpectedKind build summary schema_version must be the JSON integer 2; found '$schemaVersion'."
    }
    $summaryKind = [string](Get-MvpBuildSummaryProperty -Value $summary -Name 'summary_kind' -Label "F5 $ExpectedKind build summary")
    if (-not $summaryKind.Equals($ExpectedKind, [StringComparison]::Ordinal)) {
        throw "F5 build summary kind '$summaryKind' differs from expected '$ExpectedKind'."
    }
    $sourceFingerprint = [string](Get-MvpBuildSummaryProperty -Value $summary -Name 'source_fingerprint' -Label "F5 $ExpectedKind build summary")
    if ([string]::IsNullOrWhiteSpace($sourceFingerprint) -or
        -not $sourceFingerprint.Equals($ExpectedSourceFingerprint, [StringComparison]::Ordinal)) {
        throw "F5 $ExpectedKind build summary source_fingerprint '$sourceFingerprint' differs from staging source fingerprint '$ExpectedSourceFingerprint'."
    }
    $status = [string](Get-MvpBuildSummaryProperty -Value $summary -Name 'status' -Label "F5 $ExpectedKind build summary")
    if (-not $status.Equals('passed', [StringComparison]::Ordinal)) {
        throw "F5 $ExpectedKind build summary status must be 'passed'; found '$status'."
    }
    Assert-MvpBuildGateRegistryReceipt `
        -Receipt (Get-MvpBuildSummaryProperty -Value $summary -Name 'gate_registry' -Label "F5 $ExpectedKind build summary") `
        -ExpectedSnapshot $gateRegistrySnapshot

    $requiredContracts = @(Get-MvpBuildGateContract -SummaryKind $ExpectedKind -RegistrySnapshot $gateRegistrySnapshot)
    $gates = @(Get-MvpBuildSummaryProperty -Value $summary -Name 'gates' -Label "F5 $ExpectedKind build summary")
    if ($gates.Count -ne $requiredContracts.Count) {
        throw "F5 $ExpectedKind build summary does not contain the exact required gate set; expected $($requiredContracts.Count), found $($gates.Count)."
    }
    $gateById = @{}
    foreach ($gate in $gates) {
        if ($null -eq $gate -or $gate -is [string] -or $gate -is [ValueType]) {
            throw "F5 $ExpectedKind build summary contains a malformed gate."
        }
        Assert-MvpBuildSummaryExactProperties `
            -Value $gate `
            -ExpectedNames @('gate_id', 'command', 'status', 'started_at_utc', 'ended_at_utc', 'exit_code', 'evidence') `
            -Label "F5 $ExpectedKind build summary gate"
        $gateId = [string](Get-MvpBuildSummaryProperty -Value $gate -Name 'gate_id' -Label "F5 $ExpectedKind build summary gate")
        if ([string]::IsNullOrWhiteSpace($gateId) -or $gateById.ContainsKey($gateId)) {
            throw "F5 $ExpectedKind build summary contains an empty or duplicate gate_id '$gateId'."
        }
        $gateById[$gateId] = $gate
    }

    $summaryDirectory = [IO.Path]::GetDirectoryName($resolvedPath)
    if ([string]::IsNullOrWhiteSpace($summaryDirectory)) {
        throw "F5 $ExpectedKind build summary '$resolvedPath' does not have a parent directory."
    }
    $gateArtifacts = [System.Collections.Generic.List[object]]::new()
    foreach ($contract in $requiredContracts) {
        $gateId = [string]$contract.gate_id
        if (-not $gateById.ContainsKey($gateId)) {
            throw "F5 $ExpectedKind build summary does not contain the exact required gate set; missing '$gateId'."
        }
        $gate = $gateById[$gateId]
        $command = [string](Get-MvpBuildSummaryProperty -Value $gate -Name 'command' -Label "F5 $ExpectedKind gate '$gateId'")
        if (-not $command.Equals([string]$contract.command, [StringComparison]::Ordinal)) {
            throw "F5 $ExpectedKind gate '$gateId' command differs from the canonical contract."
        }
        $gateStatus = [string](Get-MvpBuildSummaryProperty -Value $gate -Name 'status' -Label "F5 $ExpectedKind gate '$gateId'")
        if (-not $gateStatus.Equals('passed', [StringComparison]::Ordinal)) {
            throw "F5 $ExpectedKind gate '$gateId' status must be 'passed'; found '$gateStatus'."
        }
        $exitCode = Get-MvpBuildSummaryProperty -Value $gate -Name 'exit_code' -Label "F5 $ExpectedKind gate '$gateId'"
        if (-not ($exitCode -is [int] -or $exitCode -is [long]) -or [long]$exitCode -ne 0) {
            throw "F5 $ExpectedKind gate '$gateId' exit_code must be the JSON integer 0; found '$exitCode'."
        }
        $startedAt = ConvertFrom-MvpBuildTimestamp `
            -Value (Get-MvpBuildSummaryProperty -Value $gate -Name 'started_at_utc' -Label "F5 $ExpectedKind gate '$gateId'") `
            -Name 'started_at_utc' `
            -Label "F5 $ExpectedKind gate '$gateId'"
        $endedAt = ConvertFrom-MvpBuildTimestamp `
            -Value (Get-MvpBuildSummaryProperty -Value $gate -Name 'ended_at_utc' -Label "F5 $ExpectedKind gate '$gateId'") `
            -Name 'ended_at_utc' `
            -Label "F5 $ExpectedKind gate '$gateId'"
        if ($endedAt -lt $startedAt) {
            throw "F5 $ExpectedKind gate '$gateId' ended before it started."
        }

        $evidence = Get-MvpBuildSummaryProperty -Value $gate -Name 'evidence' -Label "F5 $ExpectedKind gate '$gateId'"
        Assert-MvpBuildSummaryExactProperties `
            -Value $evidence `
            -ExpectedNames @('path', 'sha256', 'size_bytes') `
            -Label "F5 $ExpectedKind gate '$gateId' evidence"
        $relativeEvidencePath = [string](Get-MvpBuildSummaryProperty -Value $evidence -Name 'path' -Label "F5 $ExpectedKind gate '$gateId' evidence")
        $expectedEvidencePath = "logs/$gateId.log"
        if (-not $relativeEvidencePath.Equals($expectedEvidencePath, [StringComparison]::Ordinal)) {
            throw "F5 $ExpectedKind gate '$gateId' evidence path must be '$expectedEvidencePath'; found '$relativeEvidencePath'."
        }
        $sourceEvidencePath = Join-ZirconWindowsPath -Path $summaryDirectory -ChildPath $relativeEvidencePath.Replace('/', '\')
        $resolvedEvidencePath = (Resolve-ZirconWindowsPath -Path $sourceEvidencePath).OperationalPath
        Assert-MvpBuildSummaryOperationalChildPath `
            -RootPath $summaryDirectory `
            -CandidatePath $resolvedEvidencePath `
            -Label "F5 $ExpectedKind gate '$gateId' evidence"
        if (-not [IO.File]::Exists($resolvedEvidencePath)) {
            throw "F5 $ExpectedKind gate '$gateId' evidence '$sourceEvidencePath' does not exist."
        }
        $evidenceBytes = Read-MvpBuildSummaryBoundedBytes `
            -Path $resolvedEvidencePath `
            -MaximumBytes $script:MvpBuildSummaryMaximumGateEvidenceBytes `
            -Label "F5 $ExpectedKind gate '$gateId' evidence"
        $evidenceHash = Get-MvpBytesSha256 -Bytes $evidenceBytes
        $declaredHash = [string](Get-MvpBuildSummaryProperty -Value $evidence -Name 'sha256' -Label "F5 $ExpectedKind gate '$gateId' evidence")
        if (-not $evidenceHash.Equals($declaredHash, [StringComparison]::OrdinalIgnoreCase)) {
            throw "F5 $ExpectedKind gate '$gateId' evidence hash mismatch."
        }
        $declaredSize = Get-MvpBuildSummaryProperty -Value $evidence -Name 'size_bytes' -Label "F5 $ExpectedKind gate '$gateId' evidence"
        if (-not ($declaredSize -is [int] -or $declaredSize -is [long]) -or [long]$declaredSize -ne $evidenceBytes.LongLength) {
            throw "F5 $ExpectedKind gate '$gateId' evidence size mismatch."
        }
        $gateArtifacts.Add([pscustomobject]@{
            relative_path = "build/$relativeEvidencePath"
            content_bytes = $evidenceBytes
            sha256 = $evidenceHash
            size_bytes = [Int64]$evidenceBytes.LongLength
        }) | Out-Null
    }

    $summaryRelativePath = switch ($ExpectedKind) {
        'profile-contract' { 'build/profile-contract-summary.json' }
        'workspace' { 'build/workspace-summary.json' }
    }
    $summaryHash = Get-MvpBytesSha256 -Bytes $summaryBytes
    return [pscustomobject]@{
        relative_path = $summaryRelativePath
        content_bytes = $summaryBytes
        sha256 = $summaryHash
        size_bytes = [Int64]$summaryBytes.LongLength
        gate_artifacts = $gateArtifacts.ToArray()
        manifest_evidence = [ordered]@{
            path = $summaryRelativePath
            sha256 = $summaryHash
            size_bytes = [Int64]$summaryBytes.LongLength
            schema_version = 2
            summary_kind = $ExpectedKind
            status = 'passed'
            gate_count = $requiredContracts.Count
            gate_registry = $gateRegistrySnapshot.receipt
        }
    }
}

function Write-MvpValidatedArtifact {
    param(
        [Parameter(Mandatory)]$Artifact,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$CompatibleWriteLeaseRoot
    )

    $relativePath = ([string]$Artifact.relative_path).Replace('/', '\')
    if ([IO.Path]::IsPathRooted($relativePath) -or $relativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "Published F5 build artifact '$($Artifact.relative_path)' has an unsafe relative path."
    }
    $destinationPath = Join-Path $EvidenceRoot $relativePath
    $relativeDirectory = Split-Path -Parent $relativePath
    if (-not [string]::IsNullOrWhiteSpace($relativeDirectory)) {
        Ensure-MvpAcceptanceDirectoryPathNoFollow `
            -RootPath $EvidenceRoot `
            -RelativePath $relativeDirectory `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot | Out-Null
    }

    $writtenBytes = Write-MvpAcceptanceNewFileNoFollow `
        -Path $destinationPath `
        -ContentBytes ([byte[]]$Artifact.content_bytes) `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
    if ($writtenBytes.LongLength -ne [Int64]$Artifact.size_bytes -or
        -not (Get-MvpBytesSha256 -Bytes $writtenBytes).Equals([string]$Artifact.sha256, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Published F5 build artifact '$($Artifact.relative_path)' differs from its validated bytes."
    }
}

function Copy-MvpBuildSummaryEvidence {
    param(
        [Parameter(Mandatory)]$Summary,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$CompatibleWriteLeaseRoot
    )

    Write-MvpValidatedArtifact `
        -Artifact $Summary `
        -EvidenceRoot $EvidenceRoot `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
    foreach ($artifact in @($Summary.gate_artifacts)) {
        Write-MvpValidatedArtifact `
            -Artifact $artifact `
            -EvidenceRoot $EvidenceRoot `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
    }
}

Export-ModuleMember -Function Assert-MvpBuildSummaryEvidence, Copy-MvpBuildSummaryEvidence
