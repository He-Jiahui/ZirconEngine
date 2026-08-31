function Get-ZirconUiProfileManagedCargoTargetRoots {
    return @(
        "D:\cargo-targets",
        "E:\cargo-targets",
        "F:\cargo-targets",
        "D:\targets",
        "E:\targets",
        "F:\targets"
    )
}

function Get-ZirconUiProfileManagedProductRoots {
    return @(
        "D:\ZirconBuilds",
        "E:\ZirconBuilds",
        "F:\ZirconBuilds"
    )
}

function Resolve-ZirconUiProfileManagedChildPath {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string[]]$ManagedRoots,
        [Parameter(Mandatory = $true)]
        [string]$ErrorMessage
    )

    try {
        if ([string]::IsNullOrWhiteSpace($Path) -or
            -not [System.IO.Path]::IsPathRooted($Path)) {
            throw $ErrorMessage
        }
        $candidate = [System.IO.Path]::GetFullPath($Path).TrimEnd([char[]]"\/")
    }
    catch {
        throw $ErrorMessage
    }

    $managedRoot = $ManagedRoots |
        ForEach-Object { [System.IO.Path]::GetFullPath($_).TrimEnd([char[]]"\/") } |
        Where-Object {
            $rootPrefix = $_ + [System.IO.Path]::DirectorySeparatorChar
            $candidate.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)
        } |
        Select-Object -First 1
    if ($null -eq $managedRoot) {
        throw $ErrorMessage
    }

    if ($null -eq (Get-Command Assert-ZirconProfilePathContainsNoReparsePoint -ErrorAction SilentlyContinue)) {
        throw "UI profile product resolution requires profile-capture-paths.ps1."
    }
    if (Test-Path -LiteralPath $managedRoot -PathType Container) {
        Assert-ZirconProfilePathContainsNoReparsePoint -Root $managedRoot -Candidate $candidate
    }
    return $candidate
}

function Resolve-ZirconUiProfileProductDirectory {
    param(
        [AllowEmptyString()]
        [string]$ProductDirectory = "",
        [AllowEmptyString()]
        [string]$CargoTargetDir = ""
    )

    if (-not [string]::IsNullOrWhiteSpace($ProductDirectory)) {
        return Resolve-ZirconUiProfileManagedChildPath `
            -Path $ProductDirectory `
            -ManagedRoots (Get-ZirconUiProfileManagedProductRoots) `
            -ErrorMessage "UI profile product directory must resolve below an approved managed product root."
    }

    if ([string]::IsNullOrWhiteSpace($CargoTargetDir)) {
        throw "ProductDirectory or CARGO_TARGET_DIR must identify a managed UI profiling product."
    }
    $cargoTarget = Resolve-ZirconUiProfileManagedChildPath `
        -Path $CargoTargetDir `
        -ManagedRoots (Get-ZirconUiProfileManagedCargoTargetRoots) `
        -ErrorMessage "CARGO_TARGET_DIR must resolve below an approved managed Cargo target root."
    return Join-Path $cargoTarget "profiling"
}
