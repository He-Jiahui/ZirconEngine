$ErrorActionPreference = 'Stop'

$processJournalModule = Join-Path $PSScriptRoot 'MvpProcessLifecycleJournal.psm1'
Import-Module $processJournalModule -ErrorAction Stop
$qualificationContextModule = Join-Path $PSScriptRoot 'MvpProcessQualificationContext.psm1'
Import-Module $qualificationContextModule -ErrorAction Stop
$artifactStoragePolicyModule = Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1'
Import-Module $artifactStoragePolicyModule -ErrorAction Stop

$script:MvpStagingTerminalReceiptSchemaVersion = 3
$script:MvpStagingTerminalReceiptKind = 'zircon_mvp_staging_terminal'
$script:MvpStagingTerminalReceiptDirectoryName = '.mvp-staging-receipts'
$script:MvpStagingTerminalReceiptMaximumBytes = 16384

function Get-MvpStagingTerminalReceiptMessageSha256 {
    param([Parameter(Mandatory)][string]$Message)

    return Get-MvpProcessJournalSha256 -Bytes ([Text.Encoding]::UTF8.GetBytes($Message))
}

function ConvertTo-MvpStagingTerminalReceiptTime {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        return [DateTimeOffset]::Parse(
            $Value,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind)
    }
    catch {
        throw "MVP staging terminal receipt $Label '$Value' is not a round-trip timestamp."
    }
}

function Get-MvpStagingTerminalReceiptPath {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RunId
    )

    $root = [IO.Path]::GetFullPath($StagingRoot)
    $receiptRoot = [IO.Path]::GetFullPath(
        [IO.Path]::Combine($root, $script:MvpStagingTerminalReceiptDirectoryName))
    return [IO.Path]::GetFullPath([IO.Path]::Combine($receiptRoot, "$RunId.json"))
}

function Write-MvpStagingTerminalReceipt {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RunId,
        [Parameter(Mandatory)][ValidateSet('succeeded', 'failed', 'timed_out', 'cancelled')][string]$Outcome,
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,63}$')][string]$Phase,
        [Parameter(Mandatory)][string]$StartedAtUtc,
        [string]$EndedAtUtc = ([DateTimeOffset]::UtcNow.ToString('o')),
        [Parameter(Mandatory)][bool]$StagingDirectoryPublished,
        [Parameter(Mandatory)][ValidateSet('succeeded', 'failed', 'not_required')][string]$CleanupOutcome,
        [AllowNull()][string]$CleanupMessage,
        [AllowNull()][string]$FailureKind,
        [AllowNull()][string]$FailureMessage,
        [AllowNull()]$QualificationContextSetReceipt,
        [AllowNull()]$StorageCapabilityEvidence,
        [Nullable[Int64]]$RequiredFreeSpaceBytes,
        [AllowNull()][string]$StagingManifestSha256
    )

    $requiredFreeSpaceBytesWasBound = $PSBoundParameters.ContainsKey('RequiredFreeSpaceBytes')
    $startedAt = ConvertTo-MvpStagingTerminalReceiptTime -Value $StartedAtUtc -Label 'start time'
    $endedAt = ConvertTo-MvpStagingTerminalReceiptTime -Value $EndedAtUtc -Label 'end time'
    if ($endedAt -lt $startedAt) {
        throw 'MVP staging terminal receipt end time precedes its start time.'
    }
    if ($CleanupOutcome -eq 'failed') {
        if ([string]::IsNullOrWhiteSpace($CleanupMessage)) {
            throw 'A failed MVP staging cleanup requires a failure message.'
        }
    }
    elseif (-not [string]::IsNullOrEmpty($CleanupMessage)) {
        throw "MVP staging cleanup outcome '$CleanupOutcome' cannot carry a failure message."
    }
    if ($Outcome -eq 'succeeded') {
        if (-not $StagingDirectoryPublished) {
            throw 'A successful MVP staging terminal receipt requires a published staging directory.'
        }
        if (-not [string]::IsNullOrEmpty($FailureKind) -or -not [string]::IsNullOrEmpty($FailureMessage)) {
            throw 'A successful MVP staging terminal receipt cannot carry failure evidence.'
        }
    }
    else {
        if ($FailureKind -notmatch '^[a-z0-9][a-z0-9._-]{0,63}$' -or [string]::IsNullOrWhiteSpace($FailureMessage)) {
            throw "MVP staging outcome '$Outcome' requires bounded failure identity and message evidence."
        }
    }
    if ($StagingDirectoryPublished) {
        if ($StagingManifestSha256 -notmatch '^[0-9a-f]{64}$') {
            throw 'A published MVP staging directory requires a lower-case staging manifest SHA-256.'
        }
    }
    elseif (-not [string]::IsNullOrEmpty($StagingManifestSha256)) {
        throw 'An unpublished MVP staging directory cannot carry a staging manifest SHA-256.'
    }
    $validatedQualificationContextSetReceipt = $null
    if ($null -ne $QualificationContextSetReceipt) {
        $validatedQualificationContextSetReceipt = Assert-MvpProcessQualificationContextSetReceipt `
            -Receipt $QualificationContextSetReceipt `
            -ExpectedRunId $RunId
    }
    elseif ($StagingDirectoryPublished) {
        throw 'A published MVP staging terminal receipt requires a process qualification context-set receipt.'
    }
    $validatedStorageCapabilityEvidence = $null
    if ($null -ne $StorageCapabilityEvidence) {
        if (-not $requiredFreeSpaceBytesWasBound -or $null -eq $RequiredFreeSpaceBytes) {
            throw 'MVP staging terminal receipt storage capability evidence requires a free-space byte budget.'
        }
        $validatedStorageCapabilityEvidence = Assert-MvpArtifactStorageCapabilityEvidence `
            -Evidence $StorageCapabilityEvidence `
            -ExpectedPath $StagingRoot `
            -ExpectedRequiredFreeSpaceBytes ([Int64]$RequiredFreeSpaceBytes)
    }
    elseif ($requiredFreeSpaceBytesWasBound -and $null -ne $RequiredFreeSpaceBytes) {
        throw 'MVP staging terminal receipt cannot carry a free-space byte budget without storage capability evidence.'
    }
    elseif ($StagingDirectoryPublished) {
        throw 'A published MVP staging terminal receipt requires storage capability evidence.'
    }

    $path = Get-MvpStagingTerminalReceiptPath -StagingRoot $StagingRoot -RunId $RunId
    $receiptRoot = [IO.Path]::GetDirectoryName($path)
    [IO.Directory]::CreateDirectory($receiptRoot) | Out-Null
    if ([IO.File]::Exists($path)) {
        throw "Refusing to overwrite existing MVP staging terminal receipt: $path"
    }

    $receipt = [ordered]@{
        schema_version = $script:MvpStagingTerminalReceiptSchemaVersion
        receipt_kind = $script:MvpStagingTerminalReceiptKind
        run_id = $RunId
        outcome = $Outcome
        phase = $Phase
        started_at_utc = $startedAt.ToUniversalTime().ToString('o')
        ended_at_utc = $endedAt.ToUniversalTime().ToString('o')
        staging_directory_published = $StagingDirectoryPublished
        qualification_context_set = $validatedQualificationContextSetReceipt
        storage_capability = $validatedStorageCapabilityEvidence
        cleanup = [ordered]@{
            outcome = $CleanupOutcome
        }
    }
    if ($CleanupOutcome -eq 'failed') {
        $receipt.cleanup['message_sha256'] = Get-MvpStagingTerminalReceiptMessageSha256 -Message $CleanupMessage
    }
    if ($StagingDirectoryPublished) {
        $receipt['staging_manifest_sha256'] = $StagingManifestSha256
    }
    if ($Outcome -ne 'succeeded') {
        $receipt['failure'] = [ordered]@{
            kind = $FailureKind
            message_sha256 = Get-MvpStagingTerminalReceiptMessageSha256 -Message $FailureMessage
        }
    }
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        (($receipt | ConvertTo-Json -Depth 8 -Compress) + [Environment]::NewLine))
    if ($bytes.Length -gt $script:MvpStagingTerminalReceiptMaximumBytes) {
        throw "MVP staging terminal receipt exceeds its $($script:MvpStagingTerminalReceiptMaximumBytes)-byte budget."
    }

    $temporaryPath = [IO.Path]::GetFullPath(
        [IO.Path]::Combine($receiptRoot, ".$RunId.pending-$([guid]::NewGuid().ToString('N')).tmp"))
    try {
        $stream = [IO.File]::Open(
            $temporaryPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None)
        try {
            $stream.Write($bytes, 0, $bytes.Length)
            $stream.Flush($true)
        }
        finally {
            $stream.Dispose()
        }
        if ([IO.File]::Exists($path)) {
            throw "Refusing to overwrite existing MVP staging terminal receipt: $path"
        }
        [IO.File]::Move($temporaryPath, $path)
    }
    finally {
        if ([IO.File]::Exists($temporaryPath)) {
            [IO.File]::Delete($temporaryPath)
        }
    }
    return [pscustomobject]@{
        path = $path
        bytes = [Int64]$bytes.Length
        sha256 = Get-MvpProcessJournalSha256 -Bytes $bytes
    }
}

Export-ModuleMember -Function @(
    'Get-MvpStagingTerminalReceiptPath',
    'Write-MvpStagingTerminalReceipt'
)
