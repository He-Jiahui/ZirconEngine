Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpRunArtifactBudgetSchemaVersion = 1
$script:MvpRunArtifactBudgetPolicyKind = 'zircon.mvp-run-artifact-budget'
$script:MvpRunArtifactBudgetMeasurementKind = 'zircon.mvp-run-artifact-budget-measurement'
$script:MvpRunArtifactBudgetMaximumScannedFileCount = 100000
$script:MvpRunArtifactBudgetMaximumDirectoryDepth = 64
$script:MvpRunArtifactBudgetLowerHexDigits = [char[]]'0123456789abcdef'
$script:MvpRunArtifactBudgetUtf8 = [Text.UTF8Encoding]::new($false, $true)
$script:MvpRunArtifactBudgetByteArrayPool = $null
$mvpRunArtifactBudgetArrayPoolType = 'System.Buffers.ArrayPool`1[System.Byte]' -as [type]
if ($null -ne $mvpRunArtifactBudgetArrayPoolType) {
    $script:MvpRunArtifactBudgetByteArrayPool =
        $mvpRunArtifactBudgetArrayPoolType.GetProperty('Shared').GetValue($null)
}
Remove-Variable -Name mvpRunArtifactBudgetArrayPoolType

function ConvertTo-MvpRunArtifactBudgetLowerHex {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpRunArtifactBudgetLowerHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpRunArtifactBudgetLowerHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Resolve-MvpRunArtifactBudgetRoot {
    param([Parameter(Mandatory)][string]$Root)

    $fullPath = [IO.Path]::GetFullPath($Root)
    if (-not [IO.Directory]::Exists($fullPath)) {
        throw "MVP run artifact budget root '$Root' does not exist."
    }
    $rootInfo = [IO.DirectoryInfo]::new($fullPath)
    if (($rootInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP run artifact budget root '$Root' is a reparse directory."
    }
    return $rootInfo
}

function Add-MvpRunArtifactBudgetLength {
    param(
        [Parameter(Mandatory)][Int64]$Current,
        [Parameter(Mandatory)][Int64]$Additional,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Additional -lt 0 -or $Current -gt [Int64]::MaxValue - $Additional) {
        throw "MVP run artifact $Label exceeds the supported 64-bit byte range."
    }
    return [Int64]($Current + $Additional)
}

function Get-MvpRunArtifactBudgetFileLengths {
    param(
        [Parameter(Mandatory)][IO.DirectoryInfo]$RootDirectory,
        [AllowNull()][Collections.Generic.Dictionary[string, Int64]]$BaselineLengths,
        [AllowNull()][Collections.Generic.Stack[IO.DirectoryInfo]]$DirectoriesScratch,
        [AllowNull()][Collections.Generic.Stack[int]]$DirectoryDepthsScratch,
        [AllowNull()][Collections.Generic.HashSet[string]]$SeenPathsScratch,
        [AllowNull()][string]$RootPrefix,
        [AllowNull()]$ResultScratch
    )

    $rootPath = $RootDirectory.FullName
    if ([string]::IsNullOrEmpty($RootPrefix)) {
        $RootPrefix = $rootPath.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    }
    $lengths = $null
    $seenPaths = $null
    if ($null -eq $BaselineLengths) {
        $lengths = [Collections.Generic.Dictionary[string, Int64]]::new([StringComparer]::OrdinalIgnoreCase)
    }
    else {
        $seenPaths = $SeenPathsScratch
        if ($null -eq $seenPaths) {
            $seenPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        }
        else {
            $seenPaths.Clear()
        }
    }
    $directories = $DirectoriesScratch
    if ($null -eq $directories) {
        $directories = [Collections.Generic.Stack[IO.DirectoryInfo]]::new()
    }
    else {
        $directories.Clear()
    }
    $directoryDepths = $DirectoryDepthsScratch
    if ($null -eq $directoryDepths) {
        $directoryDepths = [Collections.Generic.Stack[int]]::new()
    }
    else {
        $directoryDepths.Clear()
    }
    $directories.Push($RootDirectory)
    $directoryDepths.Push(0)
    [Int64]$totalBytes = 0
    [Int64]$additionalBytes = 0
    $fileCount = 0
    $additionalFileCount = 0
    while ($directories.Count -gt 0) {
        $directory = $directories.Pop()
        $directoryDepth = $directoryDepths.Pop()
        foreach ($entry in $directory.EnumerateFileSystemInfos()) {
            if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "MVP run artifact budget root contains reparse entry '$($entry.FullName)'."
            }
            if (($entry.Attributes -band [IO.FileAttributes]::Directory) -ne 0) {
                $childDepth = $directoryDepth + 1
                if ($childDepth -gt $script:MvpRunArtifactBudgetMaximumDirectoryDepth) {
                    throw "MVP run artifact budget directory depth exceeds $($script:MvpRunArtifactBudgetMaximumDirectoryDepth)."
                }
                $directories.Push([IO.DirectoryInfo]$entry)
                $directoryDepths.Push($childDepth)
                continue
            }
            if ($fileCount -ge $script:MvpRunArtifactBudgetMaximumScannedFileCount) {
                throw "MVP run artifact budget file count exceeds its scan limit of $($script:MvpRunArtifactBudgetMaximumScannedFileCount)."
            }
            $fullPath = $entry.FullName
            if (-not $fullPath.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw "MVP run artifact path '$fullPath' escapes budget root '$rootPath'."
            }
            $relativePath = $fullPath.Substring($RootPrefix.Length).Replace('\', '/')
            $length = [Int64]$entry.Length
            if ($null -eq $BaselineLengths) {
                if ($lengths.ContainsKey($relativePath)) {
                    throw "MVP run artifact budget encountered duplicate path '$relativePath'."
                }
                $lengths.Add($relativePath, $length)
            }
            else {
                if (-not $seenPaths.Add($relativePath)) {
                    throw "MVP run artifact budget encountered duplicate path '$relativePath'."
                }
                [Int64]$baselineLength = 0
                if ($BaselineLengths.TryGetValue($relativePath, [ref]$baselineLength)) {
                    $growth = [Math]::Max([Int64]0, $length - $baselineLength)
                    $additionalBytes = Add-MvpRunArtifactBudgetLength `
                        -Current $additionalBytes `
                        -Additional $growth `
                        -Label 'additional bytes'
                }
                else {
                    $additionalFileCount++
                    $additionalBytes = Add-MvpRunArtifactBudgetLength `
                        -Current $additionalBytes `
                        -Additional $length `
                        -Label 'additional bytes'
                }
            }
            $fileCount++
            $totalBytes = Add-MvpRunArtifactBudgetLength `
                -Current $totalBytes `
                -Additional $length `
                -Label 'directory size'
        }
    }
    if ($null -eq $ResultScratch) {
        return [pscustomobject]@{
            lengths = $lengths
            file_count = $fileCount
            total_bytes = $totalBytes
            additional_bytes = $additionalBytes
            additional_file_count = $additionalFileCount
        }
    }
    $ResultScratch.lengths = $lengths
    $ResultScratch.file_count = $fileCount
    $ResultScratch.total_bytes = $totalBytes
    $ResultScratch.additional_bytes = $additionalBytes
    $ResultScratch.additional_file_count = $additionalFileCount
    return $ResultScratch
}

function Get-MvpRunArtifactBudgetBaselineSha256 {
    param(
        [Parameter(Mandatory)][Collections.Generic.Dictionary[string, Int64]]$Lengths
    )

    $paths = [string[]]@($Lengths.Keys)
    [Array]::Sort($paths, [StringComparer]::Ordinal)
    $maximumPathCharacterCount = 0
    foreach ($path in $paths) {
        if ($path.Length -gt $maximumPathCharacterCount) {
            $maximumPathCharacterCount = $path.Length
        }
    }
    $pathBufferLength = $script:MvpRunArtifactBudgetUtf8.GetMaxByteCount($maximumPathCharacterCount)
    [byte[]]$pathBuffer = $null
    if ($null -ne $script:MvpRunArtifactBudgetByteArrayPool) {
        $pathBuffer = $script:MvpRunArtifactBudgetByteArrayPool.Rent($pathBufferLength)
    }
    else {
        $pathBuffer = [byte[]]::new($pathBufferLength)
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    $hashStream = [Security.Cryptography.CryptoStream]::new(
        [IO.Stream]::Null,
        $hasher,
        [Security.Cryptography.CryptoStreamMode]::Write,
        $true)
    $writer = [IO.BinaryWriter]::new($hashStream, $script:MvpRunArtifactBudgetUtf8, $true)
    try {
        foreach ($path in $paths) {
            $pathByteCount = $script:MvpRunArtifactBudgetUtf8.GetBytes(
                $path,
                0,
                $path.Length,
                $pathBuffer,
                0)
            $writer.Write([int]$pathByteCount)
            $writer.Write($pathBuffer, 0, $pathByteCount)
            $writer.Write([Int64]$Lengths[$path])
        }
        $writer.Flush()
        $hashStream.FlushFinalBlock()
        return ConvertTo-MvpRunArtifactBudgetLowerHex -Bytes $hasher.Hash
    }
    finally {
        $writer.Dispose()
        $hashStream.Dispose()
        $hasher.Dispose()
        if ($null -ne $script:MvpRunArtifactBudgetByteArrayPool) {
            $script:MvpRunArtifactBudgetByteArrayPool.Return($pathBuffer, $false)
        }
    }
}

function Assert-MvpRunArtifactBudgetPolicy {
    param([Parameter(Mandatory)]$Budget)

    if ([int]$Budget.schema_version -ne $script:MvpRunArtifactBudgetSchemaVersion -or
        -not [string]::Equals(
            [string]$Budget.policy_kind,
            $script:MvpRunArtifactBudgetPolicyKind,
            [StringComparison]::Ordinal)) {
        throw 'MVP run artifact budget has an unsupported policy schema.'
    }
    if ($Budget.baseline_lengths -isnot [Collections.Generic.Dictionary[string, Int64]]) {
        throw 'MVP run artifact budget is missing its typed baseline length map.'
    }
    if ($Budget.scan_directories_scratch -isnot [Collections.Generic.Stack[IO.DirectoryInfo]] -or
        $Budget.scan_directory_depths_scratch -isnot [Collections.Generic.Stack[int]] -or
        $Budget.scan_seen_paths_scratch -isnot [Collections.Generic.HashSet[string]] -or
        $Budget.scan_result_scratch -isnot [pscustomobject]) {
        throw 'MVP run artifact budget is missing its typed heartbeat scan scratch.'
    }
    if ($Budget.root_directory -isnot [IO.DirectoryInfo] -or
        [string]::IsNullOrEmpty([string]$Budget.root_prefix) -or
        -not $Budget.root_directory.FullName.Equals(
            [string]$Budget.root_path,
            [StringComparison]::OrdinalIgnoreCase) -or
        $Budget.root_prefix.Length -ne $Budget.root_directory.FullName.Length + 1 -or
        -not $Budget.root_prefix.StartsWith(
            $Budget.root_directory.FullName,
            [StringComparison]::OrdinalIgnoreCase) -or
        $Budget.root_prefix[$Budget.root_prefix.Length - 1] -ne [IO.Path]::DirectorySeparatorChar) {
        throw 'MVP run artifact budget is missing its validated run root metadata.'
    }
}

function New-MvpRunArtifactBudget {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$PolicyId,
        [Parameter(Mandatory)][ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumAdditionalBytes,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$MaximumAdditionalFileCount
    )

    $rootDirectory = Resolve-MvpRunArtifactBudgetRoot -Root $Root
    $rootPath = $rootDirectory.FullName
    $rootPrefix = $rootPath.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    $baseline = Get-MvpRunArtifactBudgetFileLengths `
        -RootDirectory $rootDirectory `
        -RootPrefix $rootPrefix
    return [pscustomobject]@{
        schema_version = $script:MvpRunArtifactBudgetSchemaVersion
        policy_kind = $script:MvpRunArtifactBudgetPolicyKind
        policy_id = $PolicyId
        root_path = $rootPath
        maximum_additional_bytes = $MaximumAdditionalBytes
        maximum_additional_file_count = $MaximumAdditionalFileCount
        baseline_file_count = $baseline.file_count
        baseline_bytes = $baseline.total_bytes
        baseline_sha256 = Get-MvpRunArtifactBudgetBaselineSha256 -Lengths $baseline.lengths
        baseline_lengths = $baseline.lengths
        root_directory = $rootDirectory
        root_prefix = $rootPrefix
        scan_directories_scratch = [Collections.Generic.Stack[IO.DirectoryInfo]]::new()
        scan_directory_depths_scratch = [Collections.Generic.Stack[int]]::new()
        scan_seen_paths_scratch = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        scan_result_scratch = [pscustomobject]@{
            lengths = $null
            file_count = 0
            total_bytes = [Int64]0
            additional_bytes = [Int64]0
            additional_file_count = 0
        }
    }
}

function Measure-MvpRunArtifactBudget {
    param(
        [Parameter(Mandatory)]$Budget,
        [AllowNull()]$ResultScratch
    )

    Assert-MvpRunArtifactBudgetPolicy -Budget $Budget
    $Budget.root_directory.Refresh()
    if (-not $Budget.root_directory.Exists) {
        throw "MVP run artifact budget root '$($Budget.root_path)' does not exist."
    }
    if (($Budget.root_directory.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP run artifact budget root '$($Budget.root_path)' is a reparse directory."
    }
    $current = Get-MvpRunArtifactBudgetFileLengths `
        -RootDirectory $Budget.root_directory `
        -RootPrefix $Budget.root_prefix `
        -BaselineLengths $Budget.baseline_lengths `
        -DirectoriesScratch $Budget.scan_directories_scratch `
        -DirectoryDepthsScratch $Budget.scan_directory_depths_scratch `
        -SeenPathsScratch $Budget.scan_seen_paths_scratch `
        -ResultScratch $Budget.scan_result_scratch
    $additionalBytes = [Int64]$current.additional_bytes
    $additionalFileCount = [int]$current.additional_file_count
    $maximumAdditionalBytes = [Int64]$Budget.maximum_additional_bytes
    $maximumAdditionalFileCount = [int]$Budget.maximum_additional_file_count
    if ($null -eq $ResultScratch) {
        $ResultScratch = [pscustomobject]@{
            schema_version = 0
            measurement_kind = $null
            policy_id = $null
            measured_at_utc = $null
            additional_bytes = [Int64]0
            additional_file_count = 0
            current_bytes = [Int64]0
            current_file_count = 0
            remaining_bytes = [Int64]0
            remaining_file_count = 0
            within_budget = $false
        }
    }
    elseif ($ResultScratch -isnot [pscustomobject]) {
        throw 'MVP run artifact measurement scratch must be a PSCustomObject.'
    }
    $ResultScratch.schema_version = $script:MvpRunArtifactBudgetSchemaVersion
    $ResultScratch.measurement_kind = $script:MvpRunArtifactBudgetMeasurementKind
    $ResultScratch.policy_id = [string]$Budget.policy_id
    $ResultScratch.measured_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    $ResultScratch.additional_bytes = $additionalBytes
    $ResultScratch.additional_file_count = $additionalFileCount
    $ResultScratch.current_bytes = $current.total_bytes
    $ResultScratch.current_file_count = $current.file_count
    $ResultScratch.remaining_bytes = [Math]::Max([Int64]0, $maximumAdditionalBytes - $additionalBytes)
    $ResultScratch.remaining_file_count = [Math]::Max(0, $maximumAdditionalFileCount - $additionalFileCount)
    $ResultScratch.within_budget = ($additionalBytes -le $maximumAdditionalBytes -and
        $additionalFileCount -le $maximumAdditionalFileCount)
    return $ResultScratch
}

function Get-MvpRunArtifactBudgetPolicyReceipt {
    param([Parameter(Mandatory)]$Budget)

    Assert-MvpRunArtifactBudgetPolicy -Budget $Budget
    return [pscustomobject]@{
        schema_version = [int]$Budget.schema_version
        policy_kind = [string]$Budget.policy_kind
        policy_id = [string]$Budget.policy_id
        root_path = [string]$Budget.root_path
        maximum_additional_bytes = [Int64]$Budget.maximum_additional_bytes
        maximum_additional_file_count = [int]$Budget.maximum_additional_file_count
        baseline_file_count = [int]$Budget.baseline_file_count
        baseline_bytes = [Int64]$Budget.baseline_bytes
        baseline_sha256 = [string]$Budget.baseline_sha256
    }
}

function Assert-MvpRunArtifactBudget {
    param([Parameter(Mandatory)]$Budget)

    $measurement = Measure-MvpRunArtifactBudget -Budget $Budget
    if ([Int64]$measurement.additional_bytes -gt [Int64]$Budget.maximum_additional_bytes) {
        throw "MVP run artifact byte quota exceeded: $($measurement.additional_bytes) > $($Budget.maximum_additional_bytes)."
    }
    if ([int]$measurement.additional_file_count -gt [int]$Budget.maximum_additional_file_count) {
        throw "MVP run artifact file-count quota exceeded: $($measurement.additional_file_count) > $($Budget.maximum_additional_file_count)."
    }
    return $measurement
}

Export-ModuleMember -Function @(
    'New-MvpRunArtifactBudget',
    'Measure-MvpRunArtifactBudget',
    'Get-MvpRunArtifactBudgetPolicyReceipt',
    'Assert-MvpRunArtifactBudget'
)
