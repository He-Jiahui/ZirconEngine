Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-MvpAcceptanceProjectionRelativePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    $absoluteRoot = [IO.Path]::GetFullPath($Root).TrimEnd([char[]]@('\', '/'))
    $absolutePath = [IO.Path]::GetFullPath($Path)
    $prefix = $absoluteRoot + [IO.Path]::DirectorySeparatorChar
    if (-not $absolutePath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Acceptance projection path '$absolutePath' escapes root '$absoluteRoot'."
    }
    return $absolutePath.Substring($prefix.Length).Replace('\', '/')
}

function Get-MvpAcceptanceProjectionFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') })
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
        return -join ($hasher.ComputeHash($Bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpAcceptanceProjectionDescriptor {
    param([Parameter(Mandatory)][string]$Path)

    $item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop
    if ([bool]($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Acceptance projection path '$($item.FullName)' is a reparse point."
    }
    if ($item.PSIsContainer) {
        return [pscustomobject]@{
            is_directory = $true
            size_bytes = $null
            sha256 = $null
        }
    }
    return [pscustomobject]@{
        is_directory = $false
        size_bytes = [Int64]$item.Length
        sha256 = Get-MvpAcceptanceProjectionFileSha256 -Path $item.FullName
    }
}

function Add-MvpAcceptanceProjectionDescriptor {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)]$Descriptor
    )

    $existing = $Projection.entries[$RelativePath]
    if ($null -ne $existing) {
        if ($existing.is_directory -eq $Descriptor.is_directory -and
            $existing.size_bytes -eq $Descriptor.size_bytes -and
            $existing.sha256 -eq $Descriptor.sha256) {
            return
        }
        throw "Acceptance projection entry '$RelativePath' has conflicting descriptors."
    }
    $Projection.entries.Add($RelativePath, $Descriptor)
}

function New-MvpAcceptanceStagingProjection {
    param([Parameter(Mandatory)][string]$Root)

    return [pscustomobject]@{
        root = [IO.Path]::GetFullPath($Root)
        entries = [System.Collections.Generic.Dictionary[string, object]]::new(
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
    if ($descriptor.is_directory -ne $IsDirectory) {
        throw "Acceptance source '$SourcePath' changed its directory shape while being copied."
    }
    $relativePath = Get-MvpAcceptanceProjectionRelativePath `
        -Root $Projection.root `
        -Path $DestinationPath
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

    $absoluteRoot = [IO.Path]::GetFullPath($Projection.root).TrimEnd([char[]]@('\', '/'))
    $currentPath = [IO.Path]::GetFullPath($Path)
    while (-not $currentPath.Equals($absoluteRoot, [StringComparison]::OrdinalIgnoreCase)) {
        $descriptor = Get-MvpAcceptanceProjectionDescriptor -Path $currentPath
        $relativePath = Get-MvpAcceptanceProjectionRelativePath `
            -Root $absoluteRoot `
            -Path $currentPath
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

    $absoluteRoot = [IO.Path]::GetFullPath($Projection.root).TrimEnd([char[]]@('\', '/'))
    $currentPath = [IO.Path]::GetFullPath($Path)
    $relativePath = Get-MvpAcceptanceProjectionRelativePath -Root $absoluteRoot -Path $currentPath
    Add-MvpAcceptanceProjectionDescriptor `
        -Projection $Projection `
        -RelativePath $relativePath `
        -Descriptor ([pscustomobject]@{
            is_directory = $false
            size_bytes = [Int64]$ContentBytes.LongLength
            sha256 = Get-MvpAcceptanceProjectionBytesSha256 -Bytes $ContentBytes
        })

    $currentPath = [IO.Directory]::GetParent($currentPath).FullName
    while (-not $currentPath.Equals($absoluteRoot, [StringComparison]::OrdinalIgnoreCase)) {
        $relativePath = Get-MvpAcceptanceProjectionRelativePath -Root $absoluteRoot -Path $currentPath
        Add-MvpAcceptanceProjectionDescriptor `
            -Projection $Projection `
            -RelativePath $relativePath `
            -Descriptor ([pscustomobject]@{
                is_directory = $true
                size_bytes = $null
                sha256 = $null
            })
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
    $excluded = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase)
    foreach ($path in @($ExcludedPaths)) {
        if (-not [string]::IsNullOrWhiteSpace($path)) {
            $null = $excluded.Add([IO.Path]::GetFullPath($path))
        }
    }

    $actualPaths = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase)
    foreach ($item in @(Get-ChildItem -LiteralPath $absoluteRoot -Recurse -Force -ErrorAction Stop)) {
        $absolutePath = [IO.Path]::GetFullPath($item.FullName)
        if ($excluded.Contains($absolutePath)) {
            continue
        }
        $relativePath = Get-MvpAcceptanceProjectionRelativePath -Root $absoluteRoot -Path $absolutePath
        $expected = $Projection.entries[$relativePath]
        if ($null -eq $expected) {
            throw "Acceptance projection contains unexpected entry '$relativePath'."
        }
        $actual = Get-MvpAcceptanceProjectionDescriptor -Path $absolutePath
        if ($actual.is_directory -ne $expected.is_directory -or
            $actual.size_bytes -ne $expected.size_bytes -or
            $actual.sha256 -ne $expected.sha256) {
            throw "Acceptance projection entry '$relativePath' differs from its expected source or generated output."
        }
        $null = $actualPaths.Add($relativePath)
    }

    foreach ($relativePath in $Projection.entries.Keys) {
        if (-not $actualPaths.Contains($relativePath)) {
            throw "Acceptance projection is missing expected entry '$relativePath'."
        }
    }
}

Export-ModuleMember -Function New-MvpAcceptanceStagingProjection, Add-MvpAcceptanceStagingProjectionSourceEntry, Add-MvpAcceptanceStagingProjectionOwnedFile, Assert-MvpAcceptanceStagingProjection
