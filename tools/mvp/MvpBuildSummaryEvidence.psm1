Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop
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

function Get-MvpBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($Bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpBuildGateContract {
    param([Parameter(Mandatory)][ValidateSet('profile-contract', 'workspace')][string]$SummaryKind)

    if ($SummaryKind -eq 'profile-contract') {
        return @(
            [pscustomobject]@{ gate_id = 'zircon-app-target-server'; command = 'cargo check -p zircon_app --no-default-features --features target-server --locked' },
            [pscustomobject]@{ gate_id = 'zircon-app-target-client-platform'; command = 'cargo check -p zircon_app --bin zircon_runtime --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked' },
            [pscustomobject]@{ gate_id = 'zircon-app-target-editor-host'; command = 'cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked' },
            [pscustomobject]@{ gate_id = 'zircon-app-target-client-shader-pbr-viewer'; command = 'cargo check -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked' },
            [pscustomobject]@{ gate_id = 'zircon-runtime-target-client'; command = 'cargo check -p zircon_runtime --no-default-features --features target-client --locked' },
            [pscustomobject]@{ gate_id = 'zircon-runtime-target-editor-host'; command = 'cargo check -p zircon_runtime --no-default-features --features target-editor-host --locked' },
            [pscustomobject]@{ gate_id = 'zircon-runtime-target-server'; command = 'cargo check -p zircon_runtime --no-default-features --features target-server --locked' }
        )
    }
    return @(
        [pscustomobject]@{ gate_id = 'workspace-build'; command = 'cargo build --workspace --locked' },
        [pscustomobject]@{ gate_id = 'workspace-test'; command = 'cargo test --workspace --locked' }
    )
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
    $summaryBytes = [IO.File]::ReadAllBytes($resolvedPath)
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

    $schemaVersion = Get-MvpBuildSummaryProperty -Value $summary -Name 'schema_version' -Label "F5 $ExpectedKind build summary"
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 1) {
        throw "F5 $ExpectedKind build summary schema_version must be the JSON integer 1; found '$schemaVersion'."
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

    $requiredContracts = @(Get-MvpBuildGateContract -SummaryKind $ExpectedKind)
    $gates = @(Get-MvpBuildSummaryProperty -Value $summary -Name 'gates' -Label "F5 $ExpectedKind build summary")
    if ($gates.Count -ne $requiredContracts.Count) {
        throw "F5 $ExpectedKind build summary does not contain the exact required gate set; expected $($requiredContracts.Count), found $($gates.Count)."
    }
    $gateById = @{}
    foreach ($gate in $gates) {
        if ($null -eq $gate -or $gate -is [string] -or $gate -is [ValueType]) {
            throw "F5 $ExpectedKind build summary contains a malformed gate."
        }
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
        $evidenceBytes = [IO.File]::ReadAllBytes($resolvedEvidencePath)
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
            schema_version = 1
            summary_kind = $ExpectedKind
            status = 'passed'
            gate_count = $requiredContracts.Count
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
