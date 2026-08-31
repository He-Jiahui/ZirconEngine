Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:MvpAcceptanceStagingTreeManifestUpperHexDigits = [char[]]'0123456789ABCDEF'
$script:MvpAcceptanceStagingTreeManifestMaximumBytes = [Int64]67108864
$script:MvpAcceptanceStagingTreeManifestMaximumEntries = 100000

function Get-MvpAcceptanceStagingTreeManifestPath {
    param([Parameter(Mandatory)][string]$StagingRoot)

    return [IO.Path]::Combine(
        [IO.Path]::GetFullPath($StagingRoot),
        'staging-tree-manifest.json')
}

function ConvertTo-MvpAcceptanceStagingTreeManifestRelativePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    $absoluteRoot = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
    $absolutePath = [IO.Path]::GetFullPath($Path)
    $prefix = $absoluteRoot + [IO.Path]::DirectorySeparatorChar
    if (-not $absolutePath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance staging tree entry '$absolutePath' escapes root '$absoluteRoot'."
    }
    return $absolutePath.Substring($prefix.Length).Replace('\', '/')
}

function Resolve-MvpAcceptanceStagingTreeManifestNormalizedEntryPath {
    param(
        [Parameter(Mandatory)][string]$AbsoluteRoot,
        [Parameter(Mandatory)][string]$RootPrefix,
        [Parameter(Mandatory)][string]$NormalizedRelativePath
    )

    if ([string]::IsNullOrWhiteSpace($NormalizedRelativePath) -or
        [IO.Path]::IsPathRooted($NormalizedRelativePath) -or
        $NormalizedRelativePath.Contains(':')) {
        throw "Acceptance staging tree manifest path '$NormalizedRelativePath' is not a relative path."
    }
    $containsDotComponent =
        $NormalizedRelativePath.Equals('.', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.Equals('..', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.StartsWith('./', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.StartsWith('../', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.EndsWith('/.', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.EndsWith('/..', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.IndexOf('/./', [StringComparison]::Ordinal) -ge 0 -or
        $NormalizedRelativePath.IndexOf('/../', [StringComparison]::Ordinal) -ge 0
    if ($NormalizedRelativePath.StartsWith('/', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.EndsWith('/', [StringComparison]::Ordinal) -or
        $NormalizedRelativePath.IndexOf('//', [StringComparison]::Ordinal) -ge 0 -or
        $containsDotComponent) {
        throw "Acceptance staging tree manifest path '$NormalizedRelativePath' is not normalized."
    }

    $platformRelativePath = $NormalizedRelativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
    $candidate = [IO.Path]::GetFullPath([IO.Path]::Combine($AbsoluteRoot, $platformRelativePath))
    if (-not $candidate.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance staging tree manifest path '$NormalizedRelativePath' escapes root '$AbsoluteRoot'."
    }
    return $candidate
}

function Resolve-MvpAcceptanceStagingTreeManifestEntryPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$RelativePath
    )

    $absoluteRoot = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
    $rootPrefix = $absoluteRoot + [IO.Path]::DirectorySeparatorChar
    $normalizedRelativePath = $RelativePath.Replace('\', '/')
    return Resolve-MvpAcceptanceStagingTreeManifestNormalizedEntryPath `
        -AbsoluteRoot $absoluteRoot `
        -RootPrefix $rootPrefix `
        -NormalizedRelativePath $normalizedRelativePath
}

function ConvertTo-MvpAcceptanceStagingTreeManifestUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpAcceptanceStagingTreeManifestUpperHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpAcceptanceStagingTreeManifestUpperHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpAcceptanceStagingTreeManifestSha256 {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Security.Cryptography.SHA256]$Hasher
    )

    $ownsHasher = $null -eq $Hasher
    if ($ownsHasher) {
        $Hasher = [Security.Cryptography.SHA256]::Create()
    }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        return ConvertTo-MvpAcceptanceStagingTreeManifestUpperHex -Bytes $Hasher.ComputeHash($stream)
    }
    finally {
        $stream.Dispose()
        if ($ownsHasher) {
            $Hasher.Dispose()
        }
    }
}

function Get-MvpAcceptanceStagingTreeManifestEntries {
    param([Parameter(Mandatory)][string]$StagingRoot)

    $absoluteRoot = [IO.Path]::GetFullPath($StagingRoot)
    $rootPrefix = $absoluteRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    $manifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteRoot
    $manifestEntries = [System.Collections.Generic.List[object]]::new()
    $directories = [System.Collections.Generic.Queue[IO.DirectoryInfo]]::new()
    $directories.Enqueue([IO.DirectoryInfo]::new($absoluteRoot))
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        while ($directories.Count -gt 0) {
            $directory = $directories.Dequeue()
            foreach ($child in $directory.EnumerateFileSystemInfos()) {
                if ([bool]($child.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                    throw "Acceptance staging tree contains reparse point '$($child.FullName)'."
                }
                $childPath = $child.FullName
                if (-not $childPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "Acceptance staging tree entry '$childPath' escapes root '$absoluteRoot'."
                }
                if ($childPath.Equals($manifestPath, [StringComparison]::OrdinalIgnoreCase)) {
                    continue
                }
                $relativePath = $childPath.Substring($rootPrefix.Length).Replace('\', '/')
                if ($child -is [IO.DirectoryInfo]) {
                    $manifestEntries.Add([ordered]@{
                        path = $relativePath
                        kind = 'directory'
                    }) | Out-Null
                    $directories.Enqueue([IO.DirectoryInfo]$child)
                }
                else {
                    $manifestEntries.Add([ordered]@{
                        path = $relativePath
                        kind = 'file'
                        size_bytes = [Int64]$child.Length
                        sha256 = Get-MvpAcceptanceStagingTreeManifestSha256 `
                            -Path $childPath `
                            -Hasher $hasher
                    }) | Out-Null
                }
            }
        }
    }
    finally {
        $hasher.Dispose()
    }
    return @($manifestEntries | Sort-Object -Property path)
}

function Write-MvpAcceptanceStagingTreeManifest {
    param([Parameter(Mandatory)][string]$StagingRoot)

    $absoluteRoot = [IO.Path]::GetFullPath($StagingRoot)
    if (-not [IO.Directory]::Exists($absoluteRoot)) {
        throw "Acceptance staging tree root '$absoluteRoot' is not a directory."
    }
    $manifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteRoot
    $entries = Get-MvpAcceptanceStagingTreeManifestEntries -StagingRoot $absoluteRoot
    $value = [ordered]@{
        schema_version = 1
        entries = $entries
    }
    $temporaryPath = $manifestPath + '.partial-' + [guid]::NewGuid().ToString('N')
    $backupPath = $manifestPath + '.backup-' + [guid]::NewGuid().ToString('N')
    try {
        [IO.File]::WriteAllText(
            $temporaryPath,
            ($value | ConvertTo-Json -Depth 8),
            [Text.UTF8Encoding]::new($false))
        if ([IO.File]::Exists($manifestPath)) {
            [IO.File]::Replace($temporaryPath, $manifestPath, $backupPath)
        }
        else {
            [IO.File]::Move($temporaryPath, $manifestPath)
        }
    }
    finally {
        if ([IO.File]::Exists($temporaryPath)) {
            [IO.File]::Delete($temporaryPath)
        }
        if ([IO.File]::Exists($backupPath)) {
            [IO.File]::Delete($backupPath)
        }
    }
    return $manifestPath
}

function Read-MvpAcceptanceStagingTreeManifest {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumManifestBytes = $script:MvpAcceptanceStagingTreeManifestMaximumBytes,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumEntryCount = $script:MvpAcceptanceStagingTreeManifestMaximumEntries
    )

    $absoluteRoot = [IO.Path]::GetFullPath($StagingRoot)
    $rootPrefix = $absoluteRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    $manifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteRoot
    if (-not [IO.File]::Exists($manifestPath)) {
        throw "Acceptance staging tree '$absoluteRoot' is missing required 'staging-tree-manifest.json'."
    }
    $stream = $null
    $reader = $null
    try {
        $stream = [IO.File]::Open($manifestPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        if ($stream.Length -gt $MaximumManifestBytes) {
            throw "Acceptance staging tree manifest exceeds the manifest-byte budget of $MaximumManifestBytes."
        }
        $reader = [IO.StreamReader]::new($stream, [Text.UTF8Encoding]::new($false, $true), $true)
        $manifest = $reader.ReadToEnd() | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Acceptance staging tree manifest '$manifestPath' is not valid JSON: $($_.Exception.Message)"
    }
    finally {
        if ($null -ne $reader) {
            $reader.Dispose()
        }
        elseif ($null -ne $stream) {
            $stream.Dispose()
        }
    }
    if ($null -eq $manifest -or [Int64]$manifest.schema_version -ne 1 -or $null -eq $manifest.entries) {
        throw "Acceptance staging tree manifest '$manifestPath' has an unsupported schema."
    }
    $manifestEntries = $manifest.entries
    $manifestEntryCount = if ($manifestEntries -is [array]) {
        $manifestEntries.Length
    }
    else {
        1
    }
    if ($manifestEntryCount -gt $MaximumEntryCount) {
        throw "Acceptance staging tree manifest exceeds the entry-count budget of $MaximumEntryCount."
    }

    $seenPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $entries = [System.Collections.Generic.List[object]]::new()
    foreach ($entry in $manifestEntries) {
        if ($null -eq $entry -or [string]::IsNullOrWhiteSpace([string]$entry.path) -or
            [string]::IsNullOrWhiteSpace([string]$entry.kind)) {
            throw "Acceptance staging tree manifest '$manifestPath' contains an incomplete entry."
        }
        $relativePath = [string]$entry.path
        $normalizedRelativePath = $relativePath.Replace('\', '/')
        $resolvedPath = Resolve-MvpAcceptanceStagingTreeManifestNormalizedEntryPath `
            -AbsoluteRoot $absoluteRoot `
            -RootPrefix $rootPrefix `
            -NormalizedRelativePath $normalizedRelativePath
        if ($resolvedPath.Equals($manifestPath, [StringComparison]::OrdinalIgnoreCase) -or
            -not $seenPaths.Add($resolvedPath)) {
            throw "Acceptance staging tree manifest '$manifestPath' contains duplicate or reserved path '$relativePath'."
        }
        $kind = [string]$entry.kind
        if ($kind -cne 'file' -and $kind -cne 'directory') {
            throw "Acceptance staging tree manifest '$manifestPath' has unsupported entry kind '$kind'."
        }
        if ($kind -eq 'file' -and
            ($null -eq $entry.size_bytes -or [string]::IsNullOrWhiteSpace([string]$entry.sha256) -or
                [Int64]$entry.size_bytes -lt 0 -or [string]$entry.sha256 -notmatch '^[0-9A-F]{64}$')) {
            throw "Acceptance staging tree manifest '$manifestPath' has invalid file evidence for '$relativePath'."
        }
        $sortDepth = 1
        for ($index = 0; $index -lt $normalizedRelativePath.Length; $index++) {
            if ($normalizedRelativePath[$index] -eq '/') {
                $sortDepth++
            }
        }
        $entries.Add([pscustomobject]@{
                path = $resolvedPath
                relative_path = $normalizedRelativePath
                sort_depth = $sortDepth
                kind = $kind
                size_bytes = if ($kind -eq 'file') { [Int64]$entry.size_bytes } else { $null }
                sha256 = if ($kind -eq 'file') { [string]$entry.sha256 } else { $null }
            }) | Out-Null
    }
    $sortedEntries = @($entries | Sort-Object -Property sort_depth, relative_path)
    foreach ($sortedEntry in $sortedEntries) {
        $sortedEntry.PSObject.Properties.Remove('sort_depth')
    }
    return $sortedEntries
}

Export-ModuleMember -Function Get-MvpAcceptanceStagingTreeManifestPath, Resolve-MvpAcceptanceStagingTreeManifestEntryPath, Get-MvpAcceptanceStagingTreeManifestSha256, Write-MvpAcceptanceStagingTreeManifest, Read-MvpAcceptanceStagingTreeManifest
