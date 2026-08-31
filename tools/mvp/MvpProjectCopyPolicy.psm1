Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpProjectCopyPolicyMaximumBytes = 64KB
$script:MvpProjectCopyPolicySchemaVersion = 1
$script:MvpProjectCopyPolicyKind = 'zircon.mvp-project-copy-policy'
$script:MvpProjectCopyPolicyUpperHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-MvpProjectCopyPolicyUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    for ($index = 0; $index -lt $Bytes.Length; $index++) {
        $value = $Bytes[$index]
        $characters[$index * 2] = $script:MvpProjectCopyPolicyUpperHexDigits[$value -shr 4]
        $characters[$index * 2 + 1] = $script:MvpProjectCopyPolicyUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-MvpProjectCopyPolicyBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpProjectCopyPolicyUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Assert-MvpProjectCopyPolicyExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value -or $Value -is [array] -or $Value -is [string] -or $Value -is [ValueType]) {
        throw "$Label must be one JSON object."
    }
    foreach ($name in $ExpectedNames) {
        if ($null -eq $Value.PSObject.Properties[$name]) {
            throw "$Label is missing required property '$name'."
        }
    }
    foreach ($property in $Value.PSObject.Properties) {
        if ($ExpectedNames -cnotcontains $property.Name) {
            throw "$Label contains unknown property '$($property.Name)'."
        }
    }
}

function Read-MvpProjectCopyPolicyBytes {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP project copy policy does not exist or is not a file: $Path"
    }
    $stream = [IO.File]::Open($resolvedPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt $script:MvpProjectCopyPolicyMaximumBytes) {
            throw "MVP project copy policy exceeds its byte budget of $($script:MvpProjectCopyPolicyMaximumBytes) bytes."
        }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw 'MVP project copy policy changed while it was being read.'
            }
            $offset += $read
        }
        Write-Output -NoEnumerate $bytes
    }
    finally {
        $stream.Dispose()
    }
}

function Assert-MvpProjectCopyPolicyRelativeDirectory {
    param([Parameter(Mandatory)][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value) -or
        $Value -cne $Value.ToLowerInvariant() -or
        $Value.Contains('\') -or
        $Value.StartsWith('/') -or
        $Value.EndsWith('/') -or
        $Value.Contains('//')) {
        throw "MVP project copy policy relative_directory '$Value' is not one canonical relative directory."
    }
    foreach ($component in $Value.Split('/')) {
        if ($component -in @('.', '..') -or $component -notmatch '^[a-z0-9.][a-z0-9._-]*$') {
            throw "MVP project copy policy relative_directory '$Value' is invalid."
        }
    }
    return $Value
}

function Get-MvpProjectCopyPolicySnapshot {
    param([Parameter(Mandatory)][string]$Path)

    [byte[]]$bytes = Read-MvpProjectCopyPolicyBytes -Path $Path
    try {
        $policy = ([Text.UTF8Encoding]::new($false, $true)).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP project copy policy is not valid strict UTF-8 JSON: $($_.Exception.Message)"
    }

    Assert-MvpProjectCopyPolicyExactProperties `
        -Value $policy `
        -ExpectedNames @('schema_version', 'policy_kind', 'policy_id', 'path_comparison', 'default', 'rules') `
        -Label 'MVP project copy policy'
    $schemaVersion = $policy.schema_version
    if (($schemaVersion -isnot [int] -and $schemaVersion -isnot [long]) -or
        [Int64]$schemaVersion -ne $script:MvpProjectCopyPolicySchemaVersion) {
        throw "MVP project copy policy schema_version must be the JSON integer $($script:MvpProjectCopyPolicySchemaVersion)."
    }
    if ([string]$policy.policy_kind -cne $script:MvpProjectCopyPolicyKind) {
        throw "MVP project copy policy has unsupported policy_kind '$($policy.policy_kind)'."
    }
    $policyId = [string]$policy.policy_id
    if ($policyId -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        throw "MVP project copy policy policy_id '$policyId' is invalid."
    }
    if ([string]$policy.path_comparison -cne 'ordinal-ignore-case') {
        throw "MVP project copy policy path_comparison must be 'ordinal-ignore-case'."
    }
    Assert-MvpProjectCopyPolicyExactProperties `
        -Value $policy.default `
        -ExpectedNames @('ownership', 'copy_policy') `
        -Label 'MVP project copy policy default'
    if ([string]$policy.default.ownership -cne 'source') {
        throw "MVP project copy policy default ownership must be 'source'."
    }
    if ([string]$policy.default.copy_policy -cne 'include') {
        throw "MVP project copy policy default copy_policy must be 'include'."
    }
    if ($policy.rules -isnot [array] -or $policy.rules.Count -eq 0 -or $policy.rules.Count -gt 64) {
        throw 'MVP project copy policy rules must be one non-empty array of at most 64 entries.'
    }

    $rules = [Collections.Generic.List[object]]::new()
    $seenPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $previousPath = $null
    foreach ($rule in @($policy.rules)) {
        Assert-MvpProjectCopyPolicyExactProperties `
            -Value $rule `
            -ExpectedNames @('relative_directory', 'ownership', 'copy_policy') `
            -Label 'MVP project copy policy rule'
        $relativeDirectory = Assert-MvpProjectCopyPolicyRelativeDirectory -Value ([string]$rule.relative_directory)
        if (-not $seenPaths.Add($relativeDirectory)) {
            throw "MVP project copy policy contains duplicate relative_directory '$relativeDirectory'."
        }
        if ($null -ne $previousPath -and [StringComparer]::Ordinal.Compare($previousPath, $relativeDirectory) -ge 0) {
            throw 'MVP project copy policy rule paths must be ordinally sorted.'
        }
        foreach ($existingRule in $rules) {
            $existingPath = [string]$existingRule.relative_directory
            if ($relativeDirectory.StartsWith($existingPath + '/', [StringComparison]::OrdinalIgnoreCase) -or
                $existingPath.StartsWith($relativeDirectory + '/', [StringComparison]::OrdinalIgnoreCase)) {
                throw "MVP project copy policy contains overlapping rule paths '$existingPath' and '$relativeDirectory'."
            }
        }
        $ownership = [string]$rule.ownership
        if ($ownership -cnotin @('derived', 'generated')) {
            throw "MVP project copy policy rule '$relativeDirectory' ownership '$ownership' is unsupported."
        }
        if ([string]$rule.copy_policy -cne 'exclude-subtree') {
            throw "MVP project copy policy rule '$relativeDirectory' copy_policy must be 'exclude-subtree'."
        }
        $rules.Add([pscustomobject][ordered]@{
                relative_directory = $relativeDirectory
                match_prefix = $relativeDirectory + '/'
                ownership = $ownership
                copy_policy = 'exclude-subtree'
            }) | Out-Null
        $previousPath = $relativeDirectory
    }

    return [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = $script:MvpProjectCopyPolicySchemaVersion
            policy_kind = $script:MvpProjectCopyPolicyKind
            policy_id = $policyId
            sha256 = Get-MvpProjectCopyPolicyBytesSha256 -Bytes $bytes
            size_bytes = [Int64]$bytes.LongLength
        }
        default = [pscustomobject][ordered]@{
            ownership = 'source'
            copy_policy = 'include'
        }
        rules = $rules.ToArray()
    }
}

function Test-MvpProjectCopyPolicyPathIncluded {
    param(
        [Parameter(Mandatory)]$PolicySnapshot,
        [Parameter(Mandatory)][string]$RelativePath
    )

    $normalized = $RelativePath.Replace('\', '/')
    if ([string]::IsNullOrWhiteSpace($normalized) -or
        $normalized.StartsWith('/') -or
        $normalized.EndsWith('/') -or
        $normalized.Contains('//')) {
        throw "MVP project copy policy cannot classify invalid relative path '$RelativePath'."
    }
    $containsDotComponent =
        $normalized.Equals('.', [StringComparison]::Ordinal) -or
        $normalized.Equals('..', [StringComparison]::Ordinal) -or
        $normalized.StartsWith('./', [StringComparison]::Ordinal) -or
        $normalized.StartsWith('../', [StringComparison]::Ordinal) -or
        $normalized.EndsWith('/.', [StringComparison]::Ordinal) -or
        $normalized.EndsWith('/..', [StringComparison]::Ordinal) -or
        $normalized.IndexOf('/./', [StringComparison]::Ordinal) -ge 0 -or
        $normalized.IndexOf('/../', [StringComparison]::Ordinal) -ge 0
    if ($containsDotComponent) {
        throw "MVP project copy policy cannot classify escaping relative path '$RelativePath'."
    }
    foreach ($rule in $PolicySnapshot.rules) {
        $excluded = [string]$rule.relative_directory
        $excludedPrefix = [string]$rule.match_prefix
        if ($normalized.Equals($excluded, [StringComparison]::OrdinalIgnoreCase) -or
            $normalized.StartsWith($excludedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            return $false
        }
    }
    return $true
}

Export-ModuleMember -Function @(
    'Get-MvpProjectCopyPolicySnapshot',
    'Test-MvpProjectCopyPolicyPathIncluded'
)
