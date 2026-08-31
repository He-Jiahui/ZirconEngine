Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpAcceptanceSnapshotAdmissionReceiptKind = 'zircon.mvp-acceptance-snapshot-admission'
$script:MvpAcceptanceSnapshotMaximumManifestBytes = [Int64]67108864
$script:MvpAcceptanceSnapshotMaximumEntryCount = 100000
$script:MvpAcceptanceSnapshotMaximumTotalFileBytes = [Int64]17179869184
$script:MvpAcceptanceSnapshotMaximumDepth = 64
$script:MvpAcceptanceSnapshotMaximumDurationSeconds = 600

function Assert-MvpAcceptanceSnapshotAdmissionExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value) {
        throw "$Label is missing."
    }
    $actualNames = @($Value.PSObject.Properties.Name)
    foreach ($actualName in $actualNames) {
        if ($actualName -notin $ExpectedNames) {
            throw "$Label contains unknown property '$actualName'."
        }
    }
    foreach ($expectedName in $ExpectedNames) {
        if ($expectedName -notin $actualNames) {
            throw "$Label is missing property '$expectedName'."
        }
    }
}

function Get-MvpAcceptanceSnapshotAdmissionDepth {
    param([Parameter(Mandatory)][string]$RelativePath)

    $depth = 1
    for ($index = 0; $index -lt $RelativePath.Length; $index++) {
        if ($RelativePath[$index] -eq '/') {
            $depth++
        }
    }
    return $depth
}

function ConvertTo-MvpAcceptanceSnapshotAdmissionTimestamp {
    param([Parameter(Mandatory)][DateTimeOffset]$Value)

    return $Value.ToUniversalTime().ToString('o', [Globalization.CultureInfo]::InvariantCulture)
}

function ConvertFrom-MvpAcceptanceSnapshotAdmissionTimestamp {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParseExact(
        $Value,
        'o',
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$parsed)) {
        throw "$Label must be an ISO-8601 round-trip timestamp."
    }
    return $parsed.ToUniversalTime()
}

function Get-MvpAcceptanceSnapshotAdmissionDefaultLimits {
    return [pscustomobject][ordered]@{
        maximum_manifest_bytes = $script:MvpAcceptanceSnapshotMaximumManifestBytes
        maximum_entry_count = $script:MvpAcceptanceSnapshotMaximumEntryCount
        maximum_total_file_bytes = $script:MvpAcceptanceSnapshotMaximumTotalFileBytes
        maximum_depth = $script:MvpAcceptanceSnapshotMaximumDepth
        maximum_duration_seconds = $script:MvpAcceptanceSnapshotMaximumDurationSeconds
    }
}

function New-MvpAcceptanceSnapshotAdmission {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Entries,
        [Parameter(Mandatory)][string]$RootPath,
        [ValidateRange(0, [Int64]::MaxValue)][Int64]$ManifestSizeBytes = 0,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumManifestBytes = $script:MvpAcceptanceSnapshotMaximumManifestBytes,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumEntryCount = $script:MvpAcceptanceSnapshotMaximumEntryCount,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumTotalFileBytes = $script:MvpAcceptanceSnapshotMaximumTotalFileBytes,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumDepth = $script:MvpAcceptanceSnapshotMaximumDepth,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumDurationSeconds = $script:MvpAcceptanceSnapshotMaximumDurationSeconds,
        [DateTimeOffset]$StartedAtUtc = [DateTimeOffset]::UtcNow
    )

    $absoluteRoot = [IO.Path]::GetFullPath($RootPath)
    $sourceRootName = [IO.Path]::GetFileName($absoluteRoot.TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar))
    if ([string]::IsNullOrWhiteSpace($sourceRootName)) {
        throw 'Acceptance snapshot admission requires a named source root.'
    }
    if ($ManifestSizeBytes -gt $MaximumManifestBytes) {
        throw "Acceptance snapshot manifest exceeds the manifest-byte budget of $MaximumManifestBytes."
    }

    $startedAt = $StartedAtUtc.ToUniversalTime()
    $deadline = $startedAt.AddSeconds($MaximumDurationSeconds)
    if ([DateTimeOffset]::UtcNow -gt $deadline) {
        throw 'Acceptance snapshot admission deadline was exceeded before manifest accounting completed.'
    }

    $entryCount = 0
    $fileCount = 0
    $directoryCount = 0
    [Int64]$totalFileBytes = 0
    $maximumObservedDepth = 0
    foreach ($entry in $Entries) {
        $entryCount++
        if ($entryCount -gt $MaximumEntryCount) {
            throw "Acceptance snapshot manifest exceeds the entry-count budget of $MaximumEntryCount."
        }
        if ($null -eq $entry -or
            $null -eq $entry.PSObject.Properties['relative_path'] -or
            [string]::IsNullOrWhiteSpace([string]$entry.relative_path) -or
            $null -eq $entry.PSObject.Properties['kind']) {
            throw 'Acceptance snapshot admission received an incomplete manifest entry.'
        }

        $relativePath = [string]$entry.relative_path
        $depth = Get-MvpAcceptanceSnapshotAdmissionDepth -RelativePath $relativePath
        if ($depth -gt $MaximumDepth) {
            throw "Acceptance snapshot manifest exceeds the depth budget of $MaximumDepth."
        }
        if ($depth -gt $maximumObservedDepth) {
            $maximumObservedDepth = $depth
        }

        switch ([string]$entry.kind) {
            'directory' {
                $directoryCount++
            }
            'file' {
                if ($null -eq $entry.PSObject.Properties['size_bytes']) {
                    throw 'Acceptance snapshot admission received a file without size evidence.'
                }
                try {
                    [Int64]$entrySize = $entry.size_bytes
                }
                catch {
                    throw 'Acceptance snapshot admission received a file with invalid size evidence.'
                }
                if ($entrySize -lt 0) {
                    throw 'Acceptance snapshot admission received a file with negative size evidence.'
                }
                if ($entrySize -gt ($MaximumTotalFileBytes - $totalFileBytes)) {
                    throw "Acceptance snapshot manifest exceeds the file-byte budget of $MaximumTotalFileBytes."
                }
                $totalFileBytes += $entrySize
                $fileCount++
            }
            default {
                throw "Acceptance snapshot admission received unsupported entry kind '$($entry.kind)'."
            }
        }

        if (($entryCount % 256) -eq 0 -and [DateTimeOffset]::UtcNow -gt $deadline) {
            throw 'Acceptance snapshot admission deadline was exceeded during manifest accounting.'
        }
    }

    if ([DateTimeOffset]::UtcNow -gt $deadline) {
        throw 'Acceptance snapshot admission deadline was exceeded during manifest accounting.'
    }

    return [pscustomobject][ordered]@{
        schema_version = 1
        receipt_kind = $script:MvpAcceptanceSnapshotAdmissionReceiptKind
        source_root_name = $sourceRootName
        started_at_utc = ConvertTo-MvpAcceptanceSnapshotAdmissionTimestamp -Value $startedAt
        deadline_utc = ConvertTo-MvpAcceptanceSnapshotAdmissionTimestamp -Value $deadline
        limits = [pscustomobject][ordered]@{
            maximum_manifest_bytes = $MaximumManifestBytes
            maximum_entry_count = $MaximumEntryCount
            maximum_total_file_bytes = $MaximumTotalFileBytes
            maximum_depth = $MaximumDepth
            maximum_duration_seconds = $MaximumDurationSeconds
        }
        observed = [pscustomobject][ordered]@{
            manifest_size_bytes = $ManifestSizeBytes
            entry_count = $entryCount
            file_count = $fileCount
            directory_count = $directoryCount
            total_file_bytes = $totalFileBytes
            maximum_depth = $maximumObservedDepth
        }
    }
}

function Assert-MvpAcceptanceSnapshotAdmissionActive {
    param(
        [Parameter(Mandatory)]$Admission,
        [Parameter(Mandatory)][ValidatePattern('^[a-z][a-z0-9-]{0,63}$')][string]$Phase,
        [DateTimeOffset]$NowUtc = [DateTimeOffset]::UtcNow
    )

    Assert-MvpAcceptanceSnapshotAdmissionExactProperties `
        -Value $Admission `
        -ExpectedNames @(
            'schema_version',
            'receipt_kind',
            'source_root_name',
            'started_at_utc',
            'deadline_utc',
            'limits',
            'observed') `
        -Label 'Acceptance snapshot admission receipt'
    Assert-MvpAcceptanceSnapshotAdmissionExactProperties `
        -Value $Admission.limits `
        -ExpectedNames @(
            'maximum_manifest_bytes',
            'maximum_entry_count',
            'maximum_total_file_bytes',
            'maximum_depth',
            'maximum_duration_seconds') `
        -Label 'Acceptance snapshot admission limits'
    Assert-MvpAcceptanceSnapshotAdmissionExactProperties `
        -Value $Admission.observed `
        -ExpectedNames @(
            'manifest_size_bytes',
            'entry_count',
            'file_count',
            'directory_count',
            'total_file_bytes',
            'maximum_depth') `
        -Label 'Acceptance snapshot admission observations'

    if ($Admission.schema_version -isnot [int] -or $Admission.schema_version -ne 1) {
        throw 'Acceptance snapshot admission schema_version must be the integer 1.'
    }
    if ([string]$Admission.receipt_kind -ne $script:MvpAcceptanceSnapshotAdmissionReceiptKind) {
        throw 'Acceptance snapshot admission receipt_kind is unsupported.'
    }
    $sourceRootName = [string]$Admission.source_root_name
    if ([string]::IsNullOrWhiteSpace($sourceRootName) -or
        [IO.Path]::GetFileName($sourceRootName) -ne $sourceRootName) {
        throw 'Acceptance snapshot admission source_root_name is invalid.'
    }

    $startedAt = ConvertFrom-MvpAcceptanceSnapshotAdmissionTimestamp `
        -Value ([string]$Admission.started_at_utc) `
        -Label 'Acceptance snapshot admission started_at_utc'
    $deadline = ConvertFrom-MvpAcceptanceSnapshotAdmissionTimestamp `
        -Value ([string]$Admission.deadline_utc) `
        -Label 'Acceptance snapshot admission deadline_utc'
    try {
        [Int64]$maximumManifestBytes = $Admission.limits.maximum_manifest_bytes
        [int]$maximumEntryCount = $Admission.limits.maximum_entry_count
        [Int64]$maximumTotalFileBytes = $Admission.limits.maximum_total_file_bytes
        [int]$maximumDepth = $Admission.limits.maximum_depth
        [int]$maximumDurationSeconds = $Admission.limits.maximum_duration_seconds
        [Int64]$manifestSizeBytes = $Admission.observed.manifest_size_bytes
        [Int64]$entryCount = $Admission.observed.entry_count
        [Int64]$fileCount = $Admission.observed.file_count
        [Int64]$directoryCount = $Admission.observed.directory_count
        [Int64]$totalFileBytes = $Admission.observed.total_file_bytes
        [int]$observedDepth = $Admission.observed.maximum_depth
    }
    catch {
        throw 'Acceptance snapshot admission receipt contains invalid integer evidence.'
    }
    if ($maximumManifestBytes -le 0 -or $maximumEntryCount -le 0 -or
        $maximumTotalFileBytes -le 0 -or
        $maximumDepth -le 0 -or $maximumDurationSeconds -le 0) {
        throw 'Acceptance snapshot admission receipt contains a non-positive limit.'
    }
    if ($manifestSizeBytes -lt 0 -or $entryCount -lt 0 -or
        $fileCount -lt 0 -or $directoryCount -lt 0 -or
        $totalFileBytes -lt 0 -or $observedDepth -lt 0 -or
        $fileCount + $directoryCount -ne $entryCount) {
        throw 'Acceptance snapshot admission receipt contains invalid observations.'
    }
    if ($manifestSizeBytes -gt $maximumManifestBytes -or
        $entryCount -gt $maximumEntryCount -or
        $totalFileBytes -gt $maximumTotalFileBytes -or
        $observedDepth -gt $maximumDepth) {
        throw 'Acceptance snapshot admission receipt exceeds its recorded limits.'
    }
    if ($deadline -ne $startedAt.AddSeconds($maximumDurationSeconds)) {
        throw 'Acceptance snapshot admission deadline does not match its recorded duration.'
    }
    if ($NowUtc.ToUniversalTime() -gt $deadline) {
        throw "Acceptance snapshot admission deadline exceeded during '$Phase'."
    }
}

Export-ModuleMember -Function Get-MvpAcceptanceSnapshotAdmissionDefaultLimits, New-MvpAcceptanceSnapshotAdmission, Assert-MvpAcceptanceSnapshotAdmissionActive
