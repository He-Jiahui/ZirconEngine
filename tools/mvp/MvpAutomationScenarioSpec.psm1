Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpAutomationScenarioSchemaVersion = 1
$script:MvpAutomationScenarioKind = 'zircon.mvp-editor-automation-scenario'
$script:MvpAutomationScenarioMaximumBytes = 65536

function Assert-MvpAutomationScenarioExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames
    )

    if ($null -eq $Value -or $Value -is [Array]) {
        throw 'MVP automation scenario must contain one JSON object.'
    }
    $actualNames = @($Value.PSObject.Properties | ForEach-Object { $_.Name })
    if ($actualNames.Count -ne $ExpectedNames.Count) {
        $unknownNames = @($actualNames | Where-Object { $_ -notin $ExpectedNames })
        if ($unknownNames.Count -gt 0) {
            throw "MVP automation scenario contains unknown property '$($unknownNames[0])'."
        }
        throw "MVP automation scenario property count differs from $($ExpectedNames.Count)."
    }
    foreach ($name in $actualNames) {
        if ($name -notin $ExpectedNames) {
            throw "MVP automation scenario contains unknown property '$name'."
        }
    }
}

function Assert-MvpAutomationScenarioSpec {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]
        [ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')]
        [string]$ExpectedScenarioId
    )

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP automation scenario does not exist: $resolvedPath"
    }
    $file = [IO.FileInfo]::new($resolvedPath)
    if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP automation scenario is a reparse point: $resolvedPath"
    }
    if ($file.Length -eq 0 -or $file.Length -gt $script:MvpAutomationScenarioMaximumBytes) {
        throw "MVP automation scenario must contain 1..$($script:MvpAutomationScenarioMaximumBytes) bytes."
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($resolvedPath)
    try {
        $scenario = [Text.UTF8Encoding]::new($false, $true).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP automation scenario is not strict UTF-8 JSON: $($_.Exception.Message)"
    }
    Assert-MvpAutomationScenarioExactProperties `
        -Value $scenario `
        -ExpectedNames @('schema_version', 'scenario_kind', 'scenario_id', 'bindings')
    if (($scenario.schema_version -isnot [int] -and $scenario.schema_version -isnot [long]) -or
        [Int64]$scenario.schema_version -ne $script:MvpAutomationScenarioSchemaVersion) {
        throw "MVP automation scenario has unsupported schema version '$($scenario.schema_version)'."
    }
    if ([string]$scenario.scenario_kind -cne $script:MvpAutomationScenarioKind) {
        throw "MVP automation scenario has unsupported kind '$($scenario.scenario_kind)'."
    }
    if ([string]$scenario.scenario_id -cne $ExpectedScenarioId) {
        throw "MVP automation scenario '$($scenario.scenario_id)' differs from expected '$ExpectedScenarioId'."
    }
    $bindings = @($scenario.bindings)
    if ($bindings.Count -eq 0 -or $bindings.Count -gt 64) {
        throw 'MVP automation scenario must contain 1..64 editor UI bindings.'
    }
    return [pscustomobject][ordered]@{
        schema_version = [int]$scenario.schema_version
        scenario_kind = [string]$scenario.scenario_kind
        scenario_id = [string]$scenario.scenario_id
        binding_count = $bindings.Count
        bytes = $bytes.Length
    }
}

Export-ModuleMember -Function 'Assert-MvpAutomationScenarioSpec'
