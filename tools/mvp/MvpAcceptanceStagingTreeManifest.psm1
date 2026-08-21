Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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

function Resolve-MvpAcceptanceStagingTreeManifestEntryPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$RelativePath
    )

    if ([string]::IsNullOrWhiteSpace($RelativePath) -or
        [IO.Path]::IsPathRooted($RelativePath) -or
        $RelativePath.Contains(':')) {
        throw "Acceptance staging tree manifest path '$RelativePath' is not a relative path."
    }
    $segments = @($RelativePath.Replace('/', '\').Split('\') | Where-Object { $_ -ne '' })
    if ($segments.Count -eq 0 -or $segments.Count -ne $RelativePath.Replace('/', '\').Split('\').Count -or
        $segments -contains '.' -or $segments -contains '..') {
        throw "Acceptance staging tree manifest path '$RelativePath' is not normalized."
    }

    $absoluteRoot = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
    $candidate = [IO.Path]::GetFullPath([IO.Path]::Combine(
            $absoluteRoot,
            ($segments -join [IO.Path]::DirectorySeparatorChar)))
    $prefix = $absoluteRoot + [IO.Path]::DirectorySeparatorChar
    if (-not $candidate.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance staging tree manifest path '$RelativePath' escapes root '$absoluteRoot'."
    }
    return $candidate
}

function Get-MvpAcceptanceStagingTreeManifestSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-MvpAcceptanceStagingTreeManifestEntries {
    param([Parameter(Mandatory)][string]$StagingRoot)

    $absoluteRoot = [IO.Path]::GetFullPath($StagingRoot)
    $manifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteRoot
    $manifestEntries = [System.Collections.Generic.List[object]]::new()
    $directories = [System.Collections.Generic.Queue[string]]::new()
    $directories.Enqueue($absoluteRoot)
    while ($directories.Count -gt 0) {
        $directoryPath = $directories.Dequeue()
        foreach ($child in @(Get-ChildItem -LiteralPath $directoryPath -Force -ErrorAction Stop)) {
            if ([bool]($child.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                throw "Acceptance staging tree contains reparse point '$($child.FullName)'."
            }
            $childPath = [IO.Path]::GetFullPath($child.FullName)
            if ($childPath.Equals($manifestPath, [StringComparison]::OrdinalIgnoreCase)) {
                continue
            }
            $relativePath = ConvertTo-MvpAcceptanceStagingTreeManifestRelativePath `
                -Root $absoluteRoot `
                -Path $childPath
            if ($child.PSIsContainer) {
                $manifestEntries.Add([ordered]@{
                    path = $relativePath
                    kind = 'directory'
                }) | Out-Null
                $directories.Enqueue($childPath)
            }
            else {
                $manifestEntries.Add([ordered]@{
                    path = $relativePath
                    kind = 'file'
                    size_bytes = [Int64]$child.Length
                    sha256 = Get-MvpAcceptanceStagingTreeManifestSha256 -Path $childPath
                }) | Out-Null
            }
        }
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
    param([Parameter(Mandatory)][string]$StagingRoot)

    $absoluteRoot = [IO.Path]::GetFullPath($StagingRoot)
    $manifestPath = Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot $absoluteRoot
    if (-not [IO.File]::Exists($manifestPath)) {
        throw "Acceptance staging tree '$absoluteRoot' is missing required 'staging-tree-manifest.json'."
    }
    $stream = $null
    $reader = $null
    try {
        $stream = [IO.File]::Open($manifestPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
        $reader = [IO.StreamReader]::new($stream, [Text.UTF8Encoding]::new($false), $true)
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

    $seenPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $entries = [System.Collections.Generic.List[object]]::new()
    foreach ($entry in @($manifest.entries)) {
        if ($null -eq $entry -or [string]::IsNullOrWhiteSpace([string]$entry.path) -or
            [string]::IsNullOrWhiteSpace([string]$entry.kind)) {
            throw "Acceptance staging tree manifest '$manifestPath' contains an incomplete entry."
        }
        $relativePath = [string]$entry.path
        $resolvedPath = Resolve-MvpAcceptanceStagingTreeManifestEntryPath -Root $absoluteRoot -RelativePath $relativePath
        if ($resolvedPath.Equals($manifestPath, [StringComparison]::OrdinalIgnoreCase) -or
            -not $seenPaths.Add($resolvedPath)) {
            throw "Acceptance staging tree manifest '$manifestPath' contains duplicate or reserved path '$relativePath'."
        }
        $kind = [string]$entry.kind
        if ($kind -notin @('file', 'directory')) {
            throw "Acceptance staging tree manifest '$manifestPath' has unsupported entry kind '$kind'."
        }
        if ($kind -eq 'file' -and
            ($null -eq $entry.size_bytes -or [string]::IsNullOrWhiteSpace([string]$entry.sha256) -or
                [Int64]$entry.size_bytes -lt 0 -or [string]$entry.sha256 -notmatch '^[0-9A-F]{64}$')) {
            throw "Acceptance staging tree manifest '$manifestPath' has invalid file evidence for '$relativePath'."
        }
        $entries.Add([pscustomobject]@{
                path = $resolvedPath
                relative_path = $relativePath.Replace('\', '/')
                kind = $kind
                size_bytes = if ($kind -eq 'file') { [Int64]$entry.size_bytes } else { $null }
                sha256 = if ($kind -eq 'file') { [string]$entry.sha256 } else { $null }
            }) | Out-Null
    }
    return @($entries | Sort-Object -Property { $_.relative_path.Split('/').Count }, relative_path)
}

Export-ModuleMember -Function Get-MvpAcceptanceStagingTreeManifestPath, Resolve-MvpAcceptanceStagingTreeManifestEntryPath, Get-MvpAcceptanceStagingTreeManifestSha256, Write-MvpAcceptanceStagingTreeManifest, Read-MvpAcceptanceStagingTreeManifest
