$ErrorActionPreference = 'Stop'

$processJournalModule = Join-Path $PSScriptRoot 'MvpProcessLifecycleJournal.psm1'
Import-Module $processJournalModule -ErrorAction Stop

$script:MvpProcessEnvironmentPolicySchemaVersion = 1
$script:MvpProcessEnvironmentPolicyKind = 'zircon.mvp-process-environment-policy'
$script:MvpProcessHostEnvironmentNames = @(
    'ComSpec',
    'NUMBER_OF_PROCESSORS',
    'OS',
    'PATH',
    'PATHEXT',
    'PROCESSOR_ARCHITECTURE',
    'PROCESSOR_IDENTIFIER',
    'PROCESSOR_LEVEL',
    'PROCESSOR_REVISION',
    'SystemRoot',
    'TEMP',
    'TMP',
    'WINDIR'
)
$script:MvpProcessDeclaredEnvironmentNames = @(
    'ZIRCON_ASSET_ROOT',
    'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG',
    'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME',
    'ZIRCON_LOG_FILTER',
    'ZIRCON_LOG_ROOT',
    'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG',
    'ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME',
    'ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES',
    'ZIRCON_RUNTIME_LIBRARY',
    'ZIRCON_RUNTIME_MVP_INPUT_PROBE'
)

function Get-MvpProcessEnvironmentSensitivity {
    param([Parameter(Mandatory)][string]$Name)

    if ($Name -match '(?i)(credential|cookie|key|password|secret|token)') {
        return 'sensitive'
    }
    return 'non_sensitive'
}

function ConvertTo-MvpProcessEnvironmentNameSet {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Names,
        [Parameter(Mandatory)][string[]]$AllowedNames,
        [Parameter(Mandatory)][string]$Kind
    )

    $result = [System.Collections.Generic.List[string]]::new()
    $seen = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($name in $Names) {
        if ([string]::IsNullOrWhiteSpace($name) -or $name -notmatch '^[A-Za-z_][A-Za-z0-9_]*$') {
            throw "The process environment policy contains an invalid $Kind name '$name'."
        }
        if ($AllowedNames -notcontains $name) {
            throw "The process environment $Kind '$name' is not in the supervisor $Kind allowlist."
        }
        if (-not $seen.Add($name)) {
            throw "The process environment policy contains duplicate $Kind name '$name'."
        }
        $result.Add($name) | Out-Null
    }
    return @($result.ToArray() | Sort-Object)
}

function Assert-MvpProcessEnvironmentPolicySchema {
    param([Parameter(Mandatory)]$Policy)

    $expectedNames = @(
        'schema_version',
        'policy_kind',
        'policy_id',
        'inherited_names',
        'declared_names')
    $actualNames = @($Policy.PSObject.Properties | ForEach-Object { $_.Name })
    if ($actualNames.Count -ne $expectedNames.Count) {
        throw "MVP process environment policy property count differs from $($expectedNames.Count)."
    }
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $expectedNames) {
        $expected.Add($name) | Out-Null
    }
    foreach ($name in $actualNames) {
        if (-not $expected.Contains($name)) {
            throw "MVP process environment policy contains unknown property '$name'."
        }
    }
    if (($Policy.schema_version -isnot [int] -and $Policy.schema_version -isnot [long]) -or
        [Int64]$Policy.schema_version -ne $script:MvpProcessEnvironmentPolicySchemaVersion) {
        throw "MVP process environment policy has unsupported schema version '$($Policy.schema_version)'."
    }
    if ([string]$Policy.policy_kind -cne $script:MvpProcessEnvironmentPolicyKind) {
        throw "MVP process environment policy has unsupported kind '$($Policy.policy_kind)'."
    }
}

function New-MvpProcessEnvironmentPolicy {
    param(
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,63}$')][string]$PolicyId,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$InheritedNames,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$DeclaredNames
    )

    $inherited = ConvertTo-MvpProcessEnvironmentNameSet `
        -Names $InheritedNames `
        -AllowedNames $script:MvpProcessHostEnvironmentNames `
        -Kind 'host'
    $declared = ConvertTo-MvpProcessEnvironmentNameSet `
        -Names $DeclaredNames `
        -AllowedNames $script:MvpProcessDeclaredEnvironmentNames `
        -Kind 'declared'
    return [pscustomobject][ordered]@{
        schema_version = $script:MvpProcessEnvironmentPolicySchemaVersion
        policy_kind = $script:MvpProcessEnvironmentPolicyKind
        policy_id = $PolicyId
        inherited_names = @($inherited)
        declared_names = @($declared)
    }
}

function Set-MvpProcessEnvironmentPolicy {
    param(
        [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$StartInfo,
        [Parameter(Mandatory)]$Policy,
        [hashtable]$DeclaredEnvironment = @{}
    )

    Assert-MvpProcessEnvironmentPolicySchema -Policy $Policy
    $validatedPolicy = New-MvpProcessEnvironmentPolicy `
        -PolicyId ([string]$Policy.policy_id) `
        -InheritedNames @($Policy.inherited_names) `
        -DeclaredNames @($Policy.declared_names)
    $StartInfo.EnvironmentVariables.Clear()
    $records = [System.Collections.Generic.List[object]]::new()
    foreach ($name in $validatedPolicy.inherited_names) {
        $value = [Environment]::GetEnvironmentVariable($name, [EnvironmentVariableTarget]::Process)
        if ([string]::IsNullOrWhiteSpace($value)) {
            continue
        }
        $StartInfo.EnvironmentVariables[$name] = $value
        $records.Add([ordered]@{
            name = $name
            source = 'supervisor_inherited'
            sensitivity = Get-MvpProcessEnvironmentSensitivity -Name $name
            value_sha256 = Get-MvpProcessJournalSha256 -Bytes ([Text.Encoding]::UTF8.GetBytes($value))
        }) | Out-Null
    }
    foreach ($name in @($DeclaredEnvironment.Keys | ForEach-Object { [string]$_ } | Sort-Object)) {
        if ($validatedPolicy.declared_names -notcontains $name) {
            throw "The staged process environment variable '$name' is not allowed by environment policy '$($validatedPolicy.policy_id)'."
        }
        $sensitivity = Get-MvpProcessEnvironmentSensitivity -Name $name
        if ($sensitivity -eq 'sensitive') {
            throw "The staged process environment variable '$name' is sensitive and cannot be declared."
        }
        $value = [string]$DeclaredEnvironment[$name]
        $StartInfo.EnvironmentVariables[$name] = $value
        $records.Add([ordered]@{
            name = $name
            source = 'scenario_declared'
            sensitivity = $sensitivity
            value_sha256 = Get-MvpProcessJournalSha256 -Bytes ([Text.Encoding]::UTF8.GetBytes($value))
        }) | Out-Null
    }
    return [pscustomobject][ordered]@{
        schema_version = $validatedPolicy.schema_version
        policy_kind = $validatedPolicy.policy_kind
        policy_id = $validatedPolicy.policy_id
        variables = @($records.ToArray())
    }
}

Export-ModuleMember -Function @(
    'New-MvpProcessEnvironmentPolicy',
    'Set-MvpProcessEnvironmentPolicy'
)
