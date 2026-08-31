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
        [Parameter(Mandatory = $true)]
        [string]$Root,
        [Parameter(Mandatory = $true)]
        [string]$Candidate
    )

    $normalizedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd('\\')
    $normalizedCandidate = [System.IO.Path]::GetFullPath($Candidate)
    $rootPrefix = $normalizedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $normalizedCandidate.Equals($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase) -and
        -not $normalizedCandidate.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Profile capture path must resolve beneath $normalizedRoot."
    }

    $componentPath = $normalizedRoot
    $relativePath = [System.IO.Path]::GetRelativePath($normalizedRoot, $normalizedCandidate)
    $pathComponents = if ($relativePath -eq '.') {
        @()
    }
    else {
        @($relativePath -split '[\\/]' | Where-Object { -not [string]::IsNullOrEmpty($_) })
    }
    foreach ($component in @('.') + $pathComponents) {
        if ($component -ne '.') {
            $componentPath = Join-Path $componentPath $component
        }
        if (-not (Test-Path -LiteralPath $componentPath)) {
            break
        }
        $attributes = (Get-Item -LiteralPath $componentPath -Force).Attributes
        if (($attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
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
    return $resolvedCandidate
}

function Resolve-ZirconProfileOutputRoot {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot,
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        throw "Profile output root must not be empty."
    }

    $resolvedPath = if ([System.IO.Path]::IsPathRooted($Path)) {
        [System.IO.Path]::GetFullPath($Path)
    }
    else {
        [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $Path))
    }
    $profileRoot = [System.IO.Path]::GetFullPath("E:\zircon-profiles").TrimEnd('\\')
    $profileRootWithSeparator = $profileRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $resolvedPath.Equals($profileRoot, [System.StringComparison]::OrdinalIgnoreCase) -and
        -not $resolvedPath.StartsWith($profileRootWithSeparator, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Profile output root must resolve beneath E:\zircon-profiles."
    }

    Assert-ZirconProfilePathContainsNoReparsePoint -Root $profileRoot -Candidate $resolvedPath

    return $resolvedPath
}
