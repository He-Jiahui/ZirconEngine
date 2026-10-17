$ErrorActionPreference = 'Stop'

$outputRoot = [System.IO.Path]::GetFullPath($env:ZR_CI_OUTPUT_ROOT).TrimEnd('\')
$runPrefix = 'E:\cargo-targets\zircon-local\ci\runs\' + $env:GITHUB_RUN_ID + '-' + $env:GITHUB_RUN_ATTEMPT + '\'
if (-not $outputRoot.StartsWith($runPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Zig output must belong to this isolated CI run.'
}
$archiveName = 'zig-x86_64-windows-0.14.1.zip'
$archiveUrl = 'https://ziglang.org/download/0.14.1/zig-x86_64-windows-0.14.1.zip'
$archiveSha256 = '554f5378228923ffd558eac35e21af020c73789d87afeabf4bfd16f2e6feed2c'
$archiveBytes = 82229343L
$installName = 'zig-0.14.1'
$archivePath = Join-Path $outputRoot $archiveName
$partialPath = Join-Path $outputRoot ($archiveName + '.partial')
$installPath = Join-Path $outputRoot $installName
$stagingPath = Join-Path $outputRoot ($installName + '.extracting')
$zigPath = Join-Path $installPath 'zig.exe'

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.ZipFile

function Get-FullPath([string]$Path) {
    return [System.IO.Path]::GetFullPath($Path).TrimEnd('\')
}

function Assert-NoReparseAncestors([string]$Path) {
    $fullPath = Get-FullPath $Path
    if (-not $fullPath.StartsWith(($outputRoot + '\'), [System.StringComparison]::OrdinalIgnoreCase) -and
        -not $fullPath.Equals($outputRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Zig output is outside the registered E: CI root: $fullPath"
    }

    $current = 'E:\'
    foreach ($segment in $fullPath.Substring(3).Split('\')) {
        if ([string]::IsNullOrWhiteSpace($segment)) { continue }
        $current = Join-Path $current $segment
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Zig output path contains a reparse point: $current"
            }
        }
    }
}

function Get-EntryHash([System.IO.Compression.ZipArchiveEntry]$Entry) {
    $stream = $Entry.Open()
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            return [Convert]::ToHexString($sha.ComputeHash($stream)).ToLowerInvariant()
        } finally {
            $sha.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

function Get-ZipFiles([System.IO.Compression.ZipArchive]$Archive) {
    $prefix = 'zig-x86_64-windows-0.14.1/'
    $files = @{}
    foreach ($entry in $Archive.Entries) {
        $unixMode = ($entry.ExternalAttributes -shr 16) -band 0xF000
        if ($unixMode -eq 0xA000) { throw "Pinned Zig archive contains a symbolic link: $($entry.FullName)" }
        if (-not $entry.FullName.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
            throw "Unexpected entry in pinned Zig archive: $($entry.FullName)"
        }
        if ($entry.FullName -eq $prefix) { continue }
        $relative = $entry.FullName.Substring($prefix.Length).Replace('/', '\')
        if ([string]::IsNullOrWhiteSpace($relative) -or [System.IO.Path]::IsPathRooted($relative) -or
            $relative.Split('\') -contains '..') {
            throw "Unsafe path in pinned Zig archive: $($entry.FullName)"
        }
        if ($entry.FullName.EndsWith('/')) { continue }
        if ($files.ContainsKey($relative)) { throw "Duplicate path in pinned Zig archive: $($entry.FullName)" }
        $files[$relative] = $entry
    }
    if (-not $files.ContainsKey('zig.exe')) { throw 'Pinned Zig archive has no expected zig.exe.' }
    return $files
}

function Assert-InstallMatchesArchive([string]$Directory, [hashtable]$ExpectedFiles) {
    Assert-NoReparseAncestors $Directory
    $actualFiles = @{}
    $pendingDirectories = [System.Collections.Generic.Stack[string]]::new()
    $pendingDirectories.Push($Directory)
    while ($pendingDirectories.Count -gt 0) {
        $currentDirectory = $pendingDirectories.Pop()
        foreach ($item in Get-ChildItem -LiteralPath $currentDirectory -Force) {
            if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Zig installation contains a reparse point: $($item.FullName)"
            }
            if ($item.PSIsContainer) {
                $pendingDirectories.Push($item.FullName)
            } else {
                $relative = [System.IO.Path]::GetRelativePath($Directory, $item.FullName)
                $actualFiles[$relative] = $item.FullName
            }
        }
    }
    if ($actualFiles.Count -ne $ExpectedFiles.Count) {
        throw "Cached Zig installation file count differs from the verified archive: $Directory"
    }
    foreach ($relative in $ExpectedFiles.Keys) {
        if (-not $actualFiles.ContainsKey($relative)) {
            throw "Cached Zig installation is missing $relative"
        }
        $expectedHash = Get-EntryHash $ExpectedFiles[$relative]
        $actualHash = (Get-FileHash -LiteralPath $actualFiles[$relative] -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualHash -ne $expectedHash) { throw "Cached Zig installation failed integrity verification: $relative" }
    }
}

$resolvedRoot = Get-FullPath $outputRoot
if ($resolvedRoot -ne $outputRoot) { throw "Unexpected Zig output root: $resolvedRoot" }
$drive = [System.IO.DriveInfo]::new('E:\')
if (-not $drive.IsReady -or $drive.DriveType -ne [System.IO.DriveType]::Fixed) {
    throw 'The required E: output volume is not a ready fixed drive.'
}
$substitutions = & subst.exe 2>$null
if ($LASTEXITCODE -ne 0 -or ($substitutions -match '(?im)^E:\\:\s*=>')) {
    throw 'The E: output root is unavailable or aliases another path through SUBST.'
}
Assert-NoReparseAncestors $outputRoot
Assert-NoReparseAncestors $archivePath
Assert-NoReparseAncestors $partialPath
Assert-NoReparseAncestors $installPath
Assert-NoReparseAncestors $stagingPath

if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf) -or
    (Get-Item -LiteralPath $archivePath).Length -ne $archiveBytes -or
    (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $archiveSha256) {
    Assert-NoReparseAncestors $partialPath
    Invoke-WebRequest -Uri $archiveUrl -OutFile $partialPath
    if ((Get-Item -LiteralPath $partialPath).Length -ne $archiveBytes -or
        (Get-FileHash -LiteralPath $partialPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $archiveSha256) {
        throw 'Downloaded Zig 0.14.1 archive failed its pinned size or SHA-256 check.'
    }
    Move-Item -LiteralPath $partialPath -Destination $archivePath -Force
}
Assert-NoReparseAncestors $archivePath

$zip = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
$createdStage = $false
try {
    $expectedFiles = Get-ZipFiles $zip
    if (Test-Path -LiteralPath $installPath) {
        if (-not (Test-Path -LiteralPath $installPath -PathType Container)) {
            throw "Pinned Zig installation path exists but is not a directory: $installPath"
        }
        Assert-InstallMatchesArchive $installPath $expectedFiles
    } else {
        if (Test-Path -LiteralPath $stagingPath) {
            if (-not (Test-Path -LiteralPath $stagingPath -PathType Container)) {
                throw "Prior Zig extraction path is not a directory: $stagingPath"
            }
            Assert-InstallMatchesArchive $stagingPath $expectedFiles
            Move-Item -LiteralPath $stagingPath -Destination $installPath
        } else {
            Assert-NoReparseAncestors $stagingPath
            New-Item -ItemType Directory -Path $stagingPath | Out-Null
            $createdStage = $true
            foreach ($relative in $expectedFiles.Keys) {
                $destination = Join-Path $stagingPath $relative
                $fullDestination = Get-FullPath $destination
                if (-not $fullDestination.StartsWith(($stagingPath + '\'), [System.StringComparison]::OrdinalIgnoreCase)) {
                    throw "Zig archive entry escapes its staging directory: $relative"
                }
                $parent = Split-Path -Parent $destination
                New-Item -ItemType Directory -Force -Path $parent | Out-Null
                Assert-NoReparseAncestors $destination
                $input = $expectedFiles[$relative].Open()
                try {
                    $output = [System.IO.File]::Open($destination, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
                    try { $input.CopyTo($output) } finally { $output.Dispose() }
                } finally { $input.Dispose() }
            }
            Assert-InstallMatchesArchive $stagingPath $expectedFiles
            Move-Item -LiteralPath $stagingPath -Destination $installPath
            $createdStage = $false
        }
    }
} finally {
    $zip.Dispose()
    if ($createdStage -and (Test-Path -LiteralPath $stagingPath)) {
        Assert-NoReparseAncestors $stagingPath
        Remove-Item -LiteralPath $stagingPath -Recurse -Force
    }
}

if (-not (Test-Path -LiteralPath $zigPath -PathType Leaf)) { throw "Verified Zig executable is missing: $zigPath" }
Assert-NoReparseAncestors $zigPath
Assert-NoReparseAncestors $env:ZIG_LOCAL_CACHE_DIR
Assert-NoReparseAncestors $env:ZIG_GLOBAL_CACHE_DIR
New-Item -ItemType Directory -Force -Path $env:ZIG_LOCAL_CACHE_DIR, $env:ZIG_GLOBAL_CACHE_DIR | Out-Null
Assert-NoReparseAncestors $env:ZIG_LOCAL_CACHE_DIR
Assert-NoReparseAncestors $env:ZIG_GLOBAL_CACHE_DIR
Write-Output "Verified Zig 0.14.1 C/C++ toolchain at $zigPath"
