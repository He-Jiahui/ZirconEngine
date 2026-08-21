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

    return $resolvedPath
}
