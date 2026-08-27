Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpBuildSetSchemaVersion = 1
$script:MvpBuildSetKind = 'zircon_mvp_product_build_set'
$script:MvpBuildSetUtf8 = [Text.UTF8Encoding]::new($false)
$script:MvpBuildSetLineSeparators = [string[]]@("`r`n", "`n")
$script:MvpBuildSetNulSeparator = [char[]]@([char]0)
$script:MvpBuildSetManifestPropertyNames = [string[]] @(
    'schema_version',
    'build_set_kind',
    'status',
    'build_set_id',
    'created_utc',
    'snapshot_relative_path',
    'source_policy',
    'git_revision',
    'dirty_overlay_sha256',
    'files'
)
$script:MvpBuildSetFileEntryPropertyNames = [string[]]@(
    'relative_path',
    'sha256',
    'byte_length'
)
$script:MvpBuildSetUnsafeRelativePathPattern = [Text.RegularExpressions.Regex]::new(
    '(?:^|/)\.{0,2}(?:/|$)',
    [Text.RegularExpressions.RegexOptions]::CultureInvariant)

function ConvertTo-MvpBuildSetUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    return [BitConverter]::ToString($Bytes).Replace('-', '')
}

function Get-MvpBuildSetFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpBuildSetUpperHex -Bytes $hasher.ComputeHash($stream)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Invoke-MvpBuildSetGit {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    $quotedArguments = [string[]]::new($Arguments.Length)
    for ($index = 0; $index -lt $Arguments.Length; $index++) {
        $argument = $Arguments[$index]
        if ($argument.Contains('"')) {
            throw 'MVP BuildSet Git arguments must not contain double quotes.'
        }
        $quotedArguments[$index] = '"{0}"' -f $argument
    }
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $GitPath
    $startInfo.WorkingDirectory = $RepositoryRoot
    $startInfo.Arguments = [string]::Join(' ', $quotedArguments)
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        if (-not $process.Start()) {
            throw "Could not start MVP BuildSet git command '$($Arguments -join ' ')'"
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        [Threading.Tasks.Task]::WaitAll(@($stdoutTask, $stderrTask))
        if ($process.ExitCode -ne 0) {
            $detail = $stderrTask.Result.Trim()
            if ([string]::IsNullOrWhiteSpace($detail)) {
                $detail = $stdoutTask.Result.Trim()
            }
            throw "MVP BuildSet git command failed: git -C $RepositoryRoot $($Arguments -join ' ')`n$detail"
        }
        return $stdoutTask.Result.Split(
            $script:MvpBuildSetLineSeparators,
            [StringSplitOptions]::RemoveEmptyEntries)
    }
    finally {
        $process.Dispose()
    }
}

function Invoke-MvpBuildSetGitBytes {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Arguments
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $GitPath
    $startInfo.WorkingDirectory = $RepositoryRoot
    $startInfo.Arguments = $Arguments
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $output = [IO.MemoryStream]::new()
    try {
        if (-not $process.Start()) {
            throw "Could not start MVP BuildSet git command '$Arguments'."
        }
        $copyTask = $process.StandardOutput.BaseStream.CopyToAsync($output)
        $errorTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        [Threading.Tasks.Task]::WaitAll(@($copyTask, $errorTask))
        if ($process.ExitCode -ne 0) {
            throw "MVP BuildSet git command '$Arguments' failed: $($errorTask.Result.Trim())"
        }
        if ($output.Length -gt [Int32]::MaxValue) {
            throw "MVP BuildSet git command '$Arguments' produced more than $([Int32]::MaxValue) bytes."
        }
        return [Tuple[object, int]]::new($output.GetBuffer(), [int]$output.Length)
    }
    finally {
        $output.Dispose()
        $process.Dispose()
    }
}

function Assert-MvpBuildSetSourceIndexModePolicy {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )

    $indexCapture = Invoke-MvpBuildSetGitBytes `
            -GitPath $GitPath `
            -RepositoryRoot $RepositoryRoot `
            -Arguments 'ls-files --stage -z'
    $indexBuffer = [byte[]]$indexCapture.Item1
    $entries = $script:MvpBuildSetUtf8.GetString(
        $indexBuffer,
        0,
        $indexCapture.Item2).Split(
            $script:MvpBuildSetNulSeparator,
            [StringSplitOptions]::RemoveEmptyEntries)
    $indexBuffer = $null
    $indexCapture = $null
    foreach ($entry in $entries) {
        $separator = $entry.IndexOf([char]9)
        if ($separator -le 0) {
            throw 'MVP BuildSet received an invalid source Git index record.'
        }
        $metadata = $entry.Substring(0, $separator).Split(' ')
        if ($metadata.Count -ne 3 -or $metadata[0] -notmatch '^\d{6}$' -or $metadata[2] -ne '0') {
            throw 'MVP BuildSet received an unsupported source Git index record.'
        }
        $relativePath = $entry.Substring($separator + 1).Replace('\', '/')
        if ($metadata[0] -eq '120000') {
            # Windows can materialize a link patch as plain text. Check the source index
            # before it reaches the worktree so that link intent is never downgraded.
            throw "MVP BuildSet rejects symbolic link '$relativePath' from the source index."
        }
        if ($metadata[0] -eq '160000') {
            throw "MVP BuildSet rejects Git submodule '$relativePath' from the source index because its source closure is not materialized."
        }
    }
}

function Resolve-MvpBuildSetChildPathNormalized {
    param(
        [Parameter(Mandatory)][string]$NormalizedRoot,
        [Parameter(Mandatory)][string]$RootPrefix,
        [Parameter(Mandatory)][string]$RelativePath
    )

    $normalizedRelativePath = $RelativePath.Replace('\', '/')
    if ([string]::IsNullOrWhiteSpace($RelativePath) -or
        [IO.Path]::IsPathRooted($RelativePath) -or
        $script:MvpBuildSetUnsafeRelativePathPattern.IsMatch($normalizedRelativePath)) {
        throw "MVP BuildSet contains an unsafe relative path '$RelativePath'."
    }
    $platformRelativePath = $normalizedRelativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
    $candidate = [IO.Path]::GetFullPath([IO.Path]::Combine($NormalizedRoot, $platformRelativePath))
    if (-not $candidate.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "MVP BuildSet path escapes its snapshot root: '$RelativePath'."
    }
    return $candidate
}

function Resolve-MvpBuildSetChildPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$RelativePath
    )

    $normalizedRoot = [IO.Path]::GetFullPath($Root).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar)
    $rootPrefix = $normalizedRoot + [IO.Path]::DirectorySeparatorChar
    return Resolve-MvpBuildSetChildPathNormalized `
        -NormalizedRoot $normalizedRoot `
        -RootPrefix $rootPrefix `
        -RelativePath $RelativePath
}

function Assert-MvpBuildSetExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value -or $Value -is [Array]) {
        throw "$Label must contain one JSON object."
    }
    $actualCount = 0
    foreach ($property in $Value.PSObject.Properties) {
        $actualCount++
        $known = $false
        foreach ($name in $ExpectedNames) {
            if ([string]::Equals($property.Name, $name, [StringComparison]::Ordinal)) {
                $known = $true
                break
            }
        }
        if (-not $known) {
            throw "$Label contains unknown property '$($property.Name)'."
        }
    }
    if ($actualCount -ne $ExpectedNames.Count) {
        throw "$Label property count differs from $($ExpectedNames.Count)."
    }
}

function Get-MvpBuildSetSnapshotFilesNoFollow {
    param([Parameter(Mandatory)][string]$SnapshotRoot)

    $root = [IO.Path]::GetFullPath($SnapshotRoot).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar)
    $rootPrefix = $root + [IO.Path]::DirectorySeparatorChar
    $gitMetadataPath = [IO.Path]::Combine($root, '.git')
    $files = [Collections.Generic.List[string]]::new()
    $pending = [Collections.Generic.Stack[IO.DirectoryInfo]]::new()
    $pending.Push([IO.DirectoryInfo]::new($root))
    while ($pending.Count -gt 0) {
        $directoryInfo = $pending.Pop()
        if (($directoryInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "MVP BuildSet snapshot contains a reparse-point directory: $($directoryInfo.FullName)"
        }
        foreach ($entry in $directoryInfo.EnumerateFileSystemInfos()) {
            $entryPath = $entry.FullName
            $entryAttributes = $entry.Attributes
            if (($entryAttributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                $kind = if (($entryAttributes -band [IO.FileAttributes]::Directory) -ne 0) { 'directory' } else { 'file' }
                throw "MVP BuildSet snapshot contains a reparse-point ${kind}: $entryPath"
            }
            if (($entryAttributes -band [IO.FileAttributes]::Directory) -ne 0) {
                $pending.Push([IO.DirectoryInfo]$entry)
                continue
            }
            if ($entryPath.Equals($gitMetadataPath, [StringComparison]::OrdinalIgnoreCase)) {
                continue
            }
            if (-not $entryPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw "MVP BuildSet path '$entryPath' escapes its snapshot root '$root'."
            }
            $files.Add($entryPath.Substring($rootPrefix.Length).Replace('\', '/'))
        }
    }
    $files.Sort([StringComparer]::Ordinal)
    return ,$files
}

function Get-MvpBuildSetTrackedFiles {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$SnapshotRoot
    )

    $normalizedSnapshotRoot = [IO.Path]::GetFullPath($SnapshotRoot).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar)
    $snapshotRootPrefix = $normalizedSnapshotRoot + [IO.Path]::DirectorySeparatorChar
    $pathCapture = Invoke-MvpBuildSetGitBytes `
        -GitPath $GitPath `
        -RepositoryRoot $normalizedSnapshotRoot `
        -Arguments 'ls-files --stage -z'
    $pathBuffer = [byte[]]$pathCapture.Item1
    $entries = $script:MvpBuildSetUtf8.GetString(
        $pathBuffer,
        0,
        $pathCapture.Item2).Split(
            $script:MvpBuildSetNulSeparator,
            [StringSplitOptions]::RemoveEmptyEntries)
    $pathBuffer = $null
    $pathCapture = $null
    $paths = [Collections.Generic.List[string]]::new($entries.Length)
    foreach ($entry in $entries) {
        $separator = $entry.IndexOf([char]9)
        if ($separator -le 0) {
            throw 'MVP BuildSet received an invalid Git index record.'
        }
        $metadata = $entry.Substring(0, $separator).Split(' ')
        if ($metadata.Count -ne 3 -or $metadata[0] -notmatch '^\d{6}$' -or $metadata[2] -ne '0') {
            throw 'MVP BuildSet received an unsupported Git index record.'
        }
        $relativePath = $entry.Substring($separator + 1).Replace('\', '/')
        if ($metadata[0] -eq '160000') {
            throw "MVP BuildSet rejects Git submodule '$relativePath' because its source closure is not materialized."
        }
        if ($metadata[0] -eq '120000') {
            throw "MVP BuildSet rejects symbolic link '$relativePath' because its source closure is not materialized."
        }
        $paths.Add($relativePath)
    }
    $entries = $null
    $paths.Sort([StringComparer]::Ordinal)

    $files = [Collections.Generic.List[object]]::new($paths.Count)
    $materializedFilePrefixBuffer = [byte[]]::new(128)
    $contentHasher = [Security.Cryptography.SHA256]::Create()
    try {
        foreach ($relativePath in $paths) {
            $path = Resolve-MvpBuildSetChildPathNormalized `
                -NormalizedRoot $normalizedSnapshotRoot `
                -RootPrefix $snapshotRootPrefix `
                -RelativePath $relativePath
            $item = [IO.FileInfo]::new($path)
            if (-not $item.Exists) {
                # A dirty overlay may intentionally delete a tracked file.
                continue
            }
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "MVP BuildSet rejects reparse-point source file '$relativePath'."
            }
            $contentStream = [IO.File]::OpenRead($path)
            try {
                $prefixLength = [Math]::Min(
                    [int64]$materializedFilePrefixBuffer.LongLength,
                    $contentStream.Length)
                $read = $contentStream.Read(
                    $materializedFilePrefixBuffer,
                    0,
                    [int]$prefixLength)
                $text = [Text.Encoding]::ASCII.GetString(
                    $materializedFilePrefixBuffer,
                    0,
                    $read)
                if ($text -match '^version https://git-lfs\.github\.com/spec/v1(?:\r?\n|$)') {
                    throw "MVP BuildSet rejects an unmaterialized Git LFS pointer '$relativePath'."
                }
                $contentStream.Position = 0
                $sha256 = ConvertTo-MvpBuildSetUpperHex `
                    -Bytes $contentHasher.ComputeHash($contentStream)
            }
            finally {
                $contentStream.Dispose()
            }
            $files.Add([ordered]@{
                    relative_path = $relativePath
                    sha256 = $sha256
                    byte_length = [int64]$item.Length
                })
        }
    }
    finally {
        $contentHasher.Dispose()
    }
    if ($files.Count -eq 0) {
        throw 'MVP BuildSet contains no tracked source files.'
    }
    return ,$files
}

function Get-MvpBuildSetId {
    param(
        [Parameter(Mandatory)][string]$GitRevision,
        [Parameter(Mandatory)][string]$DirtyOverlaySha256,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.IEnumerable[object]]$Files
    )

    $encoding = $script:MvpBuildSetUtf8
    $hasher = [Security.Cryptography.SHA256]::Create()
    $cryptoStream = $null
    $writer = $null
    try {
        $cryptoStream = [Security.Cryptography.CryptoStream]::new(
            [IO.Stream]::Null,
            $hasher,
            [Security.Cryptography.CryptoStreamMode]::Write,
            $true)
        $writer = [IO.BinaryWriter]::new($cryptoStream, $encoding, $true)
        $segments = [string[]]::new(3)
        $segments[0] = 'zircon-mvp-build-set-v1'
        $segments[1] = $GitRevision
        $segments[2] = $DirtyOverlaySha256
        foreach ($segment in $segments) {
            [byte[]]$bytes = $encoding.GetBytes($segment)
            $writer.Write([int64]$bytes.LongLength)
            $writer.Write($bytes)
        }
        foreach ($file in $Files) {
            [byte[]]$bytes = $encoding.GetBytes([string]$file.relative_path)
            $writer.Write([int64]$bytes.LongLength)
            $writer.Write($bytes)
            [byte[]]$bytes = $encoding.GetBytes([string]$file.sha256)
            $writer.Write([int64]$bytes.LongLength)
            $writer.Write($bytes)
            [byte[]]$bytes = $encoding.GetBytes([string][int64]$file.byte_length)
            $writer.Write([int64]$bytes.LongLength)
            $writer.Write($bytes)
        }
        $writer.Flush()
        $cryptoStream.FlushFinalBlock()
        return ConvertTo-MvpBuildSetUpperHex -Bytes $hasher.Hash
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        if ($null -ne $cryptoStream) {
            $cryptoStream.Dispose()
        }
        $hasher.Dispose()
    }
}

function Write-MvpBuildSetJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    $bytes = $script:MvpBuildSetUtf8.GetBytes(($Value | ConvertTo-Json -Depth 12))
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.WriteByte([byte]10)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Write-MvpBuildSetIncompleteReceipt {
    param(
        [Parameter(Mandatory)][string]$BuildSetRoot,
        [Parameter(Mandatory)][string]$FailureMessage
    )

    if (-not [IO.Directory]::Exists($BuildSetRoot)) {
        return
    }
    $path = Join-Path $BuildSetRoot 'build-set-incomplete.json'
    if ([IO.File]::Exists($path)) {
        return
    }
    try {
        Write-MvpBuildSetJson -Path $path -Value ([ordered]@{
                schema_version = $script:MvpBuildSetSchemaVersion
                build_set_kind = $script:MvpBuildSetKind
                status = 'incomplete'
                failed_utc = [DateTime]::UtcNow.ToString('o')
                failure = $FailureMessage.Substring(0, [Math]::Min($FailureMessage.Length, 1024))
            })
    }
    catch {
        # Preserve the initial failure when the diagnostic receipt itself cannot be written.
    }
}

function Assert-MvpBuildSetInventory {
    param(
        [Parameter(Mandatory)][Collections.Generic.List[string]]$ActualFiles,
        [Parameter(Mandatory)][Collections.Generic.List[string]]$ExpectedFiles
    )

    $inventoryMatches = [Linq.Enumerable]::SequenceEqual(
        $ActualFiles,
        $ExpectedFiles,
        [StringComparer]::Ordinal)
    if ($inventoryMatches) {
        return
    }

    $unexpected = [Collections.Generic.List[string]]::new(3)
    $missing = [Collections.Generic.List[string]]::new(3)
    $actualIndex = 0
    $manifestIndex = 0
    while ($actualIndex -lt $ActualFiles.Count -and $manifestIndex -lt $ExpectedFiles.Count -and
        ($unexpected.Count -lt 3 -or $missing.Count -lt 3)) {
        $actualPath = $ActualFiles[$actualIndex]
        $manifestPath = $ExpectedFiles[$manifestIndex]
        $comparison = [StringComparer]::Ordinal.Compare($actualPath, $manifestPath)
        if ($comparison -eq 0) {
            $actualIndex++
            $manifestIndex++
        }
        elseif ($comparison -lt 0) {
            if ($unexpected.Count -lt 3) {
                $unexpected.Add($actualPath)
            }
            $actualIndex++
        }
        else {
            if ($missing.Count -lt 3) {
                $missing.Add($manifestPath)
            }
            $manifestIndex++
        }
    }
    while ($actualIndex -lt $ActualFiles.Count -and $unexpected.Count -lt 3) {
        $unexpected.Add($ActualFiles[$actualIndex])
        $actualIndex++
    }
    while ($manifestIndex -lt $ExpectedFiles.Count -and $missing.Count -lt 3) {
        $missing.Add($ExpectedFiles[$manifestIndex])
        $manifestIndex++
    }
    throw "MVP BuildSet snapshot contains files outside its tracked manifest. Unexpected: $($unexpected -join ', '); missing: $($missing -join ', ')."
}

function New-MvpProductBuildSet {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$BuildSetRoot
    )

    $git = Get-Command git -ErrorAction SilentlyContinue
    if ($null -eq $git) {
        throw 'MVP BuildSet requires git.'
    }
    $repoRoot = [IO.Path]::GetFullPath($RepositoryRoot)
    if (-not [IO.Directory]::Exists($repoRoot)) {
        throw "MVP BuildSet repository root does not exist: $repoRoot"
    }
    $reportedRoot = [string](Invoke-MvpBuildSetGit `
            -GitPath $git.Source `
            -RepositoryRoot $repoRoot `
            -Arguments @('rev-parse', '--show-toplevel') | Select-Object -First 1)
    if ([IO.Path]::GetFullPath($reportedRoot.Trim()) -ne $repoRoot) {
        throw "MVP BuildSet repository root must be the Git worktree root: $repoRoot"
    }
    $finalRoot = [IO.Path]::GetFullPath($BuildSetRoot)
    if ([IO.Directory]::Exists($finalRoot) -or [IO.File]::Exists($finalRoot)) {
        throw "MVP BuildSet root must not already exist: $finalRoot"
    }
    $repoPrefix = $repoRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if ($finalRoot.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'MVP BuildSet root must be outside the active repository worktree.'
    }

    $parent = Split-Path -Parent $finalRoot
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    [IO.Directory]::CreateDirectory($finalRoot) | Out-Null
    $snapshotRoot = $null
    $worktreeAdded = $false
    $pendingManifestPath = $null
    try {
        $revision = [string](Invoke-MvpBuildSetGit `
                -GitPath $git.Source `
                -RepositoryRoot $repoRoot `
                -Arguments @('rev-parse', 'HEAD') | Select-Object -First 1).Trim()
        if ($revision -notmatch '^[0-9a-f]{40}$') {
            throw "MVP BuildSet Git revision is invalid: $revision"
        }
        Assert-MvpBuildSetSourceIndexModePolicy `
            -GitPath $git.Source `
            -RepositoryRoot $repoRoot
        $snapshotRoot = Join-Path $finalRoot 'source'
        Invoke-MvpBuildSetGit `
            -GitPath $git.Source `
            -RepositoryRoot $repoRoot `
            -Arguments @('worktree', 'add', '--detach', $snapshotRoot, $revision) | Out-Null
        $worktreeAdded = $true

        $overlayPath = Join-Path $finalRoot 'tracked-dirty-overlay.patch'
        Invoke-MvpBuildSetGit `
            -GitPath $git.Source `
            -RepositoryRoot $repoRoot `
            -Arguments @('diff', '--binary', '--no-ext-diff', 'HEAD', '--output', $overlayPath) | Out-Null
        $overlaySha256 = Get-MvpBuildSetFileSha256 -Path $overlayPath
        if ([IO.FileInfo]::new($overlayPath).Length -gt 0) {
            Invoke-MvpBuildSetGit `
                -GitPath $git.Source `
                -RepositoryRoot $snapshotRoot `
                -Arguments @('apply', '--index', '--binary', '--whitespace=nowarn', $overlayPath) | Out-Null
        }
        # The private index preserves Git object modes. Re-indexing on Windows could reduce
        # a 120000 symbolic-link entry to ordinary text before the allowlist rejects it.
        Remove-Item -LiteralPath $overlayPath -Force -ErrorAction Stop

        [Collections.Generic.List[object]]$files = Get-MvpBuildSetTrackedFiles -GitPath $git.Source -SnapshotRoot $snapshotRoot
        $buildSetId = Get-MvpBuildSetId `
            -GitRevision $revision `
            -DirtyOverlaySha256 $overlaySha256 `
            -Files $files
        $manifestPath = Join-Path $finalRoot 'build-set.json'
        $pendingManifestPath = Join-Path $finalRoot 'build-set-pending.json'
        $manifest = [ordered]@{
            schema_version = $script:MvpBuildSetSchemaVersion
            build_set_kind = $script:MvpBuildSetKind
            status = 'completed'
            build_set_id = $buildSetId
            created_utc = [DateTime]::UtcNow.ToString('o')
            snapshot_relative_path = 'source'
            source_policy = 'tracked_head_plus_tracked_dirty_overlay'
            git_revision = $revision
            dirty_overlay_sha256 = $overlaySha256
            files = $files
        }
        # Validate a private candidate before the final, completed-name manifest becomes visible.
        Write-MvpBuildSetJson -Path $pendingManifestPath -Value $manifest
        $validated = Assert-MvpProductBuildSet -ManifestPath $pendingManifestPath
        [IO.File]::Move($pendingManifestPath, $manifestPath)
        $validated.manifest_path = $manifestPath
        return $validated
    }
    catch {
        if ($null -ne $pendingManifestPath -and [IO.File]::Exists($pendingManifestPath)) {
            try {
                Remove-Item -LiteralPath $pendingManifestPath -Force -ErrorAction Stop
            }
            catch {
                # Preserve the source failure even when pending-manifest cleanup is unavailable.
            }
        }
        if ($worktreeAdded -and $null -ne $snapshotRoot -and [IO.Directory]::Exists($snapshotRoot)) {
            try {
                Invoke-MvpBuildSetGit `
                    -GitPath $git.Source `
                    -RepositoryRoot $repoRoot `
                    -Arguments @('worktree', 'remove', '--force', $snapshotRoot) | Out-Null
            }
            catch {
                # The incomplete receipt preserves the original failure even if Git cleanup also fails.
            }
        }
        Write-MvpBuildSetIncompleteReceipt -BuildSetRoot $finalRoot -FailureMessage $_.Exception.Message
        throw
    }
}

function Assert-MvpProductBuildSet {
    param([Parameter(Mandatory)][string]$ManifestPath)

    $resolvedManifestPath = [IO.Path]::GetFullPath($ManifestPath)
    if (-not [IO.File]::Exists($resolvedManifestPath)) {
        throw "MVP BuildSet manifest does not exist: $resolvedManifestPath"
    }
    try {
        $manifest = [IO.File]::ReadAllText($resolvedManifestPath, $script:MvpBuildSetUtf8) | ConvertFrom-Json
    }
    catch {
        throw "MVP BuildSet manifest is malformed: $resolvedManifestPath"
    }
    Assert-MvpBuildSetExactProperties `
        -Value $manifest `
        -ExpectedNames $script:MvpBuildSetManifestPropertyNames `
        -Label 'MVP BuildSet manifest'
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [Int64]$manifest.schema_version -ne $script:MvpBuildSetSchemaVersion -or
        [string]$manifest.build_set_kind -cne $script:MvpBuildSetKind -or
        [string]$manifest.status -cne 'completed') {
        throw "MVP BuildSet manifest has an unexpected schema or status: $resolvedManifestPath"
    }
    if ([string]$manifest.build_set_id -notmatch '^[0-9A-F]{64}$' -or
        [string]$manifest.git_revision -notmatch '^[0-9a-f]{40}$' -or
        [string]$manifest.dirty_overlay_sha256 -notmatch '^[0-9A-F]{64}$' -or
        [string]$manifest.source_policy -cne 'tracked_head_plus_tracked_dirty_overlay') {
        throw 'MVP BuildSet manifest contains invalid source identity.'
    }
    $createdUtcValue = $manifest.created_utc
    $createdUtc = [DateTimeOffset]::MinValue
    $createdUtcIsValid = if ($createdUtcValue -is [DateTime]) {
        $createdUtcValue.Kind -eq [DateTimeKind]::Utc
    }
    elseif ($createdUtcValue -is [DateTimeOffset]) {
        $createdUtcValue.Offset -eq [TimeSpan]::Zero
    }
    else {
        [DateTimeOffset]::TryParse(
            [string]$createdUtcValue,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$createdUtc) -and $createdUtc.Offset -eq [TimeSpan]::Zero
    }
    if (-not $createdUtcIsValid) {
        throw 'MVP BuildSet manifest created_utc must be an ISO-8601 UTC timestamp.'
    }
    $snapshotRelativePath = [string]$manifest.snapshot_relative_path
    if ($snapshotRelativePath -cne 'source') {
        throw "MVP BuildSet snapshot_relative_path must be the direct 'source' child."
    }
    $manifestDirectory = [IO.Path]::GetDirectoryName($resolvedManifestPath)
    $snapshotRoot = [IO.Path]::GetFullPath(
        [IO.Path]::Combine($manifestDirectory, $snapshotRelativePath)).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar)
    $snapshotRootPrefix = $snapshotRoot + [IO.Path]::DirectorySeparatorChar
    if (-not [IO.Directory]::Exists($snapshotRoot)) {
        throw "MVP BuildSet snapshot root is unavailable: $snapshotRoot"
    }
    [Collections.Generic.List[string]]$actualFiles = Get-MvpBuildSetSnapshotFilesNoFollow -SnapshotRoot $snapshotRoot
    [object[]]$files = $manifest.files
    if ($files.Count -eq 0) {
        throw 'MVP BuildSet manifest has no tracked source files.'
    }
    $manifestPaths = [Collections.Generic.List[string]]::new($files.Count)
    $previousPath = $null
    $contentHasher = [Security.Cryptography.SHA256]::Create()
    try {
        foreach ($file in $files) {
            Assert-MvpBuildSetExactProperties `
                -Value $file `
                -ExpectedNames $script:MvpBuildSetFileEntryPropertyNames `
                -Label 'MVP BuildSet manifest file entry'
            $relativePath = [string]$file.relative_path
            if ([string]$file.sha256 -notmatch '^[0-9A-F]{64}$' -or
                ($file.byte_length -isnot [int] -and $file.byte_length -isnot [long]) -or
                [Int64]$file.byte_length -lt 0) {
                throw "MVP BuildSet manifest file entry '$relativePath' has invalid content identity."
            }
            if ($null -ne $previousPath -and
                [StringComparer]::Ordinal.Compare($previousPath, $relativePath) -ge 0) {
                throw 'MVP BuildSet manifest file paths must be unique and ordinally sorted.'
            }
            $previousPath = $relativePath
            $manifestPaths.Add($relativePath)
            $path = Resolve-MvpBuildSetChildPathNormalized `
                -NormalizedRoot $snapshotRoot `
                -RootPrefix $snapshotRootPrefix `
                -RelativePath $relativePath
            $item = [IO.FileInfo]::new($path)
            if (-not $item.Exists) {
                throw "MVP BuildSet snapshot file is missing: $relativePath"
            }
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "MVP BuildSet snapshot file is a reparse point: $relativePath"
            }
            if ([int64]$file.byte_length -ne [int64]$item.Length) {
                throw "MVP BuildSet snapshot file content differs from its manifest: $relativePath"
            }
            $contentStream = [IO.File]::OpenRead($path)
            try {
                $actualSha256 = ConvertTo-MvpBuildSetUpperHex `
                    -Bytes $contentHasher.ComputeHash($contentStream)
            }
            finally {
                $contentStream.Dispose()
            }
            if ([string]$file.sha256 -ne $actualSha256) {
                throw "MVP BuildSet snapshot file content differs from its manifest: $relativePath"
            }
        }
    }
    finally {
        $contentHasher.Dispose()
    }
    Assert-MvpBuildSetInventory `
        -ActualFiles $actualFiles `
        -ExpectedFiles $manifestPaths
    $buildSetId = Get-MvpBuildSetId `
        -GitRevision ([string]$manifest.git_revision) `
        -DirtyOverlaySha256 ([string]$manifest.dirty_overlay_sha256) `
        -Files $files
    if ([string]$manifest.build_set_id -ne $buildSetId) {
        throw 'MVP BuildSet manifest build_set_id does not match its source tree.'
    }
    return [pscustomobject]@{
        schema_version = $script:MvpBuildSetSchemaVersion
        build_set_kind = $script:MvpBuildSetKind
        build_set_id = $buildSetId
        snapshot_root = $snapshotRoot
        manifest_path = $resolvedManifestPath
        git_revision = [string]$manifest.git_revision
        dirty_overlay_sha256 = [string]$manifest.dirty_overlay_sha256
        files = $files
    }
}

Export-ModuleMember -Function @(
    'New-MvpProductBuildSet',
    'Assert-MvpProductBuildSet'
)
