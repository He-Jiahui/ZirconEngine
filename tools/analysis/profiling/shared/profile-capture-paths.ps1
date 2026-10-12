function ConvertTo-ZirconProfileSessionBasename {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$SessionId
    )

    $maxBasenameBytes = 96
    $hashSuffixBytes = 17
    $bytes = [Text.Encoding]::UTF8.GetBytes($SessionId)
    $readable = [Text.StringBuilder]::new($bytes.Length)
    foreach ($byte in $bytes) {
        if (
            ($byte -ge [byte][char]'0' -and $byte -le [byte][char]'9') -or
            ($byte -ge [byte][char]'A' -and $byte -le [byte][char]'Z') -or
            ($byte -ge [byte][char]'a' -and $byte -le [byte][char]'z') -or
            $byte -in @([byte][char]'-', [byte][char]'_', [byte][char]'.')
        ) {
            [void]$readable.Append([char]$byte)
        }
        else {
            [void]$readable.Append('_')
        }
    }

    $basename = $readable.ToString().Trim('.')
    if ([string]::IsNullOrEmpty($basename)) {
        $basename = 'session'
    }
    $stem = $basename.Split('.')[0]
    if (
        $stem -in @('CON', 'PRN', 'AUX', 'NUL') -or
        $stem -match '^(?:COM|LPT)[1-9]$'
    ) {
        $basename = "session_$basename"
    }
    $maxPrefixBytes = $maxBasenameBytes - $hashSuffixBytes
    if ($basename.Length -gt $maxPrefixBytes) {
        $basename = $basename.Substring(0, $maxPrefixBytes)
    }

    $modulus = [Numerics.BigInteger]::One -shl 64
    $hash = [Numerics.BigInteger]::Parse('14695981039346656037')
    $prime = [Numerics.BigInteger]::Parse('1099511628211')
    foreach ($byte in $bytes) {
        $hash = (($hash -bxor [int]$byte) * $prime) % $modulus
    }
    $hashHex = $hash.ToString('x').PadLeft(16, '0')
    if ($hashHex.Length -gt 16) {
        $hashHex = $hashHex.Substring($hashHex.Length - 16)
    }
    return '{0}-{1}' -f $basename, $hashHex
}

function Assert-ZirconProfilePathContainsNoReparsePoint {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$Candidate
    )

    $normalizedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd([char[]]"\/")
    $normalizedCandidate = [System.IO.Path]::GetFullPath($Candidate)
    $rootPrefix = $normalizedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $normalizedCandidate.Equals($normalizedRoot, [StringComparison]::OrdinalIgnoreCase) -and
        -not $normalizedCandidate.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Profile capture path must resolve beneath $normalizedRoot."
    }

    # Include ancestors above Root: an approved-looking root can itself sit below a junction.
    $componentPath = [System.IO.Path]::GetPathRoot($normalizedCandidate)
    $pathComponents = @($normalizedCandidate.Substring($componentPath.Length) -split '[\\/]' |
        Where-Object { -not [string]::IsNullOrEmpty($_) })
    foreach ($component in @('.') + $pathComponents) {
        if ($component -ne '.') {
            $componentPath = Join-Path $componentPath $component
        }
        try {
            $item = Get-Item -LiteralPath $componentPath -Force -ErrorAction Stop
        }
        catch [System.Management.Automation.ItemNotFoundException] {
            break
        }
        if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Profile capture path contains a reparse point: $componentPath"
        }
    }
}

function Assert-ZirconProfileChildName {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$PathSegment
    )

    if ([string]::IsNullOrWhiteSpace($PathSegment) -or
        [System.IO.Path]::IsPathRooted($PathSegment) -or
        $PathSegment -in @('.', '..') -or
        $PathSegment.Contains('\') -or
        $PathSegment.Contains('/')) {
        throw "Profile path component must be a plain child name: '$PathSegment'."
    }

    if ($PathSegment.IndexOfAny([System.IO.Path]::GetInvalidFileNameChars()) -ge 0 -or
        $PathSegment.EndsWith('.') -or $PathSegment.EndsWith(' ')) {
        throw "Profile path component must be a literal Windows child name: '$PathSegment'."
    }
    $stem = $PathSegment.Split('.')[0]
    if ($stem -match '^(?i:(?:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9]))$') {
        throw "Profile path component must not be a reserved Windows device name: '$PathSegment'."
    }
}

function Resolve-ZirconProfileContainedPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root,
        [Parameter(Mandatory = $true)]
        [string[]]$PathSegments
    )

    if ([string]::IsNullOrWhiteSpace($Root)) {
        throw 'Profile capture root must not be empty.'
    }
    if ($PathSegments.Count -eq 0) {
        throw 'Profile capture path requires at least one child component.'
    }

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root)
    $candidate = $resolvedRoot
    foreach ($pathSegment in $PathSegments) {
        Assert-ZirconProfileChildName -PathSegment $pathSegment
        $candidate = Join-Path $candidate $pathSegment
    }
    $resolvedCandidate = [System.IO.Path]::GetFullPath($candidate)
    Assert-ZirconProfilePathContainsNoReparsePoint `
        -Root $resolvedRoot `
        -Candidate $resolvedCandidate
    # Production profile roots also validate each derived directory/file, including hard links.
    # The generic containment helper retains its existing non-writing test-root contract.
    if ($resolvedRoot -match '^[D-Fd-f]:\\cargo-targets\\zircon-local(?:\\|$)') {
        $repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\..'))
        return Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path $resolvedCandidate
    }
    return $resolvedCandidate
}

function Resolve-ZirconProfileOutputRoot {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $boundaryError = "Profile output root must be an absolute literal path beneath physical D/E/F:\cargo-targets\zircon-local."
    if ([string]::IsNullOrWhiteSpace($Path) -or $Path -notmatch '^[D-Fd-f]:[\\/]') {
        throw $boundaryError
    }
    $pathComponents = @($Path.Substring(3) -split '[\\/]')
    for ($index = 0; $index -lt $pathComponents.Count; $index++) {
        if ($index -eq $pathComponents.Count - 1 -and $pathComponents[$index] -eq '') {
            continue
        }
        Assert-ZirconProfileChildName -PathSegment $pathComponents[$index]
    }
    $resolvedPath = [System.IO.Path]::GetFullPath($Path).TrimEnd([char[]]"\/")
    $profileRoot = $null
    foreach ($drive in @('D', 'E', 'F')) {
        $root = "${drive}:\cargo-targets\zircon-local"
        if ($resolvedPath.Equals($root, [StringComparison]::OrdinalIgnoreCase) -or
            $resolvedPath.StartsWith($root + '\', [StringComparison]::OrdinalIgnoreCase)) {
            $profileRoot = $root
            break
        }
    }
    if ($null -eq $profileRoot) {
        throw $boundaryError
    }
    $profileDrive = Get-PSDrive -Name ($resolvedPath.Substring(0, 1)) -PSProvider FileSystem -ErrorAction Stop
    $driveRoot = $resolvedPath.Substring(0, 3)
    if (-not $driveRoot.Equals([string]$profileDrive.Root, [StringComparison]::OrdinalIgnoreCase) -or
        -not [string]::IsNullOrEmpty([string]$profileDrive.DisplayRoot)) {
        throw "Profile output provider drive aliases are not allowed."
    }
    Assert-ZirconProfilePathContainsNoReparsePoint -Root $profileRoot -Candidate $Path

    # The interpreter is anchored to this source file, never to caller-controlled output data.
    $pythonPath = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\..\.jenkins\runtime\python\python.exe'))
    if (-not (Test-Path -LiteralPath $pythonPath -PathType Leaf)) {
        throw "Profile output validation requires the repository bundled Python: $pythonPath"
    }
    $physicalPolicy = @'
from pathlib import Path
import json
import stat
import sys

def inspect_ancestry(path):
    current = Path(path.anchor)
    for component in (None, *path.parts[1:]):
        if component is not None:
            current /= component
        try:
            identity = current.lstat()
            attributes = identity.st_file_attributes
        except FileNotFoundError:
            break
        if attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT:
            raise ValueError('Profile capture path contains a reparse point: ' + str(current))
        if stat.S_ISREG(identity.st_mode) and identity.st_nlink > 1:
            raise ValueError('Profile output file aliases are not allowed: ' + str(current))

try:
    requested = Path(sys.argv[1])
    if not requested.is_absolute() or any(p in ('.', '..') for p in sys.argv[1].replace('/', '\\').split('\\')[1:]):
        raise ValueError('Profile output path must be literal and absolute.')
    if not Path(requested.anchor).is_dir():
        raise ValueError('Profile output drive must exist physically.')
    inspect_ancestry(requested)
    physical = requested.resolve()
    inspect_ancestry(physical)
    if physical != requested:
        raise ValueError('Profile output path aliases are not allowed: ' + str(requested))
    roots = tuple(Path(d + ':\\cargo-targets\\zircon-local') for d in 'DEF')
    root = next((r for r in roots if requested == r or requested.is_relative_to(r)), None)
    if root is None:
        raise ValueError('Profile output path is outside the approved physical roots.')
    inspect_ancestry(root)
    physical_root = root.resolve()
    inspect_ancestry(physical_root)
    if physical_root != root:
        raise ValueError('Profile output root aliases are not allowed: ' + str(root))
    print(json.dumps({'schema_version': 1, 'ok': True, 'requested': str(requested), 'resolved': str(physical)}, ensure_ascii=True))
except (OSError, ValueError) as error:
    print(json.dumps({'schema_version': 1, 'ok': False, 'error': str(error)}, ensure_ascii=True))
    sys.exit(1)
'@
    $pythonOutput = @(& $pythonPath -B -X utf8 -c $physicalPolicy $Path)
    $pythonExitCode = $LASTEXITCODE
    if ($pythonOutput.Count -ne 1) {
        throw "Profile output validation returned an invalid receipt; exit code $pythonExitCode."
    }
    try {
        $receipt = [string]$pythonOutput[0] | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Profile output validation returned an unreadable receipt; exit code $pythonExitCode."
    }
    if ($pythonExitCode -ne 0 -or $receipt.schema_version -ne 1 -or
        $receipt.ok -isnot [bool] -or -not $receipt.ok) {
        throw "Profile output validation failed; exit code ${pythonExitCode}: $($receipt.error)"
    }
    if (-not $resolvedPath.Equals([string]$receipt.requested, [StringComparison]::OrdinalIgnoreCase) -or
        -not $resolvedPath.Equals([string]$receipt.resolved, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Profile output validation returned a different physical path."
    }
    Assert-ZirconProfilePathContainsNoReparsePoint -Root $profileRoot -Candidate ([string]$receipt.resolved)
    return $resolvedPath
}
