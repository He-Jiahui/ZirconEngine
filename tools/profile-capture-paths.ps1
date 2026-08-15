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
