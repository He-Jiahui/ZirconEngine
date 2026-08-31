Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:MvpAcceptanceProjectionUpperHexDigits = [char[]]'0123456789ABCDEF'
$script:MvpAcceptanceProjectionDirectoryDescriptor = [Tuple[bool, Nullable[Int64], string]]::new(
    $true,
    $null,
    [Management.Automation.Language.NullString]::Value)

function Get-MvpAcceptanceProjectionNormalizedRelativePath {
    param(
        [Parameter(Mandatory)][string]$AbsoluteRoot,
        [Parameter(Mandatory)][string]$RootPrefix,
        [Parameter(Mandatory)][string]$AbsolutePath
    )

    if (-not $AbsolutePath.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance projection path '$AbsolutePath' escapes root '$AbsoluteRoot'."
    }
    return $AbsolutePath.Substring($RootPrefix.Length).Replace('\', '/')
}

function Get-MvpAcceptanceProjectionRelativePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    $absoluteRoot = [IO.Path]::GetFullPath($Root).TrimEnd([char[]]@('\', '/'))
    $absolutePath = [IO.Path]::GetFullPath($Path)
    $prefix = $absoluteRoot + [IO.Path]::DirectorySeparatorChar
    return Get-MvpAcceptanceProjectionNormalizedRelativePath `
        -AbsoluteRoot $absoluteRoot `
        -RootPrefix $prefix `
        -AbsolutePath $absolutePath
}

function ConvertTo-MvpAcceptanceProjectionUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpAcceptanceProjectionUpperHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpAcceptanceProjectionUpperHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpAcceptanceProjectionFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpAcceptanceProjectionUpperHex -Bytes $hasher.ComputeHash($stream)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-MvpAcceptanceProjectionBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpAcceptanceProjectionUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpAcceptanceProjectionDescriptor {
    param(
        [string]$Path,
        [IO.FileSystemInfo]$Item
    )

    if ($null -eq $Item) {
        $Item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop
    }
    if ([bool]($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Acceptance projection path '$($item.FullName)' is a reparse point."
    }
    if ($item -is [IO.DirectoryInfo]) {
        return $script:MvpAcceptanceProjectionDirectoryDescriptor
    }
    return [Tuple[bool, Nullable[Int64], string]]::new(
        $false,
        [Int64]$item.Length,
        (Get-MvpAcceptanceProjectionFileSha256 -Path $item.FullName))
}

function Add-MvpAcceptanceProjectionDescriptor {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)][Tuple[bool, Nullable[Int64], string]]$Descriptor
    )

    $existing = $Projection.entries[$RelativePath]
    if ($null -ne $existing) {
        if ($existing.Item1 -eq $Descriptor.Item1 -and
            $existing.Item2 -eq $Descriptor.Item2 -and
            $existing.Item3 -eq $Descriptor.Item3) {
            return
        }
        throw "Acceptance projection entry '$RelativePath' has conflicting descriptors."
    }
    $Projection.entries.Add($RelativePath, $Descriptor)
}

function New-MvpAcceptanceStagingProjection {
    param([Parameter(Mandatory)][string]$Root)

    $absoluteRoot = [IO.Path]::GetFullPath($Root)
    $rootPrefix = $absoluteRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    return [pscustomobject]@{
        root = $absoluteRoot
        root_prefix = $rootPrefix
        entries = [System.Collections.Generic.Dictionary[
                string, Tuple[bool, Nullable[Int64], string]]]::new(
            [StringComparer]::OrdinalIgnoreCase)
    }
}

function Add-MvpAcceptanceStagingProjectionSourceEntry {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath,
        [Parameter(Mandatory)][bool]$IsDirectory
    )

    $descriptor = Get-MvpAcceptanceProjectionDescriptor -Path $SourcePath
    if ($descriptor.Item1 -ne $IsDirectory) {
        throw "Acceptance source '$SourcePath' changed its directory shape while being copied."
    }
    $absoluteDestinationPath = [IO.Path]::GetFullPath($DestinationPath)
    $relativePath = Get-MvpAcceptanceProjectionNormalizedRelativePath `
        -AbsoluteRoot $Projection.root `
        -RootPrefix $Projection.root_prefix `
        -AbsolutePath $absoluteDestinationPath
    Add-MvpAcceptanceProjectionDescriptor `
        -Projection $Projection `
        -RelativePath $relativePath `
        -Descriptor $descriptor
}

function Add-MvpAcceptanceStagingProjectionOwnedPath {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$Path
    )

    $absoluteRoot = [string]$Projection.root
    $currentPath = [IO.Path]::GetFullPath($Path)
    while (-not $currentPath.Equals($absoluteRoot, [StringComparison]::OrdinalIgnoreCase)) {
        $descriptor = Get-MvpAcceptanceProjectionDescriptor -Path $currentPath
        $relativePath = Get-MvpAcceptanceProjectionNormalizedRelativePath `
            -AbsoluteRoot $absoluteRoot `
            -RootPrefix $Projection.root_prefix `
            -AbsolutePath $currentPath
        Add-MvpAcceptanceProjectionDescriptor `
            -Projection $Projection `
            -RelativePath $relativePath `
            -Descriptor $descriptor
        $parent = [IO.Directory]::GetParent($currentPath)
        if ($null -eq $parent) {
            throw "Acceptance projection path '$currentPath' has no parent below '$absoluteRoot'."
        }
        $currentPath = $parent.FullName
    }
}

function Add-MvpAcceptanceStagingProjectionOwnedFile {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][byte[]]$ContentBytes
    )

    $absoluteRoot = [string]$Projection.root
    $currentPath = [IO.Path]::GetFullPath($Path)
    $relativePath = Get-MvpAcceptanceProjectionNormalizedRelativePath `
        -AbsoluteRoot $absoluteRoot `
        -RootPrefix $Projection.root_prefix `
        -AbsolutePath $currentPath
    Add-MvpAcceptanceProjectionDescriptor `
        -Projection $Projection `
        -RelativePath $relativePath `
        -Descriptor ([Tuple[bool, Nullable[Int64], string]]::new(
            $false,
            [Int64]$ContentBytes.LongLength,
            (Get-MvpAcceptanceProjectionBytesSha256 -Bytes $ContentBytes)))

    $currentPath = [IO.Directory]::GetParent($currentPath).FullName
    while (-not $currentPath.Equals($absoluteRoot, [StringComparison]::OrdinalIgnoreCase)) {
        $relativePath = Get-MvpAcceptanceProjectionNormalizedRelativePath `
            -AbsoluteRoot $absoluteRoot `
            -RootPrefix $Projection.root_prefix `
            -AbsolutePath $currentPath
        Add-MvpAcceptanceProjectionDescriptor `
            -Projection $Projection `
            -RelativePath $relativePath `
            -Descriptor $script:MvpAcceptanceProjectionDirectoryDescriptor
        $parent = [IO.Directory]::GetParent($currentPath)
        if ($null -eq $parent) {
            throw "Acceptance projection path '$currentPath' has no parent below '$absoluteRoot'."
        }
        $currentPath = $parent.FullName
    }
}

function Assert-MvpAcceptanceStagingProjection {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)]$Projection,
        [string[]]$ExcludedPaths = @()
    )

    $absoluteRoot = [IO.Path]::GetFullPath($Root)
    if (-not $absoluteRoot.Equals($Projection.root, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance projection root '$absoluteRoot' does not match '$($Projection.root)'."
    }
    $rootPrefix = [string]$Projection.root_prefix
    if ($rootPrefix.Length -ne $absoluteRoot.TrimEnd([char[]]@('\', '/')).Length + 1 -or
        -not $rootPrefix.StartsWith($absoluteRoot.TrimEnd([char[]]@('\', '/')), [StringComparison]::OrdinalIgnoreCase) -or
        $rootPrefix[$rootPrefix.Length - 1] -ne [IO.Path]::DirectorySeparatorChar) {
        throw "Acceptance projection root prefix '$rootPrefix' does not match '$absoluteRoot'."
    }
    $excluded = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase)
    foreach ($path in $ExcludedPaths) {
        if (-not [string]::IsNullOrWhiteSpace($path)) {
            $null = $excluded.Add([IO.Path]::GetFullPath($path))
        }
    }

    $actualPaths = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase)
    $directories = [System.Collections.Generic.Stack[IO.DirectoryInfo]]::new()
    $directories.Push([IO.DirectoryInfo]::new($absoluteRoot))
    while ($directories.Count -gt 0) {
        $directory = $directories.Pop()
        $enumerator = $directory.EnumerateFileSystemInfos().GetEnumerator()
        try {
            while ($enumerator.MoveNext()) {
                $item = $enumerator.Current
                if ([bool]($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                    throw "Acceptance projection path '$($item.FullName)' is a reparse point."
                }
                if ($item -is [IO.DirectoryInfo]) {
                    $directories.Push([IO.DirectoryInfo]$item)
                }
                $absolutePath = $item.FullName
                if (-not $absolutePath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "Acceptance projection path '$absolutePath' escapes root '$absoluteRoot'."
                }
                if ($excluded.Contains($absolutePath)) {
                    continue
                }
                $relativePath = $absolutePath.Substring($rootPrefix.Length).Replace('\', '/')
                $expected = $Projection.entries[$relativePath]
                if ($null -eq $expected) {
                    throw "Acceptance projection contains unexpected entry '$relativePath'."
                }
                $actualIsDirectory = $item -is [IO.DirectoryInfo]
                if ($actualIsDirectory -ne $expected.Item1 -or
                    ($actualIsDirectory -and ($null -ne $expected.Item2 -or $null -ne $expected.Item3)) -or
                    (-not $actualIsDirectory -and [Int64]$item.Length -ne [Int64]$expected.Item2)) {
                    throw "Acceptance projection entry '$relativePath' differs from its expected source or generated output."
                }
                if (-not $actualIsDirectory -and
                    (Get-MvpAcceptanceProjectionFileSha256 -Path $absolutePath) -ne $expected.Item3) {
                    throw "Acceptance projection entry '$relativePath' differs from its expected source or generated output."
                }
                $null = $actualPaths.Add($relativePath)
            }
        }
        finally {
            $enumerator.Dispose()
        }
    }

    foreach ($relativePath in $Projection.entries.Keys) {
        if (-not $actualPaths.Contains($relativePath)) {
            throw "Acceptance projection is missing expected entry '$relativePath'."
        }
    }
}

Export-ModuleMember -Function New-MvpAcceptanceStagingProjection, Add-MvpAcceptanceStagingProjectionSourceEntry, Add-MvpAcceptanceStagingProjectionOwnedFile, Assert-MvpAcceptanceStagingProjection
